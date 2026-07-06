import { ipcMain } from 'electron';
import { existsSync } from 'fs';
import { stat } from 'fs/promises';
import metadataHandler from '../handlers/metadataHandler';
import videoProbeHandler from '../handlers/videoProbeHandler';
import transcodeCacheHandler from '../handlers/transcodeCacheHandler';
import subtitleHandler from '../handlers/subtitleHandler';
import aniSkipHandler from '../handlers/aniSkipHandler';
import { findFileEpisode } from '../../shared/fileEpisode';
import type { FileEpisodeEntry } from '../../shared/fileEpisode';
import { probeCodecs, needsTranscode } from '../utils/transcodeProbe';
import { getViewHistory, markViewed } from '../services/viewHistory';
import type { SubtitleState } from '../../shared/subtitleSupport';
import type { WindowGetter } from './types';
import type { TranscodeQueueSnapshot } from '../preload';

// Map a source filePath back to its seriesId — the top-level metadata key
// whose fileEpisodes[] contains the file. (findFileEpisode only returns the
// entry, not which series owns it.) Returns null for untracked files.
function seriesIdForPath(meta: Record<string, unknown>, filePath: string): string | null {
  for (const [seriesId, series] of Object.entries(meta)) {
    const s = series as { fileEpisodes?: FileEpisodeEntry[] };
    if (!Array.isArray(s.fileEpisodes)) continue;
    if (s.fileEpisodes.some((f) => f.filePath === filePath)) return seriesId;
  }
  return null;
}

// Resolve the raw {active, queued} path split into a series-level status map.
// The active series → 'encoding'; each queued series → 'queued' UNLESS it's
// already 'encoding'. Used both for the on-demand snapshot and the broadcast.
export async function resolveQueueSnapshot(
  raw: { activePath: string | null; queuedPaths: string[] },
): Promise<TranscodeQueueSnapshot> {
  let meta: Record<string, unknown> = {};
  try {
    meta = await metadataHandler.loadMetadata();
  } catch {
    // No metadata → no way to resolve series; return an empty map.
    return {};
  }
  const snapshot: TranscodeQueueSnapshot = {};
  for (const filePath of raw.queuedPaths) {
    const seriesId = seriesIdForPath(meta, filePath);
    if (seriesId && snapshot[seriesId] !== 'encoding') snapshot[seriesId] = 'queued';
  }
  if (raw.activePath) {
    const seriesId = seriesIdForPath(meta, raw.activePath);
    if (seriesId) snapshot[seriesId] = 'encoding';
  }
  return snapshot;
}

