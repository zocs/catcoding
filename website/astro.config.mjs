import { defineConfig } from 'astro/config';

export default defineConfig({
  site: 'https://catcoding.org',
  output: 'static',
  build: {
    assets: '_assets',
  },
});
