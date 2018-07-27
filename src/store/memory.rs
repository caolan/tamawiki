//! An in-memory store, useful for testing.

use super::*;
use document::Update;

use actix::{Actor, Context, Handler};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use futures::stream::Stream;
use futures::{Poll, Async};


impl Store for MemoryStore {}

/// An asynchronous stream of Update objects
pub struct MemoryStoreStream {
    updates: Arc<Mutex<Vec<Update>>>,
    seq: usize
}

impl Stream for MemoryStoreStream {
    type Item = Update;
    type Error = StoreError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.updates.lock() {
            Err(_) => Err(StoreError::ConnectionError),
            Ok(data) => {
                self.seq += 1;
                if self.seq <= data.len() {
                    Ok(Async::Ready(Some(data[self.seq - 1].clone())))
                } else {
                    Ok(Async::Ready(None))
                }
            }
        }
    }
}

/// An in-memory Store, useful for testing.
#[derive(Default)]
pub struct MemoryStore {
    documents: HashMap<PathBuf, Arc<Mutex<Vec<Update>>>>,
}

impl MemoryStore {
    fn seq(&self, path: &Path) -> Result<usize, StoreError> {
        match self.documents.get(path) {
            Some(history) => {
                match history.lock() {
                    Err(_) => Err(StoreError::ConnectionError),
                    Ok(data) => Ok(data.len()),
                }
            },
            None => Err(StoreError::NotFound),
        }
    }

    fn since(&self, path: &Path, seq: usize) -> Result<MemoryStoreStream, StoreError> {
        match self.documents.get(path) {
            Some(history) => {
                match history.lock() {
                    Ok(data) => {
                        if seq <= data.len() {
                            Ok(MemoryStoreStream {
                                updates: history.clone(),
                                seq: seq,
                            })
                        } else {
                            Err(StoreError::InvalidSequenceNumber)
                        }
                    },
                    Err(_) => {
                        Err(StoreError::ConnectionError)
                    }
                }
            },
            None => Err(StoreError::NotFound),
        }
    }

    fn content_at(&self, path: &Path, seq: usize) -> Result<Document, StoreError> {
        match self.documents.get(path) {
            Some(history) => {
                match history.lock() {
                    Err(_) => Err(StoreError::ConnectionError),
                    Ok(data) => {
                        if seq <= data.len() {
                            let mut doc: Document = Default::default();
                            for update in &data[..seq] {
                                if let Err(_) = doc.apply(update) {
                                    return Err(StoreError::InvalidDocument);
                                }
                            }
                            Ok(doc)
                        } else {
                            Err(StoreError::InvalidSequenceNumber)
                        }
                    },
                }
            },
            None => Err(StoreError::NotFound),
        }
    }
}

impl Actor for MemoryStore {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("MemoryStore started");
    }
    
    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("MemoryStore stopped");
    }
}

impl Handler<Push> for MemoryStore {
    type Result = Result<usize, StoreError>;

    fn handle(&mut self, msg: Push, _ctx: &mut Context<Self>) -> Self::Result {
        let history = self.documents.entry(msg.path).or_insert_with(|| {
            Arc::new(Mutex::new(Vec::new()))
        });
        match history.lock() {
            Ok(mut data) => {
                data.push(msg.update);
                Ok(data.len())
            },
            Err(_) => {
                Err(StoreError::ConnectionError)
            }
        }
    }
}

impl Handler<Seq> for MemoryStore {
    type Result = Result<usize, StoreError>;

    fn handle(&mut self, msg: Seq, _ctx: &mut Context<Self>) -> Self::Result {
        self.seq(msg.path.as_path())
    }
}

impl Handler<Since> for MemoryStore {
    type Result = Result<Box<Stream<Item=Update, Error=StoreError>>, StoreError>;

