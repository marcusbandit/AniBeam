import { ipcMain } from 'electron';
import { existsSync } from 'fs';
import metadataHandler from '../handlers/metadataHandler';
import videoProbeHandler from '../handlers/videoProbeHandler';
import subtitleHandler from '../handlers/subtitleHandler';
import aniSkipHandler from '../handlers/aniSkipHandler';
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
    // decode it (HEVC, etc.) tell the renderer so it can offer system mpv
    // instead of handing <video> a URL it'll just choke on.
    const probe = await probeCodecs(filePath);
    if (probe && needsTranscode(probe)) {
      return {
        kind: 'unsupported' as const,
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

  ipcMain.handle('aniskip:fetch', async (_event, seriesId: string, malId: number, episodeNumber: number, episodeLength: number, filePath?: string) => {
    if (!seriesId || typeof episodeNumber !== 'number' || typeof episodeLength !== 'number') {
      return {};
    }
    // malId is optional now — chapters can resolve skip times without one.
    const safeMalId = typeof malId === 'number' ? malId : 0;
    const safeFilePath = typeof filePath === 'string' && filePath ? filePath : undefined;
    return aniSkipHandler.fetchAndCache(seriesId, safeMalId, episodeNumber, episodeLength, safeFilePath);
  });
}
