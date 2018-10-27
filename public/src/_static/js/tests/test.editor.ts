import { assert } from "chai";
import * as fs from "fs";
import { suite, test } from "mocha";
import * as path from "path";
import { Editor } from "../editor";

suite("Editor", () => {

    test("Use element textContent as initial CodeMirror doc value", () => {
        const div = document.createElement("div");
        div.dataset.initialSeq = "0";
        div.dataset.participants = "[]";
        div.textContent = "Example content";
        const editor = new Editor(div);
        const doc = editor.cm.getDoc();
        assert.equal(doc.getValue(), "Example content");
    });

    test("Load participants from data-participants attribute on element", () => {
        const div = document.createElement("div");
        div.dataset.initialSeq = "0";
        div.dataset.participants = JSON.stringify([
            {"id": 123, "cursor_pos": null},
            {"id": 1, "cursor_pos": 8}
        ]);
        div.textContent = "Hello, world!";
        const editor = new Editor(div);
        assert.equal(Object.keys(editor.participants).length, 2);
        assert.deepEqual(editor.participants[123], {cursor: null});
        let cursor = editor.participants[1].cursor;
        if (cursor !== null) {
            // NOTE: the type definitions for TextMarker.find() say it
            // returns a {from: CodeMirror.Position, to:
            // CodeMirror.Position} object - however, in this case it
            // will return a CodeMirror.Position object directly.
            // TODO: raise an issue with @types/codemirror for this.
            const tm = <unknown>cursor.find();
            assert.equal((<CodeMirror.Position>tm).line, 0);
            assert.equal((<CodeMirror.Position>tm).ch, 8);
        } else {
            assert.ok(false);
        }
    });

    test("Load server sequence ID from data-initial-seq attribute on element", () => {
        const div = document.createElement("div");
        div.dataset.initialSeq = "123";
        div.dataset.participants = '[{"id": 1, "cursor_pos": 0}]';
        div.textContent = "";
        const editor = new Editor(div);
        assert.deepEqual(editor.seq, 123);
        assert.deepEqual(editor.client_seq, 0);
    });

});

suite("Editor (tests/shared)", () => {

    const dir = path.resolve(__dirname, "../../../../../tests/shared");
    const index = path.resolve(dir, "INDEX.txt");
    const lines = fs.readFileSync(index, "utf-8").toString().split("\n");

    for (const line of lines) {
        if (line) {
            const filepath = path.resolve(dir, line);
            const name = path.basename(line, ".json");
            const config = JSON.parse(fs.readFileSync(filepath, "utf-8").toString());

            if (config.type === "apply") {
                test(name, () => {
                    // Set up the editor
                    const div = document.createElement("div");
                    div.dataset.initialSeq = "0";
                    div.dataset.participants = JSON.stringify(config.initial.participants);
                    div.textContent = config.initial.content;
                    const editor = new Editor(div);

                    // apply the events
                    try {
                        for (const event of config.events) {
                            editor.applyEvent(event);
                        }
                        // make sure we were not expecting an error
                        assert.equal(config.error, null);
                    } catch (e) {
                        // capture the error if we were expecting one
                        if (config.error) {
                            assert.equal(e.toString(), 'Error: ' + config.error.type);
                        } else {
                            throw e;
                        }
                    }

                    // check state of document after apply events
                    const doc = editor.cm.getDoc();
                    assert.equal(
                        doc.getValue(),
                        config.expected.content
                    );

                    // check state of participants after apply events
                    for (const p of config.expected.participants) {
                        let participant = editor.participants[p.id];
                        if (participant.cursor !== null) {
                            // NOTE: the type definitions for
                            // TextMarker.find() say it returns a
                            // {from: CodeMirror.Position, to:
                            // CodeMirror.Position} object - however,
                            // in this case it will return a
                            // CodeMirror.Position object directly.
                            // TODO: raise an issue with @types/codemirror for this.
                            let tm = <unknown>participant.cursor.find();
                            let index = doc.indexFromPos(<CodeMirror.Position>tm);
                            assert.equal(index, p.cursor_pos);
                        } else {
                            assert.ok(false);
                        }
                    }
                    assert.equal(
                        config.expected.participants.length,
                        Object.keys(editor.participants).length
                    );
                });
            } else {
                test(name, () => {
                    assert.ok(false, `Unknown test type '${config.type}'`);
                });
            }
        }
    }

});
