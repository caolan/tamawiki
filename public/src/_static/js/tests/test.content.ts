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

});
