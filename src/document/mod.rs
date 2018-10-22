//! Documents and Operations on their content.
use std::cmp;
use std::collections::HashMap;
use std::error;
use std::fmt;

// The base struct definitions are in another file so they can be used
// by build.rs to generate rust test code for the JSON tests found in
// tests/shared.
include!("./types.rs");

impl Document {
    /// Applies an Edit event to the Document's content. Either all
    /// Operations contained in the Edit are applied, or no
    /// Operations are applied and an EditError is returned.
    pub fn apply(&mut self, event: &Event) -> Result<(), EditError> {
        self.can_apply(&event)?;

        match event {
            Event::Edit(edit) => {
                for op in &edit.operations {
                    self.perform_operation(edit.author, op);
                }
            }
            Event::Join(Join { id }) => {
                self.participants
                    .entries
                    .insert(*id, DocumentParticipant { cursor_pos: 0 });
            }
            Event::Leave(Leave { id }) => {
                self.participants.entries.remove(&id);
            }
        }
        Ok(())
    }

    /// Checks that every operation inside the edit can be cleanly
    /// applied to the document, without making any changes to the
    /// document content.
    pub fn can_apply(&self, event: &Event) -> Result<(), EditError> {
        match event {
            Event::Edit(edit) => {
                if !self.participants.entries.contains_key(&edit.author) {
                    // edit author is not currently a participant
                    return Err(EditError::InvalidOperation);
                }
                let mut length = self.content.chars().count();

                for op in &edit.operations {
                    if !op.is_valid() {
                        return Err(EditError::InvalidOperation);
                    }
                    match *op {
                        Operation::Insert(Insert { pos, ref content }) => {
                            if pos > length {
                                return Err(EditError::OutsideDocument);
                            } else {
                                length += content.chars().count();
                            }
                        }
                        Operation::Delete(Delete { start, end, .. }) => {
                            if start > length || end > length {
                                return Err(EditError::OutsideDocument);
                            } else {
                                length -= end - start;
                            }
                        }
                    }
                }
                Ok(())
            }
            Event::Join(Join { id }) => {
                if self.participants.entries.contains_key(&id) {
                    // id is already a participant
                    Err(EditError::InvalidOperation)
                } else {
                    Ok(())
                }
            }
            Event::Leave(Leave { id }) => {
                if self.participants.entries.contains_key(&id) {
                    Ok(())
                } else {
                    // id is not a current participant
                    Err(EditError::InvalidOperation)
                }
            }
        }
    }

    // Applies an Operation to the Document's content, updating the
    // Document struct in-place. At this point we must already have
    // validated that the operation can be applied_cleanly using
    // can_apply_all(). If the operation cannot be applied this function will panic.
    fn perform_operation(&mut self, author: ParticipantId, op: &Operation) {
        match *op {
            Operation::Insert(ref op) => {
                match self.content.char_indices().nth(op.pos) {
                    Some((byte_pos, _)) => {
                        self.content.insert_str(byte_pos, &op.content);
                    }
                    None => {
                        if op.pos == self.content.chars().count() {
                            self.content.push_str(&op.content);
                        } else {
                            panic!("Attempted to apply an operation outside of the document")
                        }
                    }
                }
                let char_len = op.content.chars().count();
                for (id, participant) in self.participants.entries.iter_mut() {
                    if *id == author {
                        participant.cursor_pos = op.pos + char_len;
                    } else if participant.cursor_pos > op.pos {
                        participant.cursor_pos += char_len;
                    }
                }
            }
            Operation::Delete(ref op) => {
                let byte_position =
                    |content: &String, index| match content.char_indices().nth(index) {
                        Some((byte_pos, _)) => Some(byte_pos),
                        None if index == content.chars().count() => Some(content.len()),
                        None => None,
                    };
                let start = byte_position(&self.content, op.start);
                let end = byte_position(&self.content, op.end);

                if let (Some(start_byte), Some(end_byte)) = (start, end) {
                    let after = self.content.split_off(end_byte);
                    self.content.truncate(start_byte);
                    self.content.push_str(&after);
                } else {
                    panic!("Attempted to apply an operation outside of the document")
                }
                for (id, participant) in self.participants.entries.iter_mut() {
                    if *id == author {
                        participant.cursor_pos = op.start;
                    } else if participant.cursor_pos > op.start {
                        // participant.cursor_pos += op.end - op.start;
                        // TODO: test this cmp::min
                        participant.cursor_pos -=
                            cmp::min(op.end, participant.cursor_pos) - op.start;
                    }
                }
            }
        }
    }
}

