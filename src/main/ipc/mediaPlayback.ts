import { ipcMain } from 'electron';
import { existsSync } from 'fs';
import metadataHandler from '../handlers/metadataHandler';
import videoProbeHandler from '../handlers/videoProbeHandler';
import transcodeCacheHandler from '../handlers/transcodeCacheHandler';
import subtitleHandler from '../handlers/subtitleHandler';
import aniSkipHandler from '../handlers/aniSkipHandler';
import { findFileEpisode } from '../../shared/fileEpisode';
import { probeCodecs, needsTranscode } from '../utils/transcodeProbe';
import { getViewHistory, markViewed } from '../services/viewHistory';
import type { WindowGetter } from './types';

export function registerMediaPlaybackIpc(getMainWindow?: WindowGetter): void {
  const broadcastViewHistoryChanged = (): void => {
    const win = getMainWindow?.();
    if (win && !win.isDestroyed()) win.webContents.send('playback:view-history-changed');
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
    const probe = await probeCodecs(filePath);
    if (probe && needsTranscode(probe)) {
      void transcodeCacheHandler.enqueue(filePath);
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

  ipcMain.handle('subtitle:list-embedded', async (_event, videoPath: string) => {
    if (typeof videoPath !== 'string' || !videoPath) return [];
    return subtitleHandler.listEmbedded(videoPath);
  });

  ipcMain.handle('subtitle:extract', async (_event, videoPath: string, streamIndex: number, codec: string) => {
    if (typeof videoPath !== 'string' || !videoPath || typeof streamIndex !== 'number') return null;
    return subtitleHandler.extractEmbedded(videoPath, streamIndex, codec ?? '');
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
