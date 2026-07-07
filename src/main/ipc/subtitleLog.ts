import { ipcMain } from 'electron';
import { subLog } from '../services/subtitleDebugLog';

export function registerSubtitleLogIpc(): void {
  // Fire-and-forget debug lines from the renderer's subtitle machinery
  // (playback gate, retry supervisor, JASSUB init). The `renderer/` prefix is
  // added here, not in the renderer, so a line's origin in the log file is
  // always truthful.
  ipcMain.on('subtitle:log', (_event, scope: unknown, message: unknown, data?: unknown) => {
    if (typeof scope !== 'string' || typeof message !== 'string') return;
    subLog(`renderer/${scope.slice(0, 64)}`, message.slice(0, 1000), data);
  });
}
