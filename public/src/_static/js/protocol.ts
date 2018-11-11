export abstract class ClientMessage {
    public static fromJSON(data: any): ServerMessage {
        return ClientEdit.fromJSON(data);
    }

    public abstract toJSON(): any;
}

export class ClientEdit extends ClientMessage {
    public static fromJSON(data: any): ClientEdit {
        return new ClientEdit(
            data.ClientEdit.parent_seq,
            data.ClientEdit.client_seq,
            data.ClientEdit.operations.map(Operation.fromJSON),
        );
    }

    constructor(
        public parentSeq: number,
        public clientSeq: number,
        public operations: Operation[]) {
        super();
    }

    public toJSON(): any {
        return {
            ClientEdit: {
                client_seq: this.clientSeq,
                operations: this.operations.map((op) => op.toJSON()),
                parent_seq: this.parentSeq,
            },
        };
    }
}

export abstract class ServerMessage {
    public static fromJSON(data: any): ServerMessage {
        if (data.Connected) {
            return Connected.fromJSON(data);
        } else if (data.Event) {
            return ServerEvent.fromJSON(data);
        } else {
            throw new Error("Unknown ServerMessage type");
        }
    }

    public abstract toJSON(): any;
}

export class Connected extends ServerMessage {
    public static fromJSON(data: any): Connected {
        return new Connected(data.Connected.id);
    }

    constructor(public id: number) {
        super();
    }

    public transform(_other: Event): void {
        return;
    }

    public toJSON(): any {
        return { Connected: { id: this.id } };
    }
}

export class ServerEvent extends ServerMessage {
    public static fromJSON(data: any): ServerEvent {
        return new ServerEvent(
            data.Event.seq,
            data.Event.client_seq,
            Event.fromJSON(data.Event.event),
        );
    }

    constructor(
        public seq: number,
        public clientSeq: number,
        public event: Event) {
        super();
    }

    public toJSON(): any {
        return {
            Event: {
                client_seq: this.clientSeq,
                event: this.event.toJSON(),
                seq: this.seq,
            },
        };
    }
}

export class Document {
    public static fromJSON(data: any): Document {
        return new Document(
            data.content,
            data.participants.map(Participant.fromJSON),
        );
    }

    constructor(
        public content: string,
        public participants: Participant[]) { }

    public toJSON(): any {
        return {
            content: this.content,
            participants: this.participants.map((x) => x.toJSON()),
        };
    }
}

export class Participant {
    public static fromJSON(data: any): Participant {
        return new Participant(data.id, data.cursor_pos);
    }

    constructor(
        public id: number,
        public cursorPos: number) { }

    public toJSON(): any {
        return { id: this.id, cursor_pos: this.cursorPos };
    }
}

export abstract class Operation {
    public static fromJSON(data: any): Operation {
        if (data.Insert) {
            return Insert.fromJSON(data);
        } else if (data.Delete) {
            return Delete.fromJSON(data);
        } else if (data.MoveCursor) {
            return MoveCursor.fromJSON(data);
        } else {
            throw new Error("Unknown Operation type");
        }
    }

    public abstract transform(other: Operation, hasPriority: boolean): Operation[];

    public abstract toJSON(): any;
}

export class Insert extends Operation {
    public static fromJSON(data: any): Insert {
        return new Insert(data.Insert.pos, data.Insert.content);
    }

    constructor(
        public pos: number,
        public content: string) { super(); }

    public toJSON(): any {
        return { Insert: { pos: this.pos, content: this.content } };
    }

    public transform(other: Operation, hasPriority: boolean): [Insert] {
        if (other instanceof Insert) {
            if (other.pos < this.pos || (other.pos === this.pos && hasPriority)) {
                this.pos += other.content.length;
            }
        } else if (other instanceof Delete) {
            if (other.start < this.pos) {
                const end = Math.min(this.pos, other.end);
                this.pos -= end - other.start;
            }
        } else if (other instanceof MoveCursor) {
            // no change
        } else {
            throw new Error("Unknown Operation type");
        }
        return [this];
    }
}

