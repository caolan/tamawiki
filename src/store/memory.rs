//! An in-memory store, useful for testing.
use std::sync::{Arc, RwLock};
use futures::stream::Stream;
use futures::future::{self, Future};
use futures::{Poll, Async};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use super::{Store, StoreError, SequenceId};
use document::{Document, Event, Edit, Operation, Insert};


type Events = Arc<RwLock<Vec<Event>>>;
type Documents = HashMap<PathBuf, Events>;

/// Holds document data in shared memory location
#[derive(Default, Clone, Debug)]
pub struct MemoryStore {
    documents: Arc<RwLock<Documents>>
}

impl Store for MemoryStore {
    type Stream = MemoryStoreStream;
    type SinceFuture = Box<Future<Item=Self::Stream, Error=StoreError> + Send>;
    type PushFuture = Box<Future<Item=SequenceId, Error=StoreError> + Send>;
    
    fn push(&mut self, path: PathBuf, event: Event) -> Self::PushFuture {
        let mut documents = match self.documents.write() {
            Ok(documents) => documents,
            Err(_) => return Box::new(
                future::err(StoreError::ConnectionError)),
        };

        let events = documents.entry(path)
            .or_insert_with(|| Arc::new(RwLock::new(Vec::new())));
        
        let mut events = match events.write() {
            Ok(events) => events,
            Err(_) => return Box::new(
                future::err(StoreError::ConnectionError)),
        };
        
        events.push(event);
        Box::new(
            future::ok(events.len() as u64)
        )
    }

    fn seq(&self, path: &Path) ->
        Box<Future<Item=SequenceId, Error=StoreError> + Send>
    {
        let documents = match self.documents.read() {
            Ok(documents) => documents,
            Err(_) => return Box::new(
                future::err(StoreError::ConnectionError)),
        };
        
        let events = match documents.get(path) {
            Some(events) => match events.read() {
                Ok(events) => events,
                Err(_) => return Box::new(
                    future::err(StoreError::ConnectionError)),
            },
            None => return Box::new(
                future::err(StoreError::NotFound)),
        };
        
        Box::new(
            future::ok(events.len() as u64)
        )
    }

    fn since(&self, path: &Path, seq: SequenceId) -> Self::SinceFuture {
        let documents = match self.documents.read() {
            Ok(documents) => documents,
            Err(_) => return Box::new(
                future::err(StoreError::ConnectionError)),
        };
        
        let events = match documents.get(path) {
            Some(events) => events.clone(),
            None => return Box::new(
                future::err(StoreError::NotFound)),
        };

        // check sequence id is valid
        match events.read() {
            Ok(events) => if seq > events.len() as u64 {
                return Box::new(
                    future::err(StoreError::InvalidSequenceId))
            },
            Err(_) => return Box::new(
                future::err(StoreError::ConnectionError)),
        }
        
        Box::new(
            future::ok(Self::Stream { events, seq })
        )
    }

