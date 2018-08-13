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

ws.onopen = function (event) {
    console.log('CONNECTED');
};

ws.onmessage = function (event) {
    console.log('RECEIVED: ' + event.data);
};
