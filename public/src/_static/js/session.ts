import { EventEmitter } from "events";
import { Connection } from "./connection";
import * as protocol from "./protocol";

export class Session extends EventEmitter {
    // The local sequence ID for the last event sent by this client.
    public clientSeq: number;

    // The last received server sequence ID.
    public seq: number;

    // The participant Id given to this client by the server.
    public participantId?: number;

    // ClientEdit's not yet acknowledged by the server. Used to
    // transform incoming ServerMessages to accommodate concurrent
    // local events.
    public sent: protocol.ClientEdit[];

    // Operations waiting to be sent out in the next ClientEdit. These
    // will be normalized before sending, eliminating any unecessary
    // Operations.
    private outbox: protocol.Operation[];

    // The last local operation sent to the server. Used to detect
    // subsequent MoveCursor events in the outbox that have no
    // meaningful effect.
    private lastOperation?: protocol.Operation;

    constructor(seq: number, public connection: Connection) {
        super();
        this.sent = [];
        this.outbox = [];
        this.clientSeq = 0;
        this.seq = seq;
        this.connection.on("message", (msg) => this.receive(msg));
    }

    // NOTE: this must be called synchronously when an edit occurs
    // otherwise a received ServerEvent may not be transformed to
    // accommodate the current state of the content in the editor.
    public write(operations: protocol.Operation[]) {
        if (!this.outbox.length) {
            setTimeout(() => this.flush(), 0);
        }
        this.outbox = this.outbox.concat(operations);
    }

    public flush(): void {
        if (!this.outbox.length) {
            return;
        }
        let last = this.lastOperation;
        const prepared = [];
        for (let i = 0, len = this.outbox.length; i < len; i++) {
            const op = this.outbox[i];
            const moreOperations = i < len - 1;
            let keep = false;
            if (this.isContentChange(op)) {
                keep = true;
            } else if (!moreOperations) {
                keep = !last ||
                    op.cursorPositionAfter() !== last.cursorPositionAfter();
            }
            if (keep) {
                prepared.push(op);
                last = op;
            }
        }
        if (!prepared.length) {
            return;
        }
        this.outbox = [];
        this.clientSeq++;
        this.lastOperation = last;
        const msg = new protocol.ClientEdit(this.seq, this.clientSeq, prepared);
        this.connection.send(msg);
        this.sent.push(msg);
    }

    private isContentChange(op: protocol.Operation): boolean {
        if (op instanceof protocol.Insert) {
            return op.content.length > 0;
        } else if (op instanceof protocol.Delete) {
            return op.start !== op.end;
        }
        return false;
    }

    private receive(msg: protocol.ServerMessage): void {
        if (msg instanceof protocol.Connected) {
            this.participantId = msg.id;
        } else if (msg instanceof protocol.ServerEvent) {
            this.seq = msg.seq;
            // clear buffered ClientEdits which have now been
            // acknowledged by the server
            this.sent = this.sent.filter((clientEdit) => {
                return clientEdit.clientSeq > msg.clientSeq;
            });
        }
        // TODO: incoming events need to be transformed for operations
        // in the outbox, not just in 'sent'
        this.transform(msg);
        delete this.lastOperation;
        this.emit("message", msg);
    }

    // Transforms a ServerEvent to accommodate concurrent local events.
    private transform(msg: protocol.ServerMessage): void {
        if (msg instanceof protocol.ServerEvent) {
            if (!this.participantId) {
                throw new Error(
                    "participantId must be defined before transforming ServerEvents.",
                );
            }
            if (msg.event instanceof protocol.Edit) {
                // transform ServerMessage to accommodate remaining
                // ClientEdits not yet acknowledged (and therefore
                // removed in receive() call)
                for (const clientEdit of this.sent) {
                    msg.event.transform(new protocol.Edit(
                        this.participantId,
                        clientEdit.operations,
                    ));
                }
            }
        }
    }
}
