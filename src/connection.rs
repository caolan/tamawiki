//! Client communication with an EditSession via websockets
use actix_web::ws;
use std::path::PathBuf;
use actix::prelude::*;
use serde_json;

use super::TamaWikiState;
use session::{self, EditSession};
use store::Store;


/// Connection identifier, which must be unique for the current edit session
pub type ConnectionId = usize;

/// Represents a client connection to an edit session
pub struct Connection<T: Store> {
    session: Option<(ConnectionId, Addr<EditSession<T>>)>,
    path: PathBuf,
    seq: usize,
}

impl<T: Store> Connection<T> {
    pub fn new(path: PathBuf, seq: usize) -> Self {
        Self {
            session: None,
            path,
            seq,
        }
    }
}

impl<T: Store> Actor for Connection<T> {
    type Context = ws::WebsocketContext<Self, TamaWikiState<T>>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Connection started");
        let addr = ctx.address();
        let state = ctx.state();
        state.session_manager.do_send(session::Connect {
            from: addr,
            path: self.path.clone(),
            seq: self.seq,
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Connection stopped");
        if let Some((id, ref session)) = self.session {
            session.do_send(session::Disconnect { id });
        }
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

/// An enum of the possible messages the server can send to the
/// connected client
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub enum ServerMessage {
    Connected (Connected),
    Join (Join),
    Leave (Leave),
}

/// Sent when the client has benn successfully connected to an
/// EditSession
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub struct Connected {
    pub id: ConnectionId,
    pub participants: Vec<ConnectionId>,
}

/// Sent when a participant joins the EditSession
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub struct Join {
    pub id: ConnectionId,
}


/// Sent when a participant leaves the EditSession
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub struct Leave {
    pub id: ConnectionId,
}

impl<T: Store> Handler<ServerMessage> for Connection<T> {
    type Result = ();
    
    fn handle(&mut self,
              msg: ServerMessage,
              ctx: &mut ws::WebsocketContext<Connection<T>, TamaWikiState<T>>) ->
        Self::Result
    {
        ctx.text(
            serde_json::to_string::<ServerMessage>(&msg)
                .expect("failed to serialize response as JSON")
        );
    }
}


impl<T: Store> Handler<session::Connected<T>> for Connection<T> {
    type Result = ();
    
    fn handle(&mut self,
              msg: session::Connected<T>,
              ctx: &mut ws::WebsocketContext<Connection<T>, TamaWikiState<T>>) ->
        Self::Result
    {
        let session::Connected {id, session, participants} = msg;
        self.session = Some((id, session.clone()));
        ctx.text(
            serde_json::to_string(&ServerMessage::Connected(
                Connected {id, participants}
            )).expect(
                "failed to serialize response as JSON"
            )
        );
    }
}
