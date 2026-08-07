import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { mock } from 'bun:test';

// folderHandler transitively imports electron via the logger/IPC layer.
mock.module('electron', () => ({
  BrowserWindow: { getAllWindows: () => [] },
}));
// imageCacheHandler / thumbnailHandler do disk work we don't need in a
// scan-only test. Stub them to no-ops.
mock.module('../src/main/handlers/imageCacheHandler.ts', () => ({
  default: { deleteSeriesImages: async () => {} },
}));
mock.module('../src/main/handlers/thumbnailHandler.ts', () => ({
  default: { deleteSeriesThumbnails: async () => {} },
}));

const { default: folderHandler } = await import('../src/main/handlers/folderHandler.ts');

const tmp = await mkdtemp(join(tmpdir(), 'anibeam-scan-'));

async function touch(path) {
  await mkdir(join(path, '..'), { recursive: true });
  await writeFile(path, '');
}

// ---------- Fixture: real Takagi-san layout ----------
// <root>/Karakai Jouzu no Takagi-san/
//   [Erai-raws] Karakai... - 01 ~ 12 [...]/  (S1 folder with 12 episodes)
//   [Erai-raws] Karakai... 2 - 01 ~ 12 [...]/ (S2 folder with 12 episodes)
//   [Erai-raws] Karakai... 3 - 01 ~ 12 [...]/ (S3 folder with 12 episodes)
//   [Erai-raws] Karakai... - Movie [...].mkv (loose movie file)
const ROOT = tmp;
const SHOW = join(ROOT, 'Karakai Jouzu no Takagi-san');
const SEASONS = [
  '[Erai-raws] Karakai Jouzu no Takagi-san - 01 ~ 12 [1080p][Multiple Subtitle]',
  '[Erai-raws] Karakai Jouzu no Takagi-san 2 - 01 ~ 12 [1080p][Multiple Subtitle]',
  '[Erai-raws] Karakai Jouzu no Takagi-san 3 - 01 ~ 12 [1080p][Multiple Subtitle]',
];
for (const s of SEASONS) {
  const dir = join(SHOW, s);
  await mkdir(dir, { recursive: true });
  for (let ep = 1; ep <= 12; ep++) {
    const tag = ep === 12 ? ' END' : '';
    const fn = `[Erai-raws] ${s.replace(/^\[Erai-raws\] /, '').replace(/ - 01 ~ 12.*/, '')} - ${String(ep).padStart(2, '0')}${tag} [1080p][Multiple Subtitle].mkv`;
    await touch(join(dir, fn));
  }
}
await touch(join(SHOW, '[Erai-raws] Karakai Jouzu no Takagi-san - Movie [1080p][49CCAF8A].mkv'));

// A "screenshots" subdir at the wrapper level with no videos — should be
// ignored, not produce an empty series entry.
await mkdir(join(SHOW, 'screenshots'), { recursive: true });
await writeFile(join(SHOW, 'screenshots', 'cap01.png'), '');

const results = await folderHandler.scanFolder(ROOT);

// Sanity check: every result is unique by id and folderPath
const ids = new Set(results.map(r => r.id));
assert.equal(ids.size, results.length, 'duplicate ids');

const series = results.filter(r => r.type === 'series');
const movies = results.filter(r => r.type === 'movie');

assert.equal(series.length, 3, `expected 3 series entries, got ${series.length}: ${series.map(s => s.name).join(' | ')}`);
assert.equal(movies.length, 1, `expected 1 movie entry, got ${movies.length}`);

// Each series has its own folder path and its own 12 episodes — no overlap.
for (const s of series) {
  assert.equal(s.files.length, 12, `series '${s.name}' has ${s.files.length} files (expected 12)`);
  const eps = s.files.map(f => f.episodeNumber).sort((a, b) => a - b);
  assert.deepEqual(eps, [1,2,3,4,5,6,7,8,9,10,11,12], `episodes for '${s.name}': ${eps.join(',')}`);
}

