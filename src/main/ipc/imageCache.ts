import { ipcMain } from 'electron';
import imageCacheHandler from '../handlers/imageCacheHandler';
import { logger } from '../services/logger';

export function registerImageCacheIpc(): void {
  ipcMain.handle('get-image-cache-stats', async () => {
    try {
      return await imageCacheHandler.getCacheStats();
    } catch {
      logger.error('image', 'Error getting image cache stats');
      return { count: 0, sizeBytes: 0 };
    }
  });

  ipcMain.handle('clear-image-cache', async () => {
    try {
      await imageCacheHandler.clearCache();
      return true;
    } catch (error) {
      logger.error('image', 'Error clearing image cache');
      throw error;
    }
  });

  ipcMain.handle('get-image-cache-path', () => imageCacheHandler.getCachePath());
}
