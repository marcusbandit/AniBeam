// Verifies exportHandler.buildExport against the anibeam-export v1 contract
// fixed in https://github.com/marcusbandit/AniBeam/issues/11: the library
// export (sources, every series with its match) and the full export (adds
// accounts, the TMDB key, history, preferences) built from fixture config,
// metadata, tracker and view-history files. No real userData is touched.
//
// Run: bun --bun scripts/verify-export.mjs

import assert from 'node:assert/strict';
import { mock } from 'bun:test';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { mkdtemp, writeFile } from 'node:fs/promises';

const userData = await mkdtemp(join(tmpdir(), 'anibeam-export-'));

mock.module('electron', () => ({
  app: {
    getPath: () => userData,
    getVersion: () => '1.0.0',
  },
  // Not exercised: trackers.json fixture below is written already in the
  // plaintext-fallback shape (cipherEncrypted: false), the same shape
  // trackerStore itself falls back to when encryption is unavailable.
  safeStorage: {
    isEncryptionAvailable: () => false,
    encryptString: (s) => Buffer.from(s, 'utf-8'),
    decryptString: (b) => b.toString('utf-8'),
  },
  BrowserWindow: { getAllWindows: () => [] },
}));

const FRIEREN_FOLDER = '/mnt/media/anime/Sousou no Frieren';
const MOVIE_PATH = '/mnt/media/anime/Movies/Perfect Blue (1997).mkv';
const UNMATCHED_FOLDER = '/mnt/media/anime/Some Unmatched Folder';
const OP_FILE = `${FRIEREN_FOLDER}/NCOP1.mkv`;

await writeFile(join(userData, 'config.json'), JSON.stringify({
  folderSources: ['/mnt/media/anime'],
  lastScanned: '2026-08-01T00:00:00.000Z',
  version: 1,
  tmdbApiKey: 'abc123',
}));

await writeFile(join(userData, 'metadata.json'), JSON.stringify({
  sousou_no_frieren: {
    seriesId: 'sousou_no_frieren',
    title: 'Sousou no Frieren',
    folderPath: FRIEREN_FOLDER,
    type: 'series',
    hidden: false,
    source: 'anilist',
    anilistId: 154587,
    malId: 52991,
    fileEpisodes: [
      { episodeNumber: 1, filePath: `${FRIEREN_FOLDER}/01.mkv`, status: 'ready' },
    ],
  },
  movie_perfect_blue: {
    seriesId: 'movie_perfect_blue',
    title: 'Perfect Blue',
    folderPath: '/mnt/media/anime/Movies',
    type: 'movie',
    hidden: false,
    source: 'tmdb',
    tmdbId: 10494,
    tmdbKind: 'movie',
    fileEpisodes: [
      { episodeNumber: 1, filePath: MOVIE_PATH, status: 'ready' },
    ],
  },
  some_unmatched_folder: {
    seriesId: 'some_unmatched_folder',
    title: 'Some Unmatched Folder',
    folderPath: UNMATCHED_FOLDER,
    type: 'series',
    hidden: true,
    fileEpisodes: [
      { episodeNumber: 1, filePath: `${UNMATCHED_FOLDER}/01.mkv`, status: 'ready' },
    ],
  },
}));

await writeFile(join(userData, 'trackers.json'), JSON.stringify({
  anilist: {
    username: 'bandit',
    userId: 123456,
    expiresAt: null,
    lastSync: null,
    clientId: '12345',
    // Plaintext fallback shape: base64 of the plain string, cipherEncrypted: false.
    accessTokenCipher: Buffer.from('token123', 'utf-8').toString('base64'),
    refreshTokenCipher: null,
    cipherEncrypted: false,
  },
  mal: null,
  clientIds: { anilist: '12345', mal: '' },
  clientSecretCiphers: { anilist: '', mal: '' },
  clientSecretsEncrypted: false,
  mainProvider: 'anilist',
  progress: { anilist: {}, mal: {} },
  progressFetchedAt: { anilist: null, mal: null },
  version: 2,
}));

const lastViewedAt = Date.parse('2026-08-30T21:04:11Z');
await writeFile(join(userData, 'view-history.json'), JSON.stringify({
  version: 1,
  history: {
    sousou_no_frieren: { lastViewedAt, lastEpisode: 12 },
  },
}));

const { buildExport } = await import('../src/main/handlers/exportHandler.ts');

const completedAt = Date.parse('2026-08-30T21:04:11Z');
const resumeAt = Date.parse('2026-09-01T19:30:00Z');
const opResumeAt = Date.parse('2026-09-01T19:35:00Z');