// Names are anchored on the wrapper name; release-group brackets and
// episode-range suffix are gone. Trailing digit becomes a season hint.
const byName = Object.fromEntries(series.map(s => [s.name, s]));
assert.ok(byName['Karakai Jouzu no Takagi-san'],   `S1 should be named "Karakai Jouzu no Takagi-san"; got: ${series.map(s => s.name).join(' | ')}`);
assert.ok(byName['Karakai Jouzu no Takagi-san 2'], `S2 should be named "Karakai Jouzu no Takagi-san 2"; got: ${series.map(s => s.name).join(' | ')}`);
assert.ok(byName['Karakai Jouzu no Takagi-san 3'], `S3 should be named "Karakai Jouzu no Takagi-san 3"; got: ${series.map(s => s.name).join(' | ')}`);
assert.equal(byName['Karakai Jouzu no Takagi-san'].seasonNumber, null, 'S1 has no explicit season hint');
assert.equal(byName['Karakai Jouzu no Takagi-san 2'].seasonNumber, 2, 'S2 seasonNumber should be 2');
assert.equal(byName['Karakai Jouzu no Takagi-san 3'].seasonNumber, 3, 'S3 seasonNumber should be 3');

// Series folder paths must be the direct subfolders of the wrapper, not the wrapper itself.
for (const s of series) {
  assert.notEqual(s.folderPath, SHOW, `series '${s.name}' folderPath is the wrapper itself`);
  assert.ok(s.folderPath.startsWith(SHOW + '/'), `series '${s.name}' folderPath '${s.folderPath}' not under wrapper`);
}

// The movie's folderPath is the wrapper folder; title comes from the filename.
const movie = movies[0];
assert.equal(movie.folderPath, SHOW, `movie folderPath should be the wrapper; got '${movie.folderPath}'`);
assert.match(movie.name, /Karakai/i, `movie name should mention Karakai; got '${movie.name}'`);
assert.match(movie.name, /Movie/i, `movie name should mention Movie; got '${movie.name}'`);

console.log('OK: franchise wrapper produces 3 series + 1 movie');
for (const s of series) console.log(`  series: ${s.name}  (${s.files.length} eps)`);
console.log(`  movie : ${movie.name}`);

// ---------- Regression: transparent release-group wrapper stays 1 series ----------
const REG_ROOT = await mkdtemp(join(tmpdir(), 'anibeam-regress-'));
const TRANS = join(REG_ROOT, 'My Show');
const GROUP = join(TRANS, '[Some-Group]');
await mkdir(GROUP, { recursive: true });
for (let ep = 1; ep <= 3; ep++) {
  await touch(join(GROUP, `My Show - ${String(ep).padStart(2, '0')}.mkv`));
}

const regResults = await folderHandler.scanFolder(REG_ROOT);
assert.equal(regResults.length, 1, `transparent release-group wrapper produced ${regResults.length} entries (expected 1)`);
assert.equal(regResults[0].type, 'series', 'transparent wrapper should be a series');
assert.equal(regResults[0].files.length, 3, 'transparent wrapper should collect all 3 episodes');
assert.equal(regResults[0].name, 'My Show', `series name should be the outer folder; got '${regResults[0].name}'`);
console.log('OK: transparent release-group wrapper (1 subdir, no loose video) stays 1 series');

// ---------- Regression: regular flat series with episodes directly inside ----------
const FLAT_ROOT = await mkdtemp(join(tmpdir(), 'anibeam-flat-'));
const FLAT = join(FLAT_ROOT, 'Plain Show');
await mkdir(FLAT, { recursive: true });
for (let ep = 1; ep <= 4; ep++) {
  await touch(join(FLAT, `Plain Show - ${String(ep).padStart(2, '0')}.mkv`));
}
const flatResults = await folderHandler.scanFolder(FLAT_ROOT);
assert.equal(flatResults.length, 1);
assert.equal(flatResults[0].type, 'series');
assert.equal(flatResults[0].files.length, 4);
console.log('OK: plain flat series (videos directly in folder) stays 1 series');

