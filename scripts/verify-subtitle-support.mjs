import assert from 'node:assert/strict';

const {
  classifySubtitleCodec,
  isRenderableSubtitleCodec,
  deriveSubtitleState,
  derivePlaybackSubtitleState,
} = await import('../src/shared/subtitleSupport.ts');

// ---- classifySubtitleCodec --------------------------------------------------
// Text codecs we can render, bitmap codecs we can't (mpv can — this is the
// whole reason the marker exists), and the case-insensitive / unknown fallback.
const codecCases = [
  ['ass', 'ass'],
  ['SSA', 'ass'],
  ['subrip', 'vtt'],
  ['webvtt', 'vtt'],
  ['mov_text', 'vtt'],
  ['hdmv_pgs_subtitle', 'bitmap'],
  ['dvd_subtitle', 'bitmap'],
  ['DVDSUB', 'bitmap'],
  ['dvb_subtitle', 'bitmap'],
  ['eia_608', 'other'],
  ['', 'other'],
  [null, 'other'],
  [undefined, 'other'],
];
for (const [codec, expected] of codecCases) {
  assert.equal(classifySubtitleCodec(codec), expected, `classify ${String(codec)}`);
}
assert.equal(isRenderableSubtitleCodec('ass'), true);
assert.equal(isRenderableSubtitleCodec('subrip'), true);
assert.equal(isRenderableSubtitleCodec('hdmv_pgs_subtitle'), false, 'PGS is not renderable');
assert.equal(isRenderableSubtitleCodec('eia_608'), false, 'unknown is not renderable');

// ---- deriveSubtitleState (cheap probe-only sweep) ---------------------------
// An external sidecar always wins.
assert.equal(deriveSubtitleState({ hasSidecar: true, renderableCount: 0, nonRenderableCount: 0 }), 'ok');
assert.equal(deriveSubtitleState({ hasSidecar: true, renderableCount: 0, nonRenderableCount: 3 }), 'ok', 'sidecar beats bitmap-only embedded');
// A renderable embedded text stream → ok.
assert.equal(deriveSubtitleState({ hasSidecar: false, renderableCount: 1, nonRenderableCount: 0 }), 'ok');
assert.equal(deriveSubtitleState({ hasSidecar: false, renderableCount: 2, nonRenderableCount: 1 }), 'ok', 'any renderable text → ok');
// Bitmap/unknown only → the marker case. This is the "mpv shows it, AniBeam can't" file.
assert.equal(deriveSubtitleState({ hasSidecar: false, renderableCount: 0, nonRenderableCount: 1 }), 'unsupported');
// No subtitle content at all → no marker.
assert.equal(deriveSubtitleState({ hasSidecar: false, renderableCount: 0, nonRenderableCount: 0 }), null);
// The cheap sweep can NEVER return 'failed' — that needs an extraction attempt.
for (const c of codecCases) void c; // (kept for readability; nothing to assert here)

// ---- derivePlaybackSubtitleState (authoritative play-time outcome) ----------
// Something actually loaded → ok.
assert.equal(derivePlaybackSubtitleState({ loadedCount: 1, candidateStreamCount: 2 }), 'ok');
assert.equal(derivePlaybackSubtitleState({ loadedCount: 3, candidateStreamCount: 0 }), 'ok', 'sidecars-only still ok');
// Text streams were present but none loaded → failed (the silent-extract-failure case).
assert.equal(derivePlaybackSubtitleState({ loadedCount: 0, candidateStreamCount: 1 }), 'failed');
// Nothing to attempt → null, so the player never clobbers a proactively-set
// 'unsupported' (bitmap-only files report zero embedded TEXT streams here).
assert.equal(derivePlaybackSubtitleState({ loadedCount: 0, candidateStreamCount: 0 }), null);

console.log('verify-subtitle-support: all assertions passed');
