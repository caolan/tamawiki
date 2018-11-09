import { assert } from "chai";
import { Connected, Delete, Edit, Event, Insert, ServerEvent, ServerMessage } from "../protocol";
import { Session } from "../session";
import { TestConnection } from "./utils";

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
        session.send([new Delete(1, 5)]);
        session.send([new Insert(1, "i")]);
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
        session.send([new Delete(1, 5)]);
        session.send([new Delete(0, 1)]);
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
        session.send([new Insert(3, "d")]);
        assert.equal(session.sent.length, 1);
    });

});