const rendererState = {
  videoProgress: JSON.stringify({
    'sousou_no_frieren::13': { t: 612.4, d: 1420.0, updated: resumeAt },
    [`sousou_no_frieren::x:${OP_FILE}`]: { t: 30.1, d: 90.0, updated: opResumeAt },
  }),
  videoLastEpisode: JSON.stringify({
    sousou_no_frieren: { ep: 12, updated: completedAt },
  }),
  titleLanguage: 'JP',
  libraryTab: 'all',
  librarySortKey: 'alpha',
  librarySortDir: 'asc',
  feedSort: 'recent',
};

// --- library export (unticked) --------------------------------------------

const library = await buildExport(false, rendererState);

assert.equal(library.format, 'anibeam-export');
assert.equal(library.version, 1);
assert.equal(library.private, false);
assert.equal(library.exportedBy.app, 'anibeam');
assert.equal(library.exportedBy.line, 'electron');
assert.equal(library.exportedBy.version, '1.0.0');
assert.ok(!Number.isNaN(Date.parse(library.exportedAt)), 'exportedAt is a parseable instant');
assert.deepEqual(library.sources, [{ path: '/mnt/media/anime' }]);

assert.equal(library.series.length, 3, 'every metadata.json entry is included, matched or not');

const frieren = library.series.find((s) => s.id === 'sousou_no_frieren');
assert.deepEqual(frieren, {
  kind: 'series',
  path: FRIEREN_FOLDER,
  id: 'sousou_no_frieren',
  title: 'Sousou no Frieren',
  hidden: false,
  match: { provider: 'anilist', anilistId: 154587, malId: 52991 },
});

const movie = library.series.find((s) => s.id === 'movie_perfect_blue');
assert.deepEqual(movie, {
  kind: 'movie',
  // A film's identity is its file, not the shared "Movies" folder several
  // films sit under.
  path: MOVIE_PATH,
  id: 'movie_perfect_blue',
  title: 'Perfect Blue',
  hidden: false,
  match: { provider: 'tmdb', tmdbId: 10494, tmdbKind: 'movie' },
});

const unmatched = library.series.find((s) => s.id === 'some_unmatched_folder');
assert.deepEqual(unmatched, {
  kind: 'series',
  path: UNMATCHED_FOLDER,
  id: 'some_unmatched_folder',
  title: 'Some Unmatched Folder',
  hidden: true,
  match: null,
});

for (const key of ['accounts', 'keys', 'history', 'preferences']) {
  assert.ok(!(key in library), `library export must not carry ${key}`);
}

// --- full export (ticked) --------------------------------------------------

const full = await buildExport(true, rendererState);

assert.equal(full.private, true);
assert.equal(full.accounts.main, 'anilist');
assert.deepEqual(full.accounts.anilist, {
  userId: 123456,
  username: 'bandit',
  clientId: '12345',
  clientSecret: null,
  accessToken: 'token123',
  refreshToken: null,
  expiresAt: null,
});
assert.equal(full.accounts.mal, null, 'a disconnected provider exports as null, not a partial record');
assert.deepEqual(full.keys, { tmdb: 'abc123' });

assert.equal(full.history.views.length, 1);
assert.deepEqual(full.history.views[0], {
  series: FRIEREN_FOLDER,
  lastEpisode: 12,
  at: new Date(lastViewedAt).toISOString(),
});

assert.equal(full.history.completed.length, 1);
assert.deepEqual(full.history.completed[0], {
  series: FRIEREN_FOLDER,
  episode: 12,
  at: new Date(completedAt).toISOString(),
});

assert.equal(full.history.resumePoints.length, 2);
const bySeriesEp = full.history.resumePoints.find((p) => 'episode' in p);
assert.deepEqual(bySeriesEp, {
  series: FRIEREN_FOLDER,
  episode: 13,
  position: 612.4,
  duration: 1420.0,
  at: new Date(resumeAt).toISOString(),
});
const byFile = full.history.resumePoints.find((p) => 'file' in p);
assert.deepEqual(byFile, {
  file: OP_FILE,
  position: 30.1,
  duration: 90.0,
  at: new Date(opResumeAt).toISOString(),
});

assert.deepEqual(full.preferences, {
  titleLanguage: 'romaji',
  libraryTab: 'all',
  librarySort: { key: 'alpha', direction: 'asc' },
  feedSort: 'recent',
});

console.log('verify-export: all assertions passed');
