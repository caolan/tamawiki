//! Defines messages for client/server communication during an EditSession

use document::ParticipantId;

/// Message sent from the server to the client
#[derive(Serialize, Debug)]
pub enum ServerMessage {
    /// Client successfully connected
    Connected (Connected)
}

/// Client successfully connected
#[derive(Serialize, Debug)]
pub struct Connected {
    /// The new client's participant ID
    pub id: ParticipantId
}

/// Message sent from the client to the server
#[derive(Deserialize, Debug)]
pub enum ClientMessage {}
