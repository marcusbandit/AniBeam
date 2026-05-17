import { ipcMain } from 'electron';
import { existsSync } from 'fs';
import metadataHandler from '../handlers/metadataHandler';
import videoProbeHandler from '../handlers/videoProbeHandler';
import subtitleHandler from '../handlers/subtitleHandler';
import aniSkipHandler from '../handlers/aniSkipHandler';
import transcodeCacheHandler from '../handlers/transcodeCacheHandler';
import { findFileEpisode } from '../../shared/fileEpisode';
import { probeCodecs, needsTranscode } from '../utils/transcodeProbe';

export function registerMediaPlaybackIpc(): void {
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
    // decode it (HEVC, exotic pixel formats, etc.) kick off a background
    // transcode so the renderer can wait for it to finish, then play
    // the cached MP4. Returns 'transcoding' so the renderer shows a
    // progress state and listens for the file's status to flip to
    // 'ready', at which point it re-invokes this handler and gets a
    // 'direct' URL.
    const probe = await probeCodecs(filePath);
    if (probe && needsTranscode(probe)) {
      void transcodeCacheHandler.enqueue(filePath).catch(() => {/* status emit covers errors */});
      return {
        kind: 'transcoding' as const,
        vCodec: probe.vCodec,
        aCodec: probe.aCodec,
      };
    }
    return { kind: 'direct' as const, url: `file://${filePath}` };
  });

  ipcMain.handle('subtitle:list-embedded', async (_event, videoPath: string) => {
    if (typeof videoPath !== 'string' || !videoPath) return [];
    return subtitleHandler.listEmbedded(videoPath);
  });

  ipcMain.handle('subtitle:extract', async (_event, videoPath: string, streamIndex: number, codec: string) => {
    if (typeof videoPath !== 'string' || !videoPath || typeof streamIndex !== 'number') return null;
    return subtitleHandler.extractEmbedded(videoPath, streamIndex, codec ?? '');
  });

  ipcMain.handle('aniskip:fetch', async (_event, seriesId: string, malId: number, episodeNumber: number, episodeLength: number) => {
    if (!seriesId || typeof malId !== 'number' || typeof episodeNumber !== 'number' || typeof episodeLength !== 'number') {
      return {};
    }
    return aniSkipHandler.fetchAndCache(seriesId, malId, episodeNumber, episodeLength);
  });
}
