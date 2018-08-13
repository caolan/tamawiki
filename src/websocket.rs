//! Websocket upgrade handling for hyper
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

use hyper;
use hyper::{Body, Request, Response};
use http::header::{UPGRADE, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION};
use http;
use base64;
use tungstenite::protocol;
use futures::{future, Async, AsyncSink, Future, Poll, Sink, StartSend, Stream};
use sha1::{Digest, Sha1};
use std::io::ErrorKind::WouldBlock;
use std::fmt;

use error::HttpError;


pub fn is_upgrade_request(req: &Request<Body>) -> bool {
    match req.headers().get(UPGRADE) {
        Some(value) => match value.to_str() {
            Ok(protocol) => "websocket".eq_ignore_ascii_case(protocol),
            // invalid ascii chars in header value
            Err(_) => false,
        },
        _ => false
    }
}

pub fn websocket<F, U>(req: Request<Body>, fun: F) ->
    Box<Future<Item=Response<Body>, Error=HttpError> + Send>
where
    F: Fn(WebSocket) -> U + Sync + Send + 'static,
    U: Future<Item = (), Error = ()> + Send + 'static,
{
    let upgrade = is_upgrade_request(&req);
    let version = match req.headers().get(SEC_WEBSOCKET_VERSION) {
        Some(value) => match value.to_str() {
            Ok(protocol) => "13" == protocol,
            // invalid ascii chars in header value
            Err(_) => false,
        },
        _ => false
    };
    let key = req.headers().get(SEC_WEBSOCKET_KEY).map(|k| {
        let mut sha1 = Sha1::default();
        sha1.input(k.as_bytes());
        sha1.input(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
        base64::encode(&sha1.result())
    });

    if !upgrade || !version || key.is_none() {
        return Box::new(future::err(HttpError::BadRequest));
    }
    
    let on_upgrade = req
        .into_body()
        .on_upgrade()
        .map_err(|err| eprintln!("upgrade error: {}", err))
        .and_then(move |upgraded| {
            let io = protocol::WebSocket::from_raw_socket(
                upgraded,
                protocol::Role::Server,
                None
            );
            fun(WebSocket {inner: io})
        });

    hyper::rt::spawn(on_upgrade);

    Box::new(
        future::ok(
            http::Response::builder()
                .status(101)
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-accept", key.unwrap().as_str())
                .body(Default::default())
                .unwrap()
        )
    )
}


pub struct WebSocket {
    inner: protocol::WebSocket<::hyper::upgrade::Upgraded>,
}

impl Stream for WebSocket {
    type Item = Message;
    type Error = ::tungstenite::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            let msg = match self.inner.read_message() {
                Ok(item) => item,
                Err(::tungstenite::Error::Io(ref err)) if err.kind() == WouldBlock => return Ok(Async::NotReady),
                Err(::tungstenite::Error::ConnectionClosed(_frame)) => {
                    return Ok(Async::Ready(None));
                },
                Err(e) => {
                    return Err(e)
                }
            };

            match msg {
                msg @ protocol::Message::Text(..) |
                msg @ protocol::Message::Binary(..) => {
                    return Ok(Async::Ready(Some(Message {
                        inner: msg,
                    })));
                },
                protocol::Message::Ping(_payload) => {
                    // tungstenite automatically responds to pings, so this
                    // branch should actually never happen...
                }
                protocol::Message::Pong(_payload) => {
                }
            }
        }
    }
}

impl Sink for WebSocket {
    type SinkItem = Message;
    type SinkError = ::tungstenite::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        match self.inner.write_message(item.inner) {
            Ok(()) => Ok(AsyncSink::Ready),
            Err(::tungstenite::Error::SendQueueFull(inner)) => {
                Ok(AsyncSink::NotReady(Message { inner }))
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

pub struct Message {
    inner: protocol::Message,
}

impl Message {
    /// Construct a new Text `Message`.
    pub fn text<S: Into<String>>(s: S) -> Message {
        Message {
            inner: protocol::Message::text(s),
        }
    }

    // /// Construct a new Binary `Message`.
    // pub fn binary<V: Into<Vec<u8>>>(v: V) -> Message {
    //     Message {
    //         inner: protocol::Message::binary(v),
    //     }
    // }

    // /// Returns true if this message is a Text message.
    // pub fn is_text(&self) -> bool {
    //     self.inner.is_text()
    // }

    // /// Returns true if this message is a Binary message.
    // pub fn is_binary(&self) -> bool {
    //     self.inner.is_binary()
    // }

    /// Try to get a reference to the string text, if this is a Text message.
    pub fn to_str(&self) -> Result<&str, ()> {
        match self.inner {
            protocol::Message::Text(ref s) => Ok(s),
            _ => Err(())
        }
    }

    // /// Return the bytes of this message.
    // pub fn as_bytes(&self) -> &[u8] {
    //     match self.inner {
    //         protocol::Message::Text(ref s) => s.as_bytes(),
    //         protocol::Message::Binary(ref v) => v,
    //         _ => unreachable!(),
    //     }
    // }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}
