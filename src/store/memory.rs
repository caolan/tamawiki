//! An in-memory store, useful for testing.
use std::sync::{Arc, Mutex};
use futures::stream::Stream;
use futures::future::{self, Future};
use futures::{Poll, Async};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use super::{Store, StoreClient, StoreError, SequenceId};
use document::{Document, Update};


type Updates = Arc<Mutex<Vec<Update>>>;
type Documents = Arc<Mutex<HashMap<PathBuf, Updates>>>;

/// Holds document data in shared memory location
#[derive(Default, Debug, Clone)]
pub struct MemoryStore {
    documents: Documents
}

impl Store for MemoryStore {
    type Client = MemoryStoreClient;
    
    fn client(&self) -> Self::Client {
        Self::Client {
            documents: self.documents.clone()
        }
    }
}

/// An asynchronous stream of Update objects cloned from memory
pub struct MemoryStoreStream {
    updates: Updates,
    seq: SequenceId,
}

impl Stream for MemoryStoreStream {
    type Item = (SequenceId, Update);
    type Error = StoreError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.updates.lock() {
            Ok(updates) => {
                self.seq += 1;
                if self.seq <= updates.len() as u64 {
                    Ok(Async::Ready(Some(
                        (self.seq, updates[(self.seq - 1) as usize].clone())
                    )))
                } else {
                    Ok(Async::Ready(None))
                }
            },
            Err(_) => Err(StoreError::ConnectionError),
        }
    }
}

/// A thread-safe client interface to the shared document data memory
#[derive(Debug, Clone)]
pub struct MemoryStoreClient {
    documents: Documents
}

impl StoreClient for MemoryStoreClient {
    type Stream = MemoryStoreStream;
    
