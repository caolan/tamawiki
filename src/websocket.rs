//! Tungstenite WebSocket wrapper for use with Hyper
//!
//! Most of this code is lifted directly from warp's (v0.1.0) ws
//! module with some small modifications and extensions.
//! https://github.com/seanmonstar/warp
//!
//!
//! Copyright (c) 2018 Sean McArthur
//! 
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//! 
//! The above copyright notice and this permission notice shall be included in
//! all copies or substantial portions of the Software.
//! 
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
//! THE SOFTWARE.

use tungstenite::protocol;
use futures::{Async, AsyncSink, Poll, Sink, StartSend, Stream};
use std::io::ErrorKind::WouldBlock;
use hyper::upgrade::Upgraded;
use std::fmt;

// re-export tungstenite Message
pub use tungstenite::protocol::Message;


pub struct WebSocket {
    inner: protocol::WebSocket<Upgraded>,
}

impl From<protocol::WebSocket<Upgraded>> for WebSocket {
    fn from(ws: protocol::WebSocket<Upgraded>) -> Self {
        Self {inner: ws}
    }
}

impl Stream for WebSocket {
    type Item = Message;
    type Error = ::tungstenite::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.inner.read_message() {
            Ok(item) => Ok(Async::Ready(Some(item))),
            Err(::tungstenite::Error::Io(ref err)) if err.kind() == WouldBlock => {
                Ok(Async::NotReady)
            },
            Err(::tungstenite::Error::ConnectionClosed(_frame)) => {
                Ok(Async::Ready(None))
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

impl Sink for WebSocket {
    type SinkItem = Message;
    type SinkError = ::tungstenite::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        match self.inner.write_message(item) {
            Ok(()) => Ok(AsyncSink::Ready),
            Err(::tungstenite::Error::SendQueueFull(inner)) => {
                Ok(AsyncSink::NotReady(inner))
            },
            Err(::tungstenite::Error::Io(ref err)) if err.kind() == WouldBlock => {
                // the message was accepted and partly written, so this
                // isn't an error.
                Ok(AsyncSink::Ready)
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        match self.inner.write_pending() {
            Ok(()) => Ok(Async::Ready(())),
            Err(::tungstenite::Error::Io(ref err)) if err.kind() == WouldBlock => {
                Ok(Async::NotReady)
            },
            Err(err) => {
                Err(err)
            }
        }
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        match self.inner.close(None) {
            Ok(()) => Ok(Async::Ready(())),
            Err(::tungstenite::Error::Io(ref err)) if err.kind() == WouldBlock => {
                Ok(Async::NotReady)
            },
            Err(err) => {
                Err(err)
            }
        }
    }
}

impl fmt::Debug for WebSocket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WebSocket")
            .finish()
    }
}

pub fn websocket_text(ws: WebSocket) ->
impl Stream<Item=String, Error=::tungstenite::Error>
    + Sink<SinkItem=String, SinkError=::tungstenite::Error>
{
    ws.and_then(|msg| msg.into_text()).with(|text| Ok(Message::Text(text)))
}
