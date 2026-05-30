import assert from 'node:assert/strict';
import { mkdtemp, readFile, access } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { mock } from 'bun:test';

// --- Stub the Electron + metadata layer the crawler imports ----------------
// A unique userData dir per run so the real readers/writers in franchiseGraph.ts
// operate on throwaway temp files (atomic tmp+rename and all).
const userData = await mkdtemp(join(tmpdir(), 'anibeam-crawler-'));

// `owned` is the saved-metadata map the crawler treats as the library. Tests
// mutate it before each case so the right ids count as owned (→ get indexed).
let owned = {};
mock.module('electron', () => ({
  app: { getPath: () => userData },
  BrowserWindow: { getAllWindows: () => [] },
}));
mock.module('../src/main/handlers/metadataHandler.ts', () => ({
  default: { loadMetadata: async () => owned },
}));
// The default fetcher must never run — every test injects its own fetch. Stub
// anilistHandler so an accidental real call would blow up loudly instead of
// hitting the network.
mock.module('../src/main/handlers/anilistHandler.ts', () => ({
  default: { fetchRelations: async () => { throw new Error('default fetcher must not be called'); } },
}));

const { crawlFranchiseLive, crawlLibraryGaps } = await import('../src/main/services/franchiseCrawler.ts');
const { readIndex, readFranchiseFile, writeFranchiseFile } = await import('../src/main/services/franchiseGraph.ts');

// --- helpers ----------------------------------------------------------------
const FIXED = 1_700_000_000_000;
const clock = () => FIXED;

// A node self-record as fetchRelations would return it.
const self = (id) => ({
  anilistId: id, malId: null, type: 'ANIME', format: 'TV', status: 'FINISHED',
  seasonYear: 2000 + (id % 100), startYear: null, siteUrl: null,
  titleRomaji: `T${id}`, titleEnglish: null, poster: null,
});
// A relation edge entry.
const rel = (id, relationType) => ({
  relationType, anilistId: id, malId: null, type: 'ANIME', format: 'TV', status: 'FINISHED',
  seasonYear: 2000 + (id % 100), startYear: null, siteUrl: null,
  titleRomaji: `T${id}`, titleEnglish: null, poster: null,
});

const exists = async (p) => { try { await access(p); return true; } catch { return false; } };

// ===========================================================================
// Case 1 — Brand-new series whose fake graph reaches a SMALLER id.
//   Seed 104462 → PREQUEL → 6213 ; 6213 → SEQUEL → 104462.
//   Final key must be franchise-6213, byId holds the whole component, owned seed
//   is indexed pointing at franchise-6213, provisional franchise-104462.json gone.
// ===========================================================================
{
  owned = { a: { anilistId: 104462 } }; // only the seed is owned
  const graph = new Map([
    [104462, [rel(6213, 'PREQUEL')]],
    [6213, [rel(104462, 'SEQUEL')]],
  ]);
  const calls = [];
  const fetch = async (id) => {
    calls.push(id);
    return { self: self(id), relations: graph.get(id) ?? [], ok: true };
  };

  await crawlFranchiseLive(104462, { fetch, now: clock });

  const file = await readFranchiseFile('franchise-6213');
  assert.ok(file, 'case1: franchise-6213 file should exist');
  assert.equal(file.rootId, 6213, 'case1: rootId is the smallest id');
  assert.deepEqual(
    Object.keys(file.byId).map(Number).sort((a, b) => a - b),
    [6213, 104462],
    'case1: byId holds the whole component',
  );
  assert.equal(file.byId['104462'].fetchedAt, FIXED, 'case1: seed fetched with fixed clock');
  assert.equal(file.byId['6213'].fetchedAt, FIXED, 'case1: discovered node fetched too');

  const index = await readIndex();
  assert.ok(index.library['104462'], 'case1: owned seed is indexed');
  assert.equal(index.library['104462'].franchise, 'franchise-6213', 'case1: index points at franchise-6213');
  // 6213 is NOT owned, so it must not be in the library index.
  assert.equal(index.library['6213'], undefined, 'case1: non-owned member not indexed');

  // Provisional file under the seed key must be gone.
  const { franchiseFilePath } = await import('../src/main/services/franchiseGraph.ts');
  assert.equal(await exists(franchiseFilePath('franchise-104462')), false, 'case1: provisional franchise-104462.json removed');

  console.log('OK case1: brand-new series re-roots to smallest id, provisional removed');
}

