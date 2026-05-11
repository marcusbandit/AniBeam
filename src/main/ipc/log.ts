import { ipcMain } from 'electron';
import { logger } from '../services/logger';

export function registerLogIpc(): void {
  ipcMain.handle('log:get-buffer', () => logger.getBuffer());
  ipcMain.handle('log:clear', () => { logger.clear(); });
}
