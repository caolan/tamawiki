//! Persists document updates using paths as keys.
//!
//! Every store backend must implement the Store trait.
//!
//! A document's content is obtained by the accumulated application
//! of updates within the store up until a given point in time. A
//! store implementation may checkpoint or cache these update
//! applications in order to speed up this process.

use std::fmt::{self, Display};
use std::error::Error;
use std::path::{PathBuf, Path};
use futures::stream::Stream;
use futures::future::Future;

use document::{Document, Update};

pub mod memory;


/// The sequence number for an Update. The first Update for a Document
/// is SequenceId=1 (not 0). This is so requesting updates since
/// SequenceId=0 will return *all* Updates for a Document.
pub type SequenceId = u64;

/// Every store backend must provide an interface to the store using
/// the StoreClient trait, which defines the API used by TamaWiki.
pub trait StoreClient {
    type Stream: Stream<Item=(SequenceId, Update), Error=StoreError>;
    
    /// Adds a new Update to the document at 'path' and returns the
    /// new SequenceId. If the document does not exist, the act of
    /// pushing an Update creates it.
    fn push(&mut self, path: PathBuf, update: Update) ->
        Box<Future<Item=SequenceId, Error=StoreError>>;

    /// Requests the current SequenceId for the document at 'path',
    /// or StoreError::NotFound if it does not exist.
    fn seq(&self, path: &Path) ->
        Box<Future<Item=SequenceId, Error=StoreError>>;

    /// Requests a stream of Updates starting *after* the provided
    /// SequenceId. Requesting the current (head) SequenceId is not an
    /// error, but will return an empty stream. Requesting Updates
    /// since a SequenceId that does not exist yet is a
    /// StoreError::InvalidSequenceId.
    fn since(&self, path: &Path, seq: SequenceId) ->
        Box<Future<Item=Self::Stream, Error=StoreError>>;

    /// Requests the current SequenceId and content for the document
    /// at 'path' (with all updates applied), or StoreError::NotFound
    /// if the document does not exist.
    fn content(&self, path: &Path) ->
        Box<Future<Item=(SequenceId, Document), Error=StoreError>>;

    /// Requests a snapshot of the document's content at a specific
    /// SequenceId. All updates from SequenceId=1 (inclusive) to
    /// SequenceId=seq will be applied. Results in a
    /// StoreError::NotFound if the document does not exist, and
    /// StoreError::InvalidSequenceId if the SequenceId does not
    /// exist.
    fn content_at(&self, path: &Path, seq: SequenceId) ->
        Box<Future<Item=Document, Error=StoreError>>;
}

/// Error conditions for reading data from, or writing data to, the
/// store.
#[derive(Debug, PartialEq)]
pub enum StoreError {
    /// The document path is missing
    NotFound,
    /// A non-existant SequenceId was requested for the document
    InvalidSequenceId,
    /// When requesting content, the store failed to build a
    /// representation of the document from the updates in the store
    InvalidDocument,
    /// There is a problem communicating with the storage backend
    ConnectionError,
}

impl Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StoreError::NotFound => 
                write!(f, "NotFound"),
            StoreError::InvalidSequenceId =>
                write!(f, "InvalidSequenceId"),
            StoreError::InvalidDocument =>
                write!(f, "InvalidDocument"),
            StoreError::ConnectionError =>
                write!(f, "ConnectionError"),
        }
    }
}

impl Error for StoreError {
    fn description(&self) -> &str {
        match *self {
            StoreError::NotFound =>
                "StoreError: the document path is missing",
            StoreError::InvalidSequenceId =>
                "StoreError: sequence id has no matching update",
            StoreError::InvalidDocument =>
                "StoreError: failed to build document content",
            StoreError::ConnectionError =>
                "StoreError: connection error",
        }
    }
}
