import chokidar, { type FSWatcher } from 'chokidar';
import { logger } from './logger';

export interface WatcherCallbacks {
  onAdd: (filePath: string) => void;
  onUnlink: (filePath: string) => void;
  onUnlinkDir: (dirPath: string) => void;
}

const VIDEO_EXTS = new Set(['.mkv', '.mp4', '.avi', '.mov', '.webm', '.m4v', '.ts']);
const IGNORED_PATTERNS = [/(^|[\/\\])\../, /\.part$/i, /\.crdownload$/i, /\.tmp$/i];

let watcher: FSWatcher | null = null;
let activeRoots: string[] = [];
let callbacks: WatcherCallbacks | null = null;
const debounceMap = new Map<string, NodeJS.Timeout>();
const DEBOUNCE_MS = 1000;

function isVideo(path: string): boolean {
  const dot = path.lastIndexOf('.');
  if (dot < 0) return false;
  return VIDEO_EXTS.has(path.slice(dot).toLowerCase());
}

function debounce(key: string, fn: () => void): void {
  const existing = debounceMap.get(key);
  if (existing) clearTimeout(existing);
  debounceMap.set(key, setTimeout(() => {
    debounceMap.delete(key);
    fn();
  }, DEBOUNCE_MS));
}

function attach(w: FSWatcher): void {
  w.on('add', (path) => {
    if (!isVideo(path)) return;
    debounce(`add:${path}`, () => {
      logger.info('watch', `New file`, { file: path });
      callbacks?.onAdd(path);
    });
  });
  w.on('unlink', (path) => {
    if (!isVideo(path)) return;
    debounce(`unlink:${path}`, () => {
      logger.info('watch', `Removed file`, { file: path });
      callbacks?.onUnlink(path);
    });
  });
  w.on('unlinkDir', (path) => {
    debounce(`unlinkDir:${path}`, () => {
      logger.info('watch', `Removed directory`, { file: path });
      callbacks?.onUnlinkDir(path);
    });
  });
  w.on('error', (err) => {
    logger.error('watch', `Watcher error: ${(err as Error).message}`);
  });
  w.on('ready', () => {
    logger.info('watch', `Watching ${activeRoots.length} root(s)`);
  });
}

export const fileWatcher = {
  async start(roots: string[], cb: WatcherCallbacks): Promise<void> {
    callbacks = cb;
    activeRoots = roots.slice();
    if (roots.length === 0) {
      logger.info('watch', 'No library roots — watcher idle');
      return;
    }
    watcher = chokidar.watch(roots, {
      ignored: IGNORED_PATTERNS,
      ignoreInitial: true,
      persistent: true,
      awaitWriteFinish: { stabilityThreshold: 2000, pollInterval: 500 },
    });
    attach(watcher);
  },

  async restart(roots: string[]): Promise<void> {
    await this.stop();
    if (callbacks) await this.start(roots, callbacks);
  },

  async stop(): Promise<void> {
    for (const t of debounceMap.values()) clearTimeout(t);
    debounceMap.clear();
    if (watcher) {
      await watcher.close();
      watcher = null;
    }
    activeRoots = [];
  },
};
