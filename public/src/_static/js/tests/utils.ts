import { Connection } from "../connection";
import * as protocol from "../protocol";

export class TestConnection extends Connection {
    public sent: protocol.ClientMessage[];

    constructor(
        public path: string,
        public seq: number) {
        super();
        this.sent = [];
    }

    public send(msg: protocol.ClientMessage): void {
        this.sent.push(msg);
    }
}
