import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: './',
  // Allow ANIBEAM_* env vars (alongside Vite's default VITE_*) so the same
  // .env.local feeds both main and renderer.
  envPrefix: ['VITE_', 'ANIBEAM_'],
  resolve: {
    extensions: ['.tsx', '.ts', '.jsx', '.js', '.mjs', '.json'],
  },
  // Treat .wasm as a raw asset; the renderer fetches and re-blobs it with the
  // correct MIME before passing to JASSUB (Vite's dev static handler serves
  // .wasm with the wrong Content-Type and overrides any middleware fix).
  assetsInclude: ['**/*.wasm'],
  // JASSUB's bundle declares its worker as IIFE. That works in dev, but Vite's
  // production build is code-split and rejects IIFE workers. Force ES-module
  // workers everywhere — modern Chromium (Electron) supports them natively.
  worker: { format: 'es' },
});
