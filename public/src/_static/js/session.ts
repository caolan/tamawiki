import { Connection } from "./connection";

export class Session {
    public clientSeq: number;
    public participantId?: number;

    constructor(
        public connection: Connection,
        public seq: number) {
        this.clientSeq = 0;

        this.connection.on("connected", (id) => {
            this.participantId = id;
            console.log(["Connected", id]);
        });
    }
}
