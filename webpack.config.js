const UglifyJsPlugin = require('uglifyjs-webpack-plugin');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const OptimizeCSSAssetsPlugin = require("optimize-css-assets-webpack-plugin");
const path = require('path');


module.exports = {
    mode: "production",
    devtool: "source-map",
    entry: "./public/src/_static/js/editor.ts",
    output: {
        path: path.resolve(__dirname, "public/dist"),
        filename: "./_static/js/[name].js"
    },
    resolve: {
        extensions: [".ts", ".tsx", ".js", ".css"]
    },
    module: {
        rules: [
            {test: /\.tsx?$/, loader: 'ts-loader'},
            {test: /\.css$/, use: [
                MiniCssExtractPlugin.loader,
                'css-loader'
            ]}
        ]
    },
    optimization: {
        minimizer: [
            new UglifyJsPlugin({
                cache: true,
                parallel: true,
                sourceMap: true
            }),
            new OptimizeCSSAssetsPlugin({})
        ]
    },
    plugins: [
        new MiniCssExtractPlugin({
            filename: "./_static/css/[name].css",
            chunkFilename: "[id].css"
        })
    ]
};
