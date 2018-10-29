import * as CodeMirror from "codemirror";
import "codemirror/lib/codemirror.css";
import "../css/style.css";

export class Editor {
    public cm: CodeMirror.Editor;
    public participants: {[id: number]: IParticipant};
    public client_seq: number;
    public seq: number;

    constructor(public element: HTMLElement) {
        this.client_seq = 0;
        this.seq = Number(element.dataset.initialSeq);
        
        this.cm = CodeMirror(element, {
            lineWrapping: true,
            mode: "text",
            value: element.textContent,
        });
        
        this.participants = {};
        JSON.parse(element.dataset.participants || "[]").forEach(
            (p: {id: number, cursor_pos: number}) => {
                this.participants[p.id] = {cursor: null};
                this.setParticipantCursor(p.id, p.cursor_pos);
            },
        );
    }

    setParticipantCursor(id: number, index: number): void {
        let doc = this.cm.getDoc();
        var pos = doc.posFromIndex(index);
        var participant = this.participants[id];
        if (participant.cursor !== null) {
            participant.cursor.clear();
            participant.cursor = null;
        }
        if (index === null) {
            return;
        }
        var cursorCoords = this.cm.cursorCoords(pos);
        var el = document.createElement('span');
        el.className = 'participant-cursor';
        el.style.top = `${cursorCoords.bottom}px`;
        participant.cursor = doc.setBookmark(pos, {
            widget: el
        });
    }

    findCursors(start: CodeMirror.Position, end: CodeMirror.Position): {participant: number, bookmark: CodeMirror.TextMarker}[] {
        var results = [];
        let doc = this.cm.getDoc();
        var marks = doc.findMarks(start, end);
        for (const mark of marks) {
            for (const id in this.participants) {
                const p = this.participants[id];
                if (p.cursor && p.cursor === mark) {
                    results.push({participant: Number(id), bookmark: mark});
                }
            }
        }
        return results;
    }

    applyEvent(event: ServerEvent): void {
        event.apply(this);
    }
}

export interface IParticipant {
    cursor: null | CodeMirror.TextMarker;
}

export type ServerEvent = Edit | Join | Leave;

export class Edit {
    constructor(public author: number, public operations: Operation[]) {}
    
    transform(concurrent: ServerEvent): void {
        if (concurrent instanceof Edit) {
            for (const concurrentOperation of concurrent.operations) {
                let operations: Operation[] = [];
                for (const op of this.operations) {
                    operations = operations.concat(
                        op.transform(
                            concurrentOperation,
                            this.author < concurrent.author
                        )
                    );
                }
                this.operations = operations;
            }
        }
    }

    canApply(editor: Editor): void {
        if (!editor.participants.hasOwnProperty(this.author)) {
            throw new Error("InvalidOperation");
        }
        let length = editor.cm.getDoc().getValue().length;
        for (const op of this.operations) {
            length = op.canApply(length);
        }
    }
    
    apply(editor: Editor): void {
        this.canApply(editor);
        for (const op of this.operations) {
            op.apply(editor, this.author);
        }
    }
}

export class Join {
    constructor(public id: number) {}
    
    transform(_concurrent: ServerEvent): void {}

    canApply(editor: Editor): void {
        if (editor.participants.hasOwnProperty(this.id)) {
            throw new Error("InvalidOperation");
        }
    }

    apply(editor: Editor): void {
        this.canApply(editor);
        editor.participants[this.id] = {cursor: null};
        editor.setParticipantCursor(this.id, 0);
    }
}

export class Leave {
    constructor(public id: number) {}
    
    transform(_concurrent: ServerEvent): void {}

    canApply(editor: Editor): void {
        if (!editor.participants.hasOwnProperty(this.id)) {
            throw new Error("InvalidOperation");
        }
    }

    apply(editor: Editor): void {
        this.canApply(editor);
        delete editor.participants[this.id];
    }
}

export type Operation = Insert | Delete;

export class Insert {
    constructor(public pos: number, public content: string) {}

