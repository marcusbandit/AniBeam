import { ipcMain, dialog } from 'electron';
import { basename } from 'path';
import configHandler from '../handlers/configHandler';
import folderHandler, { type ScannedMedia } from '../handlers/folderHandler';
import metadataHandler from '../handlers/metadataHandler';
import { logger } from '../services/logger';
import type { WindowGetter } from './types';

export function registerFolderIpc(getMainWindow: WindowGetter): void {
  ipcMain.handle('select-folder', async () => {
    const win = getMainWindow();
    if (!win) return null;
    const result = await dialog.showOpenDialog(win, {
      properties: ['openDirectory'],
      title: 'Select Anime Folder',
    });
    if (!result.canceled && result.filePaths.length > 0) return result.filePaths[0];
    return null;
  });

  ipcMain.handle('scan-folder', async (_event, folderPath: string) => {
    try {
      return await folderHandler.scanFolder(folderPath);
    } catch (error) {
      logger.error('folder', 'Error scanning folder');
      throw error;
    }
  });

  // Pure filesystem walk of every configured library root, joined with just
  // the bits of metadata.json the renderer actually displays right now
  // (poster URL/local path + match state). No episode-level metadata, no
  // banners, no online thumbnails. The renderer renders directly off this —
  // folder names verbatim, posters when matched, placeholder when not.
  ipcMain.handle('library:walk', async () => {
    const roots = await configHandler.getFolderSources();
    const all: ScannedMedia[] = [];
    for (const root of roots) {
      try {
        const items = await folderHandler.scanFolder(root, [root]);
        all.push(...items);
      } catch (err) {
        logger.warn('folder', `library:walk failed for ${root}: ${(err as Error).message}`, { file: root });
      }
    }
    const meta = await metadataHandler.loadMetadata();
    return all.map((m) => {
      const stored = (meta[m.id] ?? {}) as {
        poster?: string | null;
        posterLocal?: string | null;
        posterMatched?: boolean;
        posterMatchAttempted?: boolean;
        matchSource?: 'mal' | 'anilist';
        matchedTitle?: string | null;
        titleRomaji?: string | null;
        titleEnglish?: string | null;
        format?: string | null;
        status?: string | null;
        startDate?: string | null;
        totalEpisodes?: number | null;
        anilistId?: number;
        malId?: number | null;
        averageScore?: number | null;
        source?: string | null;
        episodes?: Array<{ episodeNumber: number; airDate: string | null }>;
      };
      // The folder scanner classifies by structure alone, so a franchise
      // sub-entry that's really a film (e.g. "Girls und Panzer der Film")
      // gets tagged 'series'. Metadata knows better — when the matched
      // format is MOVIE, trust it so the Movies tab, detail page, and
      // metadata view all agree on the type.
      const resolvedType = stored.format === 'MOVIE' ? 'movie' : m.type;
      return {
        id: m.id,
        // Movies share the "Movies" parent folder, so basename(folderPath)
        // would be "Movies" for every movie. Use the scanner-derived title
        // (m.name) as the display fallback so the card shows "Kimi no Na Wa"
        // rather than "Movies".
        folderName: m.type === 'movie' ? m.name : basename(m.folderPath),
        folderPath: m.folderPath,
        type: resolvedType,
        poster: stored.poster ?? null,
        posterLocal: stored.posterLocal ?? null,
        posterMatched: stored.posterMatched ?? false,
        posterMatchAttempted: stored.posterMatchAttempted ?? false,
        matchSource: stored.matchSource ?? null,
        matchedTitle: stored.matchedTitle ?? null,
        titleRomaji: stored.titleRomaji ?? stored.matchedTitle ?? null,
        titleEnglish: stored.titleEnglish ?? null,
        status: stored.status ?? null,
        startDate: stored.startDate ?? null,
        totalEpisodes: stored.totalEpisodes ?? null,
        anilistId: stored.anilistId ?? null,
        malId: stored.malId ?? null,
        averageScore: stored.averageScore ?? null,
        source: stored.source ?? null,
        episodes: (stored.episodes ?? []).map((e) => ({
          episodeNumber: e.episodeNumber,
          airDate: e.airDate ?? null,
        })),
        files: m.files.map((f) => ({
          filename: f.filename,
          filePath: f.filePath,
          title: f.title,
          episodeNumber: f.episodeNumber,
          seasonNumber: f.seasonNumber,
          subtitlePath: f.subtitlePath,
          subtitlePaths: f.subtitlePaths,
          mtime: f.mtime,
          kind: f.kind,
          extraIndex: f.extraIndex,
          extraVariant: f.extraVariant,
          rawLabel: f.rawLabel,
        })),
      };
    });
  });

  ipcMain.handle('scan-all-folders', async () => {
    try {
      const folderSources = await configHandler.getFolderSources();
      if (folderSources.length === 0) return [];
      return await folderHandler.scanMultipleFolders(folderSources);
    } catch (error) {
      logger.error('folder', 'Error scanning all folders');
      throw error;
    }
  });
}