export function registerMediaPlaybackIpc(getMainWindow?: WindowGetter): void {
  const broadcastViewHistoryChanged = (): void => {
    const win = getMainWindow?.();
    if (win && !win.isDestroyed()) win.webContents.send('playback:view-history-changed');
  };

  const broadcastSubtitleStateChanged = (filePath: string, subtitleState: SubtitleState): void => {
    const win = getMainWindow?.();
    if (win && !win.isDestroyed()) {
      win.webContents.send('metadata:subtitle-state-changed', { filePath, subtitleState });
    }
  };

  ipcMain.handle('probe:retry', (_event, filePath: string) => {
    if (typeof filePath === 'string' && filePath.length > 0) {
      videoProbeHandler.retry(filePath);
    }
  });

  ipcMain.handle('video:open', async (_event, filePath: unknown) => {
    if (typeof filePath !== 'string' || !filePath) {
      throw new Error('filePath required');
    }
    // Prefer a pre-transcoded copy if one is recorded in metadata AND
    // exists on disk. Two independent checks because metadata can be stale
    // (user deleted cache outside the app) and the cache file can exist
    // without metadata (rare; scan-time enqueue path always persists).
    try {
      const meta = await metadataHandler.loadMetadata();
      const hit = findFileEpisode(meta, filePath);
      if (hit?.transcodedPath && existsSync(hit.transcodedPath)) {
        return { kind: 'direct' as const, url: `file://${hit.transcodedPath}` };
      }
    } catch {
      // Fall through to codec check — better to keep trying than to fail
      // the open entirely.
    }
    // No transcoded copy on disk. Probe the original — if Chromium can't
    // decode it (HEVC, etc.), kick off (or join) a transcode and tell the
    // renderer so it can show progress while ffmpeg runs. The renderer
    // re-calls openVideo when it sees a 'ready' status for this file.
    // enqueue() is idempotent — duplicates collapse into the same encode.
    // priority: true jumps this specific episode to the absolute front of
    // the queue — the user is actively waiting on it, so it encodes next
    // (ahead of any bulk series sweep already queued).
    const probe = await probeCodecs(filePath);
    if (probe && needsTranscode(probe)) {
      void transcodeCacheHandler.enqueue(filePath, { priority: true });
      return {
        kind: 'transcoding' as const,
        vCodec: probe.vCodec,
        aCodec: probe.aCodec,
      };
    }
    return { kind: 'direct' as const, url: `file://${filePath}` };
  });

  // Opening a series page should kick off re-encoding for every episode that
  // needs it — without the user clicking each one. The renderer hands us the
  // series' file paths (in episode order); we classify each and priority-
  // enqueue the ones that need transcoding so they encode next, ahead of the
  // bulk startup sweep. Returns each file's state so the page can draw a
  // progress bar where the "Re-encoded" tag will eventually sit.
  ipcMain.handle('transcode:ensure-series', async (_event, filePaths: unknown) => {
    if (!Array.isArray(filePaths)) return [];
    const paths = filePaths.filter((p): p is string => typeof p === 'string' && p.length > 0);
    if (paths.length === 0) return [];
    let meta: Record<string, unknown> = {};
    try {
      meta = await metadataHandler.loadMetadata();
    } catch {
      // Fall through — we can still probe codecs without metadata.
    }
    // Classify in parallel (one ffprobe each), preserving input order.
    const results = await Promise.all(paths.map(async (filePath) => {
      try {
        const hit = findFileEpisode(meta, filePath);
        if (hit?.transcodedPath && existsSync(hit.transcodedPath)) {
          return { filePath, state: 'cached' as const };
        }
        if (!existsSync(filePath)) return { filePath, state: 'none' as const };
        const probe = await probeCodecs(filePath);
        if (probe && needsTranscode(probe)) return { filePath, state: 'pending' as const };
        return { filePath, state: 'none' as const };
      } catch {
        return { filePath, state: 'none' as const };
      }
    }));
    // Enqueue pending files in reverse so the FIRST listed episode ends up at
    // the front of the queue after the unshifts — earliest episode encodes
    // first. enqueue() is idempotent; already-queued files just move up.
    const pending = results.filter((r) => r.state === 'pending');
    for (let i = pending.length - 1; i >= 0; i--) {
      void transcodeCacheHandler.enqueue(pending[i].filePath, { priority: true });
    }
    return results;
  });

  // On-demand pull of the series-level transcode queue (for a renderer that
  // just mounted). Not debounced — the live broadcast on 'transcode:queue-
  // changed' handles streaming updates.
  ipcMain.handle('transcode:queue-snapshot', async (): Promise<TranscodeQueueSnapshot> => {
    return resolveQueueSnapshot(transcodeCacheHandler.queueSnapshot());
  });

  ipcMain.handle('subtitle:list-embedded', async (_event, videoPath: string) => {
    if (typeof videoPath !== 'string' || !videoPath) return [];
    return subtitleHandler.listEmbedded(videoPath);
  });

  ipcMain.handle('subtitle:extract', async (_event, videoPath: string, streamIndex: number, codec: string) => {
    if (typeof videoPath !== 'string' || !videoPath || typeof streamIndex !== 'number') return null;
    return subtitleHandler.extractEmbedded(videoPath, streamIndex, codec ?? '');
  });

  // Fire-and-forget cache warm-up so subtitles don't extract on the play-time
  // critical path (the cold-extract takes ~an OP's length). Returns nothing.
  ipcMain.handle('subtitle:prewarm', async (_event, videoPath: string) => {
    if (typeof videoPath !== 'string' || !videoPath) return;
    void subtitleHandler.prewarm(videoPath);
  });

  // Opening a series page sweeps every episode for subtitle availability so the
  // list can flag the ones whose subs won't display (bitmap-only / unreadable).
  // Cheap probe-only check, mirroring transcode:ensure-series: one ffprobe per
  // file, skipping any already checked against the current on-disk mtime. The
  // computed states are persisted (so a reload shows them instantly) and also
  // returned so the page can paint markers without waiting for a reload.
  ipcMain.handle('subtitle:evaluate-series', async (_event, filePaths: unknown) => {
    if (!Array.isArray(filePaths)) return [];
    const paths = filePaths.filter((p): p is string => typeof p === 'string' && p.length > 0);
    if (paths.length === 0) return [];
    let meta: Record<string, unknown> = {};
    try {
      meta = await metadataHandler.loadMetadata();
    } catch {
      // Without metadata we can't know about sidecars or persist — bail.
      return [];
    }
    const evaluated = await Promise.all(paths.map(async (filePath) => {
      try {
        const entry = findFileEpisode(meta, filePath);
        const hasSidecar = !!entry?.subtitlePath || !!(entry?.subtitlePaths && entry.subtitlePaths.length);
        if (!existsSync(filePath)) {
          return { filePath, state: entry?.subtitleState ?? null, fresh: false };
        }
        const { mtimeMs } = await stat(filePath);
        // Reuse a state already computed against this exact file revision.
        if (entry && entry.subtitleState !== undefined
          && typeof entry.subtitleCheckedAt === 'number' && entry.subtitleCheckedAt >= mtimeMs) {
          return { filePath, state: entry.subtitleState ?? null, fresh: false };
        }
        const state = await subtitleHandler.evaluateAvailability(filePath, hasSidecar);
        return { filePath, state, checkedAt: mtimeMs, fresh: true };
      } catch {
        return { filePath, state: null, fresh: false };
      }
    }));
    // Persist the freshly-computed states in one transaction.
    const fresh = evaluated.filter((r): r is typeof r & { checkedAt: number } => r.fresh === true);
    if (fresh.length > 0) {
      const byPath = new Map(fresh.map((r) => [r.filePath, r]));
      await metadataHandler.transaction<boolean>(async (m) => {
        let changed = false;
        for (const series of Object.values(m)) {
          const s = series as { fileEpisodes?: FileEpisodeEntry[] };
          if (!Array.isArray(s.fileEpisodes)) continue;
          for (const file of s.fileEpisodes) {
            const r = byPath.get(file.filePath);
            if (!r) continue;
            // Don't clobber a fresher result. The authoritative play-time report
            // stamps wall-clock time (Date.now, always >> a file's mtime), so if
            // one landed between our load and this commit it outranks this
            // mtime-stamped probe and must win.
            if (file.subtitleCheckedAt != null && file.subtitleCheckedAt >= r.checkedAt) continue;
            file.subtitleState = r.state;
            file.subtitleCheckedAt = r.checkedAt;
            changed = true;
          }
        }
        return { result: changed, updated: changed ? m : null };
      });
    }
    return evaluated.map((r) => ({ filePath: r.filePath, state: r.state }));
  });

  // Authoritative play-time outcome from the player's buildSubs: 'ok' once a
  // track actually loaded, 'failed' when embedded text streams existed but none
  // could be extracted (the case the cheap probe can't see). Persisted + pushed
  // so an open series page updates live.
  ipcMain.handle('subtitle:report-state', async (_event, filePath: unknown, state: unknown) => {
    if (typeof filePath !== 'string' || !filePath) return;
    if (state !== 'ok' && state !== 'unsupported' && state !== 'failed') return;
    const touched = await metadataHandler.transaction<boolean>(async (m) => {
      let changed = false;
      for (const series of Object.values(m)) {
        const s = series as { fileEpisodes?: FileEpisodeEntry[] };
        if (!Array.isArray(s.fileEpisodes)) continue;
        for (const file of s.fileEpisodes) {
          if (file.filePath === filePath) {
            file.subtitleState = state;
            file.subtitleCheckedAt = Date.now();
            changed = true;
          }
        }
      }
      return { result: changed, updated: changed ? m : null };
    });
    if (touched) broadcastSubtitleStateChanged(filePath, state);
  });

  ipcMain.handle('aniskip:fetch', async (_event, seriesId: string, malId: number, episodeNumber: number, episodeLength: number, filePath?: string) => {
    if (!seriesId || typeof episodeNumber !== 'number' || typeof episodeLength !== 'number') {
      return {};
    }
    // malId is optional now — chapters can resolve skip times without one.
    const safeMalId = typeof malId === 'number' ? malId : 0;
    const safeFilePath = typeof filePath === 'string' && filePath ? filePath : undefined;
    return aniSkipHandler.fetchAndCache(seriesId, safeMalId, episodeNumber, episodeLength, safeFilePath);
  });

  ipcMain.handle('playback:get-view-history', async () => {
    return getViewHistory();
  });

  ipcMain.handle('playback:viewed', async (_event, payload: unknown) => {
    if (!payload || typeof payload !== 'object') return false;
    const p = payload as { seriesId?: unknown; episodeNumber?: unknown; ts?: unknown };
    if (typeof p.seriesId !== 'string' || !p.seriesId) return false;
    if (typeof p.episodeNumber !== 'number') return false;
    const ts = typeof p.ts === 'number' ? p.ts : Date.now();
    const changed = await markViewed(p.seriesId, p.episodeNumber, ts);
    if (changed) broadcastViewHistoryChanged();
    return changed;
  });
}
