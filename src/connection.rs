//! Handles websocket connections to collaborative edit sessions
use actix_web::ws;
use std::path::PathBuf;
use actix::prelude::*;
use std::marker::PhantomData;
use serde_json;

use super::TamaWikiState;
use store::Store;


/// Connection identifier, which must be unique for the current edit session
pub type ConnectionId = usize;

/// Represents a client connection to an edit session
pub struct Connection<T: Store> {
    store: PhantomData<T>,
}

impl<T: Store> Connection<T> {
    pub fn new(_path: PathBuf, _seq: usize) -> Self {
        Self {
            store: PhantomData,
        }
    }
}

impl<T: Store> Actor for Connection<T> {
    type Context = ws::WebsocketContext<Self, TamaWikiState<T>>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Connection started");
        ctx.text(
            serde_json::to_string::<ConnectionMessage>(
                &ConnectionMessage::Connected(Connected {id: 1})
            ).expect("failed to serialize response as JSON")
        );
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Connection stopped");
    }
}

/// Handler for ws::Message message
impl<T: Store> StreamHandler<ws::Message, ws::ProtocolError> for Connection<T> {
    fn handle(&mut self, msg: ws::Message, _ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(_msg) => (),
            ws::Message::Text(_text) => (),
            ws::Message::Binary(_bin) => (),
            _ => (),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ConnectionMessage {
    Connected (Connected),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Connected {
    pub id: ConnectionId
}
