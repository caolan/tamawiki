var editor_el = document.getElementById('editor');
var value = editor_el.textContent;
editor_el.innerHTML = '';

var editor = CodeMirror(editor_el, {
    mode: "text",
    value: value,
    lineWrapping: true
});

var seq = editor_el.dataset.initialSeq;
var host = window.location.host;
var pathname = window.location.pathname;
var ws_url = 'ws://' + host + pathname + '?seq=' + seq;

console.log('Connecting to: ' + ws_url);
var ws = new WebSocket(ws_url);

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

ws.onopen = function (event) {
    console.log('CONNECTED');
};

ws.onmessage = function (event) {
    var msg = JSON.parse(event.data);
    console.log('RECEIVED: ' + JSON.stringify(msg));
    if (msg.Connected) {
        connection_id = msg.Connected.id;
        displayParticipants();
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
};
