import "@webcomponents/custom-elements";
import * as CodeMirror from "codemirror";
import { EventEmitter } from "events";
import * as protocol from "./protocol";

import "codemirror/lib/codemirror.css";

export class ContentElement extends HTMLElement {
    public codemirror: CodeMirror.Editor;
    public events: EventEmitter;
    public otherParticipants: { [id: number]: { marker?: CodeMirror.TextMarker } };
    public seq: number;
    private applyingEvent: boolean;

    constructor() {
        super();
        this.seq = 0;
        this.events = new EventEmitter();
        this.otherParticipants = [];
        this.applyingEvent = false;
        this.codemirror = CodeMirror(this, {
            lineWrapping: true,
            mode: "text",
            value: "",
        });

        this.codemirror.on("changes", (_instance, changes) => {
            // don't emit changes when applying event from server
            if (this.applyingEvent) {
                return;
            }
            const operations = [];
            const doc = this.codemirror.getDoc();
            for (const change of changes) {
                const start = doc.indexFromPos(change.from);
                const inserted = change.text.join("\n");
                const removed = (change.removed || []).join("\n");
                if (removed) {
                    operations.push(new protocol.Delete(
                        start,
                        start + removed.length,
                    ));
                }
                if (inserted) {
                    operations.push(new protocol.Insert(
                        start,
                        inserted,
                    ));
                    this.clearEditMarkers(
                        change.from,
                        doc.posFromIndex(start + inserted.length),
                    );
                }
            }
            if (operations.length) {
                this.events.emit("change", this.seq, operations);
            }
        });
    }

    public loadDocument(seq: number, doc: protocol.Document) {
        this.seq = seq;
        this.codemirror.setValue(doc.content);
        for (const p of doc.participants) {
            this.addParticipant(seq, p);
        }
    }

    public addParticipant(seq: number, p: protocol.Participant): void {
        this.seq = seq;
        this.otherParticipants[p.id] = {};
        this.setParticipantPosition(p.id, p.cursorPos);
    }

    public removeParticipant(seq: number, id: number): void {
        this.seq = seq;
        const participant = this.otherParticipants[id];
        if (participant && participant.marker) {
            participant.marker.clear();
        }
        delete this.otherParticipants[id];
    }

    public getValue(): string {
        if (this.codemirror) {
            return this.codemirror.getDoc().getValue();
        }
        return "";
    }

    public getParticipantPosition(id: number): number | null {
        const participant = this.otherParticipants[id];
        if (participant && participant.marker) {
            const doc = this.codemirror.getDoc();
            // sometimes .find() returns a Position instead of {to:
            // Position, from: Position} as @types/codemirror declares
            const pos = participant.marker.find() as unknown;
            return doc.indexFromPos(pos as CodeMirror.Position);
        }
        return null;
    }

    public applyEvent(seq: number, event: protocol.Event): void {
        // check the event can be applied cleanly before making any
        // changes to the document
        this.canApplyEvent(event);
        this.seq = seq;

        this.applyingEvent = true;
        if (event instanceof protocol.Join) {
            this.otherParticipants[event.id] = {};
            this.setParticipantPosition(event.id, 0);
        } else if (event instanceof protocol.Leave) {
            delete this.otherParticipants[event.id];
        } else if (event instanceof protocol.Edit) {
            for (const op of event.operations) {
                this.applyOperation(event.author, op);
            }
        }
        this.applyingEvent = false;
    }

    private canApplyEvent(event: protocol.Event): void {
        if (event instanceof protocol.Join) {
            if (this.otherParticipants[event.id]) {
                throw new Error("InvalidOperation");
            }
        } else if (event instanceof protocol.Leave) {
            if (!this.otherParticipants[event.id]) {
                throw new Error("InvalidOperation");
            }
        } else if (event instanceof protocol.Edit) {
            if (!this.otherParticipants[event.author]) {
                throw new Error("InvalidOperation");
            }
            let length = this.codemirror.getDoc().getValue().length;
            for (const op of event.operations) {
                length = this.canApplyOperation(op, length);
            }
        }
    }

    private clearEditMarkers(from: CodeMirror.Position, to: CodeMirror.Position): void {
        const doc = this.codemirror.getDoc();
        doc.findMarks(from, to).forEach(function(mark) {
            const range = mark.find();
            mark.clear();
            if (CodeMirror.cmpPos(range.from, from) < 0) {
                doc.markText(range.from, from, { className: "edit" });
            }
            if (CodeMirror.cmpPos(to, range.to) < 0) {
                doc.markText(to, range.to, { className: "edit" });
            }
        });
    }

    private setParticipantPosition(id: number, index: number): void {
        const doc = this.codemirror.getDoc();
        const pos = doc.posFromIndex(index);
        const participant = this.otherParticipants[id];
        if (participant.marker) {
            participant.marker.clear();
            delete participant.marker;
        }
        if (index === null) {
            return;
        }
        const cursorCoords = this.codemirror.cursorCoords(pos);
        const el = document.createElement("span");
        el.className = "participant-cursor";
        el.style.height = `${(cursorCoords.bottom - cursorCoords.top)}px`;
        participant.marker = doc.setBookmark(pos, {
            widget: el,
        });
    }

    /**
     * Returns an array of IDs corresponding to Participants whose
     * cursor markers are currently in the given document range.
     */
    private participantsInRange(start: CodeMirror.Position, end: CodeMirror.Position): number[] {
        const results = [];
        const doc = this.codemirror.getDoc();
        const marks = doc.findMarks(start, end);
        for (const mark of marks) {
            for (const id in this.otherParticipants) {
                if (this.otherParticipants.hasOwnProperty(id)) {
                    const p = this.otherParticipants[id];
                    if (p.marker && p.marker === mark) {
                        results.push(Number(id));
                    }
                }
            }
        }
        return results;
    }

    /**
     * Checks the operation can be applied cleanly to the document and
     * returns what the length of the document content would be after
     * the operation is applied.
     */
    private canApplyOperation(op: protocol.Operation, length: number): number {
        if (op instanceof protocol.Insert) {
            if (op.pos > length) {
                throw new Error("OutsideDocument");
            }
            return length + op.content.length;
        } else if (op instanceof protocol.Delete) {
            if (op.start > op.end) {
                throw new Error("InvalidOperation");
            }
            if (op.end > length) {
                throw new Error("OutsideDocument");
            }
            return length - (op.end - op.start);
        } else {
            throw new Error(`Unknown Operation type: ${op}`);
        }
    }

    private applyOperation(author: number, op: protocol.Operation): void {
        const doc = this.codemirror.getDoc();

        if (op instanceof protocol.Insert) {
            const start = doc.posFromIndex(op.pos);
            doc.replaceRange(op.content, start);
            const end = doc.posFromIndex(op.pos + op.content.length);
            doc.markText(start, end, { className: "edit" });
            this.setParticipantPosition(
                author,
                op.pos + op.content.length,
            );
        } else if (op instanceof protocol.Delete) {
            const start = doc.posFromIndex(op.start);
            const end = doc.posFromIndex(op.end);
            const participants = this.participantsInRange(start, end);
            doc.replaceRange("", start, end);
            // update any other participant's cursors that were removed
            for (const id of participants) {
                this.setParticipantPosition(id, op.start);
            }
            this.setParticipantPosition(
                author,
                op.start,
            );
        }
    }
}

customElements.define("tw-content", ContentElement);
