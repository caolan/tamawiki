import { assert } from "chai";
import { Session } from "../session";
import { TestConnection } from "./utils";

import {
    ClientEdit,
    Connected,
    Delete,
    Edit,
    Event,
    Insert,
    MoveCursor,
    ServerEvent,
    ServerMessage,
} from "../protocol";

suite("Session", () => {

    test("transform incoming messages", function() {
        const edits: Event[] = [];
        const conn = new TestConnection("path", 1);
        const session = new Session(1, conn);
        session.on("message", (msg: ServerMessage) => {
            if (msg instanceof ServerEvent) {
                edits.push(msg.event);
            }
        });
        conn.emit("message", new Connected(2));
        conn.emit("message", new ServerEvent(
            2,
            0,
            new Edit(1, [
                new Insert(0, "Hello"),
            ]),
        ));
        session.write([new Delete(1, 5)]);
        session.flush();
        session.write([new Insert(1, "i")]);
        session.flush();
        conn.emit("message", new ServerEvent(
            2,
            0,
            new Edit(1, [
                new Insert(5, ", world"),
            ]),
        ));
        conn.emit("message", new ServerEvent(
            3,
            1, // this message acknowledges the first client edit
            new Edit(1, [
                new Insert(9, "!"),
            ]),
        ));
        assert.deepEqual(
            JSON.stringify(edits),
            JSON.stringify([
                new Edit(1, [new Insert(0, "Hello")]),
                new Edit(1, [new Insert(2, ", world")]),
                new Edit(1, [new Insert(10, "!")]),
            ]),
        );
    });

    test("clear buffered ClientEdits once acknowledged by server", function() {
        const conn = new TestConnection("path", 1);
        const session = new Session(1, conn);
        conn.emit("message", new Connected(2));
        session.write([new Delete(1, 5)]);
        session.flush();
        session.write([new Delete(0, 1)]);
        session.flush();
        assert.equal(session.sent.length, 2);
        conn.emit("message", new ServerEvent(
            2,
            0,
            new Edit(1, [
                new Insert(0, "a"),
            ]),
        ));
        assert.equal(session.sent.length, 2);
        conn.emit("message", new ServerEvent(
            3,
            1,
            new Edit(1, [
                new Insert(1, "b"),
            ]),
        ));
        assert.equal(session.sent.length, 1);
        conn.emit("message", new ServerEvent(
            4,
            2,
            new Edit(1, [
                new Insert(2, "c"),
            ]),
        ));
        assert.equal(session.sent.length, 0);
        session.write([new Insert(3, "d")]);
        session.flush();
        assert.equal(session.sent.length, 1);
    });

    suite("flush", function() {

        test("remove duplicate empty Deletes", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new Delete(0, 0), new Delete(0, 0)]);
            session.flush();
            assert.deepEqual(conn.sent, [
                new ClientEdit(1, 1, [new Delete(0, 0)]),
            ]);
        });

        test("remove duplicate empty Inserts", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new Insert(0, ""), new Insert(0, "")]);
            session.flush();
            assert.deepEqual(conn.sent, [
                new ClientEdit(1, 1, [new Insert(0, "")]),
            ]);
        });

        test("remove empty Delete before Insert with same ending cursor position", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new Delete(0, 0)]);
            session.write([new Insert(0, "")]);
            session.flush();
            assert.deepEqual(conn.sent, [
                new ClientEdit(1, 1, [new Insert(0, "")]),
            ]);
        });

        test("remove empty Insert before Delete with same ending cursor position", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new Insert(0, "")]);
            session.write([new Delete(0, 0)]);
            session.flush();
            assert.deepEqual(JSON.stringify(conn.sent), JSON.stringify([
                new ClientEdit(1, 1, [new Delete(0, 0)]),
            ]));
        });

        test("remove MoveCursor with same cursor end position as previous Insert", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new Insert(0, "test")]);
            session.write([new MoveCursor(4)]);
            session.flush();
            assert.deepEqual(conn.sent, [
                new ClientEdit(1, 1, [new Insert(0, "test")]),
            ]);
        });

        test("remove MoveCursor with same cursor end position as previous Delete", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new Delete(2, 4)]);
            session.write([new MoveCursor(2), new MoveCursor(2)]);
            session.flush();
            assert.deepEqual(conn.sent, [
                new ClientEdit(1, 1, [new Delete(2, 4)]),
            ]);
        });

        test("empty Insert which moves cursor is still valid", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new Insert(2, "foo")]);
            session.write([new Insert(4, "")]);
            session.flush();
            assert.deepEqual(conn.sent, [
                new ClientEdit(1, 1, [
                    new Insert(2, "foo"),
                    new Insert(4, ""),
                ]),
            ]);
        });

        test("empty Delete which moves cursor is still valid", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new Delete(2, 2)]);
            session.write([new Delete(4, 4)]);
            session.flush();
            assert.deepEqual(conn.sent, [
                new ClientEdit(1, 1, [new Delete(4, 4)]),
            ]);
        });

        test("remove MoveCursor if any operation follow it", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new MoveCursor(4)]);
            session.write([new Delete(1, 1)]);
            session.flush();
            assert.deepEqual(conn.sent, [
                new ClientEdit(1, 1, [new Delete(1, 1)]),
            ]);
        });

        test("remove empty Insert if any operation follows it", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new Insert(3, "")]);
            session.write([new Delete(1, 1)]);
            session.flush();
            assert.deepEqual(conn.sent, [
                new ClientEdit(1, 1, [new Delete(1, 1)]),
            ]);
        });

        test("remove empty Delete if any operation follows it", function() {
            const conn = new TestConnection("path", 1);
            const session = new Session(1, conn);
            session.write([new Delete(3, 3)]);
            session.write([new Insert(1, "test")]);
            session.flush();
            assert.deepEqual(conn.sent, [
                new ClientEdit(1, 1, [new Insert(1, "test")]),
            ]);
        });

    });

    test("remote event may make a local event valid that otherwise had no effect", function() {
        const conn = new TestConnection("path", 1);
        const session = new Session(1, conn);
        conn.emit("message", new Connected(1));

        session.write([new Insert(0, "test")]);
        session.flush();
        assert.deepEqual(conn.sent, [
            new ClientEdit(1, 1, [new Insert(0, "test")]),
        ]);
        conn.emit("message", new ServerEvent(
            2,
            1,
            new Edit(1, [
                new Insert(0, "a"),
            ]),
        ));
        session.write([new MoveCursor(4)]);
        session.flush();
        assert.deepEqual(conn.sent, [
            new ClientEdit(1, 1, [new Insert(0, "test")]),
            new ClientEdit(2, 2, [new MoveCursor(4)]),
        ]);
    });

});
