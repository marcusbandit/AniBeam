import { ipcMain } from 'electron';
import trackerHandler from '../handlers/trackerHandler';
import { logger } from '../services/logger';
import type { TrackerProvider } from '../services/trackerStore';
import type { WindowGetter } from './types';

function isProvider(v: unknown): v is TrackerProvider {
  return v === 'anilist' || v === 'mal';
}

export function registerTrackerIpc(getMainWindow: WindowGetter): void {
  const broadcastProgressChanged = (): void => {
    const win = getMainWindow();
    if (win && !win.isDestroyed()) win.webContents.send('tracker:progress-changed');
  };

  ipcMain.handle('tracker:status', async (_event, provider: unknown) => {
    if (!isProvider(provider)) throw new Error('invalid provider');
    return trackerHandler.status(provider);
  });

  ipcMain.handle('tracker:set-client-id', async (_event, provider: unknown, clientId: unknown) => {
    if (!isProvider(provider)) throw new Error('invalid provider');
    if (typeof clientId !== 'string') throw new Error('clientId must be a string');
    await trackerHandler.setClientId(provider, clientId);
    return trackerHandler.status(provider);
  });

  ipcMain.handle('tracker:get-client-id', async (_event, provider: unknown) => {
    if (!isProvider(provider)) throw new Error('invalid provider');
    return trackerHandler.getClientId(provider);
  });

  ipcMain.handle('tracker:connect', async (_event, provider: unknown, clientId: unknown, clientSecret: unknown) => {
    if (!isProvider(provider)) throw new Error('invalid provider');
    if (typeof clientId !== 'string' || !clientId.trim()) throw new Error('clientId required');
    const secret = typeof clientSecret === 'string' ? clientSecret.trim() : '';
    const status = await trackerHandler.startConnect(provider, clientId.trim(), secret);
    // Warm the progress cache so cards render with watched counts immediately.
    await trackerHandler.refreshProgress(provider);
    broadcastProgressChanged();
    return status;
  });

  ipcMain.handle('tracker:cancel-connect', async () => {
    trackerHandler.cancelConnect();
    return true;
  });

  ipcMain.handle('tracker:disconnect', async (_event, provider: unknown) => {
    if (!isProvider(provider)) throw new Error('invalid provider');
    const status = await trackerHandler.disconnect(provider);
    broadcastProgressChanged();
    return status;
  });

  ipcMain.handle('tracker:mark-episode', async (
    _event,
    provider: unknown,
    mediaId: unknown,
    episodeNumber: unknown,
    totalEpisodes: unknown,
  ) => {
    if (!isProvider(provider)) throw new Error('invalid provider');
    if (typeof mediaId !== 'number' || typeof episodeNumber !== 'number') {
      throw new Error('mediaId and episodeNumber must be numbers');
    }
    const result = await trackerHandler.markEpisode({
      provider,
      mediaId,
      episodeNumber,
      totalEpisodes: typeof totalEpisodes === 'number' ? totalEpisodes : null,
    });
    if (result.ok) broadcastProgressChanged();
    return result;
  });

  ipcMain.handle('tracker:get-progress', async () => {
    const snap = await trackerHandler.getProgress();
    logger.info(
      'tracker',
      `get-progress called: main=${snap.mainProvider} anilist=${Object.keys(snap.anilist).length} mal=${Object.keys(snap.mal).length}`,
    );
    return snap;
  });

  ipcMain.handle('tracker:refresh-progress', async (_event, provider: unknown) => {
    if (provider === undefined || provider === null) {
      await trackerHandler.refreshAllProgress();
    } else {
      if (!isProvider(provider)) throw new Error('invalid provider');
      await trackerHandler.refreshProgress(provider);
    }
    broadcastProgressChanged();
    return trackerHandler.getProgress();
  });

  ipcMain.handle('tracker:get-main-provider', async () => trackerHandler.getMainProvider());

  ipcMain.handle('tracker:set-main-provider', async (_event, provider: unknown) => {
    if (!isProvider(provider)) throw new Error('invalid provider');
    await trackerHandler.setMainProvider(provider);
    broadcastProgressChanged();
    return provider;
  });
}
