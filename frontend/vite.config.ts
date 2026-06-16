import { defineConfig } from 'vite';

export default defineConfig({
  base: '/Nazori/', // GitHub Pages URL base
  build: {
    target: 'esnext' // Ensure top-level await is supported for Wasm init if needed
  }
});
