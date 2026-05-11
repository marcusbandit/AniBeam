import { ipcMain, shell } from 'electron';
import { existsSync } from 'fs';
import { resolve, relative } from 'path';
import { spawn } from 'child_process';
import configHandler from '../handlers/configHandler';
import { logger } from '../services/logger';

export function registerShellIpc(): void {
  // Open a URL in the user's default browser. window.open() inside the
  // renderer would otherwise spawn a child Electron BrowserWindow, which is
  // not what users expect for things like "Open API config".
  ipcMain.handle('shell:open-external', async (_event, url: unknown) => {
    if (typeof url !== 'string' || !/^https?:\/\//i.test(url)) {
      throw new Error('only http(s) URLs may be opened externally');
    }
    await shell.openExternal(url);
    return true;
  });

  // Launch mpv on a local video file in a detached window. Validated against
  // configured library roots so the renderer can't request arbitrary paths.
  ipcMain.handle('shell:open-with-mpv', async (_event, filePath: unknown) => {
    if (typeof filePath !== 'string' || !filePath) {
      throw new Error('filePath required');
    }
    const normalizedPath = resolve(filePath);
    const allowedSources = await configHandler.getFolderSources();
    const isAllowed = allowedSources.some((source) => {
      try {
        const normalizedSource = resolve(source);
        const rel = relative(normalizedSource, normalizedPath);
        return !rel.startsWith('..') && !rel.startsWith('/');
      } catch {
        return false;
      }
    });
    if (!isAllowed) {
      logger.error('system', `mpv: rejected path outside library roots`, { file: filePath });
      throw new Error('path not in any configured library root');
    }
    if (!existsSync(normalizedPath)) {
      throw new Error('file not found');
    }
    try {
      const child = spawn('mpv', [normalizedPath], { detached: true, stdio: 'ignore' });
      child.on('error', (err) => {
        logger.error('system', `mpv launch failed: ${(err as Error).message}`, { file: normalizedPath });
      });
      child.unref();
      logger.info('system', `Launched mpv`, { file: normalizedPath });
      return true;
    } catch (err) {
      logger.error('system', `mpv spawn threw: ${(err as Error).message}`, { file: normalizedPath });
      throw err;
    }
  });
}