    fn push(&mut self, path: PathBuf, update: Update) ->
        Box<Future<Item=SequenceId, Error=StoreError> + Send>
    {
        let mut documents = match self.documents.lock() {
            Ok(documents) => documents,
            Err(_) => return Box::new(
                future::err(StoreError::ConnectionError)),
        };

        let updates = documents.entry(path)
            .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));
        
        let mut updates = match updates.lock() {
            Ok(updates) => updates,
            Err(_) => return Box::new(
                future::err(StoreError::ConnectionError)),
        };
        
        updates.push(update);
        Box::new(
            future::ok(updates.len() as u64)
        )
    }

    fn seq(&self, path: &Path) ->
        Box<Future<Item=SequenceId, Error=StoreError> + Send>
    {
        let documents = match self.documents.lock() {
            Ok(documents) => documents,
            Err(_) => return Box::new(
                future::err(StoreError::ConnectionError)),
        };
        
        let updates = match documents.get(path) {
            Some(updates) => match updates.lock() {
                Ok(updates) => updates,
                Err(_) => return Box::new(
                    future::err(StoreError::ConnectionError)),
            },
            None => return Box::new(
                future::err(StoreError::NotFound)),
        };
        
        Box::new(
            future::ok(updates.len() as u64)
        )
    }

    fn since(&self, path: &Path, seq: SequenceId) ->
        Box<Future<Item=Self::Stream, Error=StoreError> + Send>
    {
        let documents = match self.documents.lock() {
            Ok(documents) => documents,
            Err(_) => return Box::new(
                future::err(StoreError::ConnectionError)),
        };
        
        let updates = match documents.get(path) {
            Some(updates) => updates.clone(),
            None => return Box::new(
                future::err(StoreError::NotFound)),
        };

        // check sequence id is valid
        match updates.lock() {
            Ok(updates) => if seq > updates.len() as u64 {
                return Box::new(
                    future::err(StoreError::InvalidSequenceId))
            },
            Err(_) => return Box::new(
                future::err(StoreError::ConnectionError)),
        }
        
        Box::new(
            future::ok(Self::Stream { updates, seq })
        )
    }

    fn content(&self, path: &Path) ->
        Box<Future<Item=(SequenceId, Document), Error=StoreError> + Send>
    {
        Box::new(
            self.since(path, 0).and_then(|stream| {
                stream.fold(
                    (0, Document::default()),
                    |(_seq, mut doc), (seq, update)| {
                        match doc.apply(&update) {
                            Err(_) => future::err(StoreError::InvalidDocument),
                            Ok(_) => future::ok((seq, doc)),
                        }
                    }
                )
            })
        )
    }

    fn content_at(&self, path: &Path, seq: SequenceId) ->
        Box<Future<Item=Document, Error=StoreError> + Send>
    {
        let check_seq = self.seq(path).and_then(move |head| {
            if seq > head {
                future::err(StoreError::InvalidSequenceId)
            } else {
                future::ok(())
            }
        });
        let doc = self.since(path, 0).and_then(move |stream| {
            stream.take(seq).fold(
                Document::default(),
                |mut doc, (_seq, update)| {
                    match doc.apply(&update) {
                        Err(_) => future::err(StoreError::InvalidDocument),
                        Ok(_) => future::ok(doc),
                    }
                }
            )
        });
        Box::new(
            check_seq.and_then(move |_| doc)
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use document::{Operation, Insert};
    
    #[test]
    fn memory_store_push() {
        let mut store = MemoryStore::default().client();

        let push1 = store.push(
            PathBuf::from("/foo/bar"),
            Update {
                from: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            }
        );

        let push2 = store.push(
            PathBuf::from("/asdf"),
            Update {
                from: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            }
        );

        let push3 = store.push(
            PathBuf::from("/asdf"),
            Update {
                from: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            }
        );

        push1
            .and_then(|_| push2)
            .and_then(|_| push3)
            .map(|_| ())
            .map_err(|err| {
                panic!("{}", err);
            })
            .wait().unwrap();
    }

    #[test]
    fn memory_store_since() {
        let mut store = MemoryStore::default().client();

        let a = Update {
            from: 1,
            operations: vec![
                Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Hello")
                })
            ]
        };

        let b = Update {
            from: 1,
            operations: vec![
                Operation::Insert(Insert {
                    pos: 5,
                    content: String::from(", world")
                })
            ]
        };

        let c = Update {
            from: 1,
            operations: vec![
                Operation::Insert(Insert {
                    pos: 12,
                    content: String::from("!")
                })
            ]
        };
        
        let push1 = store.push(
            PathBuf::from("/foo/bar"),
            a.clone()
        );

        let push2 = store.push(
            PathBuf::from("/foo/bar"),
            b.clone()
        );

        let push3 = store.push(
            PathBuf::from("/foo/bar"),
            c.clone()
        );

        let a0 = a.clone();
        let b0 = b.clone();
        let c0 = c.clone();
        let since0 = store.since(
            &PathBuf::from("/foo/bar").as_path(),
            0
        ).and_then(|stream| {
            stream.collect().map_err(|err| {
                panic!("{}", err);
            })
        }).map(|updates| {
            assert_eq!(updates, vec![(1, a0), (2, b0), (3, c0)]);
        });

        let b1 = b.clone();
        let c1 = c.clone();
        let since1 = store.since(
            &PathBuf::from("/foo/bar").as_path(),
            1
        ).and_then(|stream| {
            stream.collect().map_err(|err| {
                panic!("{}", err);
            })
        }).map(|updates| {
            assert_eq!(updates, vec![(2, b1), (3, c1)]);
        });
            
        let c2 = c.clone();
        let since2 = store.since(
            &PathBuf::from("/foo/bar").as_path(),
            2
        ).and_then(|stream| {
            stream.collect().map_err(|err| {
                panic!("{}", err);
            })
        }).map(|updates| {
            assert_eq!(updates, vec![(3, c2)]);
        });

        let since3 = store.since(
            &PathBuf::from("/foo/bar").as_path(),
            3
        ).and_then(|stream| {
            stream.collect().map_err(|err| {
                panic!("{}", err);
            })
        }).map(|updates| {
            // requesting the last sequence number is valid, but would
            // return an empty result
            assert_eq!(updates, vec![]);
        });

        let since4 = store.since(
            &PathBuf::from("/foo/bar").as_path(),
            4
        ).map(|_| {
            // requesting updates since a sequence id not in the store
            // is invalid, however
            assert!(false)
        }).or_else(|err| {
            match err {
                StoreError::InvalidSequenceId => future::ok(()),
                _ => future::err(err),
            }
        });

        push1
            .and_then(|_| push2)
            .and_then(|_| push3)
            .and_then(|_| {
                since0
                    .join(since1)
                    .join(since2)
                    .join(since3)
                    .join(since4)
            })
            .map_err(|err| {
                panic!("{}", err);
            })
            .wait().unwrap();
    }

    #[test]
    fn memory_store_seq() {
        let mut store = MemoryStore::default().client();

        let push1 = store.push(
            PathBuf::from("/foo/bar"),
            Update {
                from: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            }
        );

        let push2 = store.push(
            PathBuf::from("/asdf"),
            Update {
                from: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            }
        );

        let push3 = store.push(
            PathBuf::from("/asdf"),
            Update {
                from: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            }
        );

        let seq1 = store.seq(
            &PathBuf::from("/foo/bar").as_path()
        ).map(|seq| {
            assert_eq!(seq, 1);
        });

        let seq2 = store.seq(
            &PathBuf::from("/asdf").as_path()
        ).map(|seq| {
            assert_eq!(seq, 2);
        });

        let seq3 = store.seq(
            &PathBuf::from("/not_found").as_path()
        ).map(|_seq| {
            // requesting a non-existing path is an error
            assert!(false);
        }).or_else(|err| {
            match err {
                StoreError::NotFound => future::ok(()),
                _ => future::err(err),
            }
        });
        
        push1
            .and_then(|_| push2)
            .and_then(|_| push3)
            .and_then(|_| {
                seq1.join(seq2)
                    .join(seq3)
            })
            .map_err(|err| {
                panic!("{}", err);
            })
            .wait().unwrap();
    }

    #[test]
    fn memory_store_content() {
        let mut store = MemoryStore::default().client();

        let push1 = store.push(
            PathBuf::from("/asdf"),
            Update {
                from: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            }
        );

        let push2 = store.push(
            PathBuf::from("/asdf"),
            Update {
                from: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            }
        );

        let content1 = store.content(
            &PathBuf::from("/asdf").as_path()
        ).map(|(seq, doc)| {
            assert_eq!(seq, 2);
            assert_eq!(doc, Document::from("Hello, world"));
        });

        let content2 = store.content(
            &PathBuf::from("/missing").as_path()
        ).map(|_| {
            assert!(false);
        }).or_else(|err| {
            match err {
                StoreError::NotFound => future::ok(()),
                _ => future::err(err),
            }
        });

        push1
            .and_then(|_| push2)
            .and_then(|_| content1.join(content2))
            .map_err(|err| {
                panic!("{}", err);
            })
            .wait().unwrap();
    }

    #[test]
    fn memory_store_content_at() {
        let mut store = MemoryStore::default().client();

        let push1 = store.push(
            PathBuf::from("/asdf"),
            Update {
                from: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            }
        );

        let push2 = store.push(
            PathBuf::from("/asdf"),
            Update {
                from: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            }
        );

        let content0 = store.content_at(
            &PathBuf::from("/asdf").as_path(),
            0
        ).map(|doc| {
            assert_eq!(doc, Document::from(""));
        });

        let content1 = store.content_at(
            &PathBuf::from("/asdf").as_path(),
            1
        ).map(|doc| {
            assert_eq!(doc, Document::from("Hello"));
        });

        let content2 = store.content_at(
            &PathBuf::from("/asdf").as_path(),
            2
        ).map(|doc| {
            assert_eq!(doc, Document::from("Hello, world"));
        });
        
        let content3 = store.content_at(
            &PathBuf::from("/asdf").as_path(),
            3
        ).map(|_| {
            // requesting a sequence number higher than the number of
            // updates return error
            assert!(false);
        }).or_else(|err| {
            match err {
                StoreError::InvalidSequenceId => future::ok(()),
                _ => future::err(err),
            }
        });

        let content4 = store.content_at(
            &PathBuf::from("/missing").as_path(),
            0
        ).map(|_| {
            // requesting a missing document is an error
            assert!(false);
        }).or_else(|err| {
            match err {
                StoreError::NotFound => future::ok(()),
                _ => future::err(err),
            }
        });

        push1
            .and_then(|_| push2)
            .and_then(|_| {
                content0
                    .join(content1)
                    .join(content2)
                    .join(content3)
                    .join(content4)
            })
            .map_err(|err| {
                panic!("{}", err);
            })
            .wait().unwrap();
    }

}
