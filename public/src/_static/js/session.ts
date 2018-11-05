import { EventEmitter } from "events";
import { Connection } from "./connection";
import * as protocol from "./protocol";

export class Session extends EventEmitter {
    public clientSeq: number;
    public participantId?: number;

    constructor(
        public connection: Connection,
        public seq: number) {
        super();

        this.clientSeq = 0;

        this.connection.on("message", (msg) => {
            if (msg instanceof protocol.Connected) {
                this.participantId = msg.id;
                this.emit("connected", msg.id);
            } else {
                if (msg.event instanceof protocol.Join) {
                    this.emit("join", new protocol.Participant(msg.event.id, 0));
                } else if (msg.event instanceof protocol.Leave) {
                    this.emit("leave", msg.event.id);
                }
            }
        });
    }

    send(operations: protocol.Operation[]) {
        this.clientSeq++;
        this.connection.send(
            new protocol.ClientEdit(this.seq, this.clientSeq, operations)
        );
    }
}