// ===========================================================================
// Case 2 — Keep + fill gaps.
//   Pre-seed a file: node 100 fetchedAt>0 (with a relation to missing 101),
//   node 101 present but stale (fetchedAt:0). Crawl → fetcher called ONLY for
//   101; node 100's relations untouched.
// ===========================================================================
{
  owned = { a: { anilistId: 100 } };
  const pre = {
    rootId: 100,
    byId: {
      100: { node: self(100), relations: [rel(101, 'SEQUEL')], fetchedAt: 5 },
      101: { node: self(101), relations: [], fetchedAt: 0 },
    },
  };
  await writeFranchiseFile('franchise-100', pre);

  const calls = [];
  const fetch = async (id) => {
    calls.push(id);
    // 101 relates back to 100 (already known) and nowhere new.
    return { self: self(id), relations: id === 101 ? [rel(100, 'PREQUEL')] : [], ok: true };
  };

  await crawlFranchiseLive(100, { fetch, now: clock });

  assert.deepEqual(calls, [101], 'case2: only the missing id was fetched');
  const file = await readFranchiseFile('franchise-100');
  assert.deepEqual(file.byId['100'].relations, [rel(101, 'SEQUEL')], 'case2: existing fetched node relations untouched');
  assert.equal(file.byId['100'].fetchedAt, 5, 'case2: existing fetchedAt preserved (not re-stamped)');
  assert.equal(file.byId['101'].fetchedAt, FIXED, 'case2: gap node now directly fetched');

  console.log('OK case2: keep + fill gaps — only missing id fetched, existing untouched');
}

// ===========================================================================
// Case 3 — forceRefetch.
//   Pre-seed seed node 200 fetchedAt>0. Crawl with forceRefetch:[200] →
//   fetcher IS called for 200 (refresh path).
// ===========================================================================
{
  owned = { a: { anilistId: 200 } };
  const pre = {
    rootId: 200,
    byId: { 200: { node: self(200), relations: [], fetchedAt: 5 } },
  };
  await writeFranchiseFile('franchise-200', pre);

  const calls = [];
  const fetch = async (id) => {
    calls.push(id);
    return { self: self(id), relations: [], ok: true };
  };

  await crawlFranchiseLive(200, { fetch, now: clock, forceRefetch: [200] });

  assert.deepEqual(calls, [200], 'case3: forceRefetch re-fetched the seed despite fetchedAt>0');
  const file = await readFranchiseFile('franchise-200');
  assert.equal(file.byId['200'].fetchedAt, FIXED, 'case3: seed re-stamped with the new clock');

  console.log('OK case3: forceRefetch re-fetches an already-fetched seed');
}

// ===========================================================================
// Case 4 — Rate-limit defer.
//   Seed 300 → SEQUEL → 301 ; fetcher returns ok:false for 301. That id stays
//   unfetched, the graph is incomplete, no crash, partial tree persisted.
// ===========================================================================
{
  owned = { a: { anilistId: 300 } };
  const graph = new Map([[300, [rel(301, 'SEQUEL')]]]);
  const fetch = async (id) => {
    if (id === 301) return { self: null, relations: [], ok: false }; // rate-limited
    return { self: self(id), relations: graph.get(id) ?? [], ok: true };
  };

  await crawlFranchiseLive(300, { fetch, now: clock }); // must not throw

  const file = await readFranchiseFile('franchise-300');
  assert.ok(file, 'case4: partial tree persisted despite the defer');
  assert.equal(file.byId['300'].fetchedAt, FIXED, 'case4: reachable node fetched');
  // 301 was deferred — it appears as a stale (fetchedAt:0) backfilled node, never directly fetched.
  assert.ok(file.byId['301'], 'case4: deferred node kept for display');
  assert.equal(file.byId['301'].fetchedAt, 0, 'case4: deferred node remains unfetched (stale)');

  console.log('OK case4: rate-limit defer keeps a partial tree, deferred node stays stale');
}