impl<'a> From<&'a str> for Document {
    fn from(content: &'a str) -> Self {
        Document {
            content: String::from(content),
            participants: Default::default(),
        }
    }
}

impl Operation {
    /// Returns false if the Operation would never describe a
    /// meaningful change for any given Document.
    ///
    /// Note that an Operation for which is_valid() returns true might
    /// still raise an EditError when applied to a specific Document
    /// (e.g. it references an index outside the target document's
    /// content size).
    pub fn is_valid(&self) -> bool {
        match self {
            Operation::Insert(_) => true,
            Operation::Delete(Delete { start, end, .. }) => end >= start,
        }
    }
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EditError::OutsideDocument => write!(
                f,
                "The operation's area of effect falls outside the document"
            ),
            EditError::InvalidOperation => write!(f, "The operation is invalid"),
        }
    }
}

impl error::Error for EditError {
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl Event {
    /// Modifies the Event struct to accommodate a concurrent Event
    /// which has already been applied locally.
    pub fn transform(&mut self, concurrent: &Event) {
        if let (&mut Event::Edit(ref mut a), &Event::Edit(ref b)) = (self, concurrent) {
            a.transform(b)
        }
    }
}

impl Edit {
    // Does the Edit from ParticipantId 'a' take precedence over the
    // Edit from ParticipantId 'b' if operations conflict?
    fn has_priority(a: ParticipantId, b: ParticipantId) -> bool {
        a > b
    }

