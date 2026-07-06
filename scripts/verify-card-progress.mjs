import assert from 'node:assert/strict';

const { computeCardProgress } = await import('../src/renderer/utils/airingUtils.ts');

const NOW = Date.parse('2026-07-01T00:00:00Z');
const past = (days) => new Date(NOW - days * 86400_000).toISOString();
const future = (days) => new Date(NOW + days * 86400_000).toISOString();

const near = (actual, expected, label) =>
  assert.ok(Math.abs(actual - expected) < 0.01, `${label}: expected ~${expected}, got ${actual}`);

// The known range squeezes into the track minus the 15% unknown cap.
const USABLE = 85;

// --- Releasing show, known total: blue watched + rose aired-but-unwatched ---
{
  const p = computeCardProgress({
    watched: 5,
    totalEpisodes: 12,
    episodes: [
      { episodeNumber: 8, airDate: past(1) },
      { episodeNumber: 9, airDate: future(6) },
    ],
    status: 'RELEASING',
    nowMs: NOW,
  });
  near(p.watchedPct, (5 / 12) * 100, 'releasing watched');
  near(p.behindPct, (8 / 12) * 100, 'releasing behind underlay reaches aired');
  near(p.unknownPct, 0, 'known total has no unknown cap');
}

// --- Finished show: NO rose, even with unwatched episodes ---
{
  const p = computeCardProgress({
    watched: 5,
    totalEpisodes: 12,
    episodes: [{ episodeNumber: 12, airDate: past(30) }],
    status: 'FINISHED',
    nowMs: NOW,
  });
  near(p.watchedPct, (5 / 12) * 100, 'finished watched');
  near(p.behindPct, 0, 'finished shows no rose');
  near(p.unknownPct, 0, 'finished no cap');
}

// --- All episodes aired but status still releasing: fully released → no rose ---
{
  const p = computeCardProgress({
    watched: 3,
    totalEpisodes: 12,
    episodes: [{ episodeNumber: 12, airDate: past(1) }],
    status: 'RELEASING',
    nowMs: NOW,
  });
  near(p.behindPct, 0, 'released >= total counts as fully released');
}

// --- MAL-style finished string normalizes the same way ---
{
  const p = computeCardProgress({
    watched: 4,
    totalEpisodes: 24,
    episodes: [],
    status: 'Finished Airing',
    nowMs: NOW,
  });
  near(p.behindPct, 0, 'MAL finished no rose');
  near(p.watchedPct, (4 / 24) * 100, 'MAL finished watched');
}

// --- Unknown total: known range squeezes left of the dark cap ---
{
  const p = computeCardProgress({
    watched: 5,
    totalEpisodes: null,
    episodes: [{ episodeNumber: 8, airDate: past(1) }],
    status: 'RELEASING',
    nowMs: NOW,
  });
  near(p.watchedPct, (5 / 8) * USABLE, 'unknown total watched squeezed');
  near(p.behindPct, USABLE, 'unknown total behind fills to the cap');
  near(p.unknownPct, 100 - USABLE, 'unknown cap present');
}

// --- "05/?": watched only, nothing aired known — never fully filled ---
{
  const p = computeCardProgress({
    watched: 5,
    totalEpisodes: null,
    episodes: [],
    nowMs: NOW,
  });
  near(p.watchedPct, USABLE, 'watched-only fills to the cap, not 100');
  near(p.behindPct, 0, 'no released info means no rose');
  near(p.unknownPct, 100 - USABLE, 'cap marks the unknown remainder');
}

// --- Finished but total unpublished: cap yes, rose no ---
{
  const p = computeCardProgress({
    watched: 2,
    totalEpisodes: null,
    episodes: [{ episodeNumber: 10, airDate: past(100) }],
    status: 'FINISHED',
    nowMs: NOW,
  });
  near(p.behindPct, 0, 'finished suppresses rose even without a total');
  near(p.unknownPct, 100 - USABLE, 'unpublished total still shows the cap');
}

// --- Untracked (watched null): no rose, empty blue ---
{
  const p = computeCardProgress({
    watched: null,
    totalEpisodes: 12,
    episodes: [{ episodeNumber: 8, airDate: past(1) }],
    status: 'RELEASING',
    nowMs: NOW,
  });
  near(p.watchedPct, 0, 'untracked has no blue');
  near(p.behindPct, 0, 'untracked has no rose');
}

// --- Watching-tab shape: only a future nextAiringEpisode is known.
//     Episode 9 airing later implies 8 released. ---
{
  const p = computeCardProgress({
    watched: 6,
    totalEpisodes: 12,
    episodes: [{ episodeNumber: 9, airDate: future(3) }],
    status: 'RELEASING',
    nowMs: NOW,
  });
  near(p.behindPct, (8 / 12) * 100, 'next-scheduled implies released');
}

// --- The EARLIEST future episode bounds released, not the latest ---
{
  const p = computeCardProgress({
    watched: 0,
    totalEpisodes: 12,
    episodes: [
      { episodeNumber: 5, airDate: future(1) },
      { episodeNumber: 12, airDate: future(50) },
    ],
    status: 'RELEASING',
    nowMs: NOW,
  });
  near(p.behindPct, (4 / 12) * 100, 'earliest future episode wins');
}

// --- Downloaded-but-no-airdate counts as released ---
{
  const p = computeCardProgress({
    watched: 2,
    totalEpisodes: 12,
    episodes: [],
    latestDownloadedEpisode: 7,
    status: 'RELEASING',
    nowMs: NOW,
  });
  near(p.behindPct, (7 / 12) * 100, 'on-disk counts as released');
}

// --- Clamping: watched beyond total never overflows ---
{
  const p = computeCardProgress({
    watched: 15,
    totalEpisodes: 12,
    episodes: [],
    status: 'FINISHED',
    nowMs: NOW,
  });
  near(p.watchedPct, 100, 'watched clamps at 100');
}

// --- Nothing to scale against → null (no overlay at all) ---
{
  assert.equal(
    computeCardProgress({ watched: null, totalEpisodes: null, episodes: [], nowMs: NOW }),
    null,
    'no data returns null',
  );
  assert.equal(
    computeCardProgress({ watched: 0, totalEpisodes: 0, episodes: null, nowMs: NOW }),
    null,
    'zero total returns null',
  );
}

// --- Upcoming show with a known total: bar exists but every fill is 0 ---
{
  const p = computeCardProgress({
    watched: 0,
    totalEpisodes: 12,
    episodes: [{ episodeNumber: 1, airDate: future(30) }],
    status: 'NOT_YET_RELEASED',
    nowMs: NOW,
  });
  near(p.watchedPct, 0, 'upcoming watched is 0');
  near(p.behindPct, 0, 'upcoming has no rose');
}

console.log('verify-card-progress: all assertions passed');
