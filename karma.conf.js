module.exports = function (config) {
    config.set({
        frameworks: ['mocha', 'karma-typescript'],
        basePath: 'public/src/_static/',
        files: ['js/**/*.ts'],
        preprocessors: {
            '**/*.ts': ['karma-typescript']
        },
        reporters: ['mocha', 'karma-typescript'],
        port: 9876,  // karma web server port
        colors: false,
        logLevel: config.LOG_INFO,
        browsers: ['FirefoxHeadless'],
        autoWatch: false,
        // singleRun: false, // Karma captures browsers, runs the tests and exits
        concurrency: Infinity,
        karmaTypescriptConfig: {
            compilerOptions: require('./tsconfig.json').compilerOptions
        },
        client: {
            mocha: {
                // change Karma's debug.html to the mocha web reporter
                reporter: 'html',
                ui: 'tdd'
            }
        }
    });
};