// ---------- Edge case: 1 subdir + 1 loose video → wrapper (per design) ----------
const EDGE_ROOT = await mkdtemp(join(tmpdir(), 'anibeam-edge-'));
const EDGE_SHOW = join(EDGE_ROOT, 'Edge Show');
const EDGE_SUB = join(EDGE_SHOW, '[Group]');
await mkdir(EDGE_SUB, { recursive: true });
for (let ep = 1; ep <= 3; ep++) {
  await touch(join(EDGE_SUB, `Edge Show - ${String(ep).padStart(2, '0')}.mkv`));
}
await touch(join(EDGE_SHOW, 'Edge Show - OVA.mkv'));
const edgeResults = await folderHandler.scanFolder(EDGE_ROOT);
const edgeSeries = edgeResults.filter(r => r.type === 'series');
const edgeMovies = edgeResults.filter(r => r.type === 'movie');
assert.equal(edgeSeries.length, 1, `edge case: expected 1 series, got ${edgeSeries.length}`);
assert.equal(edgeMovies.length, 1, `edge case: expected 1 movie, got ${edgeMovies.length}`);
console.log('OK: edge case (1 subdir + 1 loose video) → 1 series + 1 movie');

// ---------- Season folders behind a release folder (real Kaminomi layout) ----------
// <root>/Kami Nomi zo Shiru Sekai/
//   [Judas] Kami Nomi zo Shiru Sekai (Seasons 1-3 + OVAs) [BD 1080p][...]/
//     [Judas] Kaminomi - OVAs/   (4 files)
//     [Judas] Kaminomi - S1/     (12 episodes)
//     [Judas] Kaminomi - S2/     (12 episodes)
//     [Judas] Kaminomi - S3/     (12 episodes)
//
// The release folder holds no episodes of its own, so it's transparent: the
// four season folders below it are the real series. Before the passthrough
// case existed, hasVideosShallow's one-level peek made the release folder look
// like an ordinary `[group]/ep01.mkv` wrapper and all 40 files collapsed into a
// single 40-episode series matched against season 1.
const KN_ROOT = await mkdtemp(join(tmpdir(), 'anibeam-kaminomi-'));
const KN_SHOW = join(KN_ROOT, 'Kami Nomi zo Shiru Sekai');
const KN_RELEASE = join(
  KN_SHOW,
  '[Judas] Kami Nomi zo Shiru Sekai (The World God Only Knows) (Seasons 1-3 + OVAs) [BD 1080p][HEVC x265 10bit][Dual-Audio][Eng-Subs]',
);
for (const s of [1, 2, 3]) {
  const dir = join(KN_RELEASE, `[Judas] Kaminomi - S${s}`);
  await mkdir(dir, { recursive: true });
  for (let ep = 1; ep <= 12; ep++) {
    await touch(join(dir, `[Judas] Kami nomi zo Shiru Sekai - S0${s}E${String(ep).padStart(2, '0')}.mkv`));
  }
}
const KN_OVA = join(KN_RELEASE, '[Judas] Kaminomi - OVAs');
await mkdir(KN_OVA, { recursive: true });
for (const f of [
  '[Judas] Kaminomi OAD - Four Plus an Idol.mkv',
  '[Judas] Kaminomi OVA - Magical Star Kanon 100%.mkv',
  '[Judas] Kaminomi OVA - Tenri Arc 01.mkv',
  '[Judas] Kaminomi OVA - Tenri Arc 02.mkv',
]) {
  await touch(join(KN_OVA, f));
}

const knResults = await folderHandler.scanFolder(KN_ROOT);
const knSeries = knResults.filter(r => r.type === 'series');
assert.equal(
  knSeries.length, 4,
  `expected 4 series (S1, S2, S3, OVAs), got ${knSeries.length}: ${knSeries.map(s => s.name).join(' | ')}`,
);
assert.equal(new Set(knSeries.map(s => s.id)).size, 4, 'season entries must have distinct ids');

