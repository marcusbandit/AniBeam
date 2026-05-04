import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: './',
  resolve: {
    extensions: ['.tsx', '.ts', '.jsx', '.js', '.mjs', '.json'],
  },
  // Treat .wasm as a raw asset; the renderer fetches and re-blobs it with the
  // correct MIME before passing to JASSUB (Vite's dev static handler serves
  // .wasm with the wrong Content-Type and overrides any middleware fix).
  assetsInclude: ['**/*.wasm'],
});