    transform(other: Operation, hasPriority: boolean): Operation[] {
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
    
    canApply(length: number): number {
        if (this.pos > length) {
            throw new Error('OutsideDocument');
        }
        return length + this.content.length;
    }

    apply(editor: Editor, author: number): void {
        let doc = editor.cm.getDoc();
        var start = doc.posFromIndex(this.pos);
        var end = doc.posFromIndex(this.pos + this.content.length);
        doc.replaceRange(this.content, start);
        doc.markText(start, end, {className: 'edit'});
        editor.setParticipantCursor(author, this.pos + this.content.length);
    }
}

export class Delete {
    constructor(public start: number, public end: number) {}
    
    transform(other: Operation, _hasPriority: boolean): Operation[] {
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

    canApply(length: number): number {
        if (this.start > this.end) {
            throw new Error('InvalidOperation');
        }
        if (this.end > length) {
            throw new Error('OutsideDocument');
        }
        return length - (this.end - this.start);
    }

    apply(editor: Editor, author: number): void {
        let doc = editor.cm.getDoc();
        var start = doc.posFromIndex(this.start);
        var end = doc.posFromIndex(this.end);
        var cursors = editor.findCursors(start, end);
        doc.replaceRange("", start, end);
        // if we thiseted any bookmarks for other participant's cursors, update them now
        for (const cursor of cursors) {
            editor.setParticipantCursor(cursor.participant, this.start);
        }
        editor.setParticipantCursor(author, this.start);
    }
}


// var applying_server_edits = false;
// var host = window.location.host;
// var pathname = window.location.pathname;
// var ws_url = 'ws://' + host + pathname + '?seq=' + seq;

// console.log('Connecting to: ' + ws_url);
// var ws = new WebSocket(ws_url);

// var editor = CodeMirror(editor_el, {
//     mode: "text",
//     value: value,
//     lineWrapping: true
// });

// var connection_id = null;

// var participants_el = document.getElementById('participants');

// function displayParticipants() {
//     var ul = document.createElement('ul');
//     for (var id in participants) {
//         var li = document.createElement('li');
//         li.textContent = 'Guest ' + id;
//         if (id == connection_id) {
//             li.className = 'you';
//         }
//         ul.appendChild(li);
//     }
//     participants_el.innerHTML = '';
//     participants_el.appendChild(ul);
// }

// function send(msg) {
//     console.log('SENDING:' + JSON.stringify(msg));
//     ws.send(JSON.stringify(msg));
// }

//  function Insert(pos, content) {
//      this.pos = pos;
//      this.content = content;
//  }
//  Insert.prototype.toJSON = function () {
//      return {'Insert': {pos: this.pos, content: this.content}};
//  };

//  function Delete(start, end) {
//      this.start = start;
//      this.end = end;
//  }
//  Delete.prototype.toJSON = function () {
//      return {'Delete': {start: this.start, end: this.end}};
//  };

//  function MoveCursor(pos) {
//      this.pos = pos;
//  }
//  MoveCursor.prototype.toJSON = function () {
//      return {'MoveCursor': {pos: this.pos}};
//  };

//  var operations = [];

// function pushOperation(op) {
//     // remove any previous cursor moves as they will be superceded by the new operation
//     operations = operations.filter(function (op) {
//         return !(op instanceof MoveCursor);
//     });
//     // attempt to combine/squash consecutive events before they're sent
//     var last = operations.length && operations[operations.length - 1];
//     if (last) {
//         if (op instanceof Insert) {
//             if (last instanceof Insert) {
//                 if (last.pos + last.content.length === op.pos) {
//                     last.content += op.content;
//                     return;
//                 }
//             }
//         }
//         else if (op instanceof Delete) {
//             if (last instanceof Delete) {
//                 if (last.start === op.start) {
//                     last.end += (op.end - op.start);
//                     return;
//                 }
//                 else if (last.start === op.end) {
//                     last.start = op.start;
//                     return;
//                 }
//             }
//         }
//         if (op instanceof MoveCursor) {
//             // do not push move operation if it's already inferred by
//             // pervious Insert/Delete operation
//             if (last instanceof Insert) {
//                 if (last.pos + last.content.length === op.pos) {
//                     return;
//                 }
//             }
//             else if (last instanceof Delete) {
//                 if (last.start === op.pos) {
//                     return;
//                 }
//             }
//         }
//     }
//     operations.push(op);
// }

// function flushOperations() {
//     var flushed = operations;
//     operations = [];
//     return flushed;
// }

// function isBefore(a, b) {
//     if (a.line < b.line) {
//         return true;
//     }
//     if (a.line == b.line) {
//         return a.ch < b.ch;
//     }
//     return false;
// }

// function processChange(change) {
//     var start = editor.getDoc().indexFromPos(change.from);
//     var data = change.text.join('\n');
//     var removed = change.removed.join('\n');

//     if (removed) {
//         pushOperation(new Delete(start, start + removed.length));
//     }
//     if (data) {
//         pushOperation(new Insert(start, data));
//         var doc = editor.getDoc();
//         var from = change.from;
//         var to = doc.posFromIndex(start + data.length);
//         var markers = doc.findMarks(from, to);
//         markers.forEach(function (mark) {
//             var range = mark.find();
//             mark.clear();
//             if (isBefore(range.from, from)) {
//                 doc.markText(range.from, from, {className: 'edit'});
//             }
//             if (isBefore(to, range.to)) {
//                 doc.markText(to, range.to, {className: 'edit'});
//             }
//         });
//     }
// }

// // fired when content in the editor is changed
// editor.on('changes', function (instance, changes) {
//     if (!applying_server_edits) {
//         changes.forEach(processChange);
//     }
// });

// // fired when the cursor or selection moves, or any change is made to the editor content
// editor.on('cursorActivity', function (...args) {
//     if (!applying_server_edits) {
//         var doc = editor.getDoc();
//         var pos = doc.indexFromPos(doc.getCursor('head'));
//         pushOperation(new MoveCursor(pos));
//     }
// });

// function setParticipantCursor(id, pos) {
//     var participant = participants[id];
//     if (participant.cursor) {
//         participant.cursor.clear();
//     }
//     var cursorCoords = editor.cursorCoords(pos);
//     var el = document.createElement('span');
//     el.className = 'participant-cursor';
//     el.style.top = `${cursorCoords.bottom}px`;
//     participant.cursor = editor.getDoc().setBookmark(pos, { widget: el });
// }

// function applyOperation(author, op) {
//     var doc = editor.getDoc();

//     if (op.Insert) {
//         var start = doc.posFromIndex(op.Insert.pos);
//         doc.replaceRange(op.Insert.content, start);
//         var end = doc.posFromIndex(op.Insert.pos + op.Insert.content.length);
//         var marker = doc.markText(start, end, {className: 'edit'});
//         setParticipantCursor(author, end);
//     }
//     else if (op.Delete) {
//         var start = doc.posFromIndex(op.Delete.start);
//         var end = doc.posFromIndex(op.Delete.end);
//         doc.replaceRange("", start, end);
//         setParticipantCursor(author, start);
//     }
// }

// function applyEdit(edit) {
//     applying_server_edits = true;
//     edit.operations.forEach(function (op) {
//         applyOperation(edit.author, op);
//     });
//     seq = edit.seq;
//     applying_server_edits = false;
// }

// ws.onopen = function (event) {
//     console.log('CONNECTED');
// };

// ws.onclose = function (event) {
//     console.log('DISCONNECTED');
// };

// ws.onmessage = function (event) {
//     var msg = JSON.parse(event.data);
//     console.log('RECEIVED: ' + JSON.stringify(msg));
//     if (msg.Connected) {
//         connection_id = msg.Connected.id;
//         participants[msg.Connected.id] = {};
//         displayParticipants();
//         setInterval(function () {
//             if (operations.length) {
//                 send({
//                     'ClientEdit': {
//                         parent_seq: seq,
//                         client_seq: ++client_seq,
//                         operations: flushOperations()
//                     }
//                 });
//             }
//         }, 500);
//     }
//     else if (msg.Join) {
//         participants[msg.Join.id] = {};
//         displayParticipants();
//     }
//     else if (msg.Leave) {
//         delete participants[msg.Leave.id];
//         displayParticipants();
//     }
//     else if (msg.Edit) {
//         applyEdit(msg.Edit);
//     }
// };
