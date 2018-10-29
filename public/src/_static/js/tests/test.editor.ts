import { assert } from "chai";
import * as fs from "fs";
import { suite, test } from "mocha";
import * as path from "path";
import { Delete, Edit, Editor, Insert, Join, Leave, Operation, ServerEvent } from "../editor";

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
            {id: 123, cursor_pos: null},
            {id: 1, cursor_pos: 8},
        ]);
        div.textContent = "Hello, world!";
        const editor = new Editor(div);
        assert.equal(Object.keys(editor.participants).length, 2);
        assert.deepEqual(editor.participants[123], {cursor: null});
        const cursor = editor.participants[1].cursor;
        if (cursor !== null) {
            // NOTE: the type definitions for TextMarker.find() say it
            // returns a {from: CodeMirror.Position, to:
            // CodeMirror.Position} object - however, in this case it
            // will return a CodeMirror.Position object directly.
            // TODO: raise an issue with @types/codemirror for this.
            const tm  = (cursor.find() as unknown) as CodeMirror.Position;
            assert.equal(tm.line, 0);
            assert.equal(tm.ch, 8);
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

function parseOperation(op: any): Operation {
    if (op.Insert) {
        return new Insert(op.Insert.pos, op.Insert.content);
    } else if (op.Delete) {
        return new Delete(op.Delete.start, op.Delete.end);
    } else {
        throw new Error(`Unknown operation type: ${op}`);
    }
}

function parseEvent(event: any): ServerEvent {
    if (event.Edit) {
        return new Edit(
            event.Edit.author,
            event.Edit.operations.map(parseOperation),
        );
    } else if (event.Join) {
        return new Join(event.Join.id);
    } else if (event.Leave) {
        return new Leave(event.Leave.id);
    } else {
        throw new Error(`Unknown event type: ${event}`);
    }
}

function applyTest(config: any) {
    // Set up the editor
    const div = document.createElement("div");
    div.dataset.initialSeq = "0";
    div.dataset.participants = JSON.stringify(config.initial.participants);
    div.textContent = config.initial.content;
    const editor = new Editor(div);
    const events = config.events.map(parseEvent);

    // apply the events
    try {
        for (const event of events) {
            editor.applyEvent(event);
        }
        // make sure we were not expecting an error
        assert.equal(config.error, null);
    } catch (e) {
        // capture the error if we were expecting one
        if (config.error) {
            assert.equal(e.toString(), "Error: " + config.error.type);
        } else {
            throw e;
        }
    }

    // check state of document after apply events
    const doc = editor.cm.getDoc();
    assert.equal(
        doc.getValue(),
        config.expected.content,
    );

    // check state of participants after apply events
    for (const p of config.expected.participants) {
        const participant = editor.participants[p.id];
        if (participant.cursor !== null) {
            // NOTE: the type definitions for TextMarker.find() say it
            // returns a {from: CodeMirror.Position, to:
            // CodeMirror.Position} object - however, in this case it
            // will return a CodeMirror.Position object directly.
            // TODO: raise an issue with @types/codemirror for this.
            const tm = participant.cursor.find() as unknown;
            const i = doc.indexFromPos(tm as CodeMirror.Position);
            assert.equal(i, p.cursor_pos);
        } else {
            assert.ok(false);
        }
    }
    assert.equal(
        config.expected.participants.length,
        Object.keys(editor.participants).length,
    );
}

function transformTest(config: any) {
    const event = parseEvent(config.initial);
    const concurrent = config.concurrent.map(parseEvent);
    for (const c of concurrent) {
        event.transform(c);
    }
    assert.deepEqual(event, parseEvent(config.expected));
}

function makeTest(name: string, config: any) {
    if (config.type === "apply") {
        test(name, () => applyTest(config));
    } else if (config.type === "transform") {
        test(name, () => transformTest(config));
    } else {
        test(name, () => {
            assert.ok(false, `Unknown test type '${config.type}'`);
        });
    }
}

suite("Editor (tests/shared)", () => {

    const dir = path.resolve(__dirname, "../../../../../tests/shared");
    const index = path.resolve(dir, "INDEX.txt");
    const lines = fs.readFileSync(index, "utf-8").toString().split("\n");

    for (const line of lines) {
        if (line) {
            const filepath = path.resolve(dir, line);
            const name = path.basename(line, ".json");
            const config = JSON.parse(fs.readFileSync(filepath, "utf-8").toString());
            makeTest(name, config);
        }
    }

});
