import { EventEmitter } from "events";
import { Connection } from "./connection";
import { Join, Leave, Connected, Participant } from "./protocol";

export class Session extends EventEmitter {
    public clientSeq: number;
    public participantId?: number;

    constructor(
        public connection: Connection,
        public seq: number) {
        super();

        this.clientSeq = 0;

        this.connection.on("message", (msg) => {
            if (msg instanceof Connected) {
                this.participantId = msg.id;
                this.emit("connected", msg.id);
            } else {
                if (msg.event instanceof Join) {
                    this.emit("join", new Participant(msg.event.id, 0));
                } else if (msg.event instanceof Leave) {
                    this.emit("leave", msg.event.id);
                }
            }
        });
    }
}
