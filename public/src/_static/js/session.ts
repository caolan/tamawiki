export class Session {
    public seq: number;
    public client_seq: number;

    constructor(seq: number) {
        this.seq = seq;
        this.client_seq = 0;
    }
}
