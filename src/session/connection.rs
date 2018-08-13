//! handles message serialization / deserialization for an underlying
//! WebSocket

use session::message::{ServerMessage, ClientMessage};
use websocket::{self, WebSocket};
use futures::stream::Stream;
use futures::sink::Sink;
use futures::{Async, Poll, StartSend, AsyncSink};
use tungstenite;
use serde_json;

use std::fmt::{self, Display};
use std::error::Error;


#[derive(Debug)]
pub enum ConnectionError {
    /// Failed to serialize or deserialize message
    InvalidMessage { reason: String },
    /// Errors from underlying protocol
    Communication { error: Box<Error> },
}

impl From<tungstenite::Error> for ConnectionError {
    fn from(err: tungstenite::Error) -> Self {
        ConnectionError::Communication {
            error: Box::new(err)
        }
    }
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Connection Error: {:?}", self)
    }
}

impl Error for ConnectionError {}


pub struct WebSocketConnection {
    websocket: WebSocket,
}

impl From<WebSocket> for WebSocketConnection {
    fn from(websocket: WebSocket) -> Self {
        Self { websocket }
    }
}

impl Stream for WebSocketConnection {
    type Item = ClientMessage;
    type Error = ConnectionError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.websocket.poll() {
            Ok(Async::Ready(Some(data))) => {
                match data.to_str() {
                    Ok(raw) => {
                        serde_json::from_str(raw)
                            .map(Async::Ready)
                            .map_err(|err| ConnectionError::InvalidMessage {
                                reason: format!("{}", err),
                            })
                    },
                    Err(_err) => {
                        Err(ConnectionError::InvalidMessage {
                            reason: String::from("Not a websocket text message")
                        })
                    }
                }
            },
            Ok(Async::Ready(None)) => Ok(Async::Ready(None)), // connection closed
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => Err(ConnectionError::from(err)),
        }
    }
}

impl Sink for WebSocketConnection {
    type SinkItem = ServerMessage;
    type SinkError = ConnectionError;

    fn start_send(&mut self, item: Self::SinkItem) ->
        StartSend<Self::SinkItem, Self::SinkError>
    {
        // Converts EditSession Message to JSON string inside a
        // WebSocket text message wrapper
        match serde_json::to_string(&item) {
            Ok(data) => {
                let msg = websocket::Message::text(data);
                self.websocket.start_send(msg)
                    .map(|x| {
                        if x.is_ready() {
                            AsyncSink::Ready
                        } else {
                            AsyncSink::NotReady(item)
                        }
                    })
                    .map_err(ConnectionError::from)
            },
            Err(err) => {
                Err(ConnectionError::InvalidMessage {
                    reason: format!("{}", err)
                })
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.websocket.poll_complete().map_err(ConnectionError::from)
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.websocket.close().map_err(ConnectionError::from)
    }
}
