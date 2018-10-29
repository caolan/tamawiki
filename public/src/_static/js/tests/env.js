/* This file defines the environment 'npm test' uses to run unit tests */

let jsdom = require('jsdom');
console.log(jsdom);
console.log(jsdom.MutationObserver);
require('jsdom-global')();

console.log(window);
console.log(window.MutationObserver);

require('@webcomponents/custom-elements');

// // for codemirror
// global.document.body.createTextRange = function () {
//     return {
//         setEnd: function () {},
//         setStart: function () {},
//         getBoundingClientRect: function () {
//             return {right: 0};
//         },
//         getClientRects: function () {
//             return {
//                 length: 0,
//                 left: 0,
//                 right: 0
//             };
//         }
//     };
// };

let fs = require('fs');

// so requiring .css files doesn't error
require.extensions['.css'] = function (module, filename) {
    module.exports = fs.readFileSync(filename, 'utf8');
};
