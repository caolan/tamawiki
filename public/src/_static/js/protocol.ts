export abstract class ClientMessage {
    static fromJSON(data: any): ServerMessage {
        return ClientEdit.fromJSON(data);
    };
    abstract toJSON(): any;
}

export class ClientEdit extends ClientMessage {
    constructor(
        public parentSeq: number,
        public clientSeq: number,
        public operations: Operation[]) {
        super();
    }

    static fromJSON(data: any): ClientEdit {
        return new ClientEdit(
            data.ClientEdit.parent_seq,
            data.ClientEdit.client_seq,
            data.ClientEdit.operations.map(Operation.fromJSON),
        );
    }

    toJSON(): any {
        return {
            ClientEdit: {
                parent_seq: this.parentSeq,
                client_seq: this.clientSeq,
                operations: this.operations.map((op) => op.toJSON())
            }
        };
    }
}

export abstract class ServerMessage {
    static fromJSON(data: any): ServerMessage {
        if (data.Connected) {
            return Connected.fromJSON(data);
        } else if (data.Event) {
            return ServerEvent.fromJSON(data);
        } else {
            throw new Error("Unknown ServerMessage type");
        }
    };

    abstract toJSON(): any;
}

export class Connected extends ServerMessage {
    constructor(public id: number) {
        super();
    }

    transform(_other: Event): void { }

    static fromJSON(data: any): Connected {
        return new Connected(data.Connected.id);
    }

    toJSON(): any {
        return { Connected: { id: this.id } }
    }
}

export class ServerEvent extends ServerMessage {
    constructor(
        public seq: number,
        public clientSeq: number,
        public event: Event) {
        super();
    }

    static fromJSON(data: any): ServerEvent {
        return new ServerEvent(
            data.Event.seq,
            data.Event.client_seq,
            Event.fromJSON(data.Event.event),
        );
    }

    toJSON(): any {
        return {
            Event: {
                seq: this.seq,
                client_seq: this.clientSeq,
                event: this.event.toJSON()
            }
        }
    }
};

export class Document {
    constructor(public content: string,
        public participants: Participant[]) { }

    static fromJSON(data: any): Document {
        return new Document(
            data.content,
            data.participants.map(Participant.fromJSON)
        );
    }

    toJSON(): any {
        return {
            content: this.content,
            participants: this.participants.map((x) => x.toJSON())
        }
    }
}

export class Participant {
    constructor(
        public id: number,
        public cursor_pos: number) { }

    static fromJSON(data: any): Participant {
        return new Participant(data.id, data.cursor_pos);
    }

    toJSON(): any {
        return { id: this.id, cursor_pos: this.cursor_pos };
    }
}

export abstract class Operation {
    abstract transform(other: Operation, hasPriority: boolean): Operation[];

    static fromJSON(data: any): Operation {
        if (data.Insert) {
            return Insert.fromJSON(data);
        } else if (data.Delete) {
            return Delete.fromJSON(data);
        } else {
            throw new Error("Unknown Operation type");
        }
    }

    abstract toJSON(): any;
}

export class Insert extends Operation {
    constructor(
        public pos: number,
        public content: string) { super(); }

    static fromJSON(data: any): Insert {
        return new Insert(data.Insert.pos, data.Insert.content);
    }

    toJSON(): any {
        return { Insert: { pos: this.pos, content: this.content } };
    }

    transform(other: Operation, hasPriority: boolean): [Insert] {
        if (other instanceof Insert) {
            if (other.pos < this.pos || (other.pos === this.pos && hasPriority)) {
                this.pos += other.content.length;
            }
        } else if (other instanceof Delete) {
            if (other.start < this.pos) {
                let end = Math.min(this.pos, other.end);
                this.pos -= end - other.start;
            }
        } else {
            throw new Error("Unknown operation type");
        }
        return [this];
    }
}

export class Delete extends Operation {
    constructor(
        public start: number,
        public end: number) { super(); }

    static fromJSON(data: any): Delete {
        return new Delete(data.Delete.start, data.Delete.end);
    }

    toJSON(): any {
        return { Delete: { start: this.start, end: this.end } };
    }

    transform(other: Operation, _hasPriority: boolean): Delete[] {
        if (other instanceof Insert) {
            if (other.pos < this.start) {
                this.start += other.content.length;
                this.end += other.content.length;
            } else if (other.pos < this.end && this.end - this.start > 0) {
                // create a new Delete to cover the range before the insert
                let before = new Delete(this.start, other.pos);
                // update the current delete to cover the range after the insert
                this.start = other.pos + other.content.length;
                this.end = this.end + other.content.length;
                return [this, before];
            }
        } else if (other instanceof Delete) {
            let chars_deleted_before = 0;
            if (other.start < this.start) {
                let end = Math.min(this.start, other.end);
                chars_deleted_before = end - other.start;
            }
            let chars_deleted_inside = 0;
            if (other.start < this.start) {
                if (other.end > this.start) {
                    let end = Math.min(this.end, other.end);
                    chars_deleted_inside = end - this.start;
                }
            } else if (other.start < this.end) {
                let end = Math.min(this.end, other.end);
                chars_deleted_inside = end - other.start;
            }
            this.start -= chars_deleted_before;
            this.end -= chars_deleted_before + chars_deleted_inside;
        }
        return [this];
    }
}

export abstract class Event {
    /**
     * Mutates the Event to accommodate a concurrent event already
     * applied locally.
     */
    abstract transform(other: Event): void;

    /**
     * Accepts raw JSON data tagged with the event type and returns a
     * new event object of the appropriate class.
     */
    static fromJSON(data: any): Event {
        if (data.Edit) {
            return Edit.fromJSON(data);
        } else if (data.Join) {
            return Join.fromJSON(data);
        } else if (data.Leave) {
            return Leave.fromJSON(data);
        } else {
            throw new Error("Unknown Event type");
        }
    }

    abstract toJSON(): any;
}

export class Edit extends Event {
    constructor(
        public author: number,
        public operations: Operation[]) { super(); }

    transform(other: Event): void {
        if (!(other instanceof Edit)) {
            return;
        }
        for (const otherOperation of other.operations) {
            let operations: Operation[] = [];
            for (const op of this.operations) {
                operations = operations.concat(
                    op.transform(
                        otherOperation,
                        this.author < other.author
                    )
                );
            }
            this.operations = operations;
        }
    }

    static fromJSON(data: any): Edit {
        return new Edit(
            data.Edit.author,
            data.Edit.operations.map(Operation.fromJSON)
        );
    }

    toJSON(): any {
        return {
            Edit: {
                author: this.author,
                operations: this.operations.map((x) => x.toJSON())
            }
        }
    }
}

export class Join extends Event {
    constructor(public id: number) {
        super();
    }

    transform(_other: Event): void { }

    static fromJSON(data: any): Join {
        return new Join(data.Join.id);
    }

    toJSON(): any {
        return { Join: { id: this.id } }
    }
}

export class Leave extends Event {
    constructor(public id: number) {
        super();
    }

    transform(_other: Event): void { }

    static fromJSON(data: any): Leave {
        return new Leave(data.Leave.id);
    }

    toJSON(): any {
        return { Leave: { id: this.id } }
    }
}
