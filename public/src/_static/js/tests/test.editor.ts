import { assert } from "chai";
import { Editor } from "../editor";
import * as protocol from "../protocol";
import { TestConnection } from "./utils";

suite("Editor", () => {

    setup(function() {
        this.tmp = document.createElement("div");
        document.body.appendChild(this.tmp);
    });

    teardown(function() {
        document.body.removeChild(this.tmp);
    });

    test("codemirror editor contains textContent of tw-editor element", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "0");
        editor.setAttribute("participants", "[]");
        editor.textContent = "Example content";
        this.tmp.appendChild(editor);
        assert.equal(editor.content.getValue(), "Example content");
    });

    test("display participants loaded from attribute", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
            { id: 123, cursor_pos: 60 },
        ]));
        this.tmp.appendChild(editor);
        const items = editor.participants.querySelectorAll("li");
        assert.equal(items.length, 2);
        assert.equal(items[0].textContent, "Participant 1");
        assert.equal(items[1].textContent, "Participant 123");
    });

    test("create connection for session using initial-seq attribute", function(done) {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", "[]");
        this.tmp.appendChild(editor);
        if (editor.session) {
            editor.session.on("message", (msg) => {
                if (msg instanceof protocol.Connected) {
                    assert.equal(msg.id, 123);
                    if (editor.session) {
                        const conn = editor.session.connection as TestConnection;
                        assert.equal(conn.seq, 3);
                        assert.equal(editor.session.participantId, msg.id);
                        assert.deepEqual(editor.session.clientSeq, 0);
                        done();
                    }
                }
            });
            editor.session.connection.emit(
                "message",
                new protocol.Connected(123),
            );
        }
    });

    test("set local participant id on pariticpants element", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
        ]));
        this.tmp.appendChild(editor);
        if (editor.session) {
            editor.session.connection.emit(
                "message",
                new protocol.Connected(2),
            );
            const items = editor.participants.querySelectorAll("li");
            assert.equal(items.length, 2);
            assert.equal(items[0].textContent, "Participant 1");
            assert.equal(items[1].textContent, "Participant 2");
            assert.equal(items[1].className, "you");
        } else {
            assert.ok(false);
        }
    });

    test("Join event adds participant to list element", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
        ]));
        this.tmp.appendChild(editor);
        if (editor.session) {
            editor.session.connection.emit(
                "message",
                new protocol.Connected(2),
            );
            const before = editor.participants.querySelectorAll("li");
            assert.equal(before[0].textContent, "Participant 1");
            assert.equal(before[1].textContent, "Participant 2");
            assert.equal(before.length, 2);

            editor.session.connection.emit(
                "message",
                new protocol.ServerEvent(4, 0, new protocol.Join(3)),
            );
            const after = editor.participants.querySelectorAll("li");
            assert.equal(after[0].textContent, "Participant 1");
            assert.equal(after[1].textContent, "Participant 2");
            assert.equal(after[2].textContent, "Participant 3");
            assert.equal(after.length, 3);
        } else {
            assert.ok(false);
        }
    });

    test("Leave event removes participant from list element", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
            { id: 2, cursor_pos: 0 },
        ]));
        this.tmp.appendChild(editor);
        if (editor.session) {
            editor.session.connection.emit(
                "message",
                new protocol.Connected(3),
            );
            const before = editor.participants.querySelectorAll("li");
            assert.equal(before[0].textContent, "Participant 1");
            assert.equal(before[1].textContent, "Participant 2");
            assert.equal(before[2].textContent, "Participant 3");
            assert.equal(before.length, 3);

            editor.session.connection.emit(
                "message",
                new protocol.ServerEvent(4, 0, new protocol.Leave(1)),
            );
            const after = editor.participants.querySelectorAll("li");
            assert.equal(after[0].textContent, "Participant 2");
            assert.equal(after[1].textContent, "Participant 3");
            assert.equal(after.length, 2);
        } else {
            assert.ok(false);
        }
    });

    test("Content gets participants from editor attribute", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
            { id: 2, cursor_pos: 0 },
        ]));
        editor.textContent = "Hello";
        this.tmp.appendChild(editor);
        assert.deepEqual(
            Object.keys(editor.content.otherParticipants),
            ["1", "2"],
        );
        assert.equal(editor.content.seq, 3);
    });

    test("Leave event removes participant from Content element", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
            { id: 2, cursor_pos: 0 },
        ]));
        editor.textContent = "Hello";
        this.tmp.appendChild(editor);
        if (editor.session) {
            editor.session.connection.emit(
                "message",
                new protocol.Connected(3),
            );
            editor.session.connection.emit(
                "message",
                new protocol.ServerEvent(4, 0, new protocol.Leave(1)),
            );
            assert.deepEqual(
                Object.keys(editor.content.otherParticipants),
                ["2"],
            );
            assert.equal(editor.content.seq, 4);
        } else {
            assert.ok(false);
        }
    });

    test("Join event adds participant to Content element", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
            { id: 2, cursor_pos: 0 },
        ]));
        editor.textContent = "Hello";
        this.tmp.appendChild(editor);
        if (editor.session) {
            editor.session.connection.emit(
                "message",
                new protocol.Connected(3),
            );
            editor.session.connection.emit(
                "message",
                new protocol.ServerEvent(4, 0, new protocol.Join(4)),
            );
            assert.deepEqual(
                Object.keys(editor.content.otherParticipants),
                ["1", "2", "4"],
            );
            assert.equal(editor.content.seq, 4);
        } else {
            assert.ok(false);
        }
    });

    test("Send local edits over connection", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
            { id: 2, cursor_pos: 0 },
        ]));
        editor.textContent = "Hello";
        this.tmp.appendChild(editor);
        if (editor.session) {
            const doc = editor.content.codemirror.getDoc();
            const pos = doc.posFromIndex(5);
            doc.replaceRange(", world!", pos);
            const conn = editor.session.connection as TestConnection;
            assert.deepEqual(conn.sent, [
                new protocol.ClientEdit(3, 1, [
                    new protocol.Insert(5, ", world!"),
                ]),
            ]);
        } else {
            assert.ok(false);
        }
    });

    test("Apply edits received over connection", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "1");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
        ]));
        editor.textContent = "";
        this.tmp.appendChild(editor);
        if (editor.session) {
            editor.session.connection.emit(
                "message",
                new protocol.Connected(2),
            );
            editor.session.connection.emit(
                "message",
                new protocol.ServerEvent(
                    2,
                    0,
                    new protocol.Edit(1, [
                        new protocol.Insert(0, "Hello"),
                    ]),
                ),
            );
            assert.equal(
                editor.content.getValue(),
                "Hello",
            );
        } else {
            assert.ok(false);
        }
    });

    test("Don't send edits when applying a server event", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "1");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
        ]));
        editor.textContent = "";
        this.tmp.appendChild(editor);
        if (editor.session) {
            editor.session.connection.emit(
                "message",
                new protocol.Connected(2),
            );
            editor.session.connection.emit(
                "message",
                new protocol.ServerEvent(
                    2,
                    0,
                    new protocol.Edit(1, [
                        new protocol.Insert(0, "Hello"),
                    ]),
                ),
            );
            assert.equal(
                editor.content.getValue(),
                "Hello",
            );
            const conn = editor.session.connection as TestConnection;
            assert.deepEqual(conn.sent, []);
        } else {
            assert.ok(false);
        }
    });

    test("Use last applied event as parent seq when sending edits", function() {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "1");
        editor.setAttribute("participants", JSON.stringify([
            { id: 1, cursor_pos: 0 },
        ]));
        editor.textContent = "";
        this.tmp.appendChild(editor);
        if (editor.session) {
            editor.session.connection.emit(
                "message",
                new protocol.Connected(2),
            );
            editor.session.connection.emit(
                "message",
                new protocol.ServerEvent(
                    2,
                    0,
                    new protocol.Edit(1, [
                        new protocol.Insert(0, "Hello"),
                    ]),
                ),
            );
            assert.equal(
                editor.content.getValue(),
                "Hello",
            );
            const doc = editor.content.codemirror.getDoc();
            const pos = doc.posFromIndex(5);
            doc.replaceRange(", world", pos);
            assert.equal(
                editor.content.getValue(),
                "Hello, world",
            );
            editor.session.connection.emit(
                "message",
                new protocol.ServerEvent(
                    4,
                    1,
                    new protocol.Edit(1, [
                        new protocol.Insert(12, "!"),
                    ]),
                ),
            );
            assert.equal(
                editor.content.getValue(),
                "Hello, world!",
            );
            const start = doc.posFromIndex(7);
            const end = doc.posFromIndex(12);
            doc.replaceRange("galaxy", start, end);
            assert.equal(
                editor.content.getValue(),
                "Hello, galaxy!",
            );
            const conn = editor.session.connection as TestConnection;
            assert.deepEqual(JSON.stringify(conn.sent), JSON.stringify([
                new protocol.ClientEdit(2, 1, [
                    new protocol.Insert(5, ", world"),
                ]),
                new protocol.ClientEdit(4, 2, [
                    new protocol.Delete(7, 12),
                    new protocol.Insert(7, "galaxy"),
                ]),
            ]));
        } else {
            assert.ok(false);
        }
    });

});
