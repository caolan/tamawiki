import { assert } from "chai";
import { Editor } from "../editor";

suite("Editor", () => {

    class TestConnection {
        constructor (public path: string, public seq: number) {}
    }

    setup(function () {
        this.tmp = document.createElement("div");
        document.body.appendChild(this.tmp);
    });

    teardown(function () {
        document.body.removeChild(this.tmp);
    });

    test("codemirror editor contains textContent of tw-editor element", function () {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "0");
        editor.setAttribute("participants", "[]");
        editor.textContent = "Example content";
        this.tmp.appendChild(editor);
        assert.equal(editor.content.getValue(), "Example content");
    });

    test("display participants loaded from attribute", function () {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", JSON.stringify([
            {id: 1, cursor_pos: 0},
            {id: 123, cursor_pos: 60},
        ]));
        this.tmp.appendChild(editor);
        const items = editor.participants.querySelectorAll("li");
        assert.equal(items[0].textContent, "Participant 1");
        assert.equal(items[1].textContent, "Participant 123");
    });

    test("create connection using initial sequence id from attribute", function () {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", "[]");
        this.tmp.appendChild(editor);
        if (editor.connection) {
            assert.deepEqual(editor.connection.seq, 3);
            assert.deepEqual(editor.connection.path, window.location.pathname);
        } else {
            assert.ok(false);
        }
    });

    test("create session using initial sequence id from attribute", function () {
        const editor = new Editor(TestConnection);
        editor.setAttribute("initial-seq", "3");
        editor.setAttribute("participants", "[]");
        this.tmp.appendChild(editor);
        if (editor.session) {
            assert.deepEqual(editor.session.seq, 3);
            assert.deepEqual(editor.session.client_seq, 0);
        } else {
            assert.ok(false);
        }
    });

});
