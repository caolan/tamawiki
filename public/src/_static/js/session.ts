import { EventEmitter } from "events";
import { Connection } from "./connection";
import * as protocol from "./protocol";

export class Session extends EventEmitter {
    public clientSeq: number;
    public participantId?: number;

    constructor(public connection: Connection) {
        super();

        this.clientSeq = 0;

        this.connection.on("message", (msg) => {
            if (msg instanceof protocol.Connected) {
                this.participantId = msg.id;
                this.emit("connected", msg.id);
            } else {
                if (msg.event instanceof protocol.Join) {
                    const p = new protocol.Participant(msg.event.id, 0);
                    this.emit("join", msg.seq, p);
                } else if (msg.event instanceof protocol.Leave) {
                    this.emit("leave", msg.seq, msg.event.id);
                } else if (msg.event instanceof protocol.Edit) {
                    this.emit("edit", msg.seq, msg.event);
                }
            }
        });
    }

    send(parentSeq: number, operations: protocol.Operation[]) {
        this.clientSeq++;
        this.connection.send(
            new protocol.ClientEdit(parentSeq, this.clientSeq, operations)
        );
    }
}
