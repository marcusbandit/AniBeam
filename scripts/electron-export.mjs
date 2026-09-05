// The Electron app's own export, written without opening the app. Settings >
// Export does this from a window; the phase 1 exit check needs the same
// document from a terminal, so this drives `exportHandler.buildExport` with
// `electron` mocked exactly the way scripts/verify-export.mjs does.
//
// Run: bun scripts/electron-export.mjs [path]        (default ~/anibeam-export.json)
// The data directory is $ANIBEAM_ELECTRON_DATA, or ~/.config/anibeam.
//
// Unticked only: the document carries the sources and every series' match,
// and nothing private. Reading is all this does, to the Electron data alone.

import { mock } from 'bun:test';
import { writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

const dataDir = process.env.ANIBEAM_ELECTRON_DATA ?? join(homedir(), '.config', 'anibeam');
const out = process.argv[2] ?? join(homedir(), 'anibeam-export.json');

mock.module('electron', () => ({
  app: {
    getPath: () => dataDir,
    // The line the document is stamped with. There is no packaged app here to
    // ask, so it says so rather than inventing a number.
    getVersion: () => 'electron-export.mjs',
  },
  safeStorage: {
    isEncryptionAvailable: () => false,
    encryptString: (s) => Buffer.from(s, 'utf-8'),
    decryptString: (b) => b.toString('utf-8'),
  },
  BrowserWindow: { getAllWindows: () => [] },
}));

const { buildExport } = await import('../src/main/handlers/exportHandler.ts');

// The renderer's localStorage, which an unticked export never reads: it goes
// into the preferences and the resume points, and both are private.
const rendererState = {
  videoProgress: null,
  videoLastEpisode: null,
  titleLanguage: null,
  libraryTab: null,
  librarySortKey: null,
  librarySortDir: null,
  feedSort: null,
};

const document = await buildExport(false, rendererState);
await writeFile(out, `${JSON.stringify(document, null, 2)}\n`);
console.log(`${out}: ${document.sources.length} sources, ${document.series.length} series`);
