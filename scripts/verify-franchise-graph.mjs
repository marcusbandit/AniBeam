import assert from 'node:assert/strict';
import { closeGraph, isTraversable } from '../src/shared/franchise.ts';

const mk = (id, relationType, over = {}) => ({
  relationType, anilistId: id, malId: null, type: 'ANIME', format: 'TV',
  status: 'FINISHED', seasonYear: 2000 + id, siteUrl: null,
  titleRomaji: `T${id}`, titleEnglish: null, poster: null, ...over,
});
const node = (id) => ({
  anilistId: id, malId: null, type: 'ANIME', format: 'TV', status: 'FINISHED',
  seasonYear: 2000 + id, siteUrl: null, titleRomaji: `T${id}`, titleEnglish: null, poster: null,
});

// isTraversable excludes CHARACTER and OTHER
assert.equal(isTraversable('SEQUEL'), true);
assert.equal(isTraversable('CHARACTER'), false);
assert.equal(isTraversable('OTHER'), false);
assert.equal(isTraversable('SIDE_STORY'), true);

// Transitive closure through a non-seed node via fetch
const seedRelations = new Map([[1, [mk(2, 'SEQUEL')]]]);
const fetched = new Map([[2, [mk(1, 'PREQUEL'), mk(3, 'SEQUEL')]]]);
const g1 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations,
  fetch: async (id) => ({ relations: fetched.get(id) ?? [], ok: true }),
});
assert.deepEqual(g1.nodes.map((n) => n.anilistId).sort((a, b) => a - b), [1, 2, 3]);
assert.equal(g1.rootId, 1);
assert.equal(g1.complete, true);

// CHARACTER edges are dropped entirely (node 9 must not appear)
const g2 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations: new Map([[1, [mk(9, 'CHARACTER')]]]),
});
assert.deepEqual(g2.nodes.map((n) => n.anilistId), [1]);
assert.equal(g2.complete, true);

// OTHER nodes are displayed but not traversed (node 5 appears, its rels are not followed)
const g3 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations: new Map([[1, [mk(5, 'OTHER')]]]),
  fetch: async () => { throw new Error('should not fetch through OTHER'); },
});
assert.deepEqual(g3.nodes.map((n) => n.anilistId).sort((a, b) => a - b), [1, 5]);

// nodeCap halts discovery and marks incomplete
const g4 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations: new Map([[1, [mk(2, 'SEQUEL'), mk(3, 'SEQUEL'), mk(4, 'SEQUEL')]]]),
  nodeCap: 2,
});
assert.equal(g4.nodes.length, 2);
assert.equal(g4.complete, false);
assert.deepEqual(g4.nodes.map((n) => n.anilistId).sort((a, b) => a - b), [1, 2]);

// Duplicate relations collapse to a single edge
const g5 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations: new Map([[1, [mk(2, 'SEQUEL'), mk(2, 'SEQUEL')]]]),
});
assert.equal(g5.edges.length, 1);
assert.equal(g5.nodes.length, 2);

// Rate-limited fetch defers the node; graph is incomplete with deferred ids.
let calls = 0;
const g6 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations: new Map([[1, [mk(2, 'SEQUEL')]]]),
  fetch: async (id) => { calls++; return { relations: [], ok: false }; },
});
assert.equal(g6.complete, false);
assert.deepEqual(g6.deferred, [2]);
assert.equal(calls, 1);

// Subsequent call with the same seed but a now-succeeding fetcher closes the gap.
const g7 = await closeGraph({
  seedNodes: [node(1)],
  seedRelations: new Map([[1, [mk(2, 'SEQUEL')]]]),
  fetch: async (id) => ({ relations: id === 2 ? [mk(3, 'SEQUEL')] : [], ok: true }),
});
assert.equal(g7.complete, true);
assert.deepEqual(g7.deferred, []);
assert.deepEqual(g7.nodes.map(n => n.anilistId).sort((a,b)=>a-b), [1,2,3]);

// A deferred node that gets retried via a back-edge and succeeds clears deferred.
let g8Calls = 0;
const g8 = await closeGraph({
  seedNodes: [node(1)],
  // 1 → SEQUEL → 2 ; we'll defer the fetch for 2 once, then succeed and have 2 → PREQUEL → 1 (back-edge already known)
  seedRelations: new Map([[1, [mk(2, 'SEQUEL')]]]),
  fetch: async (id) => {
    g8Calls++;
    if (id === 2 && g8Calls === 1) return { relations: [], ok: false };
    // Second attempt for 2 (re-queued through some path) — succeed with no further relations.
    return { relations: [], ok: true };
  },
});
// Note: with just seedNodes=[1] and seedRelations=[[1,[mk(2,'SEQUEL')]]] the only re-queue path
// for 2 would be if another node references it. This case is therefore a defer-once-only test:
assert.equal(g8.complete, false);
assert.deepEqual(g8.deferred, [2]);

// Now exercise the actual same-pass-retry path: a back-edge re-queues a deferred node.
let g9Calls = 0;
const g9 = await closeGraph({
  seedNodes: [node(1)],
  // 1 → SEQUEL → 2 ; 1 also "knows" 2 again via another relation (duplicate edges dedup),
  // but to truly force a re-queue we use a 3-node setup: 1 → 2 (defer), 1 → 3 (ok with 3 → 2)
  seedRelations: new Map([
    [1, [mk(2, 'SEQUEL'), mk(3, 'SIDE_STORY')]],
    [3, [mk(2, 'PARENT')]],
  ]),
  fetch: async (id) => {
    g9Calls++;
    // Defer the FIRST fetch attempt for node 2; succeed on the second.
    if (id === 2 && g9Calls === 1) return { relations: [], ok: false };
    return { relations: [], ok: true };
  },
});
assert.equal(g9.complete, true);            // <-- the bug-fix assertion: deferred cleared on successful retry
assert.deepEqual(g9.deferred, []);
assert.ok(g9.nodes.length >= 3);

console.log('OK: franchise graph closure');
