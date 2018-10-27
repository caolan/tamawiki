/* This file defines the environment 'npm test' uses to run unit tests */

let JSDOM = require("jsdom").JSDOM;

let dom = new JSDOM(``, {
    url: "https://example.org/",
    referrer: "https://example.com/",
    contentType: "text/html",
    includeNodeLocations: true,
    storageQuota: 10000000
});

global.navigator = "gecko";
global.window = dom.window;

for (var k in dom.window) {
    if (!global[k]) {
        global[k] = dom.window[k];
    }
}

// for codemirror
global.document.body.createTextRange = function () {
    return {
        setEnd: function () {},
        setStart: function () {},
        getBoundingClientRect: function () {
            return {right: 0};
        },
        getClientRects: function () {
            return {
                length: 0,
                left: 0,
                right: 0
            };
        }
    };
};

let fs = require('fs');

// so requiring .css files doesn't error
require.extensions['.css'] = function (module, filename) {
    module.exports = fs.readFileSync(filename, 'utf8');
};
