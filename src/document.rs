//! Documents and Operations on their content.
use std::error;
use std::fmt;
use std::cmp;


/// Connection identifier, which must be unique between concurrent events
pub type ParticipantId = usize;

/// Represents some String content at a point in time
#[derive(Debug, PartialEq, Default, Clone)]
pub struct Document {
    /// Current document content
    pub content: String
}

impl Document {
    /// Applies an Edit event to the Document's content. Either all
    /// Operations contained in the Edit are applied, or no
    /// Operations are applied and an EditError is returned.
    pub fn apply(&mut self, edit: &Edit) -> Result<(), EditError> {
        self.can_apply(&edit)?;
        
        for op in &edit.operations {
            self.perform_operation(op);
        }
        Ok(())
    }

    /// Checks that every operation inside the edit can be cleanly
    /// applied to the document, without making any changes to the
    /// document content.
    pub fn can_apply(&self, edit: &Edit) -> Result<(), EditError> {
        let mut length = self.content.chars().count();

        for op in &edit.operations {
            if !op.is_valid() {
                return Err(EditError::InvalidOperation);
            }
            match *op {
                Operation::Insert(Insert {pos, ref content}) => {
                    if pos > length {
                        return Err(EditError::OutsideDocument);
                    } else {
                        length += content.chars().count();
                    }
                },
                Operation::Delete(Delete {start, end}) => {
                    if start >= length || end > length {
                        return Err(EditError::OutsideDocument);
                    } else {
                        length -= end - start;
                    }
                },
            }
        }
        
        Ok(())
    }
    
    // Applies an Operation to the Document's content, updating the
    // Document struct in-place. At this point we must already have
    // validated that the operation can be applied_cleanly using
    // can_apply_all(). If the operation cannot be applied this function will panic.
    fn perform_operation(&mut self, op: &Operation) {
        match *op {
            Operation::Insert(ref op) => {
                match self.content.char_indices().nth(op.pos) {
                    Some((byte_pos, _)) => {
                        self.content.insert_str(byte_pos, &op.content);
                    },
                    None => {
                        if op.pos == self.content.chars().count() {
                            self.content.push_str(&op.content);
                        } else {
                            panic!("Attempted to apply an operation outside of the document")
                        }
                    },
                }
            },
            Operation::Delete(ref op) => {
                let byte_position = |content: &String, index| {
                    match content.char_indices().nth(index) {
                        Some((byte_pos, _)) => Some(byte_pos),
                        None if index == content.chars().count() => Some(content.len()),
                        None => None
                    }   
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
            }
        }
    }
}

impl<'a> From<&'a str> for Document {
    fn from(content: &'a str) -> Self {
        Document {
            content: String::from(content)
        }
    }
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

impl Operation {
    /// Checks if performing the operation would have any effect on a
    /// Document's content.
    pub fn has_effect(&self) -> bool {
        match self {
            Operation::Insert(Insert { ref content, .. }) =>
                content.chars().count() > 0,
            Operation::Delete(Delete { start, end }) =>
                start != end,
        }
    }

    /// Returns false if the Operation would never describe a
    /// meaningful change for any given Document. Operations which
    /// have no effect according to has_effect() are also considered
    /// invalid.
    ///
    /// Note that an Operation for which is_valid() returns true might
    /// still raise an EditError when applied to a specific Document
    /// (e.g. it references an index outside the target document's
    /// content size).
    
    // Making operations with no effect invalid aids the discovery and
    // removal of irrelevant operations at the earliest opportunity,
    // so they don't clutter storage and communication channels.
    pub fn is_valid(&self) -> bool {
        self.has_effect() && match self {
            Operation::Insert(_) => true,
            Operation::Delete(Delete { start, end }) => end > start
        }
    }
}

/// Error conditions which may occur when applying an Operation to a
/// Document.
#[derive(Debug, PartialEq)]
pub enum EditError {
    /// The Operation's position or range falls outside the current
    /// Document.
    OutsideDocument,
    /// The operation is invalid and could not be applied meaningfully
    /// to any document.
    InvalidOperation,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EditError::OutsideDocument =>
                write!(f, "The operation's area of effect falls outside the document"),
            EditError::InvalidOperation =>
                write!(f, "The operation is invalid"),
        }
    }
}

