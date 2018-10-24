import { assert } from "chai";
import { suite, test } from "mocha";
import { Editor } from "../editor";

suite("Editor", () => {
    test("Use element textContent as initial CodeMirror doc value", () => {
        const div = document.createElement("div");
        div.id = "editor";
        div.dataset.initialSeq = "0";
        div.dataset.participants = '[{"id": 1, "cursor_pos": "test"}]';
        div.textContent = "Example content";
        const editor = new Editor(div);
        const doc = editor.cm.getDoc();
        assert.equal(doc.getValue(), "Example content");
    });
});
