const TerserPlugin = require('terser-webpack-plugin');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const OptimizeCSSAssetsPlugin = require("optimize-css-assets-webpack-plugin");
const TypedocWebpackPlugin = require('typedoc-webpack-plugin');
const path = require('path');


module.exports = {
    mode: "production",
    devtool: "source-map",
    entry: "./public/src/_static/js/index.ts",
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
            new TerserPlugin({
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
        }),
        new TypedocWebpackPlugin({
            name: 'TamaWiki',
            out: '../docs',
            theme: 'minimal'
        })
    ]
};
