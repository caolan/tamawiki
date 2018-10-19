//! Co-ordinates store updates and notifications
use futures::future::{self, Future};
use futures::stream::Stream;
use futures::task::Task;
use std::collections::HashMap;
use std::sync::{Weak, Arc, Mutex};
use std::path::{Path, PathBuf};

use store::{Store, StoreError, SequenceId};
use document::{Event, Edit, Join, Leave, ParticipantId};

pub mod participant;
pub mod message;

use self::participant::Participant;


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
    pub fn join(&self, path: &Path, start_seq: SequenceId) ->
    impl Future<Item=Participant<T>, Error=StoreError>
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
                            next_id: Default::default(),
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
        session.join(start_seq)
    }
}

/// Co-ordinates store updates and notifications for (potentially)
/// multiple clients editing a single document.
// This is a cloneable interface to DocumentSessionData.
#[derive(Clone)]
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
struct DocumentSessionData<T: Store + Sync> {
    store: T,
    path: PathBuf,
    next_id: ParticipantId,
    // The sequence id of the last event received during this session,
    // or None if no events have been received yet.
    last_seq: Option<SequenceId>,
    // Participant event streams parked until a new sequence ID is ready.
    waiting_tasks: Vec<Task>,
}

impl<T: Store + Sync> DocumentSession<T> {
    // this must only be called via DocumentSessionManager
    fn join(&mut self, start_seq: SequenceId) ->
    impl Future<Item=Participant<T>, Error=StoreError>
    {
        let id = {
            let mut data = self.data.lock().unwrap();
            data.next_id += 1;
            println!("Participant {} joined {:?}", data.next_id, data.path);
            data.next_id
        };
        let s2 = self.clone();
        let event = Event::Join(Join { id });
        
        self.write(event).map(move |_seq| {
            Participant::new(s2, id, start_seq)
        })
    }

    // Notifies the other clients that a participant has left
    fn leave(&self, id: ParticipantId) ->
    impl Future<Item=SequenceId, Error=StoreError>
    {
        {
            let data = self.data.lock().unwrap();
            println!("Participant {} left {:?}", id, data.path);
        };
        let event = Event::Leave(Leave { id });
        self.write(event)
    }

    // Wakes up the task once a new sequence id is available.
    fn notify_on_event(&self, task: Task) {
        let mut data = self.data.lock().unwrap();
        data.waiting_tasks.push(task);
    }

    // Returns a weak reference to the session data. Used by
    // DocumentSessionManager to reference active sessions without
    // preventing them from being dropped.
    fn downgrade(&self) -> WeakDocumentSession<T> {
        WeakDocumentSession {
            data: Arc::downgrade(&self.data)
        }
    }

    // Updates last_seq and notifies Participants waiting for events
    fn update_seq(&self, seq: SequenceId) {
        let mut data = self.data.lock().unwrap();
        data.last_seq = Some(seq);
        for t in data.waiting_tasks.drain(..) {
            t.notify();
        }
    }

    // Writes an event to the store.
    fn write(&self, event: Event) -> impl Future<Item=SequenceId, Error=StoreError> {
        let mut data = self.data.lock().unwrap();
        let path = data.path.clone();
        let s2 = self.clone();
        data.store.push(path, event).inspect(move |seq| {
            s2.update_seq(*seq);
        })
    }

    // First, transforms an event to accomodate concurrent events
    // already in the store, then, writes the resulting transformed
    // event to the store.
    fn write_transformed(&self, sender: ParticipantId, parent_seq: SequenceId, event: Event) ->
    impl Future<Item=SequenceId, Error=StoreError> {
        let concurrent_events = {
            let mut data = self.data.lock().unwrap();
            data.store.since(&data.path.as_path(), parent_seq)
        };
        let transformed = concurrent_events.and_then(move |stream| {
            stream.fold(event, move |mut event, (_seq, concurrent)| {
                let from = match concurrent {
                    Event::Edit(Edit {author, ..}) => author,
                    Event::Join(Join {id}) => id,
                    Event::Leave(Leave {id}) => id,
                };
                // TODO: update store.since query to exclude events
                // from sender at source instead of filtering here
                if from != sender {
                    event.transform(&concurrent);
                }
                future::ok(event)
            })
        });
        let s2 = self.clone();
        transformed.and_then(move |event| s2.write(event))
    }
}

impl<T: Store + Sync> WeakDocumentSession<T> {
    fn upgrade(&self) -> Option<DocumentSession<T>> {
        self.data.upgrade().map(|data| DocumentSession { data })
    }
}
