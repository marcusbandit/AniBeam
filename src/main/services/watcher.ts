import chokidar, { type FSWatcher } from 'chokidar';
import { logger } from './logger';

export interface WatcherCallbacks {
  onAdd: (filePath: string) => void;
  onAddDir: (dirPath: string) => void;
  onUnlink: (filePath: string) => void;
  onUnlinkDir: (dirPath: string) => void;
}

const VIDEO_EXTS = new Set(['.mkv', '.mp4', '.avi', '.mov', '.webm', '.m4v', '.ts', '.wmv', '.flv']);
const IGNORED_PATTERNS = [/(^|[\/\\])\../, /\.part$/i, /\.crdownload$/i, /\.tmp$/i];

let watcher: FSWatcher | null = null;
let activeRoots: string[] = [];
let callbacks: WatcherCallbacks | null = null;

function isVideo(path: string): boolean {
  const dot = path.lastIndexOf('.');
  if (dot < 0) return false;
  return VIDEO_EXTS.has(path.slice(dot).toLowerCase());
}

function attach(w: FSWatcher): void {
  // No debounce: fire as soon as inotify reports. awaitWriteFinish below
  // already collapses the rapid-fire events during a download into a single
  // add at write-completion, which is the only coalescing we actually need.
  w.on('add', (path) => {
    if (!isVideo(path)) return;
    logger.info('watch', `New file`, { file: path });
    callbacks?.onAdd(path);
  });
  w.on('addDir', (path) => {
    // Always notify on new dirs — files already present at dir-creation time
    // do NOT reliably get their own `add` event from chokidar, so the consumer
    // walks the subtree explicitly. Skip the roots themselves (chokidar emits
    // addDir for every watched root once at startup; ignoreInitial:true should
    // prevent it but be defensive).
    if (activeRoots.includes(path)) return;
    logger.info('watch', `New directory`, { file: path });
    callbacks?.onAddDir(path);
  });
  w.on('unlink', (path) => {
    if (!isVideo(path)) return;
    logger.info('watch', `Removed file`, { file: path });
    callbacks?.onUnlink(path);
  });
  w.on('unlinkDir', (path) => {
    logger.info('watch', `Removed directory`, { file: path });
    callbacks?.onUnlinkDir(path);
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
      // 500ms / 100ms: just enough to ensure the file is fully written before
      // we ingest it. Old setting (2000/500) was the source of the "watcher
      // feels laggy" symptom — every new episode took 2s+ to appear even
      // when the file landed atomically (mv on same FS).
      awaitWriteFinish: { stabilityThreshold: 500, pollInterval: 100 },
    });
    attach(watcher);
  },

  async restart(roots: string[]): Promise<void> {
    await this.stop();
    if (callbacks) await this.start(roots, callbacks);
  },

  async stop(): Promise<void> {
    if (watcher) {
      await watcher.close();
      watcher = null;
    }
    activeRoots = [];
  },
};
