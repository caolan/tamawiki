import { EventEmitter } from "events";
import { Connection } from "./connection";
import * as protocol from "./protocol";

export class Session extends EventEmitter {
    // The local sequence ID for the last event sent by this client.
    public clientSeq: number;

    // The participant Id given to this client by the server.
    public participantId?: number;

    // ClientEdit's not yet acknowledged by the server. Used to
    // transform incoming ServerMessages to accommodate concurrent
    // local events.
    public sent: protocol.ClientEdit[];

    constructor(public connection: Connection) {
        super();
        this.sent = [];
        this.clientSeq = 0;
        this.connection.on("message", (msg) => this.receive(msg));
    }

    // NOTE: this must be called synchronously when an edit occurs
    // otherwise a received ServerEvent may not be transformed to
    // accommodate the current state of the content in the editor.
    public send(parentSeq: number, operations: protocol.Operation[]) {
        this.clientSeq++;
        const msg = new protocol.ClientEdit(parentSeq, this.clientSeq, operations);
        this.connection.send(msg);
        this.sent.push(msg);
    }

    private receive(msg: protocol.ServerMessage): void {
        if (msg instanceof protocol.Connected) {
            this.participantId = msg.id;
        } else if (msg instanceof protocol.ServerEvent) {
            // clear buffered ClientEdits which have now been
            // acknowledged by the server
            this.sent = this.sent.filter((clientEdit) => {
                return clientEdit.clientSeq > msg.clientSeq;
            });
        }
        this.transform(msg);
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
