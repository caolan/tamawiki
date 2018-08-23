//! Co-ordinates store updates and notifications
use futures::{Async, Poll};
use futures::future::Future;
use futures::stream::{self, Stream};
use futures::task::{self, Task};
use std::collections::HashMap;
use std::sync::{Weak, Arc, Mutex};
use std::path::{Path, PathBuf};

use store::{Store, StoreError, SequenceId};
use document::{Event, Join, Leave, ParticipantId};

pub mod connection;
pub mod message;


/// Provides access to DocumentSessions.
// This struct is a cloneable interface to DocumentSessionData.
#[derive(Clone)]
pub struct DocumentSessionManager<T: Store + Sync> {
    data: Arc<Mutex<DocumentSessionManagerData<T>>>,
}

// Holds all active DocumentSessions
struct DocumentSessionManagerData<T: Store + Sync> {
    sessions: HashMap<PathBuf, WeakDocumentSession<T>>,
    store: T
}

impl<T: Store + Sync> DocumentSessionManagerData<T> {
    fn new(store: T) -> Self {
        Self { sessions: Default::default(), store }
    }
}

impl<T: Store + Sync> DocumentSessionManager<T> {
    /// Creates a new DocumentSessionManager using the provided Store
    pub fn new(store: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(
                DocumentSessionManagerData::new(store)
            ))
        }
    }
    
    /// Join an existing or new DocumentSession for the given path. A new
    /// DocumentSession is created automatically when the path has no
    /// active participants.
    pub fn join(&self, path: &Path) ->
    impl Future<Item=(ParticipantId, DocumentSession<T>), Error=StoreError>
    {
        let mut session = {
            let mut data = self.data.lock().unwrap();
            data.sessions.get(path)
                .and_then(|s| s.upgrade())
                .or_else(|| {
                    let s = DocumentSession {
                        data: Arc::new(Mutex::new(DocumentSessionData {
                            store: data.store.clone(),
                            path: PathBuf::from(path),
                            next_id: 0,
                            last_seq: None,
                            waiting_tasks: vec![],
                        }))
                    };
                    data.sessions.insert(
                        PathBuf::from(path),
                        s.downgrade()
                    );
                    Some(s)
                })
                .unwrap()
        };
        let fut = session.join();
        fut.map(move |id| (id, session))
    }
}

/// Co-ordinates store updates and notifications for (potentially)
/// multiple clients editing a single document.
// This is a cloneable interface to DocumentSessionData.
#[derive(Clone, Default)]
pub struct DocumentSession<T: Store + Sync> {
    data: Arc<Mutex<DocumentSessionData<T>>>
}

// A weak reference to a DocumentSession. These are used by
// DocumentSessionManager to prevent keeping active old sessions with
// no participants.
struct WeakDocumentSession<T: Store + Sync> {
    data: Weak<Mutex<DocumentSessionData<T>>>
}

/// Holds information about the active session.
#[derive(Default)]
struct DocumentSessionData<T: Store + Sync> {
    store: T,
    path: PathBuf,
    next_id: ParticipantId,
    // The sequence id of the last event received during this session,
    // or None if no events have been received yet.
    last_seq: Option<SequenceId>,
    // Subscriber streams parked until a new sequence ID is ready.
    waiting_tasks: Vec<Task>,
}

impl<T: Store + Sync> DocumentSession<T> {
    fn join(&mut self) -> impl Future<Item=ParticipantId, Error=StoreError> {
        let mut data = self.data.lock().unwrap();
        data.next_id += 1;
        let id = data.next_id;
        println!("Participant {} joined {:?}", id, data.path);
        let event = Event::Join(Join { id });
        let path = data.path.clone();
        let s2 = self.clone();
        data.store.push(path, event).map(move |seq| {
            s2.update_seq(seq);
            id
        })
    }

    /// Remove a participant id from the active session and notify the
    /// other clients.
    pub fn leave(&self, id: ParticipantId) -> impl Future<Item=ParticipantId, Error=StoreError> {
        let mut data = self.data.lock().unwrap();
        println!("Participant {} left {:?}", id, data.path);
        let event = Event::Leave(Leave { id });
        let path = data.path.clone();
        let s2 = self.clone();
        data.store.push(path, event).map(move |seq| {
            s2.update_seq(seq);
            id
        })
    }

    fn update_seq(&self, seq: SequenceId) {
        let mut data = self.data.lock().unwrap();
        data.last_seq = Some(seq);
        for t in data.waiting_tasks.drain(..) {
            t.notify();
        }
    }

    fn notify_on_event(&self, task: Task) {
        let mut data = self.data.lock().unwrap();
        data.waiting_tasks.push(task);
    }

    /// Returns an infinite stream of Events starting *after* the
    /// provided SequenceId. When the current head SequenceId is
    /// reached it will wait for new Events to arrive.
    pub fn subscribe(&self, since: SequenceId) -> EventStream<T> {
        let catchup = {
            let data = self.data.lock().unwrap();
            data.store.since(&data.path.as_path(), since)
        };
        EventStream {
            session: self.clone(),
            seq: since,
            state: EventStreamState::ReadingStream,
            inner: stream::futures_unordered(vec![catchup]).flatten(),
        }
    }

    fn downgrade(&self) -> WeakDocumentSession<T> {
        WeakDocumentSession {
            data: Arc::downgrade(&self.data)
        }
    }
}

impl<T: Store + Sync> WeakDocumentSession<T> {
    fn upgrade(&self) -> Option<DocumentSession<T>> {
        self.data.upgrade().map(|data| DocumentSession { data })
    }
}

/// A stream of SequenceIds and their corresponding Events.
pub struct EventStream<T: Store + Sync> {
    inner: stream::Flatten<stream::FuturesUnordered<T::SinceFuture>>,
    session: DocumentSession<T>,
    state: EventStreamState,
    seq: SequenceId,
}

#[derive(Debug)]
enum EventStreamState {
    ReadingStream,
    WaitingForEvent,
}

impl<T: Store + Sync> Stream for EventStream<T> {
    type Item = (SequenceId, Event);
    type Error = StoreError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.state {
            EventStreamState::ReadingStream => {
                match self.inner.poll() {
                    Ok(Async::Ready(None)) => {
                        self.state = EventStreamState::WaitingForEvent;
                        self.poll()
                    },
                    Ok(Async::Ready(Some((seq, event)))) => {
                        self.seq = seq;
                        Ok(Async::Ready(Some((seq, event))))
                    },
                    other => other
                }
            },
            EventStreamState::WaitingForEvent => {
                let last_seq = {
                    let data = self.session.data.lock().unwrap();
                    data.last_seq
                };
                match last_seq {
                    Some(last_seq) if last_seq > self.seq => {
                        self.inner = {
                            let data = self.session.data.lock().unwrap();
                            stream::futures_unordered(vec![
                                data.store.since(&data.path.as_path(), self.seq)
                            ]).flatten()
                        };
                        self.state = EventStreamState::ReadingStream;
                        self.poll()
                    },
                    _ => {
                        self.session.notify_on_event(task::current());
                        Ok(Async::NotReady)
                    }
                }
            },
        }
    }
}
