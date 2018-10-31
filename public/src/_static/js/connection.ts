export interface IConnectionConstructor {
    new (path: string, seq: number): IConnection;
}

export interface IConnection {
    path: string;
    seq: number;
}

export class Connection {
    constructor(public path: string, public seq: number) {}
}
