module.exports = {
    name: 'TamaWiki',
    theme: 'minimal',
    out: './public/doc/',
    readme: 'none',
    exclude: [
        '**/tests/**/*',
        '*~'
    ],
    excludeExternals: true,
    excludeNotExported: true,
    excludePrivate: true
};
