// What happens after an external player (mpv) finishes with an episode.
//
// The in-window player does three separate things as you watch: it keeps a
// resume position in the renderer's localStorage, marks the series viewed once
// you've accumulated ~30s, and bumps AniList/MAL past the auto-mark threshold.
// Watching in mpv used to do none of them, so an episode watched there was
// invisible to the app.
//
// This module applies the same three outcomes from a single end-of-session
// report. It deliberately mirrors the in-window player's thresholds rather
// than inventing new ones — an episode should count as watched the same way
// regardless of which player showed it.

import type { BrowserWindow } from 'electron';
import { logger } from '../services/logger';
import { markViewed } from '../services/viewHistory';
import metadataHandler from './metadataHandler';
import trackerHandler from './trackerHandler';
import type { MpvPlaybackReport } from '../services/mpvPlayback';
import type { TrackerProvider } from '../services/trackerStore';

/** Which episode an mpv window was showing. mpv itself only knows a path. */
export interface MpvSessionContext {
  seriesId: string | null;
  episodeNumber: number | null;
  /** Extras (OP/ED/PV/SP) share an episodeNumber with a real episode, so they
   *  must never move a tracker or the view history. */
  isExtra: boolean;
}

/** Seconds of real playback before the series counts as "viewed". Same value
 *  the in-window player uses for its Library "Last viewed" sort. */
const VIEW_THRESHOLD_SEC = 30;
/** Fraction of the episode that counts as finished. The in-window player takes
 *  the earlier of AniSkip's outro start and 85%; we have no AniSkip data for an
 *  mpv session, so it's the 85% leg alone. */
const AUTO_MARK_FRACTION = 0.85;

interface SeriesIds {
  anilistId: number | null;
  malId: number | null;
  totalEpisodes: number | null;
}

async function lookupSeriesIds(seriesId: string): Promise<SeriesIds | null> {
  try {
    const meta = await metadataHandler.getSeriesMetadata(seriesId);
    if (!meta) return null;
    const anilistId = typeof meta.anilistId === 'number' ? meta.anilistId : null;
    const malId = typeof meta.malId === 'number' ? meta.malId : null;
    const totalEpisodes = typeof meta.totalEpisodes === 'number' ? meta.totalEpisodes : null;
    return { anilistId, malId, totalEpisodes };
  } catch (err) {
    logger.warn('system', `mpv: could not read series metadata: ${(err as Error).message}`);
    return null;
  }
}

/**
 * Push the episode's progress to the two trackers the user has linked.
 * Mirrors the tracker:mark-episode IPC, including its hidden-series guard —
 * an incognito series must not sync just because it was played in mpv.
 */
async function markOnTrackers(
  ids: SeriesIds,
  episodeNumber: number,
): Promise<string[]> {
  const targets: Array<{ provider: TrackerProvider; mediaId: number }> = [];
  if (ids.anilistId) targets.push({ provider: 'anilist', mediaId: ids.anilistId });
  if (ids.malId) targets.push({ provider: 'mal', mediaId: ids.malId });
  const marked: string[] = [];
  for (const { provider, mediaId } of targets) {
    try {
      if (await metadataHandler.isMediaHidden(provider, mediaId)) continue;
      const res = await trackerHandler.markEpisode({
        provider,
        mediaId,
        episodeNumber,
        totalEpisodes: ids.totalEpisodes,
      });
      if (res.ok) marked.push(`${provider.toUpperCase()} → ep ${res.newProgress}`);
    } catch (err) {
      logger.warn('system', `mpv: ${provider} mark failed: ${(err as Error).message}`);
    }
  }
  return marked;
}

/**
 * Apply an mpv session's outcome. Called once, when mpv exits.
 *
 * Never throws: a playback session that already happened shouldn't be able to
 * fail retroactively, and there is no user-facing operation left to abort.
 */
export async function finishMpvSession(
  report: MpvPlaybackReport,
  ctx: MpvSessionContext,
  win: BrowserWindow | null,
): Promise<void> {
  try {
    // We never got a word out of mpv (socket refused, or it was killed before
    // it opened one). Reporting position 0 here would wipe a real resume point,
    // so treat it as "no information" and leave everything alone.
    if (!report.tracked) {
      logger.warn('system', 'mpv: session ended without progress data — nothing recorded', {
        file: report.filePath,
      });
      return;
    }

    const alive = win && !win.isDestroyed() ? win : null;

    // 1. Resume position. The map lives in the renderer's localStorage (it's
    //    the same store the in-window player reads on open), so main can only
    //    hand the numbers over and let the renderer apply the head/tail rules.
    alive?.webContents.send('playback:mpv-ended', {
      filePath: report.filePath,
      seriesId: ctx.seriesId,
      episodeNumber: ctx.episodeNumber,
      isExtra: ctx.isExtra,
      position: report.position,
      duration: report.duration,
    });

    // Everything below is episode-level bookkeeping; an extra isn't an episode.
    if (ctx.isExtra || !ctx.seriesId || ctx.episodeNumber == null) return;

    // 2. View history — "I watched some of this", by real playback time rather
    //    than where the playhead ended up, so scrubbing to the credits and
    //    quitting doesn't count.
    if (report.watchedSec >= VIEW_THRESHOLD_SEC) {
      const changed = await markViewed(ctx.seriesId, ctx.episodeNumber, Date.now());
      if (changed) alive?.webContents.send('playback:view-history-changed');
    }

    // 3. Trackers. Needs a duration to know what "finished" means; mpv reports
    //    one for anything seekable.
    if (report.duration <= 0) return;
    if (report.position < report.duration * AUTO_MARK_FRACTION) return;

    const ids = await lookupSeriesIds(ctx.seriesId);
    if (!ids) return;
    if (!ids.anilistId && !ids.malId) {
      // Same bail as the in-window player: almost always an unmatched series.
      logger.warn('system', 'mpv: watched to the end but the series has no AniList/MAL id', {
        series: ctx.seriesId,
      });
      return;
    }
    const marked = await markOnTrackers(ids, ctx.episodeNumber);
    if (marked.length > 0) {
      logger.info('system', `Tracked from mpv · ${marked.join(' · ')}`, { series: ctx.seriesId });
      alive?.webContents.send('tracker:progress-changed');
    }
  } catch (err) {
    logger.warn('system', `mpv: could not record session: ${(err as Error).message}`, {
      file: report.filePath,
    });
  }
}
