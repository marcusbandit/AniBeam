import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// JASSUB (libass-WASM) loads its .wasm via WebAssembly.instantiateStreaming(),
// which requires the response to have Content-Type: application/wasm. Vite's
// dev server serves .wasm as application/octet-stream by default in some
// environments, which makes the worker throw. This middleware overrides that.
const wasmMimeFix = {
  name: 'wasm-mime-fix',
  configureServer(server) {
    server.middlewares.use((req, res, next) => {
      if (req.url && req.url.endsWith('.wasm')) {
        res.setHeader('Content-Type', 'application/wasm');
      }
      next();
    });
  },
};

export default defineConfig({
  plugins: [react(), wasmMimeFix],
  base: './',
  resolve: {
    extensions: ['.tsx', '.ts', '.jsx', '.js', '.mjs', '.json'],
  },
  // .wasm and the JASSUB worker need to be left alone (no transformation).
  assetsInclude: ['**/*.wasm'],
});
