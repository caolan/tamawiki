//! Defines messages for client/server communication during an EditSession

use document::{Operation, ParticipantId, Event};
use futures::future::FutureResult;
use futures::sink::Sink;
use futures::stream::Stream;
use serde_json;
use std::error::Error;
use std::fmt::{self, Debug, Display};
use store::{SequenceId, StoreError};

/// Message sent from the server to the client
#[derive(Serialize, Debug, PartialEq)]
pub enum ServerMessage {
    /// Client successfully connected - always the first message sent
    /// to a client
    Connected(ConnectedMessage),
    /// A new event from the server
    Event(ServerEventMessage),
}

#[derive(Serialize, Debug, PartialEq)]
pub struct ServerEventMessage {
    /// The most recently applied client SequenceId
    pub client_seq: SequenceId,
    /// The SequenceId of the Event in the Store
    pub seq: SequenceId,
    /// The event
    pub event: Event,
}

/// Client successfully connected
#[derive(Serialize, Debug, PartialEq)]
pub struct ConnectedMessage {
    /// The new client's participant ID
    pub id: ParticipantId,
}

/// Message sent from the client to the server
#[derive(Deserialize, Debug, PartialEq)]
pub enum ClientMessage {
    /// A change was made to the document content
    ClientEdit(ClientEditMessage),
}

/// A change made to the document content by the client
#[derive(Deserialize, Debug, PartialEq)]
pub struct ClientEditMessage {
    /// The most recently applied server SequenceId before this client
    /// edit was made
    pub parent_seq: SequenceId,
    /// The client's own local SequenceId for this event
    pub client_seq: SequenceId,
    /// The operations which describe the change to the document
    pub operations: Vec<Operation>,
}

/// An error when attempting to read Events from a Stream, or write
/// Events to a Sink.
#[derive(Debug)]
pub enum MessageStreamError {
    /// Failed to serialize or deserialize message
    InvalidMessage {
        /// Detailed information on the error if available
        reason: String,
    },
    /// Errors from underlying protocol or event store
    Transport {
        /// The original error
        error: Box<Debug + Send>,
    },
}

impl Display for MessageStreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Message Error: {:?}", self)
    }
}

impl Error for MessageStreamError {}

impl From<StoreError> for MessageStreamError {
    fn from(err: StoreError) -> Self {
        MessageStreamError::Transport {
            error: Box::new(err),
        }
    }
}

/// Wraps an Sink + Stream which handles Strings so it can read
/// ClientMessages and write ServerMessages.
pub fn message_stream<T, E>(
    stream: T,
) -> impl Stream<Item = ClientMessage, Error = MessageStreamError>
         + Sink<SinkItem = ServerMessage, SinkError = MessageStreamError>
where
    E: Debug + Send + 'static,
    T: Stream<Item = String, Error = E> + Sink<SinkItem = String, SinkError = E>,
{
    stream
        .map_err(|err| MessageStreamError::Transport {
            error: Box::new(err),
        }).and_then(|text| {
            FutureResult::from(serde_json::from_str(&text).map_err(|err| {
                MessageStreamError::InvalidMessage {
                    reason: format!("{}", err),
                }
            }))
        }).sink_map_err(|err| MessageStreamError::Transport {
            error: Box::new(err),
        }).with(|msg| {
            FutureResult::from(serde_json::to_string(&msg).map_err(|err| {
                MessageStreamError::InvalidMessage {
                    reason: format!("{}", err),
                }
            }))
        })
}

// #[cfg(test)]
// mod tests {
//     extern crate tokio;

//     use super::*;
//     use futures::sync::mpsc::channel;
//     use futures::future::Future;
//     use document::{Operation, Insert};
//     use self::tokio::runtime::current_thread::Runtime;

//     #[test]
//     fn test_message_stream() {
//         let mut rt = Runtime::new().expect("new test runtime");
//         let (tx, rx) = channel(10);
//         let rx = message_stream(rx);
//         rt.block_on(
//             tx.send(String::from("{\"ClientEdit\":{\"seq\":0,\"client_seq\":1,\"operations\":[{\"Insert\":{\"pos\":0,\"content\":\"test\"}}]}}"))
//                 .map_err(|err| panic!(err))
//                 .and_then(|_| {
//                     rx.into_future().map(|result| {
//                         match result {
//                             (Some(msg), _rx) => {
//                                 assert_eq!(msg, ClientMessage::ClientEdit(ClientEditMessage {
//                                     seq: 0,
//                                     client_seq: 1,
//                                     operations: vec![
//                                         Operation::Insert(Insert {
//                                             pos: 0,
//                                             content: String::from("test")
//                                         })
//                                     ]
//                                 }))
//                             },
//                             (None, _rx) => panic!("Expected ClientMessage"),
//                         }
//                     }).map_err(|err| panic!(err))
//                 })
//         ).unwrap();
//     }

//     #[test]
//     fn test_message_sink() {
//         let mut rt = Runtime::new().expect("new test runtime");
//         let (tx, rx) = channel(10);
//         let tx = message_sink(tx);
//         rt.block_on(
//             tx.send(ServerMessage::Connected(ConnectedMessage {id: 123}))
//                 .map_err(|err| panic!(err))
//                 .and_then(|_| {
//                     rx.into_future().map(|result| {
//                         match result {
//                             (Some(msg), _rx) => {
//                                 assert_eq!(msg, "{\"Connected\":{\"id\":123}}")
//                             },
//                             (None, _rx) => panic!("Expected ClientMessage"),
//                     }
//                     }).map_err(|err| panic!(err))
//                 })
//         ).unwrap();
//     }
// }
