import { assert } from "chai";
import * as protocol from "../protocol";

suite("protocol", () => {

    test("Insert.fromJSON / toJSON", function () {
        const ins = new protocol.Insert(10, "hello");
        const serialized = ins.toJSON();
        const deserialized = protocol.Insert.fromJSON(serialized);
        assert.deepEqual(serialized, {Insert: {pos: 10, content: "hello"}});
        assert.deepEqual(deserialized, ins);
        assert.ok(deserialized instanceof protocol.Insert);
    });

    test("Delete.fromJSON / toJSON", function () {
        const del = new protocol.Delete(10, 20);
        const serialized = del.toJSON();
        const deserialized = protocol.Delete.fromJSON(serialized);
        assert.deepEqual(serialized, {Delete: {start: 10, end: 20}});
        assert.deepEqual(deserialized, del);
        assert.ok(deserialized instanceof protocol.Delete);
    });

    test("Operation.fromJSON / toJSON", function () {
        const ins = new protocol.Insert(10, "hello");
        const deserialized_ins = protocol.Operation.fromJSON(ins.toJSON());
        assert.deepEqual(deserialized_ins, ins);
        assert.ok(deserialized_ins instanceof protocol.Insert);
        
        const del = new protocol.Delete(10, 20);
        const deserialized_del = protocol.Operation.fromJSON(del.toJSON());
        assert.deepEqual(deserialized_del, del);
        assert.ok(deserialized_del instanceof protocol.Delete);

        assert.throws(() => {
            protocol.Operation.fromJSON({Foo: {operation: "unknown"}});
        }, /Unknown Operation type/);
    });

    test("Join.fromJSON / toJSON", function () {
        const join = new protocol.Join(123);
        const serialized = join.toJSON();
        const deserialized = protocol.Join.fromJSON(serialized);
        assert.deepEqual(serialized, {Join: {id: 123}});
        assert.deepEqual(deserialized, join);
        assert.ok(deserialized instanceof protocol.Join);
    });

    test("Leave.fromJSON / toJSON", function () {
        const leave = new protocol.Leave(123);
        const serialized = leave.toJSON();
        const deserialized = protocol.Leave.fromJSON(serialized);
        assert.deepEqual(serialized, {Leave: {id: 123}});
        assert.deepEqual(deserialized, leave);
        assert.ok(deserialized instanceof protocol.Leave);
    });

    test("Edit.fromJSON / toJSON", function () {
        const edit = new protocol.Edit(1, [
            new protocol.Insert(10, "hello"),
            new protocol.Delete(2, 3),
        ]);
        const serialized = edit.toJSON();
        const deserialized = protocol.Edit.fromJSON(serialized);
        assert.deepEqual(serialized, {Edit: {author: 1, operations: [
            {Insert: {pos: 10, content: "hello"}},
            {Delete: {start: 2, end: 3}},
        ]}});
        assert.deepEqual(deserialized, edit);
        assert.ok(deserialized instanceof protocol.Edit);
    });

    test("Event.fromJSON / toJSON", function () {
        const join = new protocol.Join(123);
        const deserialized_join = protocol.Event.fromJSON(join.toJSON());
        assert.deepEqual(deserialized_join, join);
        assert.ok(deserialized_join instanceof protocol.Join);
        
        const leave = new protocol.Leave(123);
        const deserialized_leave = protocol.Event.fromJSON(leave.toJSON());
        assert.deepEqual(deserialized_leave, leave);
        assert.ok(deserialized_leave instanceof protocol.Leave);
        
        const edit = new protocol.Edit(1, [
            new protocol.Insert(10, "hello"),
            new protocol.Delete(2, 3),
        ]);
        const deserialized_edit = protocol.Edit.fromJSON(edit.toJSON());
        assert.deepEqual(deserialized_edit, edit);
        assert.ok(deserialized_edit instanceof protocol.Edit);

        assert.throws(() => {
            protocol.Event.fromJSON({Foo: {event: "unknown"}});
        }, /Unknown Event type/);
    });


});
