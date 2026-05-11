import { ipcMain } from 'electron';
import subscriptionsHandler from '../handlers/subscriptionsHandler';

export function registerSubscriptionsIpc(): void {
  ipcMain.handle('subscriptions:list', async () => subscriptionsHandler.list());
}
