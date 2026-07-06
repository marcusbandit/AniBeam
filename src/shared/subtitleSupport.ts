// Pure, isomorphic subtitle-support logic. No Electron imports — usable from the
// main process (probing/extraction), the renderer (play-time outcome), and the
// verify scripts.
//
// Why this exists: a file can have subtitle streams that AniBeam still can't
// show. Bitmap formats (PGS/DVD, the Blu-ray kind) render fine in mpv but we
// have no OCR, and a text stream can fail to extract. In both cases the player's
// subtitle list comes back empty and the subtitle icon silently disappears, with
// nothing recorded — so the series view can't warn the user. These helpers
// classify a stream's codec and turn a probe/playback summary into a persisted
// `subtitleState` that drives the episode-row marker.

/** Persisted per-file subtitle outcome. Absent ⇒ unknown / genuinely no subs. */
export type SubtitleState = 'ok' | 'unsupported' | 'failed';

// Text codecs we can actually render: ASS/SSA go to libass (JASSUB); the rest
// convert to WebVTT for the browser's native track flow. Keep this the single
// source of truth — subtitleHandler's extraction mapping reads it too.
export const ASS_CODECS = new Set(['ass', 'ssa']);
export const VTT_CODECS = new Set(['subrip', 'webvtt', 'mov_text']);
// Image-based subtitles. We list the common ffmpeg codec names; anything that
// isn't renderable text and isn't here still counts as non-renderable ('other').
export const BITMAP_CODECS = new Set([
  'hdmv_pgs_subtitle', 'pgssub', 'dvd_subtitle', 'dvdsub', 'xsub',
  'dvb_subtitle', 'dvbsub', 'dvb_teletext',
]);

export type SubtitleCodecKind = 'ass' | 'vtt' | 'bitmap' | 'other';

/** Map an ffprobe codec_name to how (or whether) we can render it. */
export function classifySubtitleCodec(codec: string | null | undefined): SubtitleCodecKind {
  const c = (codec ?? '').toLowerCase();
  if (ASS_CODECS.has(c)) return 'ass';
  if (VTT_CODECS.has(c)) return 'vtt';
  if (BITMAP_CODECS.has(c)) return 'bitmap';
  return 'other';
}

/** True for codecs we can extract + display (ASS or VTT). */
export function isRenderableSubtitleCodec(codec: string | null | undefined): boolean {
  const k = classifySubtitleCodec(codec);
  return k === 'ass' || k === 'vtt';
}

/**
 * Decide a file's subtitle state from a cheap, probe-only summary (no extraction
 * attempted). This is what the series-view sweep persists for every episode:
 *  - any external sidecar, or any renderable embedded stream ⇒ 'ok'
 *  - subtitle streams exist but none are renderable (bitmap-only / unknown) ⇒ 'unsupported'
 *  - no subtitle content at all ⇒ null (no marker)
 *
 * It never returns 'failed' — that requires actually attempting extraction, which
 * only the play-time path does (see derivePlaybackSubtitleState).
 */
export function deriveSubtitleState(input: {
  hasSidecar: boolean;
  renderableCount: number;
  nonRenderableCount: number;
}): SubtitleState | null {
  if (input.hasSidecar || input.renderableCount > 0) return 'ok';
  if (input.nonRenderableCount > 0) return 'unsupported';
  return null;
}

/**
 * Decide a file's subtitle state from the actual play-time build result. This is
 * authoritative for the "text track exists but extraction failed" case the cheap
 * probe can't see:
 *  - at least one subtitle source loaded ⇒ 'ok'
 *  - we attempted to load embedded text streams but ended up with nothing ⇒ 'failed'
 *  - nothing to load (no sidecars, no embedded text streams) ⇒ null
 *
 * Returns null (rather than clearing) when there was nothing to attempt, so the
 * player never overwrites a proactively-detected 'unsupported' for a bitmap-only
 * file (whose text-stream count is zero).
 */
export function derivePlaybackSubtitleState(input: {
  loadedCount: number;
  candidateStreamCount: number;
}): SubtitleState | null {
  if (input.loadedCount > 0) return 'ok';
  if (input.candidateStreamCount > 0) return 'failed';
  return null;
}
