const defaultPreset = require('cssnano-preset-advanced');
const fs = require('fs');

module.exports = {
    plugins: [
        require('postcss-mixins')({
            mixinsFiles: './src/styles/_internal.mixins.pcss',
        }),
        require('postcss-nested'),
        require('postcss-preset-env')({
            stage: 3,
            features: {
                'nesting-rules': true,
                'custom-media-queries': true,
            },
        }),
        require('postcss-functions')({
            functions: {
                fluid: (min, max, size = '2vw + 1rem') =>
                    `clamp(var(${min}), ${size}, var(${max}))`,
                theme: (themeVar, overrideVar) =>
                    overrideVar
                        ? `var(${overrideVar}, var(${themeVar}))`
                        : `var(${themeVar})`,
            },
        }),
        require('cssnano')({
            preset: [
                defaultPreset,
                {
                    discardUnused: false,
                    zIndex: false,
                    reduceIdents: false,
                },
            ],
        }),
    ],
};
