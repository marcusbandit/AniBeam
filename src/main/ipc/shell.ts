import { ipcMain, shell } from 'electron';
import { existsSync } from 'fs';
import { resolve, relative } from 'path';
import { spawn } from 'child_process';
import { platform } from 'os';
import configHandler from '../handlers/configHandler';
import { logger } from '../services/logger';

export function registerShellIpc(): void {
  // Open a URL in the user's default browser. window.open() inside the
  // renderer would otherwise spawn a child Electron BrowserWindow, which is
  // not what users expect for things like "Open API config".
  //
  // On Linux we deliberately call `xdg-open` via a detached child process
  // instead of shell.openExternal. Electron's openExternal goes through its
  // own protocol chain, which on Wayland+NVIDIA setups can launch a fresh
  // browser window via a partially-initialised handler rather than routing
  // the URL into the user's running default browser. xdg-open looks up the
  // MIME default directly, and modern browsers (Firefox, Chromium) treat an
  // xdg-open URL as "open in existing instance" by default — which is the
  // behaviour users expect for "Open on AniList".
  ipcMain.handle('shell:open-external', async (_event, url: unknown) => {
    if (typeof url !== 'string' || !/^https?:\/\//i.test(url)) {
      throw new Error('only http(s) URLs may be opened externally');
    }
    if (platform() === 'linux') {
      try {
        const child = spawn('xdg-open', [url], { detached: true, stdio: 'ignore' });
        child.on('error', (err) => {
          logger.warn('system', `xdg-open failed: ${(err as Error).message} — falling back to shell.openExternal`);
          void shell.openExternal(url);
        });
        child.unref();
        return true;
      } catch (err) {
        logger.warn('system', `xdg-open spawn threw: ${(err as Error).message} — using shell.openExternal`);
        await shell.openExternal(url);
        return true;
      }
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
