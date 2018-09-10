var editor_el = document.getElementById('editor');
var value = editor_el.textContent;
editor_el.innerHTML = '';

var applying_server_edits = false;
var client_seq = 0;
var seq = Number(editor_el.dataset.initialSeq);
var host = window.location.host;
var pathname = window.location.pathname;
var ws_url = 'ws://' + host + pathname + '?seq=' + seq;

console.log('Connecting to: ' + ws_url);
var ws = new WebSocket(ws_url);

var editor = CodeMirror(editor_el, {
    mode: "text",
    value: value,
    lineWrapping: true
});


var participants = JSON.parse(editor_el.dataset.participants || '[]');
var connection_id = null;

var participants_el = document.getElementById('participants');

function displayParticipants() {
    var ul = document.createElement('ul');
    participants.forEach(function (id) {
        var li = document.createElement('li');
        li.textContent = 'Guest ' + id;
        if (id == connection_id) {
            li.className = 'you';
        }
        ul.appendChild(li);
    });
    participants_el.innerHTML = '';
    participants_el.appendChild(ul);
}

function send(msg) {
    console.log('SENDING:' + JSON.stringify(msg));
    ws.send(JSON.stringify(msg));
}

 function Insert(pos, content) {
     this.pos = pos;
     this.content = content;
 }
 Insert.prototype.toJSON = function () {
     return {'Insert': {pos: this.pos, content: this.content}};
 };
 
 function Delete(start, end) {
     this.start = start;
     this.end = end;
 }
 Delete.prototype.toJSON = function () {
     return {'Delete': {start: this.start, end: this.end}};
 };
 
 var operations = [];

function pushOperation(op) {
    var last = operations.length && operations[operations.length - 1];
    if (last && last instanceof op.constructor) {
        if (op instanceof Insert) {
            if (last.pos + last.content.length == op.pos) {
                last.content += op.content;
                return;
            }
        }
        else {
            // otherwise it's a delete
            if (last.start == op.start) {
                last.end += (op.end - op.start);
                return;
            }
            else if (last.start == op.end) {
                last.start = op.start;
                return;
            }
        }
    }
    operations.push(op);
}

function flushOperations() {
    var flushed = operations;
    operations = [];
    return flushed;
}

function processChange(change) {
    var start = editor.doc.indexFromPos(change.from);
    var data = change.text.join('\n');
    var removed = change.removed.join('\n');
    
    if (removed) {
        pushOperation(new Delete(start, start + removed.length));
    }
    if (data) {
        pushOperation(new Insert(start, data));
    }
}

editor.on('changes', function (instance, changes) {
    if (!applying_server_edits) {
        changes.forEach(processChange);
    }
});

function applyOperation(op) {
    var doc = editor.doc;
    
    if (op.Insert) {
        doc.replaceRange(
            op.Insert.content,
            doc.posFromIndex(op.Insert.pos)
        );
    }
    else if (op.Delete) {
        doc.replaceRange(
            "",
            doc.posFromIndex(op.Delete.start),
            doc.posFromIndex(op.Delete.end)
        );
    }
}

function applyEdit(edit) {
    applying_server_edits = true;
    edit.operations.forEach(applyOperation);
    seq = edit.seq;
    applying_server_edits = false;
}

ws.onopen = function (event) {
    console.log('CONNECTED');
};

ws.onmessage = function (event) {
    var msg = JSON.parse(event.data);
    console.log('RECEIVED: ' + JSON.stringify(msg));
    if (msg.Connected) {
        connection_id = msg.Connected.id;
        participants.push(msg.Connected.id);
        displayParticipants();
        // send edits every 1s
        setInterval(function () {
            if (operations.length) {
                send({
                    'ClientEdit': {
                        parent_seq: seq,
                        client_seq: ++client_seq,
                        operations: flushOperations(ws)
                    }
                });
            }
        }, 1000);
    }
    else if (msg.Join) {
        participants.push(msg.Join.id);
        displayParticipants();
    }
    else if (msg.Leave) {
        participants = participants.filter(function (id) {
            return id != msg.Leave.id;
        });
        displayParticipants();
    }
    else if (msg.Edit) {
        applyEdit(msg.Edit);
    }
};
