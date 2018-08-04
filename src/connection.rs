//! Client communication with an EditSession via websockets
use actix_web::ws;
use std::path::PathBuf;
use futures::future::Future;
use actix::prelude::*;
use serde_json;

use super::TamaWikiState;
use document::{Operation, Update};
use session::{self, EditSession, ClientUpdate};
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

    fn handle_edit(&mut self,
                   data: Edit,
                   ctx: &mut ws::WebsocketContext<Self, TamaWikiState<T>>)
    {
        match self.session {
            Some((id, ref session)) => {
                Arbiter::spawn(
                    session.send(ClientUpdate {
                        edit: data,
                        id
                    }).map(|result| {
                        if let Err(err) = result {
                            panic!("error communicating with store: {}", err)
                        }
                    }).map_err(|err| {
                        panic!("error sending edit to session: {}", err)
                    })
                );
            },
            None => {
                // TODO: write a test for this
                ctx.address().do_send(ServerMessage::Error(ServerError {
                    code: "Not connected".to_owned(),
                    description: "You must wait for a 'Connected' message before sending".to_owned()
                }));
            }
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
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                // TODO: write a test for this
                ctx.pong(&msg);
            },
            ws::Message::Text(text) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::Edit(data)) => {
                        self.handle_edit(data, ctx);
                    },
                    Err(err) => {
                        // TODO: write a test for this
                        ctx.address().do_send(ServerMessage::Error(ServerError {
                            code: "Invalid data".to_owned(),
                            description: format!("Message could not be parsed: {}", err)
                        }));
                    }
                }
                
            },
            ws::Message::Binary(_bin) => {
                // TODO: write a test for this
                ctx.close(Some(ws::CloseReason {
                    code: ws::CloseCode::Unsupported,
                    description: Some(
                        String::from("Server does not accept binary messages")
                    ),
                }));
            },
            ws::Message::Pong(_msg) => (),
            ws::Message::Close(_msg) => (),
        }
    }
}

/// An enum of the possible messages the client can send to the server
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub enum ClientMessage {
    Edit (Edit),
}

/// Sent by the client when local changes were made to their document
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub struct Edit {
    /// The last server sequence number this edit is based upon
    pub seq: usize,
    /// The number of this update on the client
    pub client_seq: usize,
    /// The operations on the document which describe this change
    pub operations: Vec<Operation>
}

/// An enum of the possible messages the server can send to the client
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub enum ServerMessage {
    Connected (Connected),
    Join (Join),
    Leave (Leave),
    Change (Change),
    Error (ServerError),
}

/// An error message sent by the server to the client
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub struct ServerError {
    pub code: String,
    pub description: String,
}

/// Sent by the server when the client has been successfully connected
/// to an EditSession
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub struct Connected {
    pub id: ConnectionId,
    pub participants: Vec<ConnectionId>,
}

/// Sent by the server when a participant joins the EditSession
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub struct Join {
    // TODO: add client_seq to this message
    pub id: ConnectionId,
}

/// Broadcast by the server to all participants, except the author,
/// when it accepts a new change to the document.
#[derive(Debug, PartialEq, Deserialize, Serialize, Message, Clone)]
pub struct Change {
    pub seq: usize,
    pub update: Update
}


/// Sent by the server when a participant leaves the EditSession
#[derive(Debug, PartialEq, Deserialize, Serialize, Message)]
pub struct Leave {
    // TODO: add client_seq to this message
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
