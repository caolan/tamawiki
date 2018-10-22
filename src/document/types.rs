// NOTE: this file is shared between build.rs and the document module,
// it should contain the minimal definitions of each type - just
// enough for build.rs to be able to deserialize them from JSON

use serde::{Deserializer, Serializer, Deserialize, Serialize};
use serde::de::{SeqAccess, Visitor};
use std::iter::FromIterator;


/// Connection identifier, which must be unique between concurrent events
pub type ParticipantId = usize;

/// A collection of DocumentParticipants keyed by Participant ID.
/// Serializes as an array of SerializedParticipant objects as JSON
/// only allows strings as keys.
#[derive(Default, Debug, PartialEq, Clone)]
pub struct Participants {
    /// The participants
    pub entries: HashMap<ParticipantId, DocumentParticipant>,
}

impl Participants {
    /// Creates a new empty Participants collection
    pub fn new() -> Self {
        Default::default()
    }
}

impl FromIterator<(usize, DocumentParticipant)> for Participants {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=(usize, DocumentParticipant)>,
    {
        Participants {entries: HashMap::from_iter(iter)}
    }
}

struct ParticipantsVisitor {}

impl<'de> Visitor<'de> for ParticipantsVisitor {
    type Value = Participants;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Participants::new())
    }

    #[inline]
    fn visit_seq<T>(self, mut access: T) -> Result<Self::Value, T::Error>
    where
        T: SeqAccess<'de>,
    {
        let mut entries = HashMap::with_capacity(
            access.size_hint().unwrap_or(0)
        );
        while let Some(SerializedParticipant {id, cursor_pos}) = access.next_element()? {
            entries.insert(id, DocumentParticipant { cursor_pos });
        }
        Ok(Participants {entries})
    }
}

impl<'de> Deserialize<'de> for Participants {
    fn deserialize<D>(deserializer: D) -> Result<Participants, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ParticipantsVisitor {})
    }
}

#[derive(Deserialize, Serialize)]
struct SerializedParticipant {
    id: ParticipantId,
    cursor_pos: usize,
}

impl Serialize for Participants {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.entries.iter().map(|(k, v)| SerializedParticipant {
            id: *k,
            cursor_pos: v.cursor_pos
        }))
    }
}

/// Edit session participant data relevant to displaying a document
#[derive(Debug, PartialEq, Eq, Hash, Default, Deserialize, Serialize, Clone)]
pub struct DocumentParticipant {
    /// Unicode Scalar Value index of the participant's cursor
    pub cursor_pos: usize
}

/// Represents some String content at a point in time
#[derive(Debug, PartialEq, Default, Deserialize, Serialize, Clone)]
pub struct Document {
    /// Current document content
    pub content: String,
    /// Current active editors
    pub participants: Participants
}

/// Inserts new content at a single position in the Document
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Insert {
    /// Insert position as number of Unicode Scalar Values preceeding
    /// the Insert operation, from the beginning of the document (not
    /// byte position, or number of grapheme clusters)
    pub pos: usize,
    /// The String content to insert
    pub content: String,
}

/// Deletes a contiguous region of content from the Document
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Delete {
    /// First Unicode Scalar Value to remove in range
    pub start: usize,
    /// Unicode Scalar Value to end the delete operation on (exclusive
    /// of the 'end' character)
    pub end: usize,
}

/// Describes an event corresponding to a single Document.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub enum Event {
    /// A new participant has joined
    Join (Join),
    /// A participant has left
    Leave (Leave),
    /// An update was made to the document
    Edit (Edit),
}

/// Describes incremental changes to a Document's content. Through the
/// accumulated application of Operations to a Document, the
/// Document's content at a point in time can be derived.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub enum Operation {
    /// Insert new content into the Docuemnt
    Insert(Insert),
    /// Remove content from the Document
    Delete(Delete),
}

/// A new participant has joined the DocumentSession
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Join {
    /// The id of the newly joined participant
    pub id: ParticipantId
}

/// A participant has left the DocumentSession
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Leave {
    /// The id of the now departed participant
    pub id: ParticipantId
}

/// An Edit combines multiple operations into a single Document
/// change (i.e. all the operations are applied together, or not at
/// all).
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Edit {
    /// The ParticipantId for the original author of this edit
    pub author: ParticipantId,
    /// The Operations which describe this Edit
    pub operations: Vec<Operation>,
}

/// Error conditions which may occur when applying an Operation to a
/// Document.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EditError {
    /// The Operation's position or range falls outside the current
    /// Document.
    OutsideDocument,
    /// The operation is invalid and could not be applied meaningfully
    /// to any document.
    InvalidOperation,
}

