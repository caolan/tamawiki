//! EditSessions write data to the store, and co-ordinate client
//! updates and notifications.

use std::collections::HashMap;
use futures::future::Future;
use futures::stream::Stream;
use std::path::PathBuf;
// use actix::{Addr, Arbiter, Actor, Context, MailboxError, Message, Handler};
use actix::prelude::*;

// use super::TamaWikiState;
use connection::{Connection, ConnectionId, ServerMessage, Join, Leave, Change, Edit};
use store::{Store, Since, Push, StoreError};
use document::Update;

/// Central registry for current EditSessions in progress. The
/// EditSessionManager lets clients join an existing EditSession if
/// one is in progress, or will create a new one when the first client
/// joins.
pub struct EditSessionManager<T: Store> {
    store: Addr<T>,
    sessions: HashMap<PathBuf, Addr<EditSession<T>>>,
}

impl<T: Store> EditSessionManager<T> {
    pub fn new(store: Addr<T>) -> Self {
        Self {
            store,
            sessions: HashMap::default()
        }
    }
}

impl<T: Store> Actor for EditSessionManager<T> {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
       println!("EditSessionManager started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
       println!("EditSessionManager stopped");
    }
}

/// Connect to an EditSession (or create one if none are in progress).
#[derive(Message)]
pub struct Connect<T: Store> {
    pub from: Addr<Connection<T>>,
    pub path: PathBuf,
    pub seq: usize,
}

/// Notify a Connection actor that it has joined an EditSession.
#[derive(Message)]
pub struct Connected<T: Store> {
    pub id: ConnectionId,
    pub session: Addr<EditSession<T>>,
    pub participants: Vec<ConnectionId>,
}

/// Leave an EditSession
#[derive(Message)]
pub struct Disconnect {
    pub id: ConnectionId
}

/// Send an update to the EditSession for processing
pub struct ClientUpdate {
    pub id: ConnectionId,
    pub edit: Edit,
}
impl Message for ClientUpdate {
    type Result = Result<(), MailboxError>;
}

/// Sent by the EditSession to the EditSessionManager after all
/// clients leave.
#[derive(Message)]
pub struct EndSession<T: Store> {
    pub path: PathBuf,
    pub from: Addr<EditSession<T>>,
}

impl<T: Store> Handler<Connect<T>> for EditSessionManager<T> {
    type Result = ();
    
    fn handle(&mut self,
              msg: Connect<T>,
              ctx: &mut Context<Self>) -> Self::Result {

        let exists = match self.sessions.get(&msg.path) {
            // check if session exists and actor is still alive
            Some(addr) => addr.connected(),
            _ => false,
        };
        if !exists {
            self.sessions.insert(
                msg.path.clone(),
                EditSession::new(
                    self.store.clone(),
                    ctx.address(),
                    msg.path.clone()
                ).start()
            );
        }
        self.sessions[&msg.path].do_send(msg);
    }
}

impl<T: Store> Handler<EndSession<T>> for EditSessionManager<T> {
    type Result = ();
    
    fn handle(&mut self,
              msg: EndSession<T>,
              _ctx: &mut Context<Self>) -> Self::Result {

        let exists = match self.sessions.get(&msg.path) {
            // check if session exists and actor is still alive
            Some(addr) => *addr == msg.from,
            None => false,
        };
        if exists {
            self.sessions.remove(&msg.path);
        }
    }
}

/// Represents a collaborative editing session for a single document.
/// Co-ordinates messages between connected clients.
pub struct EditSession<T: Store> {
    store: Addr<T>,
    next_id: ConnectionId,
    connected: HashMap<ConnectionId, (Addr<Connection<T>>, usize)>,
    manager: Addr<EditSessionManager<T>>,
    path: PathBuf,
}

impl<T: Store> EditSession<T> {
    /// Creates a new EditSession using the provided document store
    pub fn new(store: Addr<T>,
               manager: Addr<EditSessionManager<T>>,
               path: PathBuf) -> Self
    {
        Self {
            store,
            next_id: 0,
            connected: HashMap::new(),
            manager,
            path,
        }
    }
}

impl<T: Store> Actor for EditSession<T> {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("EditSession started: {}", self.path.to_str().unwrap());
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("EditSession stopped: {}", self.path.to_str().unwrap());
        self.manager.do_send(
            EndSession {
                path: self.path.clone(),
                from: ctx.address(),
            }
        );
    }
}

impl<T: Store> Handler<Connect<T>> for EditSession<T> {
    type Result = ();
    
    fn handle(&mut self, msg: Connect<T>, ctx: &mut Context<Self>) ->
        Self::Result
    {
        self.next_id += 1;
        let mut participants = Vec::new();

        for (id, (addr, _seq)) in &self.connected {
            if *id == self.next_id {
                panic!("Duplicate connection ID detected in EditSession");
            }
            participants.push(*id);
            addr.do_send(ServerMessage::Join(Join {
                id: self.next_id
            }));
        }
        
        self.connected.insert(
            self.next_id,
            (msg.from.clone(), msg.seq)
        );
        
        let addr = ctx.address();
        msg.from.do_send(Connected {
            id: self.next_id,
            session: addr,
            participants
        });

        Arbiter::spawn(
            self.store.send(Since {
                path: self.path.clone(),
                seq: msg.seq,
            }).map(|result| {
                match result {
                    Err(StoreError::NotFound) => (),
                    Err(err) => panic!("{}", err),
                    Ok(stream) => {
                        Arbiter::spawn(
                            stream.for_each(move |(seq, update)| {
                                msg.from.send(ServerMessage::Change(Change {
                                    seq,
                                    update,
                                })).map_err(|err| panic!("{}", err))
                            }).map_err(|err| panic!("{}", err))
                        );
                    }
                }
            }).map_err(|err| {
                panic!("Error communicating with store: {}", err);
            })
        );
    }
}

impl<T: Store> Handler<Disconnect> for EditSession<T> {
    type Result = ();
    
    fn handle(&mut self, msg: Disconnect, ctx: &mut Context<Self>) ->
        Self::Result
    {
        self.connected.remove(&msg.id);
        
        if self.connected.is_empty() {
            ctx.stop();
        } else {
            for (addr, _seq) in self.connected.values() {
                addr.do_send(ServerMessage::Leave(Leave {
                    id: msg.id
                }));
            }
        }
    }
}

impl<T: Store> Handler<ClientUpdate> for EditSession<T> {
    type Result = Box<Future<Item=(), Error=MailboxError>>;
    
    fn handle(&mut self, msg: ClientUpdate, ctx: &mut Context<Self>) ->
        Self::Result
    {
        let addr = ctx.address();
        Box::new(
            self.store.send(Push {
                path: self.path.clone(),
                update: Update {
                    from: msg.id,
                    operations: msg.edit.operations.clone()
                }
            }).map(move |result| {
                match result {
                    Ok(seq) => {
                        addr.do_send(Change {
                            seq,
                            update: Update {
                                from: msg.id,
                                operations: msg.edit.operations
                            }
                        });
                    },
                    Err(err) => panic!("{}", err),
                }
            })
        )
    }
}


impl<T: Store> Handler<Change> for EditSession<T> {
    type Result = ();
    
    fn handle(&mut self, msg: Change, _ctx: &mut Context<Self>) ->
        Self::Result
    {
        for (id, (addr, _seq)) in &self.connected {
            if *id != msg.update.from {
                addr.do_send(ServerMessage::Change(msg.clone()))
            }
        }
    }
}
