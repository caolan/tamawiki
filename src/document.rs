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
    /// Applies an Operation to the Document's content, updating the
    /// Document struct in-place. If the Operation cannot be applied
    /// cleanly and OperationError is returned.
    fn apply(&mut self, op: &Operation) -> Result<(), OperationError> {
        match *op {
            Operation::Insert(ref op) => {
                match self.content.char_indices().nth(op.pos) {
                    Some((byte_pos, _)) => {
                        self.content.insert_str(byte_pos, &op.content);
                        Ok(())
                    },
                    None => {
                        if op.pos == self.content.chars().count() {
                            self.content.push_str(&op.content);
                            Ok(())
                        } else {
                            Err(OperationError::OutsideDocument)
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
                    Ok(())
                } else {
                    Err(OperationError::OutsideDocument)
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

/// Errors conditions which may occur when applying an Operation to a
/// Document
#[derive(Debug, PartialEq)]
pub enum OperationError {
    /// The Operation's position or range falls outside the Document
    OutsideDocument,
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OperationError::OutsideDocument =>
                write!(f, "The operation's area of effect falls outside the document"),
        }
    }
}

impl error::Error for OperationError {
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn operation_test(initial: &'static str, op: Operation, expected: &'static str) {
        let mut doc = Document {
            content: String::from(initial),
        };
        doc.apply(&op).unwrap();
        assert_eq!(doc.content, expected);
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
            doc.apply(&Operation::Delete(Delete {
                start: 3,
                end: 7
            })),
            Err(OperationError::OutsideDocument)
        );
        // content should be unchanged
        assert_eq!(doc.content, String::from("foobar"));
        assert_eq!(
            doc.apply(&Operation::Delete(Delete {
                start: 7,
                end: 10
            })),
            Err(OperationError::OutsideDocument)
        );
        // content should be unchanged
        assert_eq!(doc.content, String::from("foobar"));
    }

    
    #[test]
    fn insert_outside_of_bounds() {
        let mut doc = Document {
            content: String::from("foobar"),
        };
        assert_eq!(
            doc.apply(&Operation::Insert(Insert {
                pos: 8,
                content: String::from("test")
            })),
            Err(OperationError::OutsideDocument)
        );
        // content should be unchanged
        assert_eq!(doc.content, String::from("foobar"));
    }

}
