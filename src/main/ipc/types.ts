import type { BrowserWindow } from 'electron';

// Lazy reference to the main window. Each IPC group is registered once at
// app startup but the window itself can be recreated (window-all-closed →
// activate on macOS), so handlers resolve it on demand instead of capturing
// a stale reference.
export type WindowGetter = () => BrowserWindow | null;