    fn handle(&mut self, msg: Since, _ctx: &mut Context<Self>) -> Self::Result {
        self.since(msg.path.as_path(), msg.seq).map(
            |stream| -> Box<Stream<Item=Update, Error=StoreError>> {
                Box::new(stream)
            }
        )
    }
}

impl Handler<ContentAt> for MemoryStore {
    type Result = Result<Document, StoreError>;

    fn handle(&mut self, msg: ContentAt, _ctx: &mut Context<Self>) -> Self::Result {
        self.content_at(msg.path.as_path(), msg.seq)
    }
}

impl Handler<Content> for MemoryStore {
    type Result = Result<(usize, Document), StoreError>;

    fn handle(&mut self, msg: Content, _ctx: &mut Context<Self>) -> Self::Result {
        let path = msg.path.as_path();
        let seq = self.seq(&path)?;
        Ok((seq, self.content_at(&path, seq)?))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use actix::prelude::*;
    use futures::future::Future;
    use document::{Operation, Insert};
    
    #[test]
    fn memory_store_push() {
        let mut sys = System::new("test");
        let store: Addr<Unsync, _> = MemoryStore::default().start();

        let push1 = store.send(Push {
            path: PathBuf::from("/foo/bar"),
            update: Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            },
        }).map(|result| {
            assert_eq!(result.unwrap(), 1);
        });

        let push2 = store.send(Push {
            path: PathBuf::from("/asdf"),
            update: Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            },
        }).map(|result| {
            assert_eq!(result.unwrap(), 1);
        });

