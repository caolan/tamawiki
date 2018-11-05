import { EventEmitter } from "events";
import { Connection } from "./connection";
import { Join, Connected, Participant } from "./protocol";

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
            } else if (msg instanceof Join) {
                this.emit("join", new Participant(msg.id, 0));
            }
        });
    }
}