// Every file is claimed exactly once — nothing lost, nothing double-counted.
const knFiles = knSeries.flatMap(s => s.files.map(f => f.filePath));
assert.equal(knFiles.length, 40, `expected 40 files across the split, got ${knFiles.length}`);
assert.equal(new Set(knFiles).size, 40, 'a file is claimed by more than one series');

// Names are anchored on the OUTER folder the user named, not the release
// folder's own tag-laden name, and are shaped so AniList can find them.
const knByName = Object.fromEntries(knSeries.map(s => [s.name, s]));
for (const [name, eps, season] of [
  ['Kami Nomi zo Shiru Sekai', 12, 1],
  ['Kami Nomi zo Shiru Sekai Season 2', 12, 2],
  ['Kami Nomi zo Shiru Sekai Season 3', 12, 3],
  ['Kami Nomi zo Shiru Sekai OVA', 4, null],
]) {
  const entry = knByName[name];
  assert.ok(entry, `expected a series named "${name}"; got: ${knSeries.map(s => s.name).join(' | ')}`);
  assert.equal(entry.files.length, eps, `"${name}" has ${entry.files.length} files (expected ${eps})`);
  assert.equal(entry.seasonNumber, season, `"${name}" seasonNumber should be ${season}`);
  assert.ok(
    entry.folderPath.startsWith(KN_RELEASE + '/'),
    `"${name}" folderPath should be a season folder under the release folder; got '${entry.folderPath}'`,
  );
}

// Each season's episodes are numbered 1-12 in their own namespace.
for (const name of ['Kami Nomi zo Shiru Sekai', 'Kami Nomi zo Shiru Sekai Season 2', 'Kami Nomi zo Shiru Sekai Season 3']) {
  const eps = knByName[name].files.map(f => f.episodeNumber).sort((a, b) => a - b);
  assert.deepEqual(eps, [1,2,3,4,5,6,7,8,9,10,11,12], `episodes for '${name}': ${eps.join(',')}`);
}

console.log('OK: season folders behind a transparent release folder split into 4 series');
for (const s of knSeries) console.log(`  series: ${s.name}  (${s.files.length} files, season ${s.seasonNumber ?? '-'})`);

// ---------- Plain "Season N" subfolders get the show's name, not "Season N" ----------
// Without a label fallback, deriveSubfolderSeriesName returned the bare
// subfolder name — so these matched AniList as a show literally called
// "Season 2".
const SEA_ROOT = await mkdtemp(join(tmpdir(), 'anibeam-seasons-'));
const SEA_SHOW = join(SEA_ROOT, 'Some Show');
for (const [dir, n] of [['Season 1', 3], ['Season 2', 3], ['Specials', 2]]) {
  const d = join(SEA_SHOW, dir);
  await mkdir(d, { recursive: true });
  for (let ep = 1; ep <= n; ep++) await touch(join(d, `${dir} - ${String(ep).padStart(2, '0')}.mkv`));
}
const seaSeries = (await folderHandler.scanFolder(SEA_ROOT)).filter(r => r.type === 'series');
const seaNames = seaSeries.map(s => s.name).sort();
assert.deepEqual(
  seaNames,
  ['Some Show', 'Some Show Season 2', 'Some Show Specials'],
  `plain season folders should be named after the show; got: ${seaNames.join(' | ')}`,
);
console.log('OK: plain "Season N" / "Specials" subfolders are named after the show');

// Cleanup
await rm(tmp, { recursive: true, force: true });
await rm(REG_ROOT, { recursive: true, force: true });
await rm(FLAT_ROOT, { recursive: true, force: true });
await rm(EDGE_ROOT, { recursive: true, force: true });
await rm(KN_ROOT, { recursive: true, force: true });
await rm(SEA_ROOT, { recursive: true, force: true });

console.log('\nAll checks passed.');
