const path = require('path');

module.exports = {
    mode: "development",
    entry: "./public/src/_static/js/tests/index.ts",
    output: {
        path: path.resolve(__dirname, "public/test"),
        filename: "./js/[name].js"
    },
    resolve: {
        extensions: [".ts", ".tsx", ".js", ".css"]
    },
    module: {
        rules: [
            {test: /\.tsx?$/, loader: "ts-loader"},
            {test: /\.css$/, loader: "raw-loader"}
        ]
    }
};
