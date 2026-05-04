import { defineConfig } from 'vite';
import { builtinModules } from 'module';

export default defineConfig({
  // Build-time env vars prefixed ANIBEAM_ (from .env.local etc.) are inlined
  // into the main bundle. Public client IDs / secrets bundled this way ship
  // with the binary; nothing reads process.env at runtime.
  envPrefix: 'ANIBEAM_',
  build: {
    lib: {
      entry: 'src/main/main.ts',
      formats: ['es'],
      fileName: () => 'main.js',
    },
    rollupOptions: {
      external: [
        'electron',
        ...builtinModules,
        ...builtinModules.map(m => `node:${m}`),
      ],
      output: {
        entryFileNames: '[name].js',
      },
    },
    minify: false,
    emptyOutDir: false,
  },
  resolve: {
    extensions: ['.ts', '.js', '.mjs', '.json'],
    mainFields: ['module', 'jsnext:main', 'jsnext'],
  },
});
