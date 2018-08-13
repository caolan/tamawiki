use document::ParticipantId;

#[derive(Serialize, Debug)]
pub enum ServerMessage {
    Connected (Connected)
}

#[derive(Serialize, Debug)]
pub struct Connected {
    pub id: ParticipantId
}

#[derive(Deserialize, Debug)]
pub enum ClientMessage {}
