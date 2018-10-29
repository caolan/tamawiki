import { assert } from "chai";
import { Editor } from "../editor";

suite("Editor", () => {

    setup(function () {
        this.tmp = document.getElementById("tmp") as HTMLElement;
        this.tmp.innerHTML = "";
    });

    test("codemirror editor contains textContent of tw-editor element", function () {
        const editor = new Editor();
        editor.setAttribute("initial-seq", "0");
        editor.setAttribute("participants", "[]");
        editor.textContent = "Example content";
        this.tmp.appendChild(editor);
        assert.equal(editor.content.getValue(), "Example content");
    });

});
