import "@webcomponents/custom-elements";
import * as CodeMirror from "codemirror";
import * as protocol from "./protocol";
import { EventEmitter } from "events";

import "codemirror/lib/codemirror.css";

export class ContentElement extends HTMLElement {
    public codemirror: CodeMirror.Editor;
    public events: EventEmitter;
    public participants: { [id: number]: { marker?: CodeMirror.TextMarker } };
    public seq: number;
    private applyingEvent: boolean;

    constructor() {
        super();
        this.seq = 0;
        this.events = new EventEmitter();
        this.participants = [];
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
                const inserted = change.text.join('\n');
                const removed = (change.removed || []).join('\n');
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
                }
            }
            if (operations.length) {
                this.events.emit("change", this.seq, operations);
            }
        });
    }

    loadDocument(seq: number, doc: protocol.Document) {
        this.seq = seq;
        this.codemirror.setValue(doc.content);
        for (const p of doc.participants) {
            this.addParticipant(seq, p);
        }
    }

    addParticipant(seq: number, p: protocol.Participant): void {
        this.seq = seq;
        this.participants[p.id] = {};
        this.setParticipantPosition(p.id, p.cursor_pos);
    }

    removeParticipant(seq: number, id: number): void {
        this.seq = seq;
        const participant = this.participants[id];
        if (participant && participant.marker) {
            participant.marker.clear();
        }
        delete this.participants[id];
    }

    getValue(): string {
        if (this.codemirror) {
            return this.codemirror.getDoc().getValue();
        }
        return "";
    }

    canApplyEvent(event: protocol.Event): void {
        if (event instanceof protocol.Join) {
            if (this.participants[event.id]) {
                throw new Error("InvalidOperation");
            }
        } else if (event instanceof protocol.Leave) {
            if (!this.participants[event.id]) {
                throw new Error("InvalidOperation");
            }
        } else if (event instanceof protocol.Edit) {
            if (!this.participants[event.author]) {
                throw new Error("InvalidOperation");
            }
            let length = this.codemirror.getDoc().getValue().length;
            for (const op of event.operations) {
                length = this.canApplyOperation(op, length);
            }
        }
    }

    applyEvent(seq: number, event: protocol.Event): void {
        // check the event can be applied cleanly before making any
        // changes to the document
        this.canApplyEvent(event);
        this.seq = seq;

        this.applyingEvent = true;
        if (event instanceof protocol.Join) {
            this.participants[event.id] = {};
            this.setParticipantPosition(event.id, 0);
        } else if (event instanceof protocol.Leave) {
            delete this.participants[event.id];
        } else if (event instanceof protocol.Edit) {
            for (const op of event.operations) {
                this.applyOperation(event.author, op);
            }
        }
        this.applyingEvent = false;
    }

    getParticipantPosition(id: number): number | null {
        const participant = this.participants[id];
        if (participant && participant.marker) {
            const doc = this.codemirror.getDoc();
            // sometimes .find() returns a Position instead of {to:
            // Position, from: Position} as @types/codemirror declares
            const pos = participant.marker.find() as unknown;
            return doc.indexFromPos(pos as CodeMirror.Position);
        }
        return null;
    }

    setParticipantPosition(id: number, index: number): void {
        let doc = this.codemirror.getDoc();
        var pos = doc.posFromIndex(index);
        var participant = this.participants[id];
        if (participant.marker) {
            participant.marker.clear();
            delete participant.marker;
        }
        if (index === null) {
            return;
        }
        var cursorCoords = this.codemirror.cursorCoords(pos);
        var el = document.createElement('span');
        el.className = 'participant-participant';
        el.style.top = `${cursorCoords.bottom}px`;
        participant.marker = doc.setBookmark(pos, {
            widget: el
        });
    }

    /**
     * Returns an array of IDs corresponding to Participants whose
     * cursor markers are currently in the given document range.
     */
    participantsInRange(start: CodeMirror.Position, end: CodeMirror.Position): number[] {
        var results = [];
        let doc = this.codemirror.getDoc();
        var marks = doc.findMarks(start, end);
        for (const mark of marks) {
            for (const id in this.participants) {
                const p = this.participants[id];
                if (p.marker && p.marker === mark) {
                    results.push(Number(id));
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
    canApplyOperation(op: protocol.Operation, length: number): number {
        if (op instanceof protocol.Insert) {
            if (op.pos > length) {
                throw new Error('OutsideDocument');
            }
            return length + op.content.length;
        } else if (op instanceof protocol.Delete) {
            if (op.start > op.end) {
                throw new Error('InvalidOperation');
            }
            if (op.end > length) {
                throw new Error('OutsideDocument');
            }
            return length - (op.end - op.start);
        } else {
            throw new Error(`Unknown Operation type: ${op}`);
        }
    }

    applyOperation(author: number, op: protocol.Operation): void {
        let doc = this.codemirror.getDoc();

        if (op instanceof protocol.Insert) {
            var start = doc.posFromIndex(op.pos);
            var end = doc.posFromIndex(op.pos + op.content.length);
            doc.replaceRange(op.content, start);
            doc.markText(start, end, { className: 'edit' });
            this.setParticipantPosition(
                author,
                op.pos + op.content.length
            );
        } else if (op instanceof protocol.Delete) {
            var start = doc.posFromIndex(op.start);
            var end = doc.posFromIndex(op.end);
            var participants = this.participantsInRange(start, end);
            doc.replaceRange("", start, end);
            // update any other participant's cursors that were removed
            for (const id of participants) {
                this.setParticipantPosition(id, op.start);
            }
            this.setParticipantPosition(
                author,
                op.start
            );
        }
    }
}

customElements.define("tw-content", ContentElement);
