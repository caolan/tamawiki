import { assert } from "chai";
import { Connection } from "../connection";
import { Editor } from "../editor";
import { ClientMessage, Connected, Join, Leave, ServerEvent } from "../protocol";

suite("Editor", () => {

    class TestConnection extends Connection {
        constructor(
            public path: string,
            public seq: number) { super(); }

        public send(msg: ClientMessage): void {
            console.log(msg);
        }
    }

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
            editor.session.on("connected", (id) => {
                assert.equal(id, 123);
                if (editor.session) {
                    assert.equal(editor.session.participantId, id);
                    assert.deepEqual(editor.session.seq, 3);
                    assert.deepEqual(editor.session.clientSeq, 0);
                } else {
                    assert.ok(false);
                }
                done();
            });
            editor.session.connection.emit(
                "message",
                new Connected(123),
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
                new Connected(2),
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
            const before = editor.participants.querySelectorAll("li");
            assert.equal(before[0].textContent, "Participant 1");
            assert.equal(before.length, 1);

            editor.session.connection.emit(
                "message",
                new ServerEvent(4, 0, new Join(2)),
            );
            const after = editor.participants.querySelectorAll("li");
            assert.equal(after[0].textContent, "Participant 1");
            assert.equal(after[1].textContent, "Participant 2");
            assert.equal(after.length, 2);
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
            const before = editor.participants.querySelectorAll("li");
            assert.equal(before[0].textContent, "Participant 1");
            assert.equal(before[1].textContent, "Participant 2");
            assert.equal(before.length, 2);

            editor.session.connection.emit(
                "message",
                new ServerEvent(4, 0, new Leave(1)),
            );
            const after = editor.participants.querySelectorAll("li");
            assert.equal(after[0].textContent, "Participant 2");
            assert.equal(after.length, 1);
        } else {
            assert.ok(false);
        }
    });

});
