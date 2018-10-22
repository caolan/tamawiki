//! Persists document events using paths as keys.
//!
//! Every store backend must implement the Store trait.
//!
//! A document's content is obtained by the accumulated application of
//! edit events within the store up until a given point in time. A
//! store implementation may checkpoint or cache these edit event
//! applications in order to speed up this process.

use futures::future::Future;
use futures::stream::Stream;
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::path::{Path, PathBuf};

use document::{Document, Event};

pub mod memory;

/// The sequence number for an Event. The first Event for a Document
/// is SequenceId=1 (not 0). This is so requesting events since
/// SequenceId=0 will return *all* Events for a Document.
pub type SequenceId = u64;

/// Every store backend must provide a client interface to the store
/// using the Store trait. This client interface will be cloned for
/// multiple requests and orchestrating concurrent access (e.g. via
/// locking and a connection pool) is left to the implementation.
pub trait Store: Clone + Debug + Send + 'static {
    /// Type for the stream of Events returned by `since()` calls
    type Stream: Stream<Item = (SequenceId, Event), Error = StoreError> + Send;
    /// The Future returned by `since()` calls
    type SinceFuture: Future<Item = Self::Stream, Error = StoreError> + Send;
    /// The Future returned by `push()` calls
    type PushFuture: Future<Item = SequenceId, Error = StoreError> + Send;

    /// Adds a new Event to the document at 'path' and returns the
    /// new SequenceId. If the document does not exist, the act of
    /// pushing an Event creates it.
    fn push(&mut self, path: PathBuf, event: Event) -> Self::PushFuture;

    /// Requests the current SequenceId for the document at 'path',
    /// or StoreError::NotFound if it does not exist.
    fn seq(&self, path: &Path) -> Box<Future<Item = SequenceId, Error = StoreError> + Send>;

    /// Requests a stream of Events starting *after* the provided
    /// SequenceId. Requesting the current (head) SequenceId is not an
    /// error, but will return an empty stream. Requesting Events
    /// since a SequenceId that does not exist yet is a
    /// StoreError::InvalidSequenceId.
    fn since(&self, path: &Path, seq: SequenceId) -> Self::SinceFuture;

    /// Requests the current SequenceId and content for the document
    /// at 'path' (with all events applied), or StoreError::NotFound
    /// if the document does not exist.
    fn content(
        &self,
        path: &Path,
    ) -> Box<Future<Item = (SequenceId, Document), Error = StoreError> + Send>;

    /// Requests a snapshot of the document's content at a specific
    /// SequenceId. All events from SequenceId=1 (inclusive) to
    /// SequenceId=seq will be applied. Results in a
    /// StoreError::NotFound if the document does not exist, and
    /// StoreError::InvalidSequenceId if the SequenceId does not
    /// exist.
    fn content_at(
        &self,
        path: &Path,
        seq: SequenceId,
    ) -> Box<Future<Item = Document, Error = StoreError> + Send>;
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
    /// representation of the document from the events in the store
    InvalidDocument,
    /// There is a problem communicating with the storage backend
    ConnectionError,
}

impl Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StoreError::NotFound => write!(f, "NotFound"),
            StoreError::InvalidSequenceId => write!(f, "InvalidSequenceId"),
            StoreError::InvalidDocument => write!(f, "InvalidDocument"),
            StoreError::ConnectionError => write!(f, "ConnectionError"),
        }
    }
}

impl Error for StoreError {
    fn description(&self) -> &str {
        match *self {
            StoreError::NotFound => "StoreError: the document path is missing",
            StoreError::InvalidSequenceId => "StoreError: sequence id has no matching event",
            StoreError::InvalidDocument => "StoreError: failed to build document content",
            StoreError::ConnectionError => "StoreError: connection error",
        }
    }
}
