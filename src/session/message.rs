//! Defines messages for client/server communication during an EditSession

use store::SequenceId;
use document::{ParticipantId, Operation};


/// Message sent from the server to the client
#[derive(Serialize, Debug)]
pub enum ServerMessage {
    /// Client successfully connected - always the first message sent
    /// to a client
    Connected (ConnectedMessage),
    /// An update was made to the Document
    Edit (EditMessage),
    /// A new participant joined the DocumentSession
    Join (JoinMessage),
}

/// Client successfully connected
#[derive(Serialize, Debug)]
pub struct ConnectedMessage {
    /// The new client's participant ID
    pub id: ParticipantId,
}

/// An update was made to the Document
#[derive(Serialize, Debug)]
pub struct EditMessage {
    /// The SequenceId of the edit Event in the Store
    pub seq: SequenceId,
    /// The id of the participant responsible for making this change
    pub author: ParticipantId,
    /// The individual Operations which describe the edit
    pub operations: Vec<Operation>,
}

/// A new participant has joined the DocumentSession
#[derive(Serialize, Debug)]
pub struct JoinMessage {
    /// The SequenceId of the join Event in the Store
    pub seq: SequenceId,
    /// The id of the newly joined participant
    pub id: ParticipantId,
}

/// Message sent from the client to the server
#[derive(Deserialize, Debug)]
pub enum ClientMessage {}