export class Delete extends Operation {
    public static fromJSON(data: any): Delete {
        return new Delete(data.Delete.start, data.Delete.end);
    }

    constructor(
        public start: number,
        public end: number) { super(); }

    public toJSON(): any {
        return { Delete: { start: this.start, end: this.end } };
    }

    public transform(other: Operation, _hasPriority: boolean): Delete[] {
        if (other instanceof Insert) {
            if (other.pos < this.start) {
                this.start += other.content.length;
                this.end += other.content.length;
            } else if (other.pos < this.end && this.end - this.start > 0) {
                // create a new Delete to cover the range before the insert
                const before = new Delete(this.start, other.pos);
                // update the current delete to cover the range after the insert
                this.start = other.pos + other.content.length;
                this.end = this.end + other.content.length;
                return [this, before];
            }
        } else if (other instanceof Delete) {
            let charsDeletedBefore = 0;
            if (other.start < this.start) {
                const end = Math.min(this.start, other.end);
                charsDeletedBefore = end - other.start;
            }
            let charsDeletedInside = 0;
            if (other.start < this.start) {
                if (other.end > this.start) {
                    const end = Math.min(this.end, other.end);
                    charsDeletedInside = end - this.start;
                }
            } else if (other.start < this.end) {
                const end = Math.min(this.end, other.end);
                charsDeletedInside = end - other.start;
            }
            this.start -= charsDeletedBefore;
            this.end -= charsDeletedBefore + charsDeletedInside;
        } else if (other instanceof MoveCursor) {
            // no change
        } else {
            throw new Error("Unknown Operation type");
        }
        return [this];
    }
}

export class MoveCursor extends Operation {
    public static fromJSON(data: any): MoveCursor {
        return new MoveCursor(data.MoveCursor.pos);
    }

    constructor(public pos: number) {
        super();
    }

    public toJSON(): any {
        return { MoveCursor: { pos: this.pos } };
    }

    public transform(other: Operation, hasPriority: boolean): [MoveCursor] {
        if (other instanceof Insert) {
            if (other.pos < this.pos) {
                this.pos += other.content.length;
            }
        } else if (other instanceof Delete) {
            if (other.start < this.pos) {
                const end = Math.min(this.pos, other.end);
                this.pos -= end - other.start;
            }
        } else if (other instanceof MoveCursor) {
            // no change
        } else {
            throw new Error("Unknown Operation type");
        }
        return [this];
    }
}

export abstract class Event {
    /**
     * Accepts raw JSON data tagged with the event type and returns a
     * new event object of the appropriate class.
     */
    public static fromJSON(data: any): Event {
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

    /**
     * Mutates the Event to accommodate a concurrent event already
     * applied locally.
     */
    public abstract transform(other: Event): void;

    public abstract toJSON(): any;
}

export class Edit extends Event {
    public static fromJSON(data: any): Edit {
        return new Edit(
            data.Edit.author,
            data.Edit.operations.map(Operation.fromJSON),
        );
    }

    constructor(
        public author: number,
        public operations: Operation[]) { super(); }

    public transform(other: Event): void {
        if (!(other instanceof Edit)) {
            return;
        }
        for (const otherOperation of other.operations) {
            let operations: Operation[] = [];
            for (const op of this.operations) {
                operations = operations.concat(
                    op.transform(
                        otherOperation,
                        this.author < other.author,
                    ),
                );
            }
            this.operations = operations;
        }
    }

    public toJSON(): any {
        return {
            Edit: {
                author: this.author,
                operations: this.operations.map((x) => x.toJSON()),
            },
        };
    }
}

export class Join extends Event {
    public static fromJSON(data: any): Join {
        return new Join(data.Join.id);
    }

    constructor(public id: number) {
        super();
    }

    public transform(_other: Event): void {
        return;
    }

    public toJSON(): any {
        return { Join: { id: this.id } };
    }
}

export class Leave extends Event {
    public static fromJSON(data: any): Leave {
        return new Leave(data.Leave.id);
    }

    constructor(public id: number) {
        super();
    }

    public transform(_other: Event): void {
        return;
    }

    public toJSON(): any {
        return { Leave: { id: this.id } };
    }
}
