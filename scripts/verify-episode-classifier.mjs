import assert from 'node:assert/strict';
import { readdirSync, existsSync } from 'node:fs';

const { classifyFile } = await import('../src/shared/episodeClassifier.ts');

// ---- Unit fixtures ----------------------------------------------------------
// Hand-picked cases that pin behaviour for the regression we care about
// (Bakemonogatari-style OP/ED/PV markers polluting the episode list) plus a
// few other naming patterns the existing parser already handled, so we don't
// regress those.

const cases = [
  // Bakemonogatari real episodes — must classify as episode 1..15.
  ['[Coalgirls]_Bakemonogatari_01_(1920x1080_Blu-ray_FLAC)_[9787055F].mkv', { kind: 'episode', episodeNumber: 1 }],
  ['[Coalgirls]_Bakemonogatari_15_(1920x1080_Blu-ray_FLAC)_[256D3923].mkv', { kind: 'episode', episodeNumber: 15 }],

  // Bakemonogatari extras — the main offenders.
  ['[Coalgirls]_Bakemonogatari_ED1_(1920x1080_Blu-ray_FLAC)_[7EE4E478].mkv', { kind: 'ed', extraIndex: 1, extraVariant: null, rawLabel: 'ED1' }],
  ['[Coalgirls]_Bakemonogatari_ED3_(1920x1080_Blu-ray_FLAC)_[8F8AC7AF].mkv', { kind: 'ed', extraIndex: 3, extraVariant: null }],
  ['[Coalgirls]_Bakemonogatari_OP2_(1920x1080_Blu-ray_FLAC)_[57D95944].mkv', { kind: 'op', extraIndex: 2, extraVariant: null }],
  ['[Coalgirls]_Bakemonogatari_OP4a_(1920x1080_Blu-ray_FLAC)_[AF4FF3CC].mkv', { kind: 'op', extraIndex: 4, extraVariant: 'a', rawLabel: 'OP4a' }],
  ['[Coalgirls]_Bakemonogatari_OP4b_(1920x1080_Blu-ray_FLAC)_[63162685].mkv', { kind: 'op', extraIndex: 4, extraVariant: 'b' }],
  ['[Coalgirls]_Bakemonogatari_OP5b_(1920x1080_Blu-ray_FLAC)_[7B7B859A].mkv', { kind: 'op', extraIndex: 5, extraVariant: 'b' }],
  ['[Coalgirls]_Bakemonogatari_PV01_(1920x1080_Blu-ray_FLAC)_[8924213A].mkv', { kind: 'pv', extraIndex: 1, extraVariant: null }],
  ['[Coalgirls]_Bakemonogatari_PV12_(1920x1080_Blu-ray_FLAC)_[17C508BF].mkv', { kind: 'pv', extraIndex: 12, extraVariant: null }],

  // Standard release patterns the legacy parser already handled.
  ['[Erai-raws] Show Name - 01 [1080p].mkv', { kind: 'episode', episodeNumber: 1 }],
  ['Show.Name.S02E07.1080p.WEB.mkv', { kind: 'episode', episodeNumber: 7, seasonNumber: 2 }],
  ['Show Name - Episode 12.mkv', { kind: 'episode', episodeNumber: 12 }],
  ['Show Name Episode 6.5.mkv', { kind: 'episode', episodeNumber: 6.5 }],

  // Creditless variants fold into op/ed.
  ['Show Name NCOP1 [1080p].mkv', { kind: 'op', extraIndex: 1 }],
  ['Show Name NCED2 [1080p].mkv', { kind: 'ed', extraIndex: 2 }],

  // Specials — both labelled and the existing "episode 0" convention.
  ['Show Name SP1 [1080p].mkv', { kind: 'sp', extraIndex: 1 }],
  ['Show Name Special [1080p].mkv', { kind: 'sp', extraIndex: null, episodeNumber: 0, seasonNumber: 0 }],

  // Standalone trailers / promos. Index only resolves when attached to the
  // token (Trailer1, Trailer01) — "Trailer 1" with a space stays index=null.
  ['Show Name Trailer1.mkv', { kind: 'pv', extraIndex: 1 }],
  ['Show Name Trailer.mkv', { kind: 'pv', extraIndex: null }],

  // Edge: an episode title containing a substring that looks like a marker
  // must NOT be misclassified — anchored ^…$ tokens protect us.
  ['Show Name - 03 - Operations of Hope.mkv', { kind: 'episode', episodeNumber: 3 }],
  ['Show Name - 04 - Edge of Tomorrow.mkv', { kind: 'episode', episodeNumber: 4 }],
];

let failures = 0;
for (const [filename, expected] of cases) {
  const actual = classifyFile(filename);
  for (const [key, want] of Object.entries(expected)) {
    if (actual[key] !== want) {
      console.error(`FAIL  ${filename}`);
      console.error(`      expected ${key}=${JSON.stringify(want)}, got ${JSON.stringify(actual[key])}`);
      console.error(`      full result:`, actual);
      failures++;
    }
  }
}

// ---- Real-library scan (if available) --------------------------------------
// Dump classification for every file in the user's actual Bakemonogatari
// release folder, so the output is something concrete to look at and react
// to rather than just synthetic pass/fail.
const BAKE_DIR = '/srv/media/Anime/Bakemonogatari/[Coalgirls]_Bakemonogatari_(1920x1080_Blu-ray_FLAC)';
if (existsSync(BAKE_DIR)) {
  console.log('\n--- Real Bakemonogatari folder ---');
  const files = readdirSync(BAKE_DIR).filter((f) => f.endsWith('.mkv')).sort();
  const buckets = { episode: [], op: [], ed: [], pv: [], sp: [], other: [] };
  for (const f of files) {
    const c = classifyFile(f);
    buckets[c.kind].push({ file: f, ...c });
  }
  for (const [kind, list] of Object.entries(buckets)) {
    if (list.length === 0) continue;
    console.log(`\n${kind.toUpperCase()} (${list.length})`);
    for (const row of list) {
      const label = row.rawLabel ? `[${row.rawLabel}]` : `ep ${row.episodeNumber}`;
      console.log(`  ${label.padEnd(8)} ${row.file}`);
    }
  }
}

if (failures > 0) {
  console.error(`\n${failures} assertion failure(s)`);
  process.exit(1);
}
console.log('\nOK: episode classifier');
