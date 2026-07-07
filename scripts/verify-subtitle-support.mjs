import assert from 'node:assert/strict';

const {
  classifySubtitleCodec,
  isRenderableSubtitleCodec,
  deriveSubtitleState,
  derivePlaybackSubtitleState,
  isEnglishSubtitleStream,
  isSignsSubtitleStream,
  pickDefaultSubtitleStream,
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

// ---- pickDefaultSubtitleStream (the ONE track the gate waits on) ------------
const stream = (streamIndex, language, title) => ({ streamIndex, codec: 'ass', language, title });

// Signs track first in stream order, both tagged eng: dialogue still wins.
{
  const signs = stream(2, 'eng', 'English (Signs & Songs)');
  const full = stream(3, 'eng', 'English');
  const pick = pickDefaultSubtitleStream([signs, full]);
  assert.equal(pick.stream.streamIndex, 3, 'dialogue outranks signs regardless of order');
  assert.equal(pick.reason, 'english dialogue');
}
// Untagged multisub (language lives in the title): the English title is found.
{
  const pick = pickDefaultSubtitleStream([
    stream(2, 'und', 'Español (Latinoamérica)'),
    stream(3, null, 'English'),
    stream(4, 'und', 'Português (Brasil)'),
  ]);
  assert.equal(pick.stream.streamIndex, 3, 'title-only English is identified');
  assert.equal(pick.reason, 'english dialogue');
}
// Only English track is forced-only: picked, but flagged as signs/forced.
{
  const pick = pickDefaultSubtitleStream([
    stream(2, 'spa', 'Español'),
    stream(3, 'eng', 'English (Forced)'),
  ]);
  assert.equal(pick.stream.streamIndex, 3, 'english forced beats wrong language');
  assert.equal(pick.reason, 'english (signs/forced)');
}
// No English anywhere: fall back to probe order (waiting for a track that
// doesn't exist would gate playback forever).
{
  const pick = pickDefaultSubtitleStream([
    stream(2, 'spa', 'Español'),
    stream(3, 'fre', 'Français'),
  ]);
  assert.equal(pick.stream.streamIndex, 2, 'no English → first stream');
  assert.equal(pick.reason, 'first stream fallback');
}
// Ties keep stream order: two full English tracks → the first one.
{
  const pick = pickDefaultSubtitleStream([
    stream(2, 'eng', 'English'),
    stream(3, 'eng', 'English (SDH)'),
  ]);
  assert.equal(pick.stream.streamIndex, 2, 'equal rank keeps probe order');
}
assert.equal(pickDefaultSubtitleStream([]), null, 'empty input → null');

// Language-tag matching: BCP47 en-* forms count, enm (Middle English) does not.
assert.equal(isEnglishSubtitleStream(stream(0, 'en-US', null)), true, 'en-US is English');
assert.equal(isEnglishSubtitleStream(stream(0, 'EN', null)), true, 'tag matching is case-insensitive');
assert.equal(isEnglishSubtitleStream(stream(0, 'enm', null)), false, 'enm alone is not English');
assert.equal(isEnglishSubtitleStream(stream(0, null, null)), false, 'nothing to identify');
// SDH/CC are full dialogue, not signs.
assert.equal(isSignsSubtitleStream(stream(0, 'eng', 'English (SDH)')), false, 'SDH is dialogue');
assert.equal(isSignsSubtitleStream(stream(0, 'eng', 'English [CC]')), false, 'CC is dialogue');
assert.equal(isSignsSubtitleStream(stream(0, 'eng', 'Signs/Songs')), true);
assert.equal(isSignsSubtitleStream(stream(0, 'eng', 'Commentary')), true);

console.log('verify-subtitle-support: all assertions passed');
