//! A store persists document updates using paths as keys. Every store
//! implementation must define an Actor capable of handling the
//! messages in this module.
//!
//! A document's content is calculated by the accumulated application
//! of updates within the store up until a given point in time. A
//! store implementation may checkpoint or cache these update
//! applications in order to speed up this process.

use document::{Document, Update};
use std::path::PathBuf;
use futures::stream::Stream;
use std::fmt::{self, Display};
use std::error::Error;
use actix::Message;

pub mod memory;


/// Adds a new Update to the document at 'path' and increments the
/// sequence number. If the document does not exist, the act of
/// pushing an update creates it.
pub struct Push {
    path: PathBuf,
    update: Update,
}
impl Message for Push {
    type Result = Result<usize, StoreError>;
}

/// Requests the current sequence number for the document at 'path',
/// or StoreError::NotFound if it does not exist.
pub struct Seq {
    path: PathBuf,
}
impl Message for Seq {
    type Result = Result<usize, StoreError>;
}

/// Requests a Stream of Updates starting *after* the provided
/// sequence number. Requesting the current sequence number is not an
/// error, but will return an empty Stream. Requesting updates since a
/// sequence number that does not exist yet is a
/// StoreError::InvalidSequenceNumber.
pub struct Since {
    path: PathBuf,
    seq: usize,
}
impl Message for Since {
    type Result = Result<Box<Stream<Item=Update, Error=StoreError>>, StoreError>;
}

/// Requests the current sequence number and content for the document
/// (with all updates applied), or StoreError::NotFound if the
/// document does not exist.
pub struct Content {
    path: PathBuf,
}
impl Message for Content {
    type Result = Result<(usize, Document), StoreError>;
}

/// Requests a snapshot of the document's content at a specific
/// sequence number. All updates from 0 to 'seq' will be applied.
/// Results in a StoreError::NotFound if the document does not exist,
/// and StoreError::InvalidSequenceNumber if the sequence number does
/// not exist.
pub struct ContentAt {
    path: PathBuf,
    seq: usize,
}
impl Message for ContentAt {
    type Result = Result<Document, StoreError>;
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

impl Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StoreError::NotFound => 
                write!(f, "NotFound"),
            StoreError::InvalidSequenceNumber =>
                write!(f, "InvalidSequenceNumber"),
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
            StoreError::InvalidSequenceNumber =>
                "StoreError: sequence number has no matching update",
            StoreError::InvalidDocument =>
                "StoreError: failed to build document content",
            StoreError::ConnectionError =>
                "StoreError: connection error",
        }
    }
}