    fn content(&self, path: &Path) ->
        Box<Future<Item=(SequenceId, Document), Error=StoreError> + Send>
    {
        Box::new(
            self.since(path, 0).and_then(|stream| {
                stream.fold(
                    (0, Document::default()),
                    |(_seq, mut doc), (seq, ref event)| {
                        match doc.apply(event) {
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
                |mut doc, (_seq, ref event)| {
                    match doc.apply(event) {
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

impl From<HashMap<String, String>> for MemoryStore {
    fn from(data: HashMap<String, String>) -> Self {
        let mut documents: Documents = Default::default();
        for (k, v) in data {
            documents.insert(
                PathBuf::from(k),
                Arc::new(RwLock::new(vec![Event::Edit(Edit {
                    author: 1,
                    operations: vec![
                        Operation::Insert(Insert {
                            pos: 0,
                            content: v
                        })
                    ]
                })]))
            );
        }
        MemoryStore {
            documents: Arc::new(RwLock::new(documents))
        }
    }
}

/// Convenient way to create a new MemoryStore with existing content.
///
/// Each document defined using this macro will consist of a single
/// Insert Operation which inserts the provided content.
///
/// # Example
///
/// ```
/// #[macro_use] extern crate tamawiki;
///
/// use tamawiki::store::memory::MemoryStore;
///
/// let store = memorystore! {
///     "example/index.html" => "My Example Page",
///     "example/blah.html" => "blah blah blah"
/// };
/// ```
#[macro_export]
macro_rules! memorystore {
    { $($path:expr => $content:expr),* } => {
        {
            let mut docs = std::collections::HashMap::<String, String>::new();
            $(
                docs.insert(String::from($path), String::from($content));
            )*
            MemoryStore::from(docs)
        }
    };
}

/// An asynchronous stream of Event objects cloned from memory
pub struct MemoryStoreStream {
    events: Events,
    seq: SequenceId,
}

impl Stream for MemoryStoreStream {
    type Item = (SequenceId, Event);
    type Error = StoreError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.events.read() {
            Ok(events) => {
                self.seq += 1;
                if self.seq <= events.len() as u64 {
                    Ok(Async::Ready(Some(
                        (self.seq, events[(self.seq - 1) as usize].clone())
                    )))
                } else {
                    Ok(Async::Ready(None))
                }
            },
            Err(_) => Err(StoreError::ConnectionError),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use document::{DocumentParticipant, Operation, Insert, Join};
    use std::collections::HashSet;
    
    #[test]
    fn memory_store_push() {
        let mut store = MemoryStore::default();

        let push1 = store.push(
            PathBuf::from("/foo/bar"),
            Event::Edit(Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            })
        );

        let push2 = store.push(
            PathBuf::from("/asdf"),
            Event::Edit(Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            })
        );

        let push3 = store.push(
            PathBuf::from("/asdf"),
            Event::Edit(Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            })
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
        let mut store = MemoryStore::default();

        let a = Event::Edit(Edit {
            author: 1,
            operations: vec![
                Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("Hello")
                })
            ]
        });

        let b = Event::Edit(Edit {
            author: 1,
            operations: vec![
                Operation::Insert(Insert {
                    pos: 5,
                    content: String::from(", world")
                })
            ]
        });

        let c = Event::Edit(Edit {
            author: 1,
            operations: vec![
                Operation::Insert(Insert {
                    pos: 12,
                    content: String::from("!")
                })
            ]
        });
        
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
        }).map(|events| {
            assert_eq!(events, vec![(1, a0), (2, b0), (3, c0)]);
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
        }).map(|events| {
            assert_eq!(events, vec![(2, b1), (3, c1)]);
        });
            
        let c2 = c.clone();
        let since2 = store.since(
            &PathBuf::from("/foo/bar").as_path(),
            2
        ).and_then(|stream| {
            stream.collect().map_err(|err| {
                panic!("{}", err);
            })
        }).map(|events| {
            assert_eq!(events, vec![(3, c2)]);
        });

        let since3 = store.since(
            &PathBuf::from("/foo/bar").as_path(),
            3
        ).and_then(|stream| {
            stream.collect().map_err(|err| {
                panic!("{}", err);
            })
        }).map(|events| {
            // requesting the last sequence number is valid, but would
            // return an empty result
            assert_eq!(events, vec![]);
        });

        let since4 = store.since(
            &PathBuf::from("/foo/bar").as_path(),
            4
        ).map(|_| {
            // requesting events since a sequence id not in the store
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
        let mut store = MemoryStore::default();

        let push1 = store.push(
            PathBuf::from("/foo/bar"),
            Event::Edit(Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            })
        );

        let push2 = store.push(
            PathBuf::from("/asdf"),
            Event::Edit(Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            })
        );

        let push3 = store.push(
            PathBuf::from("/asdf"),
            Event::Edit(Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            })
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
        let mut store = MemoryStore::default();
        // later we'll test that the cloned store has the same data
        let store_clone = store.clone();

        let push1 = store.push(
            PathBuf::from("/asdf"),
            Event::Join(Join {id: 1})
        );

        let push2 = store.push(
            PathBuf::from("/asdf"),
            Event::Edit(Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            })
        );

        let push3 = store.push(
            PathBuf::from("/asdf"),
            Event::Edit(Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            })
        );

        let content1 = store.content(
            &PathBuf::from("/asdf").as_path()
        ).map(|(seq, doc)| {
            assert_eq!(seq, 3);
            assert_eq!(doc, Document {
                content: String::from("Hello, world"),
                participants: vec![DocumentParticipant {id: 1}].into_iter().collect(),
            });
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

        // check cloned store has same data
        let content3 = store_clone.content(
            &PathBuf::from("/asdf").as_path()
        ).map(|(seq, doc)| {
            assert_eq!(seq, 3);
            assert_eq!(doc, Document {
                content: String::from("Hello, world"),
                participants: vec![DocumentParticipant {id: 1}].into_iter().collect(),
            });
        });

        push1
            .and_then(|_| push2)
            .and_then(|_| push3)
            .and_then(|_| content1.join3(content2, content3))
            .map_err(|err| {
                panic!("{}", err);
            })
            .wait().unwrap();
    }

    #[test]
    fn memory_store_content_at() {
        let mut store = MemoryStore::default();

        let push1 = store.push(
            PathBuf::from("/asdf"),
            Event::Join(Join {id: 1})
        );

        let push2 = store.push(
            PathBuf::from("/asdf"),
            Event::Edit(Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("Hello")
                    })
                ]
            })
        );

        let push3 = store.push(
            PathBuf::from("/asdf"),
            Event::Edit(Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world")
                    })
                ]
            })
        );

        let content0 = store.content_at(
            &PathBuf::from("/asdf").as_path(),
            0
        ).map(|doc| {
            assert_eq!(doc, Document {
                content: String::from(""),
                participants: HashSet::new(),
            });
        });
        
        let content1 = store.content_at(
            &PathBuf::from("/asdf").as_path(),
            1
        ).map(|doc| {
            assert_eq!(doc, Document {
                content: String::from(""),
                participants: vec![DocumentParticipant {id: 1}].into_iter().collect(),
            });
        });

        let content2 = store.content_at(
            &PathBuf::from("/asdf").as_path(),
            2
        ).map(|doc| {
            assert_eq!(doc, Document {
                content: String::from("Hello"),
                participants: vec![DocumentParticipant {id: 1}].into_iter().collect(),
            });
        });

        let content3 = store.content_at(
            &PathBuf::from("/asdf").as_path(),
            3
        ).map(|doc| {
            assert_eq!(doc, Document {
                content: String::from("Hello, world"),
                participants: vec![DocumentParticipant {id: 1}].into_iter().collect(),
            });
        });
        
        let content4 = store.content_at(
            &PathBuf::from("/asdf").as_path(),
            4
        ).map(|_| {
            // requesting a sequence number higher than the number of
            // events return error
            assert!(false);
        }).or_else(|err| {
            match err {
                StoreError::InvalidSequenceId => future::ok(()),
                _ => future::err(err),
            }
        });

        let content5 = store.content_at(
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
            .and_then(|_| push3)
            .and_then(|_| {
                content0
                    .join(content1)
                    .join(content2)
                    .join(content3)
                    .join(content4)
                    .join(content5)
            })
            .map_err(|err| {
                panic!("{}", err);
            })
            .wait().unwrap();
    }

}
