import * as CodeMirror from "codemirror";
import "codemirror/lib/codemirror.css";
import "../css/style.css";

export class Editor {
    public cm: CodeMirror.Editor;
    public participants: {[id: number]: IParticipant};

    constructor(public element: HTMLElement) {
        this.participants = {};
        JSON.parse(element.dataset.participants || "[]").forEach(
            (p: {id: number, cursor_pos: number}) => {
                this.participants[p.id] = {
                    cursor_pos: p.cursor_pos,
                };
            },
        );
        this.cm = CodeMirror(element, {
            lineWrapping: true,
            mode: "text",
            value: element.textContent,
        });
    }
}

export interface IParticipant {
    cursor_pos: number;
}

// var editor_el = document.getElementById('editor');
// var value = editor_el.textContent;
// editor_el.innerHTML = '';

// var applying_server_edits = false;
// var client_seq = 0;
// var seq = Number(editor_el.dataset.initialSeq);
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

// var participants = JSON.parse(editor_el.dataset.participants || '{}');
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
