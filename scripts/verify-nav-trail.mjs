import assert from 'node:assert/strict';

const { labelForPath, readTrail, activeTab, carryState, RAIL_TABS, MAIN_SCROLL_ID, MAX_TRAIL } =
  await import('../src/renderer/hooks/navTrailCore.ts');

// --- labelForPath: one generic name per route pattern ---
assert.equal(labelForPath('/'), 'Library');
assert.equal(labelForPath('/feed'), 'Feed');
assert.equal(labelForPath('/watching'), 'Watching');
assert.equal(labelForPath('/subscriptions'), 'Subscriptions');
assert.equal(labelForPath('/settings'), 'Settings');
assert.equal(labelForPath('/metadata'), 'Metadata');
assert.equal(labelForPath('/series/12345'), 'Series');
assert.equal(labelForPath('/player/12345/3'), 'Player');

// Prefix matching, so a tab's own sub-paths keep the tab's name.
assert.equal(labelForPath('/feed/anything'), 'Feed');

// Anything we don't recognise still gets a usable Back button.
assert.equal(labelForPath('/nonsense'), 'Back');
assert.equal(labelForPath(''), 'Back');

// --- readTrail: state comes from history and may be anything ---
assert.deepEqual(readTrail(null), []);
assert.deepEqual(readTrail(undefined), []);
assert.deepEqual(readTrail({}), []);
assert.deepEqual(readTrail({ trail: null }), []);
assert.deepEqual(readTrail({ trail: 'nope' }), []);
assert.deepEqual(readTrail({ trail: { path: '/feed', label: 'Feed' } }), []);
assert.deepEqual(readTrail('a string'), []);

// Malformed entries are dropped, well-formed neighbours survive.
assert.deepEqual(
  readTrail({
    trail: [
      null,
      'string entry',
      42,
      { label: 'no path' },
      { path: '/feed' },
      { path: 7, label: 'numeric path' },
      { path: '/watching', label: 'Watching' },
    ],
  }),
  [{ path: '/watching', label: 'Watching' }],
);

// A well-formed trail passes through in order.
assert.deepEqual(
  readTrail({ trail: [{ path: '/feed', label: 'Feed' }, { path: '/series/1', label: 'Cowboy Bebop' }] }),
  [{ path: '/feed', label: 'Feed' }, { path: '/series/1', label: 'Cowboy Bebop' }],
);

// --- readTrail: the optional scroll offset ---
// Entries written before scroll existed have no field and stay valid.
assert.deepEqual(readTrail({ trail: [{ path: '/feed', label: 'Feed' }] }), [{ path: '/feed', label: 'Feed' }]);

// A real offset rides along.
assert.deepEqual(
  readTrail({ trail: [{ path: '/feed', label: 'Feed', scroll: 840 }] }),
  [{ path: '/feed', label: 'Feed', scroll: 840 }],
);
assert.deepEqual(
  readTrail({ trail: [{ path: '/feed', label: 'Feed', scroll: 0 }] }),
  [{ path: '/feed', label: 'Feed', scroll: 0 }],
);

// A junk offset is ignored rather than poisoning the entry.
for (const junk of ['840', NaN, Infinity, null, {}]) {
  const [entry] = readTrail({ trail: [{ path: '/feed', label: 'Feed', scroll: junk }] });
  assert.deepEqual(entry, { path: '/feed', label: 'Feed' }, `scroll=${String(junk)} should be dropped`);
  assert.equal(entry.scroll, undefined);
}

// --- RAIL_TABS: the rail's destinations, in rail order ---
assert.deepEqual(RAIL_TABS.map((t) => t.path), ['/', '/feed', '/watching', '/metadata', '/settings']);
assert.ok(RAIL_TABS.every((t) => typeof t.label === 'string' && t.label.length > 0));
assert.equal(MAIN_SCROLL_ID, 'main-scroll');
assert.equal(MAX_TRAIL, 12);

// --- activeTab: which rail tab lights up ---
// A rail tab lights itself, no trail needed.
for (const tab of RAIL_TABS) {
  assert.equal(activeTab(tab.path, []), tab.path, `${tab.path} should light itself`);
}

// Subscriptions is reached from Settings and has no rail entry of its own.
assert.equal(activeTab('/subscriptions', []), '/settings');
assert.equal(activeTab('/subscriptions', [{ path: '/feed', label: 'Feed' }]), '/settings');

// A series page inherits the tab it was browsed from.
assert.equal(activeTab('/series/1', [{ path: '/feed', label: 'Feed' }]), '/feed');
assert.equal(activeTab('/series/1', [{ path: '/watching', label: 'Watching' }]), '/watching');
assert.equal(activeTab('/series/1', [{ path: '/metadata', label: 'Metadata' }]), '/metadata');

// Deeper crawls keep the ROOT of the trail, not the nearest ancestor.
assert.equal(
  activeTab('/series/3', [
    { path: '/feed', label: 'Feed' },
    { path: '/series/1', label: 'A' },
    { path: '/series/2', label: 'B' },
  ]),
  '/feed',
);

// A query string on the trail entry must not hide the tab.
assert.equal(activeTab('/series/1', [{ path: '/feed?x=1', label: 'Feed' }]), '/feed');
assert.equal(activeTab('/series/1', [{ path: '/watching?tab=behind&sort=title', label: 'Watching' }]), '/watching');

// Cold deep link (no trail at all) falls back to Library.
assert.equal(activeTab('/series/1', []), '/');

// A trail that never touches a rail tab also falls back to Library.
assert.equal(activeTab('/series/2', [{ path: '/series/1', label: 'A' }]), '/');

// The player has no rail, but asking is still safe.
assert.equal(activeTab('/player/1/3', [{ path: '/watching', label: 'Watching' }]), '/watching');
assert.equal(activeTab('/player/1/3', []), '/');

// An unknown route with no trail is Library, same as a cold deep link.
assert.equal(activeTab('/nonsense', []), '/');

// --- carryState: keep the trail, drop the one-shot scroll instruction ---

// A same-page rewrite keeps the trail intact...
assert.deepEqual(
  carryState({ trail: [{ path: '/feed', label: 'Feed', scroll: 840 }] }),
  { trail: [{ path: '/feed', label: 'Feed', scroll: 840 }] },
);

// ...but never carries `restoreScroll` forward, or every keystroke in the
// Library search box would re-fire the restore.
const carried = carryState({
  trail: [{ path: '/feed', label: 'Feed' }],
  restoreScroll: 840,
});
assert.deepEqual(carried, { trail: [{ path: '/feed', label: 'Feed' }] });
assert.ok(!('restoreScroll' in carried));

// Nothing durable to keep means no state at all, not an empty trail.
assert.equal(carryState(undefined), undefined);
assert.equal(carryState(null), undefined);
assert.equal(carryState({ restoreScroll: 840 }), undefined);
assert.equal(carryState({ trail: [] }), undefined);
assert.equal(carryState({ trail: 'junk' }), undefined);

console.log('OK: nav trail core');
