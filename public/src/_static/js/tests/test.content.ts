import { assert } from "chai";
import { ContentElement } from "../content";
import { Delete, Document, Edit, Insert, Operation, ServerEvent } from "../protocol";

suite("ContentElement", () => {

    setup(function() {
        this.tmp = document.createElement("div");
        document.body.appendChild(this.tmp);
    });

    teardown(function() {
        document.body.removeChild(this.tmp);
    });

    test("emit change: Insert operation", function(done) {
        const content = new ContentElement();
        this.tmp.appendChild(content);
        content.events.on("change", (parentSeq: number, operations: Operation[]) => {
            assert.equal(parentSeq, 0);
            assert.deepEqual(operations, [
                new Insert(0, "test"),
            ]);
            done();
        });
        const doc = content.codemirror.getDoc();
        const pos = doc.posFromIndex(0);
        doc.replaceRange("test", pos);
    });

    test("emit change: Delete operation", function(done) {
        const content = new ContentElement();
        this.tmp.appendChild(content);
        content.loadDocument(3, Document.fromJSON({
            content: "Hello, world!",
            participants: [],
        }));
        content.events.on("change", (parentSeq: number, operations: Operation[]) => {
            assert.equal(parentSeq, 3);
            assert.deepEqual(operations, [
                new Delete(5, 13),
            ]);
            done();
        });
        const doc = content.codemirror.getDoc();
        const start = doc.posFromIndex(5);
        const end = doc.posFromIndex(13);
        doc.replaceRange("", start, end);
    });

    test("emit change: replace range, Delete + Insert operations", function(done) {
        const content = new ContentElement();
        this.tmp.appendChild(content);
        content.loadDocument(3, Document.fromJSON({
            content: "Hello, world!",
            participants: [],
        }));
        content.events.on("change", (parentSeq: number, operations: Operation[]) => {
            assert.equal(parentSeq, 3);
            assert.deepEqual(operations, [
                new Delete(8, 12),
                new Insert(8, "galaxy"),
            ]);
            done();
        });
        const doc = content.codemirror.getDoc();
        const start = doc.posFromIndex(8);
        const end = doc.posFromIndex(12);
        doc.replaceRange("galaxy", start, end);
    });

    test("applyEvent updates parentSeq on emitted changes", function(done) {
        const content = new ContentElement();
        this.tmp.appendChild(content);
        content.loadDocument(2, Document.fromJSON({
            content: "",
            participants: [
                { id: 1, cursor_pos: 0 },
                { id: 2, cursor_pos: 0 },
            ],
        }));
        content.applyEvent(3, new Edit(1, [
            new Insert(0, "Hello"),
        ]));
        content.events.on("change", (parentSeq: number, operations: Operation[]) => {
            assert.equal(parentSeq, 3);
            assert.deepEqual(operations, [
                new Insert(5, ", world!"),
            ]);
            done();
        });
        const doc = content.codemirror.getDoc();
        const pos = doc.posFromIndex(5);
        doc.replaceRange(", world!", pos);
    });

    function bookmarkClassNamesAt(doc: CodeMirror.Doc, index: number): Set<string> {
        const pos = doc.posFromIndex(index);
        const markers = doc.findMarksAt(pos);
        const classNames = new Set();
        for (const marker of (markers as any[])) {
            classNames.add(marker.replacedWith.className);
        }
        return classNames;
    }

    function markerClassNames(doc: CodeMirror.Doc, from: number, to: number): Set<string> {
        const start = doc.posFromIndex(from);
        const end = doc.posFromIndex(to);
        const markers = doc.findMarks(start, end);
        const classNames = new Set();
        for (const marker of (markers as any[])) {
            classNames.add(marker.className);
        }
        return classNames;
    }

    test("bookmark is inserted for participant's cursor", function() {
        const content = new ContentElement();
        this.tmp.appendChild(content);
        content.loadDocument(4, Document.fromJSON({
            content: "Hello",
            participants: [
                { id: 1, cursor_pos: 0 },
            ],
        }));
        content.setParticipantPosition(1, 3);
        const doc = content.codemirror.getDoc();
        assert.ok(bookmarkClassNamesAt(doc, 3).has("participant-cursor"));
    });

    test("adjust edit markers after local Insert", function() {
        const content = new ContentElement();
        this.tmp.appendChild(content);
        content.loadDocument(2, Document.fromJSON({
            content: "",
            participants: [
                { id: 1, cursor_pos: 0 },
                { id: 2, cursor_pos: 0 },
            ],
        }));
        content.applyEvent(3, new Edit(1, [
            new Insert(0, "ab"),
        ]));
        const doc = content.codemirror.getDoc();
        // all remotely inserted text should have 'edit' class
        assert.ok(markerClassNames(doc, 0, 1).has("edit"));
        assert.ok(markerClassNames(doc, 1, 2).has("edit"));
        // insert char between a and b via local edit
        const pos = doc.posFromIndex(1);
        doc.replaceRange("c", pos);
        assert.ok(markerClassNames(doc, 0, 1).has("edit"));
        assert.ok(markerClassNames(doc, 2, 3).has("edit"));
        // the middle char should not have the 'edit' class
        assert.ok(!(markerClassNames(doc, 1, 2).has("edit")));
    });

});
