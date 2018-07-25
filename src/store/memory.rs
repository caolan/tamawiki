//! An in-memory store, useful for testing.

use super::{Store, StoreError};
use document::Update;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::vec;


/// An in-memory Store, useful for testing.
#[derive(Default)]
pub struct MemoryStore {
    documents: HashMap<PathBuf, Vec<Update>>,
}

impl Store for MemoryStore {
    type Iter = vec::IntoIter<Update>;
    
    fn push(&mut self, path: PathBuf, update: Update) -> Result<usize, StoreError> {
        let history = self.documents.entry(path).or_insert_with(|| Vec::new());
        history.push(update);
        Ok(history.len())
    }

    fn since(&self, path: &Path, seq: usize) -> Result<Self::Iter, StoreError> {
        match self.documents.get(path) {
            Some(history) => {
                if seq <= history.len() {
                    Ok(Vec::from(&history[seq..]).into_iter())
                } else {
                    Err(StoreError::InvalidSequenceNumber)
                }
            },
            None => Err(StoreError::NotFound),
        }
    }

    fn seq(&self, path: &Path) -> Result<usize, StoreError> {
        match self.documents.get(path) {
            Some(history) => Ok(history.len()),
            None => Err(StoreError::NotFound),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use document::{Document, Operation, Insert};
    
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
            store.since(&path.as_path(), 0).unwrap().collect::<Vec<Update>>(),
            vec![a.clone(), b.clone(), c.clone()]
        );
        assert_eq!(
            store.since(&path.as_path(), 1).unwrap().collect::<Vec<Update>>(),
            vec![b.clone(), c.clone()]
        );
        assert_eq!(
            store.since(&path.as_path(), 2).unwrap().collect::<Vec<Update>>(),
            vec![c.clone()]
        );
        // requesting the last sequence number is valid, but would
        // return an empty iterator
        assert_eq!(
            store.since(&path.as_path(), 3).unwrap().collect::<Vec<Update>>(),
            vec![]
        );
        // requesting updates since a sequence number not in the store
        // is invalid, however
        match store.since(&path.as_path(), 4) {
            Err(StoreError::InvalidSequenceNumber) => (),
            _ => assert!(false),
        }
        // requesting updates for a missing path is an error
        match store.since(&PathBuf::from("/missing").as_path(), 0) {
            Err(StoreError::NotFound) => (),
            _ => assert!(false),
        }
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
