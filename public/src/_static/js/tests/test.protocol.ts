import { assert } from "chai";
import * as protocol from "../protocol";

suite("protocol", () => {

    test("ClientEdit.fromJSON / toJSON", function() {
        const msg = new protocol.ClientEdit(1, 2, [
            new protocol.Insert(0, "Hello"),
        ]);
        const serialized = msg.toJSON();
        const deserialized = protocol.ClientEdit.fromJSON(serialized);
        assert.deepEqual(serialized, {
            ClientEdit: {
                client_seq: 2,
                operations: [
                    { Insert: { pos: 0, content: "Hello" } },
                ],
                parent_seq: 1,
            },
        });
        assert.deepEqual(deserialized, msg);
        assert.ok(deserialized instanceof protocol.ClientEdit);
    });

    test("ClientMessage.fromJSON", function() {
        const msg = new protocol.ClientEdit(1, 2, [
            new protocol.Insert(0, "Hello"),
        ]);
        const serialized = msg.toJSON();
        const deserialized = protocol.ClientMessage.fromJSON(serialized);
        assert.deepEqual(deserialized, msg);
        assert.ok(deserialized instanceof protocol.ClientMessage);
    });

    test("Connected.fromJSON / toJSON", function() {
        const msg = new protocol.Connected(1);
        const serialized = msg.toJSON();
        const deserialized = protocol.Connected.fromJSON(serialized);
        assert.deepEqual(serialized, { Connected: { id: 1 } });
        assert.deepEqual(deserialized, msg);
        assert.ok(deserialized instanceof protocol.Connected);
    });

    test("ServerEvent.fromJSON / toJSON", function() {
        const msg = new protocol.ServerEvent(1, 0, new protocol.Join(1));
        const serialized = msg.toJSON();
        const deserialized = protocol.ServerEvent.fromJSON(serialized);
        assert.deepEqual(serialized, {
            Event: {
                client_seq: 0,
                event: { Join: { id: 1 } },
                seq: 1,
            },
        });
        assert.deepEqual(deserialized, msg);
        assert.ok(deserialized instanceof protocol.ServerEvent);
    });

    test("ServerMessage.fromJSON", function() {
        const msg = new protocol.Connected(1);
        const serialized = msg.toJSON();
        const deserialized = protocol.ServerMessage.fromJSON(serialized);
        assert.deepEqual(deserialized, msg);
        assert.ok(deserialized instanceof protocol.ServerMessage);
        const msg2 = new protocol.ServerEvent(1, 0, new protocol.Join(1));
        const serialized2 = msg2.toJSON();
        const deserialized2 = protocol.ServerMessage.fromJSON(serialized2);
        assert.deepEqual(deserialized2, msg2);
        assert.ok(deserialized2 instanceof protocol.ServerMessage);
    });

    test("Participant.fromJSON / toJSON", function() {
        const participant = new protocol.Participant(123, 40);
        const serialized = participant.toJSON();
        const deserialized = protocol.Participant.fromJSON(serialized);
        assert.deepEqual(serialized, { id: 123, cursor_pos: 40 });
        assert.deepEqual(deserialized, participant);
        assert.ok(deserialized instanceof protocol.Participant);
    });

    test("Document.fromJSON / toJSON", function() {
        const doc = new protocol.Document("Hello, world!", [
            new protocol.Participant(1, 0),
            new protocol.Participant(123, 40),
        ]);
        const serialized = doc.toJSON();
        const deserialized = protocol.Document.fromJSON(serialized);
        assert.deepEqual(serialized, {
            content: "Hello, world!",
            participants: [
                { id: 1, cursor_pos: 0 },
                { id: 123, cursor_pos: 40 },
            ],
        });
        assert.deepEqual(deserialized, doc);
        assert.ok(deserialized instanceof protocol.Document);
    });

    test("Insert.fromJSON / toJSON", function() {
        const ins = new protocol.Insert(10, "hello");
        const serialized = ins.toJSON();
        const deserialized = protocol.Insert.fromJSON(serialized);
        assert.deepEqual(serialized, { Insert: { pos: 10, content: "hello" } });
        assert.deepEqual(deserialized, ins);
        assert.ok(deserialized instanceof protocol.Insert);
    });

    test("Delete.fromJSON / toJSON", function() {
        const del = new protocol.Delete(10, 20);
        const serialized = del.toJSON();
        const deserialized = protocol.Delete.fromJSON(serialized);
        assert.deepEqual(serialized, { Delete: { start: 10, end: 20 } });
        assert.deepEqual(deserialized, del);
        assert.ok(deserialized instanceof protocol.Delete);
    });

    test("Operation.fromJSON / toJSON", function() {
        const ins = new protocol.Insert(10, "hello");
        const deserializedIns = protocol.Operation.fromJSON(ins.toJSON());
        assert.deepEqual(deserializedIns, ins);
        assert.ok(deserializedIns instanceof protocol.Insert);

        const del = new protocol.Delete(10, 20);
        const deserializedDel = protocol.Operation.fromJSON(del.toJSON());
        assert.deepEqual(deserializedDel, del);
        assert.ok(deserializedDel instanceof protocol.Delete);

        assert.throws(() => {
            protocol.Operation.fromJSON({ Foo: { operation: "unknown" } });
        }, /Unknown Operation type/);
    });

    test("Join.fromJSON / toJSON", function() {
        const join = new protocol.Join(123);
        const serialized = join.toJSON();
        const deserialized = protocol.Join.fromJSON(serialized);
        assert.deepEqual(serialized, { Join: { id: 123 } });
        assert.deepEqual(deserialized, join);
        assert.ok(deserialized instanceof protocol.Join);
    });

    test("Leave.fromJSON / toJSON", function() {
        const leave = new protocol.Leave(123);
        const serialized = leave.toJSON();
        const deserialized = protocol.Leave.fromJSON(serialized);
        assert.deepEqual(serialized, { Leave: { id: 123 } });
        assert.deepEqual(deserialized, leave);
        assert.ok(deserialized instanceof protocol.Leave);
    });

    test("Edit.fromJSON / toJSON", function() {
        const edit = new protocol.Edit(1, [
            new protocol.Insert(10, "hello"),
            new protocol.Delete(2, 3),
        ]);
        const serialized = edit.toJSON();
        const deserialized = protocol.Edit.fromJSON(serialized);
        assert.deepEqual(serialized, {
            Edit: {
                author: 1,
                operations: [
                    { Insert: { pos: 10, content: "hello" } },
                    { Delete: { start: 2, end: 3 } },
                ],
            },
        });
        assert.deepEqual(deserialized, edit);
        assert.ok(deserialized instanceof protocol.Edit);
    });

    test("Event.fromJSON / toJSON", function() {
        const join = new protocol.Join(123);
        const deserializedJoin = protocol.Event.fromJSON(join.toJSON());
        assert.deepEqual(deserializedJoin, join);
        assert.ok(deserializedJoin instanceof protocol.Join);

        const leave = new protocol.Leave(123);
        const deserializedLeave = protocol.Event.fromJSON(leave.toJSON());
        assert.deepEqual(deserializedLeave, leave);
        assert.ok(deserializedLeave instanceof protocol.Leave);

        const edit = new protocol.Edit(1, [
            new protocol.Insert(10, "hello"),
            new protocol.Delete(2, 3),
        ]);
        const deserializedEdit = protocol.Edit.fromJSON(edit.toJSON());
        assert.deepEqual(deserializedEdit, edit);
        assert.ok(deserializedEdit instanceof protocol.Edit);

        assert.throws(() => {
            protocol.Event.fromJSON({ Foo: { event: "unknown" } });
        }, /Unknown Event type/);
    });

});