        let push3 = store.send(Push {
            path: PathBuf::from("/asdf"),
            update: Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            },
        }).map(|result| {
            assert_eq!(result.unwrap(), 2);
        });

        sys.run_until_complete(
            push1
                .and_then(|_| push2)
                .and_then(|_| push3)
                .map_err(|err| {
                    panic!("{}", err);
                })
        ).unwrap();
    }

    #[test]
    fn memory_store_since() {
        let mut sys = System::new("test");
        let store: Addr<Unsync, _> = MemoryStore::default().start();

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
        
        let push1 = store.send(Push {
            path: PathBuf::from("/foo/bar"),
            update: a.clone(),
        });

        let push2 = store.send(Push {
            path: PathBuf::from("/foo/bar"),
            update: b.clone(),
        });

        let push3 = store.send(Push {
            path: PathBuf::from("/foo/bar"),
            update: c.clone(),
        });

        let since0 = store.send(Since {
            path: PathBuf::from("/foo/bar"),
            seq: 0
        }).and_then(|result| {
            result.unwrap().collect().map_err(|err| {
                panic!("{}", err);
            })
        }).map(|updates| {
            assert_eq!(updates, vec![a.clone(), b.clone(), c.clone()]);
        });

        let since1 = store.send(Since {
            path: PathBuf::from("/foo/bar"),
            seq: 1
        }).and_then(|result| {
            result.unwrap().collect().map_err(|err| {
                panic!("{}", err);
            })
        }).map(|updates| {
            assert_eq!(updates, vec![b.clone(), c.clone()]);
        });

        let since2 = store.send(Since {
            path: PathBuf::from("/foo/bar"),
            seq: 2
        }).and_then(|result| {
            result.unwrap().collect().map_err(|err| {
                panic!("{}", err);
            })
        }).map(|updates| {
            assert_eq!(updates, vec![c.clone()]);
        });

        let since3 = store.send(Since {
            path: PathBuf::from("/foo/bar"),
            seq: 3
        }).and_then(|result| {
            result.unwrap().collect().map_err(|err| {
                panic!("{}", err);
            })
        }).map(|updates| {
            // requesting the last sequence number is valid, but would
            // return an empty result
            assert_eq!(updates, vec![]);
        });

        let since4 = store.send(Since {
            path: PathBuf::from("/foo/bar"),
            seq: 4
        }).map(|result| {
            // requesting updates since a sequence number not in the store
            // is invalid, however
            match result {
                Err(StoreError::InvalidSequenceNumber) => (),
                _ => assert!(false),
            }
        });

        sys.run_until_complete(
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
        ).unwrap();
    }

    #[test]
    fn memory_store_seq() {
        let mut sys = System::new("test");
        let store: Addr<Unsync, _> = MemoryStore::default().start();

        let push1 = store.send(Push {
            path: PathBuf::from("/foo/bar"),
            update: Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            },
        });

        let push2 = store.send(Push {
            path: PathBuf::from("/asdf"),
            update: Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            },
        });

        let push3 = store.send(Push {
            path: PathBuf::from("/asdf"),
            update: Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            },
        });

        let seq1 = store.send(Seq {
            path: PathBuf::from("/foo/bar")
        }).map(|result| {
            assert_eq!(result, Ok(1));
        });

        let seq2 = store.send(Seq {
            path: PathBuf::from("/asdf")
        }).map(|result| {
            assert_eq!(result, Ok(2));
        });

        let seq3 = store.send(Seq {
            path: PathBuf::from("/not_found")
        }).map(|result| {
            // requesting a non-existing path is an error
            assert_eq!(result, Err(StoreError::NotFound));
        });
        
        sys.run_until_complete(
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
        ).unwrap();
    }

    #[test]
    fn memory_store_content() {
        let mut sys = System::new("test");
        let store: Addr<Unsync, _> = MemoryStore::default().start();

        let push1 = store.send(Push {
            path: PathBuf::from("/asdf"),
            update: Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            },
        });

        let push2 = store.send(Push {
            path: PathBuf::from("/asdf"),
            update: Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            },
        });

        let content1 = store.send(Content {
            path: PathBuf::from("/asdf")
        }).map(|result| {
            assert_eq!(
                result,
                Ok((2, Document::from("Hello, world")))
            );
        });

        let content2 = store.send(Content {
            path: PathBuf::from("/missing")
        }).map(|result| {
            assert_eq!(
                result,
                Err(StoreError::NotFound)
            );
        });

        sys.run_until_complete(
            push1
                .and_then(|_| push2)
                .and_then(|_| content1.join(content2))
                .map_err(|err| {
                    panic!("{}", err);
                })
        ).unwrap();
    }

    #[test]
    fn memory_store_content_at() {
                let mut sys = System::new("test");
        let store: Addr<Unsync, _> = MemoryStore::default().start();

        let push1 = store.send(Push {
            path: PathBuf::from("/asdf"),
            update: Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            },
        });

        let push2 = store.send(Push {
            path: PathBuf::from("/asdf"),
            update: Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            },
        });

        let content0 = store.send(ContentAt {
            path: PathBuf::from("/asdf"),
            seq: 0
        }).map(|result| {
            assert_eq!(
                result,
                Ok(Document::from(""))
            );
        });

        let content1 = store.send(ContentAt {
            path: PathBuf::from("/asdf"),
            seq: 1
        }).map(|result| {
            assert_eq!(
                result,
                Ok(Document::from("Hello"))
            );
        });

        let content2 = store.send(ContentAt {
            path: PathBuf::from("/asdf"),
            seq: 2
        }).map(|result| {
            assert_eq!(
                result,
                Ok(Document::from("Hello, world"))
            );
        });
        
        let content3 = store.send(ContentAt {
            path: PathBuf::from("/asdf"),
            seq: 3
        }).map(|result| {
            // requesting a sequence number higher than the number of
            // updates return error
            assert_eq!(
                result,
                Err(StoreError::InvalidSequenceNumber)
            );
        });

        let content4 = store.send(ContentAt {
            path: PathBuf::from("/missing"),
            seq: 0
        }).map(|result| {
            // requesting a missing document is an error
            assert_eq!(
                result,
                Err(StoreError::NotFound)
            );
        });

        sys.run_until_complete(
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
        ).unwrap();
    }

}
