import { ipcMain, dialog } from 'electron';
import { writeFile } from 'fs/promises';
import { buildExport, defaultExportFileName } from '../handlers/exportHandler';
import { logger } from '../services/logger';
import type { RendererExportState, ExportWriteResult } from '../preload';
import type { WindowGetter } from './types';

export function registerExportIpc(getMainWindow: WindowGetter): void {
  ipcMain.handle('export:write', async (
    _event,
    includePrivate: unknown,
    rendererState: unknown,
  ): Promise<ExportWriteResult> => {
    const state = (rendererState ?? {}) as Partial<RendererExportState>;
    const normalized: RendererExportState = {
      videoProgress: state.videoProgress ?? null,
      videoLastEpisode: state.videoLastEpisode ?? null,
      titleLanguage: state.titleLanguage ?? null,
      libraryTab: state.libraryTab ?? null,
      librarySortKey: state.librarySortKey ?? null,
      librarySortDir: state.librarySortDir ?? null,
      feedSort: state.feedSort ?? null,
    };

    const win = getMainWindow();
    const defaultPath = defaultExportFileName(includePrivate === true);
    const dialogOpts = {
      title: 'Export library',
      defaultPath,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    };
    const result = win
      ? await dialog.showSaveDialog(win, dialogOpts)
      : await dialog.showSaveDialog(dialogOpts);
    if (result.canceled || !result.filePath) return { ok: false, reason: 'canceled' };

    try {
      const data = await buildExport(includePrivate === true, normalized);
      await writeFile(result.filePath, JSON.stringify(data, null, 2), 'utf-8');
      logger.info('export', `Wrote ${includePrivate === true ? 'full' : 'library'} export`, { file: result.filePath });
      return { ok: true, path: result.filePath };
    } catch (err) {
      logger.error('export', `Failed to write export: ${(err as Error).message}`);
      return { ok: false, reason: 'error', message: (err as Error).message };
    }
  });
}
