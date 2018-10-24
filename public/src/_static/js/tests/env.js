/* This file defines the environment 'npm test' uses to run unit tests */

let JSDOM = require("jsdom").JSDOM;

let dom = new JSDOM(``, {
    url: "https://example.org/",
    referrer: "https://example.com/",
    contentType: "text/html",
    includeNodeLocations: true,
    storageQuota: 10000000
});

global.window = dom.window;
for (var k in dom.window) {
    if (!global[k]) {
        global[k] = dom.window[k];
    }
}

let fs = require('fs');

// so requiring .css files doesn't error
require.extensions['.css'] = function (module, filename) {
    module.exports = fs.readFileSync(filename, 'utf8');
};
