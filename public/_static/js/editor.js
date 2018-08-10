var editor_el = document.getElementById('editor');
var value = editor_el.textContent;
editor_el.innerHTML = '';

var editor = CodeMirror(editor_el, {
    mode: "text",
    value: value,
    lineWrapping: true
});
