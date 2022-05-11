import autoPreprocess from 'svelte-preprocess';
import path from 'path';
import fs from 'fs';
import svg from '@poppanator/sveltekit-svg';
import auto from '@sveltejs/adapter-auto';

const mediaQueries = fs
  .readFileSync('./src/styles/_internal.media.pcss')
  .toString();

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: [
    autoPreprocess({
      postcss: {
        prependData: mediaQueries,
      },
    }),
  ],

  kit: {
    adapter: auto(),

    // hydrate the <div id="svelte"> element in src/app.html
    vite: {
      plugins: [
        svg({
          svgoOptions: {
            plugins: [
              {
                name: 'prefixIds',
                params: {
                  prefixClassNames: false,
                  delim: '-',
                },
              },
            ],
          },
        }),
      ],
      resolve: {
        alias: {
          src: path.resolve('./src'),
          lib: path.resolve('./src/lib'),
          components: path.resolve('./src/components'),
          styles: path.resolve('./src/styles'),
          consts: path.resolve('./src/consts'),
          assets: path.resolve('./src/assets'),
          icons: path.resolve('./src/assets/icons'),
          graphics: path.resolve('./src/assets/graphics'),
          social: path.resolve('./src/assets/social'),
          images: path.resolve('./src/assets/images'),
          logos: path.resolve('./src/assets/logos'),
          utils: path.resolve('./src/utils'),
          modules: path.resolve('./src/modules'),
          models: path.resolve('./src/models'),
          tokens: path.resolve('./src/assets/tokens'),
        },
      },
    },
  },
};

export default config;
