import "@webcomponents/custom-elements";
import * as CodeMirror from "codemirror";
import { EventEmitter } from "events";
import * as protocol from "./protocol";

import "codemirror/lib/codemirror.css";

export class ContentElement extends HTMLElement {
    public codemirror: CodeMirror.Editor;
    public events: EventEmitter;
    public otherParticipants: { [id: number]: { marker?: CodeMirror.TextMarker } };
    private applyingEvent: boolean;

    constructor() {
        super();
        this.events = new EventEmitter();
        this.otherParticipants = [];
        this.applyingEvent = false;
        this.codemirror = CodeMirror(this, {
            lineWrapping: true,
            mode: "text",
            value: "",
            viewportMargin: Infinity,
        });

        this.codemirror.on("cursorActivity", () => {
            // don't emit changes when applying event from server
            if (this.applyingEvent) {
                return;
            }
            const doc = this.codemirror.getDoc();
            const pos = doc.indexFromPos(doc.getCursor("head"));
            this.events.emit("change", [new protocol.MoveCursor(pos)]);
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
                this.events.emit("change", operations);
            }
        });
    }

    public loadDocument(doc: protocol.Document) {
        this.codemirror.setValue(doc.content);
        for (const p of doc.participants) {
            this.addParticipant(p);
        }
    }

    public addParticipant(p: protocol.Participant): void {
        this.otherParticipants[p.id] = {};
        this.setParticipantPosition(p.id, p.cursorPos);
    }

    public removeParticipant(id: number): void {
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

    public applyMessage(msg: protocol.ServerMessage): void {
        if (msg instanceof protocol.ServerEvent) {
            this.applyEvent(msg.event);
        }
    }

    public applyEvent(event: protocol.Event): void {
        // check the event can be applied cleanly before making any
        // changes to the document
        this.canApplyEvent(event);

        this.applyingEvent = true;
        if (event instanceof protocol.Join) {
            this.addParticipant(new protocol.Participant(event.id, 0));
        } else if (event instanceof protocol.Leave) {
            this.removeParticipant(event.id);
        } else if (event instanceof protocol.Edit) {
            for (const op of event.operations) {
                this.applyOperation(event.author, op);
            }
        }
        this.applyingEvent = false;
    }

    public setParticipantPosition(id: number, index: number): void {
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
        } else if (op instanceof protocol.MoveCursor) {
            if (op.pos > length) {
                throw new Error("OutsideDocument");
            }
            return length;
        } else {
            throw new Error(`Unknown Operation type`);
        }
    }

    private applyOperation(author: number, op: protocol.Operation): void {
        const doc = this.codemirror.getDoc();

        if (op instanceof protocol.Insert) {
            const start = doc.posFromIndex(op.pos);
            const atLocalCursor = op.pos === doc.indexFromPos(doc.getCursor());
            doc.replaceRange(op.content, start);
            const end = doc.posFromIndex(op.pos + op.content.length);
            doc.markText(start, end, { className: "edit" });
            this.setParticipantPosition(
                author,
                op.cursorPositionAfter(),
            );
            if (atLocalCursor) {
                // Need to move the local cursor back to original
                // position in the case where the remote Insert event
                // happened at the same location.
                doc.setCursor(start);
            }
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
                op.cursorPositionAfter(),
            );
        } else if (op instanceof protocol.MoveCursor) {
            this.setParticipantPosition(
                author,
                op.cursorPositionAfter(),
            );
        } else {
            throw new Error("Unknown Operation type");
        }
    }
}

customElements.define("tw-content", ContentElement);
