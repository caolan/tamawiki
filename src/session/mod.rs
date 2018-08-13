//! Co-ordinates client store updates and notifications
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};

use document::ParticipantId;

pub mod connection;
pub mod message;


/// Holds all active EditSessions
#[derive(Default)]
pub struct EditSessionManager {
    sessions: HashMap<PathBuf, Arc<Mutex<EditSession>>>
}

impl EditSessionManager {
    /// Join an existing or new EditSession for the given path. A new
    /// EditSession is created automatically when the path has no
    /// active participants.
    pub fn join(&mut self, path: &Path) -> ParticipantId {
        let session = self.sessions.entry(PathBuf::from(path))
            .or_insert_with(move || Arc::new(Mutex::new(EditSession {
                next_id: 0,
            })));
        
        session.lock().unwrap().join()
    }
}

#[derive(Default)]
struct EditSession {
    next_id: ParticipantId,
}

impl EditSession {
    fn join(&mut self) -> ParticipantId {
        self.next_id += 1;
        self.next_id
    }
} 
