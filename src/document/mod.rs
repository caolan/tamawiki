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
                        Operation::Delete(Delete { start, end }) => {
                            if start > length || end > length {
                                return Err(EditError::OutsideDocument);
                            } else {
                                length -= end - start;
                            }
                        }
                        Operation::MoveCursor(MoveCursor { pos }) => {
                            if pos > length {
                                return Err(EditError::OutsideDocument);
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
            Operation::MoveCursor(ref op) => {
                for (id, participant) in self.participants.entries.iter_mut() {
                    if *id == author {
                        participant.cursor_pos = op.pos;
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
            Operation::MoveCursor(_) => true,
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
    /// Does the Edit from ParticipantId 'a' take precedence over the
    /// Edit from ParticipantId 'b' if operations conflict?
    fn has_priority(a: ParticipantId, b: ParticipantId) -> bool {
        a > b
    }

    /// Modifies the `operations` inside the Edit struct to
    /// accommodate a concurrent Edit entry whose operations have
    /// already been applied locally.
    fn transform(&mut self, other: &Edit) {
        let mut concurrent = other.operations.clone();
        let mut next_ops = vec![];
        let mut group = vec![];
        let mut next_group = vec![];
        let mut con_group = vec![];
        let mut next_con_group = vec![];
        let mut next_concurrent = vec![];

        for op in self.operations.drain(..) {
            group.push(op);
            for con in concurrent.drain(..) {
                for g in &group {
                    // clone here because we need the previous values
                    // when transforming `con` later
                    g.clone()
                        .transform(self.author, &con, other.author, &mut next_group);
                }

                // transform con for last value of group (i.e. not `next_group`)
                con_group.push(con);
                for g in &group {
                    for c in con_group.drain(..) {
                        c.transform(other.author, g, self.author, &mut next_con_group);
                    }
                    con_group.append(&mut next_con_group);
                }

                // store transformed concurrent operation for use with
                // next `op` value
                next_concurrent.append(&mut con_group);

                // need to clear group because items were cloned from
                // it earlier instead of using drain(..)
                group.clear();
                group.append(&mut next_group);
            }
            concurrent.append(&mut next_concurrent);
            next_ops.append(&mut group);
        }

        self.operations.append(&mut next_ops);

        // NOTE: at this point it is possible operations which no
        // longer affect a documents content (e.g. Deletes where
        // start == end, or Inserts with content = "") are included
        // in the new_operations vector. This is valid because those
        // operations may modify the cursor position of
        // participants and would have been accepted if applied in
        // a different order. To make sure the cursor positions
        // converge, these otherwise useless operations must be
        // retained.

        // TODO: perhaps use the new MoveCursor operation to
        // replace the empty operations described in the above
        // comment and have empty operations fail validation
    }
}

impl Operation {
    fn transform(
        self,
        author: ParticipantId,
        other_op: &Operation,
        other_author: ParticipantId,
        output: &mut Vec<Operation>,
    ) {
        println!("transforming: {:?} for {:?}", self, other_op);
        match self {
            Operation::Insert(this) => this.transform(author, other_op, other_author, output),
            Operation::Delete(this) => this.transform(author, other_op, other_author, output),
            Operation::MoveCursor(this) => this.transform(author, other_op, other_author, output),
        }
    }
}

impl Insert {
    /// The `scratch` and `result` arguments must be empty vectors. The
    /// output of the transform will be written to `result` and
    /// `scratch` will be cleared before returning.
    fn transform(
        mut self,
        author: ParticipantId,
        other_op: &Operation,
        other_author: ParticipantId,
        output: &mut Vec<Operation>,
    ) {
        match *other_op {
            Operation::Insert(ref other) => {
                if other.pos < self.pos
                    || (other.pos == self.pos && Edit::has_priority(other_author, author))
                {
                    self.pos += other.content.chars().count();
                }
                output.push(Operation::Insert(self));
            }
            Operation::Delete(ref other) => {
                if other.start < self.pos {
                    let end = cmp::min(self.pos, other.end);
                    self.pos -= end - other.start;
                }
                output.push(Operation::Insert(self));
            }
            Operation::MoveCursor(_) => {
                output.push(Operation::Insert(self));
            }
        }
    }
}

impl Delete {
    /// The `scratch` and `result` arguments must be empty vectors. The
    /// output of the transform will be written to `result` and
    /// `scratch` will be cleared before returning.
    fn transform(
        mut self,
        _author: ParticipantId,
        other_op: &Operation,
        _other_author: ParticipantId,
        output: &mut Vec<Operation>,
    ) {
        match *other_op {
            Operation::Insert(ref other) => {
                if other.pos < self.start {
                    let len = other.content.chars().count();
                    self.start += len;
                    self.end += len;
                    output.push(Operation::Delete(self));
                } else if other.pos < self.end && self.end - self.start > 0 {
                    // need to split the delete into two parts
                    // to avoid clobbering the new insert
                    // (only split when the delete has a range
                    // greater than 0, otherwise it can only
                    // produce a duplicate event)

                    // create a new operation for the first
                    // part of the range
                    let before = Delete {
                        start: self.start,
                        end: other.pos,
                    };

                    // adjust the current operation to cover
                    // the end second part of the range
                    let len = other.content.chars().count();
                    self.start = other.pos + len;
                    self.end = self.end + len;

                    // push the operation covering the first
                    // part of the range to transformed.
                    // This means it's applied after the
                    // second part of the range and the cursor
                    // ends in the correct position.
                    output.push(Operation::Delete(self));
                    output.push(Operation::Delete(before));
                } else {
                    output.push(Operation::Delete(self));
                }
            }
            Operation::Delete(ref other) => {
                let mut chars_deleted_before = if other.start < self.start {
                    let end = cmp::min(self.start, other.end);
                    end - other.start
                } else {
                    0
                };
                let mut chars_deleted_inside = 0;
                if other.start < self.start {
                    if other.end > self.start {
                        let end = cmp::min(self.end, other.end);
                        chars_deleted_inside = end - self.start;
                    }
                } else if other.start < self.end {
                    let end = cmp::min(self.end, other.end);
                    chars_deleted_inside = end - other.start;
                }
                self.start -= chars_deleted_before;
                self.end -= chars_deleted_before + chars_deleted_inside;
                output.push(Operation::Delete(self));
            }
            Operation::MoveCursor(_) => {
                output.push(Operation::Delete(self));
            }
        }
    }
}

impl MoveCursor {
    /// The `scratch` and `result` arguments must be empty vectors. The
    /// output of the transform will be written to `result` and
    /// `scratch` will be cleared before returning.
    fn transform(
        mut self,
        _author: ParticipantId,
        other_op: &Operation,
        _other_author: ParticipantId,
        output: &mut Vec<Operation>,
    ) {
        match *other_op {
            Operation::Insert(ref other) => {
                if other.pos < self.pos {
                    self.pos += other.content.chars().count();
                }
                output.push(Operation::MoveCursor(self));
            }
            Operation::Delete(ref other) => {
                if other.start < self.pos {
                    let end = cmp::min(self.pos, other.end);
                    self.pos -= end - other.start;
                }
                output.push(Operation::MoveCursor(self));
            }
            Operation::MoveCursor(_) => {
                output.push(Operation::MoveCursor(self));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // NOTE: these tests are generated by build.rs during cargo build
    // and use the JSON test specifications found in tests/shared
    include!(concat!(env!("OUT_DIR"), "/shared_tests.rs"));

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
        fn generate_strategy() -> BoxedStrategy<Operation>;
    }

    impl GenerateStrategy for Operation {
        fn generate_strategy() -> BoxedStrategy<Operation> {
            prop_oneof![
                Insert::generate_strategy(),
                Delete::generate_strategy(),
                MoveCursor::generate_strategy(),
            ].boxed()
        }
    }

    impl GenerateStrategy for Delete {
        fn generate_strategy() -> BoxedStrategy<Operation> {
            (0usize..10usize)
                .prop_flat_map(move |start| (Just(start), start..(10usize + 1usize)).boxed())
                .prop_map(|(start, end)| Operation::Delete(Delete { start, end }))
                .boxed()
        }
    }

    impl GenerateStrategy for Insert {
        fn generate_strategy() -> BoxedStrategy<Operation> {
            (0usize..(10usize + 1usize), ".{1,10}")
                .prop_map(|v| {
                    Operation::Insert(Insert {
                        pos: v.0,
                        content: v.1,
                    })
                }).boxed()
        }
    }

    impl GenerateStrategy for MoveCursor {
        fn generate_strategy() -> BoxedStrategy<Operation> {
            (0usize..(10usize + 1usize))
                .prop_map(|v| Operation::MoveCursor(MoveCursor { pos: v }))
                .boxed()
        }
    }

    fn generate_document_data(min_size: usize) -> BoxedStrategy<String> {
        proptest::collection::vec(proptest::char::any(), min_size..(min_size + 10))
            .prop_map(|v| -> String { v.into_iter().collect() })
            .prop_filter(
                "Document must contain at least one unicode scalar value".to_owned(),
                |v| v.chars().count() > 0,
            ).boxed()
    }

    // NOTE: only use this to generate a small number of operations,
    // and they will be in reverse order, with first to be applied at
    // the end of the vector
    fn operations_strategy(num: usize) -> BoxedStrategy<(Vec<Operation>, usize)> {
        if num == 0 {
            (Just(vec![]), Just(0)).boxed()
        } else {
            Operation::generate_strategy()
                .prop_flat_map(move |op| {
                    operations_strategy(num - 1).prop_map(move |(mut ops, req_size)| {
                        let next_req_size = match op {
                            Operation::Insert(ref data) => {
                                let len = data.content.chars().count();
                                let tmp = if len < req_size { req_size - len } else { 0 };
                                cmp::max(tmp, data.pos)
                            }
                            Operation::Delete(ref data) => {
                                let removed = data.end - data.start;
                                cmp::max(req_size + removed, data.end)
                            }
                            Operation::MoveCursor(ref data) => cmp::max(req_size, data.pos),
                        };
                        ops.push(op.clone());
                        (ops, next_req_size)
                    })
                }).boxed()
        }
    }

    fn conflicting_operations(
        max: usize,
    ) -> BoxedStrategy<(String, Vec<Operation>, Vec<Operation>)> {
        (
            (1..(max + 1)).prop_flat_map(|num| operations_strategy(num)),
            (1..(max + 1)).prop_flat_map(|num| operations_strategy(num)),
        )
            .prop_flat_map(move |((mut ops1, req_size1), (mut ops2, req_size2))| {
                let req_size = cmp::max(req_size1, req_size2);
                ops1.reverse();
                ops2.reverse();
                (generate_document_data(req_size), Just(ops1), Just(ops2))
            }).boxed()
    }

    proptest! {

        #[test]
        fn check_application_order_for_one_conflicting_operation_each
            ((ref initial, ref ops1, ref ops2) in conflicting_operations(1))
        {
            let mut doc = Document::from(initial.as_str());
            doc.apply(&Event::Join(Join { id: 1 })).unwrap();
            doc.apply(&Event::Join(Join { id: 2 })).unwrap();
            let mut a1 = Event::Edit(Edit {
                author: 1,
                operations: ops1.clone(),
            });
            let b1 = Event::Edit(Edit {
                author: 2,
                operations: ops2.clone(),
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

        #[test]
        fn check_application_order_for_multiple_conflicting_operations_each_single_edit
            ((ref initial, ref ops1, ref ops2) in conflicting_operations(2))
        {
            let mut doc = Document::from(initial.as_str());
            doc.apply(&Event::Join(Join { id: 1 })).unwrap();
            doc.apply(&Event::Join(Join { id: 2 })).unwrap();
            let mut a1 = Event::Edit(Edit {
                author: 1,
                operations: ops1.clone(),
            });
            let b1 = Event::Edit(Edit {
                author: 2,
                operations: ops2.clone(),
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
            prop_assert_eq!(doc1, doc2);
        }

    }

}
