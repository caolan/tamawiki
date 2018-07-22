//! This module describes a Document, which represents some String
//! content at a point in time, and Operations, which can be applied
//! to the Document to update it's content.

use std::error;
use std::fmt;

/// Represents the current state of a document
#[derive(Debug, PartialEq, Default)]
pub struct Document {
    pub content: String
}

impl Document {
    /// Applies an Update to the Document's content. Either all
    /// Operations contained in the Update are applied, or no
    /// Operations are applied and an UpdateError is returned.
    pub fn apply(&mut self, update: &Update) -> Result<(), UpdateError> {
        self.can_apply(&update)?;
        
        for op in &update.operations {
            self.perform_operation(op);
        }
        Ok(())
    }

    /// Checks that every operation inside the update can be cleanly
    /// applied to the document, without making any changes to the
    /// document content.
    pub fn can_apply(&self, update: &Update) -> Result<(), UpdateError> {
        let mut length = self.content.chars().count();

        for op in &update.operations {
            if !op.is_valid() {
                return Err(UpdateError::InvalidOperation);
            }
            match *op {
                Operation::Insert(Insert {pos, ref content}) => {
                    if pos > length {
                        return Err(UpdateError::OutsideDocument);
                    } else {
                        length += content.chars().count();
                    }
                },
                Operation::Delete(Delete {start, end}) => {
                    if start >= length || end > length {
                        return Err(UpdateError::OutsideDocument);
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

/// Inserts new content at a single position in the Document
#[derive(Debug, PartialEq)]
pub struct Insert {
    /// Insert position as number of Unicode Scalar Values preceeding
    /// the Insert operation, from the beginning of the document (not
    /// byte position, or number of grapheme clusters)
    pub pos: usize,
    pub content: String,
}

/// Deletes a region of content from the Document
#[derive(Debug, PartialEq)]
pub struct Delete {
    /// First Unicode Scalar Value to remove in range
    pub start: usize,
    /// Unicode Scalar Value to end the delete operation on (exclusive
    /// of the 'end' character)
    pub end: usize,
}

/// Describes incremental changes to a Document's content
#[derive(Debug, PartialEq)]
pub enum Operation {
    Insert(Insert),
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
    /// still raise an UpdateError when applied to a specific Document
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
pub enum UpdateError {
    /// The Operation's position or range falls outside the current
    /// Document.
    OutsideDocument,
    /// The operation is invalid and could not be applied meaningfully
    /// to any document.
    InvalidOperation,
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UpdateError::OutsideDocument =>
                write!(f, "The operation's area of effect falls outside the document"),
            UpdateError::InvalidOperation =>
                write!(f, "The operation is invalid"),
        }
    }
}

impl error::Error for UpdateError {
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

/// An Update combines multiple operations into a single Document
/// change (i.e. all the operations are applied together, or not at
/// all).
#[derive(Debug, PartialEq)]
pub struct Update {
    pub operations: Vec<Operation>,
}


#[cfg(test)]
mod tests {
    use super::*;

    fn operation_test(initial: &'static str, op: Operation, expected: &'static str) {
        let mut doc = Document {
            content: String::from(initial),
        };
        doc.apply(&Update {
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
            doc.apply(&Update {
                operations: vec![
                    Operation::Delete(Delete {
                        start: 3,
                        end: 7
                    })
                ],
            }),
            Err(UpdateError::OutsideDocument)
        );
        // document should remain unchanged
        assert_eq!(doc, Document {
            content: String::from("foobar")
        });
        assert_eq!(
            doc.apply(&Update {
                operations: vec![
                    Operation::Delete(Delete {
                        start: 7,
                        end: 10
                    })
                ],
            }),
            Err(UpdateError::OutsideDocument)
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
            doc.apply(&Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 8,
                        content: String::from("test")
                    })
                ],
            }),
            Err(UpdateError::OutsideDocument)
        );
        // document should be unchanged
        assert_eq!(doc, Document {
            content: String::from("foobar")
        });
    }

    #[test]
    fn apply_multiple_operations_in_single_update() {
        let mut doc = Document {
            content: String::from("Hello"),
        };
        doc.apply(&Update {
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
    fn apply_update_with_single_failing_operation() {
        let mut doc = Document {
            content: String::from("a"),
        };
        assert_eq!(
            doc.apply(&Update {
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
            Err(UpdateError::OutsideDocument)
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
        doc.apply(&Update {
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
            doc.apply(&Update {
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
            Err(UpdateError::OutsideDocument)
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
            doc.apply(&Update {
                operations: vec![
                    Operation::Delete(Delete {
                        start: 2,
                        end: 2
                    })
                ],
            }),
            Err(UpdateError::InvalidOperation)
        );

        // document should remain unchanged
        assert_eq!(doc, Document {
            content: String::from("Hello")
        });

        assert_eq!(
            doc.apply(&Update {
                operations: vec![
                    Operation::Insert(Insert {
                        pos: 0,
                        content: String::new()
                    })
                ],
            }),
            Err(UpdateError::InvalidOperation)
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
            doc.can_apply(&Update {
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
            doc.can_apply(&Update {
                operations: vec![
                    Operation::Insert (Insert {
                        pos: 100,
                        content: String::from("test")
                    })
                ]
            }),
            Err(UpdateError::OutsideDocument)
        );
        assert_eq!(
            doc.can_apply(&Update {
                operations: vec![
                    Operation::Insert (Insert {
                        pos: 100,
                        content: String::from("")
                    })
                ]
            }),
            Err(UpdateError::InvalidOperation)
        );
        assert_eq!(
            doc.can_apply(&Update {
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
            doc.can_apply(&Update {
                operations: vec![
                    Operation::Delete (Delete {
                        start: 100,
                        end: 102,
                    })
                ]
            }),
            Err(UpdateError::OutsideDocument)
        );
        assert_eq!(
            doc.can_apply(&Update {
                operations: vec![
                    Operation::Delete (Delete {
                        start: 0,
                        end: 6,
                    })
                ]
            }),
            Err(UpdateError::OutsideDocument)
        );
        assert_eq!(
            doc.can_apply(&Update {
                operations: vec![
                    Operation::Delete (Delete {
                        start: 1,
                        end: 1,
                    })
                ]
            }),
            Err(UpdateError::InvalidOperation)
        );
        assert_eq!(
            doc.can_apply(&Update {
                operations: vec![
                    Operation::Delete (Delete {
                        start: 2,
                        end: 1,
                    })
                ]
            }),
            Err(UpdateError::InvalidOperation)
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

}
