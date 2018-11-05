import { EventEmitter } from "events";
import { ClientMessage, ServerMessage } from "./protocol";

export abstract class Connection extends EventEmitter {
    abstract send(msg: ClientMessage): void;
}

export interface IConnectionConstructor {
    new(path: string, seq: number): Connection
}

export class WebSocketConnection extends Connection {
    websocket: WebSocket;

    constructor(path: string, seq: number) {
        super();
        const host = window.location.host;
        this.websocket = new WebSocket(`ws://${host}${path}?seq=${seq}`);
        this.websocket.onopen = (_event) => {
            console.log("websocket open");
        };
        this.websocket.onmessage = (event) => {
            this.emit("message", ServerMessage.fromJSON(
                JSON.parse(event.data)
            ));
        };
    }

    send(msg: ClientMessage): void {
        console.log(msg);
    }
}
