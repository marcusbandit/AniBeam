import { ipcMain, shell } from 'electron';
import { existsSync } from 'fs';
import { resolve, relative } from 'path';
import { spawn } from 'child_process';
import { platform } from 'os';
import configHandler from '../handlers/configHandler';
import { logger } from '../services/logger';
import { playInMpv } from '../services/mpvPlayback';
import { finishMpvSession, type MpvSessionContext } from '../handlers/externalPlaybackHandler';
import type { WindowGetter } from './types';

export function registerShellIpc(getMainWindow?: WindowGetter): void {
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

  // Launch mpv on a local video file. Validated against configured library
  // roots so the renderer can't request arbitrary paths.
  //
  // The launch resolves as soon as mpv is up; the session keeps running in the
  // background and reports its final position through onMpvPlaybackEnded, which
  // is what turns watching in mpv into a resume point, a view-history entry and
  // a tracker bump. `context` carries which episode this is — mpv only knows a
  // file path, and an extra (OP/ED/PV) must never bump a tracker.
  ipcMain.handle('shell:open-with-mpv', async (_event, filePath: unknown, context: unknown) => {
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
    const ctx = (context && typeof context === 'object' ? context : {}) as {
      seriesId?: unknown; episodeNumber?: unknown; isExtra?: unknown; startSec?: unknown;
    };
    const session: MpvSessionContext = {
      seriesId: typeof ctx.seriesId === 'string' && ctx.seriesId ? ctx.seriesId : null,
      episodeNumber: typeof ctx.episodeNumber === 'number' && Number.isFinite(ctx.episodeNumber)
        ? ctx.episodeNumber
        : null,
      isExtra: ctx.isExtra === true,
    };
    const startSec = typeof ctx.startSec === 'number' && Number.isFinite(ctx.startSec) && ctx.startSec > 0
      ? ctx.startSec
      : undefined;

    logger.info('system', `Launched mpv`, { file: normalizedPath });
    // Fire-and-forget: the renderer's await must not block for the length of
    // an episode. Errors are reported through the same finish path.
    void playInMpv(normalizedPath, { startSec })
      .then((report) => finishMpvSession(report, session, getMainWindow?.() ?? null))
      .catch((err: Error) => {
        logger.error('system', `mpv launch failed: ${err.message}`, { file: normalizedPath });
      });
    return true;
  });
}
