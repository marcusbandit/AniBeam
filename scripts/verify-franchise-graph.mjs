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
  fetch: async (id) => fetched.get(id) ?? [],
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

console.log('OK: franchise graph closure');
