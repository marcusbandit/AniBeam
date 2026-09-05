// The Electron app's Home grid, All tab, as plain text: one title per line,
// in the order the grid puts them. It exists for the phase 1 exit check,
// which diffs this against the native core's `anibeam-cli list`, so it reads
// the same two files the app does and applies the same three rules rather
// than asking the app anything.
//
// Run: bun scripts/electron-list.mjs > /tmp/electron-list.txt
// The data directory is $ANIBEAM_ELECTRON_DATA, or ~/.config/anibeam.
//
// No imports from src/: this has to keep working against the frozen Electron
// tree without building any of it.

import { readFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { basename, join } from 'node:path';

const dataDir = process.env.ANIBEAM_ELECTRON_DATA ?? join(homedir(), '.config', 'anibeam');

async function readJson(name) {
  return JSON.parse(await readFile(join(dataDir, name), 'utf-8'));
}

const config = await readJson('config.json');
const metadata = await readJson('metadata.json');
const sources = Array.isArray(config.folderSources) ? config.folderSources : [];

const under = (path, root) => path === root || path.startsWith(`${root.replace(/\/+$/, '')}/`);

// The grid's fallback title. A show is its folder; several films share one
// "Movies" folder, so a film uses the name the scanner gave it, which is what
// the record's own `title` field holds.
function folderName(record) {
  const folder = record.folderPath ?? '';
  if (record.type === 'movie') return record.title ?? basename(folder);
  return basename(folder);
}

// TitleLanguageContext.tsx, JP: romaji, then english, then the folder name.
// An empty string counts as absent, the way `||` treated it.
function pickTitle(record) {
  return record.titleRomaji || record.titleEnglish || folderName(record);
}

const titles = Object.values(metadata)
  // Hidden series have their own tab and are never mixed into All.
  .filter((record) => record.hidden !== true)
  // The grid is built from a walk of the sources, so an entry left behind by
  // a source that is gone is not on it, and neither is one with nothing to
  // play. The native side hides the same rows as missing.
  .filter((record) => sources.some((root) => under(record.folderPath ?? '', root)))
  .filter((record) => (record.fileEpisodes ?? []).length > 0)
  .map(pickTitle);

// Lower-cased, compared code point by code point. The grid itself sorts with
// localeCompare, but phase 1's core sorts with Rust's own string compare, and
// the two orders are only allowed to differ by collation: this file is the
// core's order so the diff is about the titles rather than about ICU.
titles.sort((a, b) => {
  const [x, y] = [a.toLowerCase(), b.toLowerCase()];
  if (x === y) return 0;
  return x < y ? -1 : 1;
});

console.log(titles.join('\n'));