// ===========================================================================
// Case 5 — Incremental persistence.
//   A multi-node crawl must flush more than once (proves the tree forms live).
//   We spy by wrapping writeFranchiseFile via a counter the fetch hook can't
//   reach directly, so instead we count debounced flushes by observing the file
//   change across fetches. Simplest robust proof: count writeFranchiseFile calls
//   through a module-level spy injected by re-importing with a wrapper.
// ===========================================================================
{
  owned = { a: { anilistId: 400 } };
  // Chain 400 → 401 → 402 → 403 so closeGraph fetches several nodes; each fetch
  // schedules a debounced flush plus the final flush → multiple writes.
  const graph = new Map([
    [400, [rel(401, 'SEQUEL')]],
    [401, [rel(402, 'SEQUEL')]],
    [402, [rel(403, 'SEQUEL')]],
    [403, []],
  ]);

  // Spy: wrap the real writer. We can't monkeypatch the bound import inside the
  // module, so instead we assert the live-forming property indirectly — the
  // debounce is 200ms, and each fetch awaits the (fake, instant) fetcher, so by
  // the time closeGraph drains, at least the scheduled flush + final flush have
  // both run. We prove "more than one write" by checking the file exists with
  // the full component AND that a mid-crawl read saw a partial tree.
  let midState = null;
  const fetch = async (id) => {
    // After the first couple of fetches, peek at disk: a debounced flush should
    // have landed a partial file before the crawl finished.
    if (id === 403 && midState === null) {
      // allow any pending 200ms debounce to fire
      await new Promise((r) => setTimeout(r, 250));
      midState = await readFranchiseFile('franchise-400');
    }
    return { self: self(id), relations: graph.get(id) ?? [], ok: true };
  };

  await crawlFranchiseLive(400, { fetch, now: clock });

  assert.ok(midState, 'case5: a debounced flush persisted a partial tree mid-crawl');
  const midKeys = Object.keys(midState.byId).length;
  const final = await readFranchiseFile('franchise-400');
  const finalKeys = Object.keys(final.byId).length;
  assert.ok(midKeys >= 1, 'case5: mid-crawl file held at least one fetched node');
  assert.ok(finalKeys > midKeys, 'case5: final file has strictly more nodes than the mid-crawl snapshot (multiple flushes)');
  assert.equal(finalKeys, 4, 'case5: final component has all four nodes');

  console.log(`OK case5: incremental persistence — mid-crawl ${midKeys} node(s) → final ${finalKeys} (multiple flushes)`);
}

// ===========================================================================
// Bonus — crawlLibraryGaps skips already-covered owned series and crawls gaps.
// ===========================================================================
{
  owned = { a: { anilistId: 500 }, b: { anilistId: 600 } };
  // 500 is already covered (index entry with node + fetchedAt>0); 600 is a gap.
  await writeFranchiseFile('franchise-500', {
    rootId: 500, byId: { 500: { node: self(500), relations: [], fetchedAt: 9 } },
  });
  // Seed the index for 500 the way a prior crawl would have.
  const { writeIndex } = await import('../src/main/services/franchiseGraph.ts');
  const idx0 = await readIndex();
  idx0.library['500'] = { node: self(500), relations: [], fetchedAt: 9, franchise: 'franchise-500' };
  await writeIndex(idx0);

  const calls = [];
  const fetch = async (id) => {
    calls.push(id);
    return { self: self(id), relations: [], ok: true };
  };

  await crawlLibraryGaps({ fetch, now: clock });

  assert.ok(!calls.includes(500), 'gaps: already-covered owned series 500 was skipped');
  assert.ok(calls.includes(600), 'gaps: uncovered owned series 600 was crawled');
  const idx = await readIndex();
  assert.ok(idx.library['600'], 'gaps: 600 now indexed');

  console.log('OK gaps: crawlLibraryGaps skips covered, fills the gap');
}

console.log('OK: franchise crawler');
