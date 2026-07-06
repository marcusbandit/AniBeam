import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// This is the main vite config for development
// The renderer config is in vite.renderer.config.mjs for Electron Forge
export default defineConfig({
  plugins: [react()],
  base: './',
  root: './src/renderer',
  server: {
    port: 5173,
  },
  build: {
    outDir: '../../dist',
  },
  // JASSUB's bundle declares its worker as IIFE, which Vite's code-split
  // production build rejects. Force ES-module workers, same as
  // vite.renderer.config.mjs (the electron-forge path).
  worker: { format: 'es' },
});