    /// Modifies the `operations` inside the Edit struct to
    /// accommodate a concurrent Edit entry whose operations have
    /// already been applied locally.
    fn transform(&mut self, concurrent: &Edit) {
        use self::Operation as Op;
        let mut new_operations = vec![];

        for op in &concurrent.operations {
            for operation in &mut self.operations {
                match (operation, op) {
                    (Op::Insert(ref mut this), &Op::Insert(ref other)) => {
                        if other.pos < this.pos
                            || (other.pos == this.pos
                                && Edit::has_priority(concurrent.author, self.author))
                        {
                            this.pos += other.content.chars().count();
                        }
                    }
                    (Op::Insert(ref mut this), &Op::Delete(ref other)) => {
                        if other.start < this.pos {
                            let end = cmp::min(this.pos, other.end);
                            this.pos -= end - other.start;
                        }
                    }
                    (Op::Delete(ref mut this), &Op::Insert(ref other)) => {
                        if other.pos < this.start {
                            let len = other.content.chars().count();
                            this.start += len;
                            this.end += len;
                        } else if other.pos < this.end && this.end - this.start > 0 {
                            // need to split the delete into two parts
                            // to avoid clobbering the new insert
                            // (only split when the delete has a range
                            // greater than 0, otherwise it can only
                            // produce a duplicate event)

                            // create a new operation for the first
                            // part of the range
                            let op = Op::Delete(Delete {
                                start: this.start,
                                end: other.pos,
                            });

                            // adjust the current operation to cover
                            // the end second part of the range
                            let len = other.content.chars().count();
                            this.start = other.pos + len;
                            this.end = this.end + len;

                            // push the operation covering the first
                            // part of the range to new_operations.
                            // This means it's applied after the
                            // second part of the range and the cursor
                            // ends in the correct position.
                            new_operations.push(op);
                        }
                    }
                    (Op::Delete(ref mut this), &Op::Delete(ref other)) => {
                        let mut chars_deleted_before = if other.start < this.start {
                            let end = cmp::min(this.start, other.end);
                            end - other.start
                        } else {
                            0
                        };
                        let mut chars_deleted_inside = 0;
                        if other.start < this.start {
                            if other.end > this.start {
                                let end = cmp::min(this.end, other.end);
                                chars_deleted_inside = end - this.start;
                            }
                        } else if other.start < this.end {
                            let end = cmp::min(this.end, other.end);
                            chars_deleted_inside = end - other.start;
                        }
                        this.start -= chars_deleted_before;
                        this.end -= chars_deleted_before + chars_deleted_inside;
                    }
                }
            }
            self.operations.append(&mut new_operations);
            // NOTE: at this point it is possible operations which no
            // longer affect a documents content (e.g. Deletes where
            // start == end, or Inserts with content = "") are
            // included in the operations list. This is valid because
            // those operations may modify the cursor position of
            // participants and would have been accepted if applied in
            // a different order. To make sure the cursor positions
            // converge, these otherwise useless operations must be
            // retained.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::ops::Range;

    // NOTE: these tests are generated by build.rs during cargo build
    // and use the JSON test specifications found in tests/shared
    include!(concat!(env!("OUT_DIR"), "/shared_tests.rs"));

    #[test]
    fn transform_insert_before_insert() {
        let mut h = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("Test"),
            })],
        };
        h.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 10,
                content: String::from("foo"),
            })],
        });
        assert_eq!(
            h.operations,
            vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("Test")
            })]
        );
    }

    #[test]
    fn transform_insert_after_insert() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 10,
                content: String::from("Test"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 2,
                content: String::from("foo"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 13,
                content: String::from("Test")
            })]
        );
    }

    #[test]
    fn transform_inserts_at_same_point_checks_priority() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("Test"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("foo"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 8,
                content: String::from("Test")
            })]
        );

        let mut msg = Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("Test"),
            })],
        };
        msg.transform(&Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("foo"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("Test")
            })]
        );
    }

    #[test]
    fn transform_insert_uses_char_index_not_byte_index() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("Test"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                // 2-byte unicode scalar value
                content: String::from("д"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                // 6 is char index, 7 would be byte index
                pos: 6,
                content: String::from("Test")
            })]
        );
    }

    #[test]
    fn transform_delete_delete_non_overlapping_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 0, end: 5 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 10, end: 15 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 0, end: 5 })]
        );
    }

    #[test]
    fn transform_delete_delete_non_overlapping_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 5, end: 10 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 0, end: 1 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 4, end: 9 })]
        );
    }

    #[test]
    fn transform_delete_delete_adjacent_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 2, end: 4 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 0, end: 2 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 0, end: 2 })]
        );
    }

    #[test]
    fn transform_delete_delete_adjacent_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 0, end: 3 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 3, end: 5 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 0, end: 3 })]
        );
    }

    #[test]
    fn transform_delete_delete_overlapping_start() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 5, end: 15 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 0, end: 10 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 0, end: 5 })]
        );
    }

    #[test]
    fn transform_delete_delete_overlapping_end() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 0, end: 4 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 2, end: 6 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 0, end: 2 })]
        );
    }

    #[test]
    fn transform_delete_delete_subset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 5, end: 10 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 1, end: 20 })],
        });
        // keep the delete operation even though it will have no
        // effect on the content, since it will still set the cursor
        // position for it's author
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 1, end: 1 })]
        );
    }

    #[test]
    fn transform_delete_delete_superset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 0, end: 17 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 5, end: 10 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 0, end: 12 })]
        );
    }

    #[test]
    fn transform_delete_delete_same_range() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 0, end: 1 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 0, end: 1 })],
        });
        // keep delete which has no effect since it will still set the
        // appropriate cursor position for the author
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 0, end: 0 })]
        );
    }

    #[test]
    fn transform_insert_delete_non_overlapping_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("12345"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 10, end: 15 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("12345")
            })]
        );
    }

    #[test]
    fn transform_insert_delete_non_overlapping_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("foo"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 0, end: 1 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 4,
                content: String::from("foo")
            })]
        );
    }

    #[test]
    fn transform_insert_delete_adjacent_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 2,
                content: String::from("ab"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 0, end: 2 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("ab")
            })]
        );
    }

    #[test]
    fn transform_insert_delete_adjacent_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("foo"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 3, end: 5 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("foo")
            })]
        );
    }

    #[test]
    fn transform_insert_delete_overlapping_start() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("1234567890"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 0, end: 10 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("1234567890")
            })]
        );
    }

    #[test]
    fn transform_insert_delete_overlapping_end() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("abcd"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 2, end: 6 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("abcd")
            })]
        );
    }

    #[test]
    fn transform_insert_delete_subset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("12345"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 1, end: 20 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 1,
                content: String::from("12345")
            })]
        );
    }

    #[test]
    fn transform_insert_delete_superset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("12345678901234567"),
            })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete { start: 5, end: 10 })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("12345678901234567")
            })]
        );
    }

    #[test]
    fn transform_delete_insert_non_overlapping_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 0, end: 5 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 10,
                content: String::from("12345"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 0, end: 5 })]
        );
    }

    #[test]
    fn transform_delete_insert_non_overlapping_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 5, end: 8 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("a"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 6, end: 9 })]
        );
    }

    #[test]
    fn transform_delete_insert_adjacent_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 2, end: 4 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("ab"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 4, end: 6 })]
        );
    }

    #[test]
    fn transform_delete_insert_adjacent_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 0, end: 3 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 3,
                content: String::from("ab"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 0, end: 3 })]
        );
    }

    #[test]
    fn transform_delete_insert_same_start_position() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 2, end: 4 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 2,
                content: String::from("cd"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![
                Operation::Delete(Delete { start: 4, end: 6 }),
                Operation::Delete(Delete { start: 2, end: 2 }),
            ]
        );
    }

    #[test]
    fn transform_delete_insert_overlapping_start() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 5, end: 15 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("1234567890"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 15, end: 25 })]
        );
    }

    #[test]
    fn transform_delete_insert_overlapping_end() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 0, end: 4 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 2,
                content: String::from("abcd"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![
                Operation::Delete(Delete { start: 6, end: 8 }),
                Operation::Delete(Delete { start: 0, end: 2 }),
            ]
        );
    }

    #[test]
    fn transform_delete_insert_subset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 5, end: 10 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 1,
                content: String::from("12345678901234567890"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![Operation::Delete(Delete { start: 25, end: 30 })]
        );
    }

    #[test]
    fn transform_delete_insert_superset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete { start: 0, end: 17 })],
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("12345"),
            })],
        });
        assert_eq!(
            msg.operations,
            vec![
                Operation::Delete(Delete { start: 10, end: 22 }),
                Operation::Delete(Delete { start: 0, end: 5 }),
            ]
        );
    }

    #[test]
    fn concurrent_delete_and_insert() {
        let mut doc = Document::from("ab");
        doc.apply(&Event::Join(Join { id: 1 })).unwrap();
        doc.apply(&Event::Join(Join { id: 2 })).unwrap();

        let op1 = Operation::Delete(Delete { start: 0, end: 1 });
        let op2 = Operation::Insert(Insert {
            pos: 1,
            content: String::from("c"),
        });

        let mut a1 = Event::Edit(Edit {
            author: 1,
            operations: vec![op1.clone()],
        });
        let b1 = Event::Edit(Edit {
            author: 2,
            operations: vec![op2.clone()],
        });
        let a2 = a1.clone();
        let mut b2 = b1.clone();
        let mut doc1 = doc.clone();
        let mut doc2 = doc.clone();
        a1.transform(&b1);
        b2.transform(&a2);

        assert_eq!(
            a2,
            Event::Edit(Edit {
                author: 1,
                operations: vec![Operation::Delete(Delete { start: 0, end: 1 })]
            })
        );
        assert_eq!(
            b2,
            Event::Edit(Edit {
                author: 2,
                operations: vec![Operation::Insert(Insert {
                    pos: 0,
                    content: String::from("c")
                })]
            })
        );

        // doc1

        assert_eq!(
            doc1.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 0 }),
            ].into_iter()
            .collect()
        );

        doc1.apply(&b1).unwrap();
        assert_eq!(
            doc1.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 2 }),
            ].into_iter()
            .collect()
        );

        doc1.apply(&a1).unwrap();
        assert_eq!(
            doc1.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 1 }),
            ].into_iter()
            .collect()
        );

        // doc2

        assert_eq!(
            doc2.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 0 }),
            ].into_iter()
            .collect()
        );

        doc2.apply(&a2).unwrap();
        assert_eq!(
            doc2.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 0 }),
            ].into_iter()
            .collect()
        );

        doc2.apply(&b2).unwrap();
        assert_eq!(
            doc2.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 1 }),
            ].into_iter()
            .collect()
        );

        // check the results converge
        assert_eq!(doc1, doc2);
    }

    #[test]
    fn concurrent_delete_and_insert_2() {
        let mut doc = Document::from("a");
        doc.apply(&Event::Join(Join { id: 1 })).unwrap();
        doc.apply(&Event::Join(Join { id: 2 })).unwrap();

        let op1 = Operation::Delete(Delete { start: 0, end: 1 });
        let op2 = Operation::Insert(Insert {
            pos: 0,
            content: String::from("b"),
        });

        let mut a1 = Event::Edit(Edit {
            author: 1,
            operations: vec![op1.clone()],
        });
        let b1 = Event::Edit(Edit {
            author: 2,
            operations: vec![op2.clone()],
        });
        let a2 = a1.clone();
        let mut b2 = b1.clone();
        let mut doc1 = doc.clone();
        let mut doc2 = doc.clone();
        a1.transform(&b1);
        b2.transform(&a2);

        // assert_eq!(a2, Event::Edit(Edit {
        //     author: 1,
        //     operations: vec![Operation::Delete(Delete {
        //         start: 1,
        //         end: 2
        //     })]
        // }));
        // assert_eq!(b2, Event::Edit(Edit {
        //     author: 2,
        //     operations: vec![Operation::Insert(Insert {
        //         pos: 0,
        //         content: String::from("c")
        //     })]
        // }));

        // doc1 (insert then delete)

        assert_eq!(
            doc1.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 0 }),
            ].into_iter()
            .collect()
        );

        doc1.apply(&b1).unwrap();
        assert_eq!(
            doc1.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 1 }),
            ].into_iter()
            .collect()
        );

        doc1.apply(&a1).unwrap();
        assert_eq!(
            doc1.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 1 }),
            ].into_iter()
            .collect()
        );

        // doc2 (delete then insert)

        assert_eq!(
            doc2.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 0 }),
            ].into_iter()
            .collect()
        );

        doc2.apply(&a2).unwrap();
        assert_eq!(
            doc2.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 0 }),
            ].into_iter()
            .collect()
        );

        doc2.apply(&b2).unwrap();
        assert_eq!(
            doc2.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 1 }),
            ].into_iter()
            .collect()
        );

        // check the results converge
        assert_eq!(doc1, doc2);
    }

    #[test]
    fn concurrent_delete_and_insert_3() {
        let mut doc = Document::from("ab");
        doc.apply(&Event::Join(Join { id: 1 })).unwrap();
        doc.apply(&Event::Join(Join { id: 2 })).unwrap();

        let op1 = Operation::Delete(Delete { start: 0, end: 2 });
        let op2 = Operation::Insert(Insert {
            pos: 1,
            content: String::from("c"),
        });

        let mut a1 = Event::Edit(Edit {
            author: 1,
            operations: vec![op1.clone()],
        });
        let b1 = Event::Edit(Edit {
            author: 2,
            operations: vec![op2.clone()],
        });
        let a2 = a1.clone();
        let mut b2 = b1.clone();
        let mut doc1 = doc.clone();
        let mut doc2 = doc.clone();
        a1.transform(&b1);
        b2.transform(&a2);

        // doc1 (insert then delete)

        assert_eq!(
            doc1.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 0 }),
            ].into_iter()
            .collect()
        );

        doc1.apply(&b1).unwrap();
        assert_eq!(
            doc1.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 2 }),
            ].into_iter()
            .collect()
        );

        doc1.apply(&a1).unwrap();
        assert_eq!(
            doc1.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 1 }),
            ].into_iter()
            .collect()
        );

        // doc2 (delete then insert)

        assert_eq!(
            doc2.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 0 }),
            ].into_iter()
            .collect()
        );

        doc2.apply(&a2).unwrap();
        assert_eq!(
            doc2.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 0 }),
            ].into_iter()
            .collect()
        );

        doc2.apply(&b2).unwrap();
        assert_eq!(
            doc2.participants,
            vec![
                (1, DocumentParticipant { cursor_pos: 0 }),
                (2, DocumentParticipant { cursor_pos: 1 }),
            ].into_iter()
            .collect()
        );

        // check the results converge
        assert_eq!(doc1, doc2);
    }

    trait GenerateStrategy {
        fn generate_strategy(doc_size: usize) -> BoxedStrategy<Operation>;
    }

    impl GenerateStrategy for Delete {
        fn generate_strategy(doc_size: usize) -> BoxedStrategy<Operation> {
            (
                Range {
                    start: 0,
                    end: doc_size + 1,
                },
                Range {
                    start: 0,
                    end: doc_size + 1,
                },
            )
                .prop_filter(
                    "Delete operation start index must be smaller than end index".to_owned(),
                    |v| v.0 < v.1,
                ).prop_map(|v| {
                    Operation::Delete(Delete {
                        start: v.0,
                        end: v.1,
                    })
                }).boxed()
        }
    }

    impl GenerateStrategy for Insert {
        fn generate_strategy(doc_size: usize) -> BoxedStrategy<Operation> {
            (
                Range {
                    start: 0,
                    end: doc_size,
                },
                ".{1,100}",
            )
                .prop_map(|v| {
                    Operation::Insert(Insert {
                        pos: v.0,
                        content: v.1,
                    })
                }).boxed()
        }
    }

    fn generate_document_data() -> BoxedStrategy<String> {
        ".{1,100}"
            .prop_filter(
                "Document must contain at least one unicode scalar value".to_owned(),
                |v| v.chars().count() > 0,
            ).boxed()
    }

    fn conflicting_operations<A, B>() -> BoxedStrategy<(String, Operation, Operation)>
    where
        A: GenerateStrategy,
        B: GenerateStrategy,
    {
        generate_document_data()
            .prop_flat_map(|initial| {
                let len = initial.chars().count();
                (
                    Just(initial),
                    <A as GenerateStrategy>::generate_strategy(len),
                    <B as GenerateStrategy>::generate_strategy(len),
                )
            }).boxed()
    }

    // helper function for proptest! block below
    fn check_order_of_application(initial: &str, op1: &Operation, op2: &Operation) {
        let mut doc = Document::from(initial);
        doc.apply(&Event::Join(Join { id: 1 })).unwrap();
        doc.apply(&Event::Join(Join { id: 2 })).unwrap();

        let mut a1 = Event::Edit(Edit {
            author: 1,
            operations: vec![op1.clone()],
        });
        let b1 = Event::Edit(Edit {
            author: 2,
            operations: vec![op2.clone()],
        });
        let a2 = a1.clone();
        let mut b2 = b1.clone();
        let mut doc1 = doc.clone();
        let mut doc2 = doc.clone();
        a1.transform(&b1);
        b2.transform(&a2);
        // apply original operations
        doc1.apply(&b1).unwrap();
        doc2.apply(&a2).unwrap();
        // apply transformed operations on top
        doc1.apply(&a1).unwrap();
        doc2.apply(&b2).unwrap();
        // check the results converge
        assert_eq!(doc1, doc2);
    }

    proptest! {

        #[test]
        fn insert_insert_order_of_application
            ((ref initial, ref op1, ref op2) in conflicting_operations::<Insert, Insert>()) {
                check_order_of_application(initial, op1, op2);
            }

        #[test]
        fn delete_delete_order_of_application
            ((ref initial, ref op1, ref op2) in conflicting_operations::<Delete, Delete>()) {
                check_order_of_application(initial, op1, op2);
            }

        #[test]
        fn delete_insert_order_of_application
            ((ref initial, ref op1, ref op2) in conflicting_operations::<Delete, Insert>()) {
                check_order_of_application(initial, op1, op2);
            }

        #[test]
        fn insert_delete_order_of_application
            ((ref initial, ref op1, ref op2) in conflicting_operations::<Insert, Delete>()) {
                check_order_of_application(initial, op1, op2);
            }

    }

}
