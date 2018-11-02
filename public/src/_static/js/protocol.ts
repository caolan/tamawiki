export interface Participant {
    id: number;
    cursor_pos: number;
}

export abstract class Operation {
    abstract transform(other: Operation, hasPriority: boolean): Operation[];

    // TODO: write test to confirm for all classes in this file that
    // the fromJSON/toJSON functoins can convert back and forth
    // cleanly and without losing data
    static fromJSON(data: any): Operation {
        if (data.Insert) {
            return Insert.fromJSON(data.Insert);
        } else if (data.Delete) {
            return Delete.fromJSON(data.Delete);
        } else {
            throw new Error(`Unknown Operation type: ${data}`);
        }
    }

    abstract toJSON(): any;
}

export class Insert extends Operation {
    constructor(
        public pos: number,
        public content: string) { super(); }

    static fromJSON(data: any): Insert {
        return new Insert(data.pos, data.content);
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
            throw new Error(`Unknown operation type: ${other}`);
        }
        return [this];
    }
}

export class Delete extends Operation {
    constructor(
        public start: number,
        public end: number) { super(); }

    static fromJSON(data: any): Delete {
        return new Delete(data.start, data.end);
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
            return Edit.fromJSON(data.Edit);
        } else if (data.Join) {
            return Join.fromJSON(data.Join);
        } else if (data.Leave) {
            return Leave.fromJSON(data.Leave);
        } else {
            throw new Error(`Unknown Event type: ${data}`);
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
            data.author,
            data.operations.map(Operation.fromJSON)
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
        return new Join(data.id);
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
        return new Leave(data.id);
    }

    toJSON(): any {
        return { Leave: { id: this.id } }
    }
}
