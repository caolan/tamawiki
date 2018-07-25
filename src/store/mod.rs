//! Persists document updates using paths as keys. The current state
//! of a document at a point in time is calculated by the accumulated
//! application of updates within the store up until that point.

use document::{Document, Update};
use std::path::{Path, PathBuf};

pub mod memory;


/// The Store trait must be implemented by any storage backends and
/// describes the minimal API required by TamaWiki. Default functions
/// are provided for content() and content_at() but they replay all
/// update events from seq=0, so should be replaced with more
/// performant versions specific to the storeage backend.
pub trait Store {
    /// The Iterator returned by since()
    type Iter: Iterator<Item=Update>;

    /// Adds a new Update to the document at 'path' and increments the
    /// sequence number. If the document does not exist, the act of
    /// pushing an update creates it.
    fn push(&mut self, path: PathBuf, update: Update) -> Result<usize, StoreError>;

    /// Returns an Iterator over the Updates starting *after* the
    /// provided sequence number. Requesting the current sequence
    /// number is not an error, but will return an empty Iterator.
    /// Requesting updates since a sequence number that does not exist
    /// yet is a StoreError::InvalidSequenceNumber.
    fn since(&self, path: &Path, seq: usize) -> Result<Self::Iter, StoreError>;

    /// Returns the current sequence number for the document at
    /// 'path', or StoreError::NotFound if it does not exist.
    fn seq(&self, path: &Path) -> Result<usize, StoreError>;

    /// Returns a snapshot of the document's content at a specific
    /// sequence number. All updates from 0 to 'seq' will be applied.
    /// Results in a StoreError::NotFound if the document does not
    /// exist, and StoreError::InvalidSequenceNumber if the sequence
    /// number does not exist.
    fn content_at(&self, path: &Path, seq: usize) -> Result<Document, StoreError> {
        let mut doc: Document = Default::default();
        if seq > self.seq(&path)? {
            return Err(StoreError::InvalidSequenceNumber);
        }
        for update in self.since(&path, 0)?.into_iter().take(seq) {
            if let Err(_) = doc.apply(&update) {
                return Err(StoreError::InvalidDocument);
            }
        }
        Ok(doc)
    }

    /// Returns the current sequence number and content for the
    /// document (with all updates applied), or StoreError::NotFound
    /// if the document does not exist.
    fn content(&self, path: &Path) -> Result<(usize, Document), StoreError> {
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
