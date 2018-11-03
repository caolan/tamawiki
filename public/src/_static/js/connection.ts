import { Duplex } from "stream";

export type ConnectFunction =
    (path: string, seq: number) => Duplex;

export function websocketConnect(_path: string, _seq: number): Duplex {
    return new Duplex();
}
