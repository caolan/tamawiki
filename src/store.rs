//! Persists document updates using paths as keys. The current state
//! of a document at a point in time by calculated by the accumulated
//! application of updates within the store up until that point.

use document::{Document, Update};
use std::collections::HashMap;
use std::path::{Path, PathBuf};


/// The Store trait must be implemented by any storage backends and
/// describes the minimal API required by TamaWiki. Default functions
/// are provided for content() and content_at() but they replay all
/// update events from seq=0, so should be replaced with more
/// performant versions specific to the storeage backend.
pub trait Store<'a> {
    /// The IntoIterator returned by since()
    type Iter: IntoIterator<Item=&'a Update>;

    /// Adds a new Update to the document at 'path' and increments the
    /// sequence number. If the document does not exist, the act of
    /// pushing an update creates it.
    fn push(&'a mut self, path: PathBuf, update: Update) -> Result<usize, StoreError>;

    /// Returns an IntoIterator over the Updates starting *after* the
    /// provided sequence number. Requesting the current sequence
    /// number is not an error, but will return an empty IntoIterator.
    /// Requesting updates since a sequence number that does not exist
    /// yet is a StoreError::InvalidSequenceNumber.
    fn since(&'a self, path: &Path, seq: usize) -> Result<Self::Iter, StoreError>;

    /// Returns the current sequence number for the document at
    /// 'path', or StoreError::NotFound if it does not exist.
    fn seq(&'a self, path: &Path) -> Result<usize, StoreError>;

    /// Returns a snapshot of the document's content at a specific
    /// sequence number. All updates from 0 to 'seq' will be applied.
    /// Results in a StoreError::NotFound if the document does not
    /// exist, and StoreError::InvalidSequenceNumber if the sequence
    /// number does not exist.
    fn content_at(&'a self, path: &Path, seq: usize) -> Result<Document, StoreError> {
        let mut doc: Document = Default::default();
        if seq > self.seq(&path)? {
            return Err(StoreError::InvalidSequenceNumber);
        }
        for update in self.since(&path, 0)?.into_iter().take(seq) {
            if let Err(_) = doc.apply(update) {
                return Err(StoreError::InvalidDocument);
            }
        }
        Ok(doc)
    }

    /// Returns the current sequence number and content for the
    /// document (with all updates applied), or StoreError::NotFound
    /// if the document does not exist.
    fn content(&'a self, path: &Path)
               -> Result<(usize, Document), StoreError> {
        let seq = self.seq(&path)?;
        self.content_at(&path, seq).map(|content| (seq, content))
    }
}

/// Error conditions for reading data from or writing data to the
/// store.
#[derive(Debug, PartialEq)]
pub enum StoreError {
    /// The document path is missing
    NotFound,
    /// A non-existant sequence number was requested for the document
    InvalidSequenceNumber,
    /// When requesting content, the store failed to build a
    /// representation of the document from the updates in the store
    InvalidDocument,
    /// There is a problem communicating with the storage backend
    ConnectionError,
}

/// An in-memory Store, useful for testing.
#[derive(Default)]
pub struct MemoryStore {
    documents: HashMap<PathBuf, Vec<Update>>,
}

impl<'a> Store<'a> for MemoryStore {
    type Iter = &'a[Update];
    
    fn push(&'a mut self, path: PathBuf, update: Update) -> Result<usize, StoreError> {
        let history = self.documents.entry(path).or_insert_with(|| Vec::new());
        history.push(update);
        Ok(history.len())
    }

    fn since(&'a self, path: &Path, seq: usize) -> Result<Self::Iter, StoreError> {
        match self.documents.get(path) {
            Some(history) => {
                if seq <= history.len() {
                    Ok(&history[seq..])
                } else {
                    Err(StoreError::InvalidSequenceNumber)
                }
            },
            None => Err(StoreError::NotFound),
        }
    }

    fn seq(&'a self, path: &Path) -> Result<usize, StoreError> {
        match self.documents.get(path) {
            Some(history) => Ok(history.len()),
            None => Err(StoreError::NotFound),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use document::{Operation, Insert};
    
    #[test]
    fn memory_store_push() {
        let mut store: MemoryStore = Default::default();
        
        store.push(PathBuf::from("/foo/bar"), Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Hello")
                })
            ]
        }).unwrap();

        store.push(PathBuf::from("/asdf"), Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Hello")
                })
            ]
        }).unwrap();

        store.push(PathBuf::from("/asdf"), Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 5,
                    content: String::from(", world!")
                })
            ]
        }).unwrap();

        assert_eq!(
            store.documents.get(PathBuf::from("/foo/bar").as_path()).unwrap(),
            &vec![
                Update {
                    operations: vec![
                        Operation::Insert(Insert {
                            pos: 0,
                            content: String::from("Hello")
                        })
                    ]
                }
            ]
        );

        assert_eq!(
            store.documents.get(PathBuf::from("/asdf").as_path()).unwrap(),
            &vec![
                Update {
                    operations: vec![
                        Operation::Insert(Insert {
                            pos: 0,
                            content: String::from("Hello")
                        })
                    ]
                },
                Update {
                    operations: vec![
                        Operation::Insert(Insert {
                            pos: 5,
                            content: String::from(", world!")
                        })
                    ]
                }
            ]
        );
    }

    #[test]
    fn memory_store_since() {
        let mut store: MemoryStore = Default::default();
        
        let a = Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Hello")
                })
            ]
        };
        let b = Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 5,
                    content: String::from(", world")
                })
            ]
        };
        let c = Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 12,
                    content: String::from("!")
                })
            ]
        };

        let path = PathBuf::from("/foo/bar");
        store.push(path.clone(), a.clone()).unwrap();
        store.push(path.clone(), b.clone()).unwrap();
        store.push(path.clone(), c.clone()).unwrap();
        
        assert_eq!(
            store.since(&path.as_path(), 0),
            Ok(vec![a.clone(), b.clone(), c.clone()].as_slice())
        );
        assert_eq!(
            store.since(&path.as_path(), 1),
            Ok(vec![b.clone(), c.clone()].as_slice())
        );
        assert_eq!(
            store.since(&path.as_path(), 2),
            Ok(vec![c.clone()].as_slice())
        );
        // requesting the last sequence number is valid, but would
        // return an empty iterator
        assert_eq!(
            store.since(&path.as_path(), 3),
            Ok(vec![].as_slice())
        );
        // requesting updates since a sequence number not in the store
        // is invalid, however
        assert_eq!(
            store.since(&path.as_path(), 4),
            Err(StoreError::InvalidSequenceNumber)
        );
        // requesting updates for a missing path is an error
        assert_eq!(
            store.since(&PathBuf::from("/missing").as_path(), 0),
            Err(StoreError::NotFound)
        );
    }

    #[test]
    fn memory_store_seq() {
        let mut store: MemoryStore = Default::default();
        
        store.push(PathBuf::from("/foo/bar"), Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Hello")
                })
            ]
        }).unwrap();

        store.push(PathBuf::from("/asdf"), Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Hello")
                })
            ]
        }).unwrap();

        store.push(PathBuf::from("/asdf"), Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 5,
                    content: String::from(", world!")
                })
            ]
        }).unwrap();

        assert_eq!(
            store.seq(PathBuf::from("/foo/bar").as_path()),
            Ok(1)
        );
        assert_eq!(
            store.seq(PathBuf::from("/asdf").as_path()),
            Ok(2)
        );
        // requesting a non-existing path is an error
        assert_eq!(
            store.seq(PathBuf::from("/not_found").as_path()),
            Err(StoreError::NotFound)
        );
    }

    #[test]
    fn memory_store_content() {
        let mut store: MemoryStore = Default::default();

        store.push(PathBuf::from("/asdf"), Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Hello")
                })
            ]
        }).unwrap();

        store.push(PathBuf::from("/asdf"), Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 5,
                    content: String::from(", world!")
                })
            ]
        }).unwrap();

        assert_eq!(
            store.content(&PathBuf::from("/asdf").as_path()),
            Ok((2, Document::from("Hello, world!")))
        );
        assert_eq!(
            store.content(&PathBuf::from("/missing").as_path()),
            Err(StoreError::NotFound)
        );
    }

    #[test]
    fn memory_store_content_at() {
        let mut store: MemoryStore = Default::default();

        store.push(PathBuf::from("/asdf"), Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Hello")
                })
            ]
        }).unwrap();

        store.push(PathBuf::from("/asdf"), Update {
            operations: vec![
                Operation::Insert(Insert {
                    pos: 5,
                    content: String::from(", world!")
                })
            ]
        }).unwrap();

        assert_eq!(
            store.content_at(&PathBuf::from("/asdf").as_path(), 0),
            Ok(Document::from(""))
        );
        assert_eq!(
            store.content_at(&PathBuf::from("/asdf").as_path(), 1),
            Ok(Document::from("Hello"))
        );
        assert_eq!(
            store.content_at(&PathBuf::from("/asdf").as_path(), 2),
            Ok(Document::from("Hello, world!"))
        );
        // requesting a sequence number higher than the number of
        // updates return error
        assert_eq!(
            store.content_at(&PathBuf::from("/asdf").as_path(), 3),
            Err(StoreError::InvalidSequenceNumber)
        );
        // requesting a missing document is an error
        assert_eq!(
            store.content_at(&PathBuf::from("/missing").as_path(), 0),
            Err(StoreError::NotFound)
        );
    }

}
