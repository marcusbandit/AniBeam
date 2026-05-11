import { ipcMain } from 'electron';
import configHandler from '../handlers/configHandler';
import folderHandler from '../handlers/folderHandler';
import metadataHandler from '../handlers/metadataHandler';
import { fileWatcher } from '../services/watcher';
import { logger } from '../services/logger';

export function registerConfigIpc(): void {
  ipcMain.handle('get-folder-sources', async () => {
    try {
      return await configHandler.getFolderSources();
    } catch {
      logger.error('system', 'Error getting folder sources');
      return [];
    }
  });

  ipcMain.handle('find-movie-folders', async (_event, rootPath: string) => {
    try {
      return await folderHandler.findMovieFolders(rootPath);
    } catch (error) {
      logger.error('folder', `Error finding movie folders: ${(error as Error).message}`);
      return [];
    }
  });

  ipcMain.handle('add-folder-source', async (_event, folderPath: string) => {
    try {
      const ok = await configHandler.addFolderSource(folderPath);
      if (ok) {
        const roots = await configHandler.getFolderSources();
        await fileWatcher.restart(roots);
        logger.info('folder', `Added library root: ${folderPath}`);
      }
      return ok;
    } catch (error) {
      logger.error('folder', `Error adding folder source: ${(error as Error).message}`);
      throw error;
    }
  });

  ipcMain.handle('remove-folder-source', async (_event, folderPath: string) => {
    try {
      const ok = await configHandler.removeFolderSource(folderPath);
      if (ok) {
        logger.info('folder', `Removed library root: ${folderPath}`);
        const activeRoots = await configHandler.getFolderSources();
        await metadataHandler.transaction(async (meta) => {
          const reconciled = await folderHandler.reconcileMetadata(meta, activeRoots);
          return { updated: reconciled };
        });
        await fileWatcher.restart(activeRoots);
      }
      return ok;
    } catch (error) {
      logger.error('folder', `Error removing folder source: ${(error as Error).message}`);
      throw error;
    }
  });
}