impl error::Error for EditError {
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

/// Describes an event corresponding to a single Document.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub enum Event {
    /// A new participant has joined
    Join (Join),
    /// An update was made to the document
    Edit (Edit),
}

/// A new participant has joined the DocumentSession
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Join {
    /// The id of the newly joined participant
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

impl Edit {
    // Does the Edit from ParticipantId 'a' take precedence over the
    // Edit from ParticipantId 'b' if operations conflict?
    fn has_priority(a: ParticipantId, b: ParticipantId) -> bool {
        a > b
    }
    
    /// Modifies the `operations` inside the Edit struct to
    /// accommodate a concurrent Edit entry whose operations have
    /// already been applied locally.
    pub fn transform(&mut self, concurrent: &Edit) {
        use self::Operation as Op;
        let mut new_operations = vec![];

        for op in &concurrent.operations {
            for operation in &mut self.operations {
                match (operation, op) {
                    (Op::Insert(ref mut this), &Op::Insert(ref other)) => {
                        if other.pos < this.pos ||
                            (other.pos == this.pos &&
                             Edit::has_priority(concurrent.author, self.author)) {
                                this.pos += other.content.chars().count();
                            }
                    },
                    (Op::Insert(ref mut this), &Op::Delete(ref other)) => {
                        if other.start < this.pos {
                            let end = cmp::min(this.pos, other.end);
                            this.pos -= end - other.start;
                        }
                    },
                    (Op::Delete(ref mut this), &Op::Insert(ref other)) => {
                        if other.pos <= this.start {
                            let len = other.content.chars().count();
                            this.start += len;
                            this.end += len;
                        } else if other.pos < this.end {
                            // need to split the delete into two parts
                            // to avoid clobbering the new insert
                            let len = other.content.chars().count();
                            let start2 = other.pos + len;
                            let end2 = this.end + len;
                            this.end = other.pos;
                            let first_del_len = other.pos - this.start;
                            new_operations.push(
                                Op::Delete(Delete {
                                    // the start and end will be affected
                                    // by the first delete range, so we
                                    // need to adjust the indices
                                    // accordingly
                                    start: start2 - first_del_len,
                                    end: end2 - first_del_len
                                })
                            );
                        }
                    },
                    (Op::Delete(ref mut this), &Op::Delete(ref other)) => {
                        let mut chars_deleted_before =
                            if other.start < this.start {
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
                    },
                }
            }
            self.operations.append(&mut new_operations);
            self.operations.retain(|op| op.has_effect());
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;
    use proptest::prelude::*;
    
    fn operation_test(initial: &'static str, op: Operation, expected: &'static str) {
        let mut doc = Document {
            content: String::from(initial),
        };
        doc.apply(&Edit {
            author: 1,
            operations: vec![op]
        }).unwrap();
        
        assert_eq!(doc, Document {
            content: String::from(expected),
        });
    }
    
    #[test]
    fn apply_insert_operation_in_middle() {
        operation_test(
            "Hello, !",
            Operation::Insert(Insert {
                pos: 7,
                content: String::from("world"),
            }),
            "Hello, world!"
        );
    }

    #[test]
    fn apply_insert_operation_at_start() {
        operation_test(
            "Foo Bar",
            Operation::Insert(Insert {
                pos: 0,
                content: String::from("Baz "),
            }),
            "Baz Foo Bar"
        )
    }

    #[test]
    fn apply_insert_operation_at_end() {
        let s = "Foo Bar";
        let len = s.chars().count();
        operation_test(
            s,
            Operation::Insert(Insert {
                pos: len,
                content: String::from(" Baz"),
            }),
            "Foo Bar Baz"
        );
    }
    
    #[test]
    fn apply_insert_middle_with_multibyte_chars_in_doc() {
        operation_test(
            "Здравствуйте",
            Operation::Insert(Insert {
                pos: 6,
                content: String::from("-"),
            }),
            "Здравс-твуйте"
        );
    }

    #[test]
    fn apply_insert_at_end_with_multibyte_chars_in_doc() {
        operation_test(
            "Здравс",
            Operation::Insert(Insert {
                pos: 6,
                content: String::from("..."),
            }),
            "Здравс..."
        );
    }

    #[test]
    fn apply_insert_multibyte_char_into_doc() {
        operation_test(
            "Hello / !",
            Operation::Insert(Insert {
                pos: 8,
                content: String::from("Здравствуйте"),
            }),
            "Hello / Здравствуйте!"
        );
    }

    #[test]
    fn apply_delete_operation_in_middle() {
        operation_test(
            "Hello, world!",
            Operation::Delete(Delete {
                start: 7,
                end: 12
            }),
            "Hello, !"
        );
    }

    #[test]
    fn apply_delete_operation_at_start() {
        operation_test(
            "Foo Bar Baz",
            Operation::Delete(Delete {
                start: 0,
                end: 4
            }),
            "Bar Baz"
        );
    }

    #[test]
    fn apply_delete_operation_at_end() {
        operation_test(
            "Foo Bar Baz",
            Operation::Delete(Delete {
                start: 7,
                end: 11
            }),
            "Foo Bar"
        );
    }

    #[test]
    fn apply_delete_with_multibyte_chars() {
        operation_test(
            "Здравствуйте test",
            Operation::Delete(Delete {
                start: 6,
                end: 12
            }),
            "Здравс test"
        );
    }

    
    #[test]
    fn apply_delete_to_end_with_multibyte_chars() {
        operation_test(
            "Здравствуйте",
            Operation::Delete(Delete {
                start: 6,
                end: 12
            }),
            "Здравс"
        );
    }

    #[test]
    fn delete_outside_of_bounds() {
        let mut doc = Document {
            content: String::from("foobar"),
        };
        assert_eq!(
            doc.apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Delete(Delete {
                        start: 3,
                        end: 7
                    })
                ],
            }),
            Err(EditError::OutsideDocument)
        );
        // document should remain unchanged
        assert_eq!(doc, Document {
            content: String::from("foobar")
        });
        assert_eq!(
            doc.apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Delete(Delete {
                        start: 7,
                        end: 10
                    })
                ],
            }),
            Err(EditError::OutsideDocument)
        );
        // document should remain unchanged
        assert_eq!(doc, Document {
            content: String::from("foobar")
        });
    }

    
    #[test]
    fn insert_outside_of_bounds() {
        let mut doc = Document {
            content: String::from("foobar"),
        };
        assert_eq!(
            doc.apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 8,
                        content: String::from("test")
                    })
                ],
            }),
            Err(EditError::OutsideDocument)
        );
        // document should be unchanged
        assert_eq!(doc, Document {
            content: String::from("foobar")
        });
    }

    #[test]
    fn apply_multiple_operations_in_single_edit() {
        let mut doc = Document {
            content: String::from("Hello"),
        };
        doc.apply(&Edit {
            author: 1,
            operations: vec![
                Operation::Insert(Insert {
                    pos: 5,
                    content: String::from(", ")
                }),
                Operation::Insert(Insert {
                    pos: 7,
                    content: String::from("world!!")
                }),
                Operation::Delete(Delete {
                    start: 13,
                    end: 14
                })
            ],
        }).unwrap();
        
        assert_eq!(doc, Document {
            content: String::from("Hello, world!")
        });
    }

    #[test]
    fn apply_edit_with_single_failing_operation() {
        let mut doc = Document {
            content: String::from("a"),
        };
        assert_eq!(
            doc.apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("b")
                    }),
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::from("c")
                    }),
                    Operation::Delete(Delete {
                        start: 20,
                        end: 25
                    })
                ],
            }),
            Err(EditError::OutsideDocument)
        );
        
        // document should remain unchanged
        assert_eq!(doc, Document {
            content: String::from("a")
        });
    }

    
    #[test]
    fn apply_previous_operation_makes_later_operation_valid() {
        let mut doc = Document {
            content: String::from("Hello"),
        };
        doc.apply(&Edit {
            author: 1,
            operations: vec![
                Operation::Insert(Insert {
                    pos: 5,
                    content: String::from(", world!")
                }),
                Operation::Delete(Delete {
                    start: 7,
                    end: 12
                }),
                Operation::Insert(Insert {
                    pos: 7,
                    content: String::from("galaxy")
                }),
            ],
        }).unwrap();
        
        assert_eq!(doc, Document {
            content: String::from("Hello, galaxy!")
        });
    }

    #[test]
    fn apply_previous_operation_makes_later_operation_invalid() {
        let mut doc = Document {
            content: String::from("Hello"),
        };
        assert_eq!(
            doc.apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Delete(Delete {
                        start: 0,
                        end: 5
                    }),
                    Operation::Insert(Insert {
                        pos: 5,
                        content: String::from(", world!")
                    })
                ],
            }),
            Err(EditError::OutsideDocument)
        );

        // document should remain unchanged
        assert_eq!(doc, Document {
            content: String::from("Hello")
        });
    }

    #[test]
    fn apply_operations_that_have_no_effect() {
        let mut doc = Document {
            content: String::from("Hello"),
        };
        assert_eq!(
            doc.apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Delete(Delete {
                        start: 2,
                        end: 2
                    })
                ],
            }),
            Err(EditError::InvalidOperation)
        );

        // document should remain unchanged
        assert_eq!(doc, Document {
            content: String::from("Hello")
        });

        assert_eq!(
            doc.apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::new()
                    })
                ],
            }),
            Err(EditError::InvalidOperation)
        );
        
        // document should remain unchanged
        assert_eq!(doc, Document {
            content: String::from("Hello")
        });
    }

    #[test]
    fn can_apply() {
        let doc = Document {
            content: String::from("Hello"),
        };
        assert_eq!(
            doc.can_apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Insert (Insert {
                        pos: 0,
                        content: String::from("test")
                    })
                ]
            }),
            Ok(())
        );
        assert_eq!(
            doc.can_apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Insert (Insert {
                        pos: 100,
                        content: String::from("test")
                    })
                ]
            }),
            Err(EditError::OutsideDocument)
        );
        assert_eq!(
            doc.can_apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Insert (Insert {
                        pos: 100,
                        content: String::from("")
                    })
                ]
            }),
            Err(EditError::InvalidOperation)
        );
        assert_eq!(
            doc.can_apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Delete (Delete {
                        start: 0,
                        end: 2,
                    })
                ]
            }),
            Ok(())
        );
        assert_eq!(
            doc.can_apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Delete (Delete {
                        start: 100,
                        end: 102,
                    })
                ]
            }),
            Err(EditError::OutsideDocument)
        );
        assert_eq!(
            doc.can_apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Delete (Delete {
                        start: 0,
                        end: 6,
                    })
                ]
            }),
            Err(EditError::OutsideDocument)
        );
        assert_eq!(
            doc.can_apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Delete (Delete {
                        start: 1,
                        end: 1,
                    })
                ]
            }),
            Err(EditError::InvalidOperation)
        );
        assert_eq!(
            doc.can_apply(&Edit {
                author: 1,
                operations: vec![
                    Operation::Delete (Delete {
                        start: 2,
                        end: 1,
                    })
                ]
            }),
            Err(EditError::InvalidOperation)
        );
    }

    #[test]
    fn operation_has_effect() {
        assert!(Operation::Insert (Insert {pos: 0, content: String::from("test")}).has_effect());
        assert!(!Operation::Insert (Insert {pos: 0, content: String::from("")}).has_effect());
        assert!(Operation::Delete (Delete {start: 0, end: 10}).has_effect());
        assert!(!Operation::Delete (Delete {start: 0, end: 0}).has_effect());
    }

    #[test]
    fn operation_is_valid() {
        assert!(Operation::Insert (Insert {pos: 0, content: String::from("test")}).is_valid());
        assert!(!Operation::Insert (Insert {pos: 0, content: String::from("")}).is_valid());
        assert!(Operation::Delete (Delete {start: 0, end: 10}).is_valid());
        assert!(!Operation::Delete (Delete {start: 0, end: 0}).is_valid());
        assert!(!Operation::Delete (Delete {start: 10, end: 0}).is_valid());
    }

    
    #[test]
    fn transform_insert_before_insert() {
        let mut h = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("Test")
            })]
        };
        h.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 10,
                content: String::from("foo")
            })]
        });
        assert_eq!(h.operations, vec![Operation::Insert(Insert {
            pos: 0,
            content: String::from("Test")
        })]);
    }
    
    #[test]
    fn transform_insert_after_insert() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 10,
                content: String::from("Test")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 2,
                content: String::from("foo")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 13,
            content: String::from("Test")
        })]);
    }
    
    #[test]
    fn transform_inserts_at_same_point_checks_priority() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("Test")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("foo")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 8,
            content: String::from("Test")
        })]);
        
        let mut msg = Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("Test")
            })]
        };
        msg.transform(&Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("foo")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 5,
            content: String::from("Test")
        })]);
    }

    #[test]
    fn transform_insert_uses_char_index_not_byte_index() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("Test")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                // 2-byte unicode scalar value
                content: String::from("д")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            // 6 is char index, 7 would be byte index
            pos: 6,
            content: String::from("Test")
        })]);
    }

    #[test]
    fn transform_delete_delete_non_overlapping_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 5
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 10,
                end: 15
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 0,
            end: 5
        })]);
    }
     
    #[test]
    fn transform_delete_delete_non_overlapping_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 5,
                end: 10
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 1
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 4,
            end: 9
        })]);
    }

    #[test]
    fn transform_delete_delete_adjacent_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 2,
                end: 4
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
            start: 0,
            end: 2
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 0,
            end: 2
        })]);
    }
    
    #[test]
    fn transform_delete_delete_adjacent_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 3
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
            start: 3,
            end: 5
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 0,
            end: 3
        })]);
    }
    
    #[test]
    fn transform_delete_delete_overlapping_start() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 5,
                end: 15
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 10
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 0,
            end: 5
        })]);
    }

    #[test]
    fn transform_delete_delete_overlapping_end() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 4
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 2,
                end: 6
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 0,
            end: 2
        })]);
    }
 
    #[test]
    fn transform_delete_delete_subset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 5,
                end: 10
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 1,
                end: 20
            })]
        });
        assert_eq!(msg.operations, vec![]);
    }

    #[test]
    fn transform_delete_delete_superset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 17
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 5,
                end: 10
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 0,
            end: 12
        })]);
    }

    #[test]
    fn transform_insert_delete_non_overlapping_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("12345")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 10,
                end: 15
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 0,
            content: String::from("12345")
        })]);
    }
    
    #[test]
    fn transform_insert_delete_non_overlapping_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("foo")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 1
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 4,
            content: String::from("foo")
        })]);
    }

    #[test]
    fn transform_insert_delete_adjacent_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 2,
                content: String::from("ab")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 2
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 0,
            content: String::from("ab")
        })]);
    }
    
    #[test]
    fn transform_insert_delete_adjacent_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("foo")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 3,
                end: 5
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 0,
            content: String::from("foo")
        })]);
    }
    
    #[test]
    fn transform_insert_delete_overlapping_start() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("1234567890")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 10
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 0,
            content: String::from("1234567890")
        })]);
    }

    #[test]
    fn transform_insert_delete_overlapping_end() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("abcd")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 2,
                end: 6
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 0,
            content: String::from("abcd")
        })]);
    }

    #[test]
    fn transform_insert_delete_subset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("12345")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 1,
                end: 20
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 1,
            content: String::from("12345")
        })]);
    }

    #[test]
    fn transform_insert_delete_superset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("12345678901234567")
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Delete(Delete {
                start: 5,
                end: 10
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Insert(Insert {
            pos: 0,
            content: String::from("12345678901234567")
        })]);
    }

    #[test]
    fn transform_delete_insert_non_overlapping_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 5
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 10,
                content: String::from("12345")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 0,
            end: 5
        })]);
    }
    
    #[test]
    fn transform_delete_insert_non_overlapping_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 5,
                end: 8
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("a")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 6,
            end: 9
        })]);
    }

    #[test]
    fn transform_delete_insert_adjacent_before() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 2,
                end: 4
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("ab")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 4,
            end: 6
        })]);
    }
    
    #[test]
    fn transform_delete_insert_adjacent_after() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 3
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 3,
                content: String::from("ab")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 0,
            end: 3
        })]);
    }
    
    #[test]
    fn transform_delete_insert_same_start_position() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 2,
                end: 4
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 2,
                content: String::from("cd")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 4,
            end: 6
        })]);
    }
    
    #[test]
    fn transform_delete_insert_overlapping_start() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 5,
                end: 15
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 0,
                content: String::from("1234567890")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 15,
            end: 25
        })]);
    }

    #[test]
    fn transform_delete_insert_overlapping_end() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 4
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 2,
                content: String::from("abcd")
            })]
        });
        assert_eq!(msg.operations, vec![
            Operation::Delete(Delete {
                start: 0,
                end: 2
            }),
            // start and end indices are -2 because the first
            // operation will affect the second
            Operation::Delete(Delete {
                start: 4,
                end: 6
            }),
        ]);
    }

    #[test]
    fn transform_delete_insert_subset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 5,
                end: 10
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 1,
                content: String::from("12345678901234567890")
            })]
        });
        assert_eq!(msg.operations, vec![Operation::Delete(Delete {
            start: 25,
            end: 30
        })]);
    }

    #[test]
    fn transform_delete_insert_superset() {
        let mut msg = Edit {
            author: 1,
            operations: vec![Operation::Delete(Delete {
                start: 0,
                end: 17
            })]
        };
        msg.transform(&Edit {
            author: 2,
            operations: vec![Operation::Insert(Insert {
                pos: 5,
                content: String::from("12345")
            })]
        });
        assert_eq!(msg.operations, vec![
            Operation::Delete(Delete {
                start: 0,
                end: 5
            }),
            // start and end indices are -5 because the first
            // operation will affect the second
            Operation::Delete(Delete {
                start: 5,
                end: 17
            }),
        ]);
    }

    trait GenerateStrategy {
        fn generate_strategy(doc_size: usize) -> BoxedStrategy<Operation>;
    }

    impl GenerateStrategy for Delete {
        fn generate_strategy(doc_size: usize) -> BoxedStrategy<Operation> {
            (Range { start: 0, end: doc_size + 1 },
             Range { start: 0, end: doc_size + 1 })
                .prop_filter(
                    "Delete operation start index must be smaller than end index".to_owned(),
                    |v| v.0 < v.1
                )
                .prop_map(|v| Operation::Delete(Delete { start: v.0, end: v.1 }))
                .boxed()
        }
    }

    impl GenerateStrategy for Insert {
        fn generate_strategy(doc_size: usize) -> BoxedStrategy<Operation> {
            (Range { start: 0, end: doc_size }, ".{1,100}")
                .prop_map(|v| Operation::Insert(Insert { pos: v.0, content: v.1 }))
                .boxed()
        }
    }
    
    fn generate_document_data() -> BoxedStrategy<String> {
        ".{1,100}".prop_filter(
            "Document must contain at least one unicode scalar value".to_owned(),
            |v| v.chars().count() > 0
        ).boxed()
    }

    fn conflicting_operations<A, B>() -> BoxedStrategy<(String, Operation, Operation)>
    where A: GenerateStrategy,
          B: GenerateStrategy
    {
        generate_document_data()
            .prop_flat_map(|initial| {
                let len = initial.chars().count();
                (Just(initial),
                 <A as GenerateStrategy>::generate_strategy(len),
                 <B as GenerateStrategy>::generate_strategy(len),
                )
            })
            .boxed()
    }

    // helper function for proptest! block below
    fn check_order_of_application(initial: &str, op1: &Operation, op2: &Operation) {
        let doc = Document::from(initial);
        let mut a1 = Edit {
            author: 1,
            operations: vec![op1.clone()]
        };
        let b1 = Edit {
            author: 2,
            operations: vec![op2.clone()]
        };
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
