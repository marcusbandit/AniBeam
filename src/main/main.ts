import { app, BrowserWindow, ipcMain, dialog, Menu, protocol, net, shell } from 'electron';
import { fileURLToPath, pathToFileURL } from 'url';
import { dirname, join, isAbsolute, resolve, relative, basename } from 'path';
import { existsSync } from 'fs';
import { spawn } from 'child_process';
import axios from 'axios';
import folderHandler from './handlers/folderHandler';
import anilistHandler from './handlers/anilistHandler';
import malHandler from './handlers/malHandler';
import metadataHandler from './handlers/metadataHandler';
import configHandler from './handlers/configHandler';
import imageCacheHandler from './handlers/imageCacheHandler';
import thumbnailHandler from './handlers/thumbnailHandler';
import { initMediaProgress, updateMediaProgress } from './utils/debugUtils';
import { findBestMatch } from './utils/metadataMatcher';
import { findShowMatch, fetchEpisodeAirDates } from './utils/posterMatch';
import { logger } from './services/logger';
import type { FileStatus } from '../shared/fileStatus';
import type { ScannedMedia } from './handlers/folderHandler';
import videoProbeHandler from './handlers/videoProbeHandler';
import subtitleHandler from './handlers/subtitleHandler';
import aniSkipHandler from './handlers/aniSkipHandler';
import trackerHandler from './handlers/trackerHandler';
import type { TrackerProvider } from './services/trackerStore';
import { fileWatcher } from './services/watcher';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// When the app is launched from a .desktop entry, stdout/stderr aren't
// connected to anything — the first write that hits a closed pipe takes
// down the main process with EPIPE. Swallow EPIPE specifically so any
// stray console.log can't crash us; surface other errors normally.
for (const stream of [process.stdout, process.stderr]) {
  stream.on('error', (err: NodeJS.ErrnoException) => {
    if (err.code !== 'EPIPE') throw err;
  });
}

// Helper to check if error is a rate limit (already logged by handlers)
function isRateLimitError(error: unknown): boolean {
  if (axios.isAxiosError(error)) {
    return error.response?.status === 429;
  }
  if (error && typeof error === 'object') {
    const err = error as { response?: { status?: number }; statusCode?: number; message?: string };
    if (err.response?.status === 429 || err.statusCode === 429) {
      return true;
    }
    if (err.message && /rate.?limit/i.test(err.message)) {
      return true;
    }
  }
  return false;
}

function preserveStatus(
  seriesId: string,
  files: Array<{ filePath: string; status?: string; lastProbedAt?: number }>,
  existingMetadata: Record<string, unknown>,
): typeof files {
  const prior = existingMetadata[seriesId] as
    | { fileEpisodes?: Array<{ filePath: string; status?: string; lastProbedAt?: number }> }
    | undefined;
  if (!prior?.fileEpisodes) return files;
  const byPath = new Map(prior.fileEpisodes.map((f) => [f.filePath, f]));
  return files.map((f) => {
    const old = byPath.get(f.filePath);
    if (!old) return f;
    return { ...f, status: old.status ?? f.status, lastProbedAt: old.lastProbedAt ?? f.lastProbedAt };
  });
}

async function updateFileStatus(filePath: string, status: FileStatus): Promise<void> {
  const touched = await metadataHandler.transaction<boolean>(async (meta) => {
    let changed = false;
    for (const series of Object.values(meta)) {
      const s = series as { fileEpisodes?: Array<{ filePath: string; status?: string; lastProbedAt?: number }> };
      if (!Array.isArray(s.fileEpisodes)) continue;
      for (const file of s.fileEpisodes) {
        if (file.filePath === filePath) {
          file.status = status;
          file.lastProbedAt = Date.now();
          changed = true;
        }
      }
    }
    return { result: changed, updated: changed ? meta : null };
  });
  if (touched && mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send('metadata:file-status-changed', { filePath, status });
  }
}

async function ingestSingleFile(filePath: string): Promise<void> {
  try {
    const activeRoots = await configHandler.getFolderSources();
    const result = await folderHandler.scanSingleFile(filePath, activeRoots);
    if (!result) {
      logger.warn('watch', `scanSingleFile returned null`, { file: filePath });
      return;
    }
    const { media } = result;

    // Filesystem-only ingest. Splice the new file into the series's
    // fileEpisodes (creating a minimal entry if the series is brand new).
    // No network, no thumbnails, no metadata fetch — metadata is paused.
    let isBrandNewSeries = false;
    await metadataHandler.transaction(async (current) => {
      const existing = (current[media.id] ?? {}) as {
        fileEpisodes?: Array<{ filePath: string; status?: string; lastProbedAt?: number }>;
        title?: string;
        posterMatchAttempted?: boolean;
      };
      isBrandNewSeries = !existing.fileEpisodes || existing.fileEpisodes.length === 0;
      const byPath = new Map((existing.fileEpisodes ?? []).map((f) => [f.filePath, f]));
      const newFileEpisodes = media.files.map((f) => {
        const old = byPath.get(f.filePath);
        const isThisFile = f.filePath === filePath;
        return {
          episodeNumber: f.episodeNumber,
          seasonNumber: f.seasonNumber,
          filePath: f.filePath,
          subtitlePath: f.subtitlePath,
          subtitlePaths: f.subtitlePaths,
          filename: f.filename,
          title: f.title,
          status: isThisFile ? 'verifying' : (old?.status ?? f.status),
          lastProbedAt: isThisFile ? Date.now() : (old?.lastProbedAt ?? f.lastProbedAt),
        };
      });
      current[media.id] = {
        ...existing,
        seriesId: media.id,
        title: existing.title ?? media.name,
        fileEpisodes: newFileEpisodes,
        folderPath: media.folderPath,
        type: media.type,
      };
      logger.info('watch', `Ingest: ${media.name} (${newFileEpisodes.length} files)`, { series: media.name, file: filePath });
      return { updated: current };
    });

    videoProbeHandler.enqueue(filePath);

    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:file-status-changed', { filePath, status: 'verifying' });
    }

    // First time we see this series → background poster match. Movies share
    // the "Movies" parent folder, so the scanner-derived title (media.name)
    // is the right search input; for series, the folder name is.
    if (isBrandNewSeries) {
      const query = media.type === 'movie' ? media.name : basename(media.folderPath);
      void matchPosterForSeries(media.id, query);
    }
  } catch (err) {
    logger.error('watch', `Failed to ingest new file: ${(err as Error).message}`, { file: filePath });
  }
}

async function handleUnlink(filePath: string): Promise<void> {
  const changed = await metadataHandler.transaction<boolean>(async (meta) => {
    const activeRoots = await configHandler.getFolderSources();
    const reconciled = await folderHandler.reconcileMetadata(meta, activeRoots);
    // reconcileMetadata always returns a new outer object, so reference
    // equality is unreliable. Compare keys + per-series file counts to detect
    // actual changes and avoid pointless writes.
    const sameShape =
      Object.keys(reconciled).length === Object.keys(meta).length &&
      Object.entries(reconciled).every(([id, value]) => {
        const before = meta[id] as { fileEpisodes?: unknown[] } | undefined;
        const after = value as { fileEpisodes?: unknown[] };
        return before && (before.fileEpisodes?.length ?? 0) === (after.fileEpisodes?.length ?? 0);
      });
    return { result: !sameShape, updated: sameShape ? null : reconciled };
  });
  if (changed && mainWindow && !mainWindow.isDestroyed()) {
    // Reuse the file-status-changed channel — renderers treat any payload as
    // "metadata changed, reload." Status field is a placeholder; the entry
    // for `filePath` is gone after a reconcile.
    mainWindow.webContents.send('metadata:file-status-changed', { filePath, status: 'ready' });
  }
}

// Vite env variables (injected by @electron-forge/plugin-vite at build time)
declare const MAIN_WINDOW_VITE_DEV_SERVER_URL: string | undefined;
declare const MAIN_WINDOW_VITE_NAME: string;

let mainWindow: BrowserWindow | null = null;

let startupCatchUpInFlight = false;

async function runStartupCatchUp(): Promise<void> {
  if (startupCatchUpInFlight) return;
  startupCatchUpInFlight = true;
  try {
    const activeRoots = await configHandler.getFolderSources();
    if (activeRoots.length === 0) return;
    logger.info('watch', `Startup catch-up: ${activeRoots.length} root(s)`);
    for (const root of activeRoots) {
      try {
        await runScanAndFetch(root, activeRoots);
      } catch (err) {
        logger.warn('folder', `Startup catch-up failed for ${root}: ${(err as Error).message}`, { file: root });
      }
    }
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:file-status-changed', { filePath: '', status: 'ready' });
    }
    // Background: kick off poster matching for series that haven't been
    // tried yet. Doesn't block startup. Renderer reloads on each match
    // via the file-status-changed ping inside matchPosterForSeries.
    void matchPostersForLibrary();
  } finally {
    startupCatchUpInFlight = false;
  }
}

// Background match for one series: poster + status + per-episode air
// dates (MAL only — AniList airing-schedule not wired yet). Idempotent:
// always writes `posterMatchAttempted: true` so we don't re-hammer MAL
// on every restart. Misses leave the placeholder; future "human in the
// loop" UI will let the user pick manually.
async function matchPosterForSeries(seriesId: string, folderName: string): Promise<void> {
  try {
    const match = await findShowMatch(folderName);
    if (!match) {
      await metadataHandler.transaction(async (current) => {
        const existing = (current[seriesId] ?? {}) as Record<string, unknown>;
        if (existing.posterMatchAttempted) return { updated: null };
        current[seriesId] = { ...existing, posterMatchAttempted: true, posterMatched: false };
        return { updated: current };
      });
      return;
    }
    // findShowMatch guarantees the id matching `source` is set; the other
    // is best-effort (cross-resolved or absent).
    const primaryId = match.source === 'anilist' ? match.anilistId! : match.malId!;
    const [cached, episodeDates] = await Promise.all([
      imageCacheHandler.cacheImages([match.posterUrl]),
      fetchEpisodeAirDates(match.source, primaryId, match.totalEpisodes),
    ]);
    const posterLocal = cached.get(match.posterUrl) ?? null;
    await metadataHandler.transaction(async (current) => {
      const existing = (current[seriesId] ?? {}) as Record<string, unknown>;
      // Slim episode list — only what the feed needs to sort. We deliberately
      // do NOT keep titles/descriptions/thumbnails; metadata enrichment
      // beyond the sort key is still paused.
      const slimEpisodes = episodeDates.map((e) => ({ episodeNumber: e.episodeNumber, airDate: e.airDate }));
      // Persist BOTH provider ids when we have them. Trackers, AniSkip, and
      // the rest of the renderer key off `anilistId`/`malId` directly —
      // matchSource alone isn't enough.
      current[seriesId] = {
        ...existing,
        poster: match.posterUrl,
        posterLocal,
        posterMatchAttempted: true,
        posterMatched: true,
        matchSource: match.source,
        anilistId: match.anilistId ?? undefined,
        malId: match.malId ?? null,
        matchedTitle: match.matchedTitle,
        titleRomaji: match.titleRomaji,
        titleEnglish: match.titleEnglish,
        matchScore: match.score,
        status: match.status,
        startDate: match.startDate,
        totalEpisodes: match.totalEpisodes,
        episodes: slimEpisodes,
      };
      return { updated: current };
    });
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:file-status-changed', { filePath: '', status: 'ready' });
    }
  } catch (err) {
    logger.warn('metadata', `Match failed for ${folderName}: ${(err as Error).message}`, { series: folderName });
  }
}

// Walk every series in metadata.json and match a poster for any that
// haven't been attempted yet. Sequential so we don't blow through the
// MAL/AniList rate limiters in bursts. Designed to run in the background
// — never throw, never block the caller.
async function matchPostersForLibrary(): Promise<void> {
  const meta = await metadataHandler.loadMetadata();
  const todo: Array<{ seriesId: string; folderName: string }> = [];
  for (const [seriesId, raw] of Object.entries(meta)) {
    const s = raw as {
      posterMatchAttempted?: boolean;
      posterMatched?: boolean;
      titleRomaji?: string | null;
      titleEnglish?: string | null;
      episodes?: Array<{ episodeNumber?: number; airDate?: string | null }>;
      folderPath?: string;
      title?: string;
      type?: 'series' | 'movie';
      anilistId?: number;
      malId?: number | null;
    };
    // Re-match when:
    //  (a) never attempted, or
    //  (b) matched in an earlier iteration but the episode air dates we now
    //      need for the feed sort aren't there yet, or
    //  (c) matched but missing the explicit titleRomaji/titleEnglish fields
    //      added with the JP/EN switch, or
    //  (d) matched but missing anilistId / malId — older entries written
    //      before the matcher persisted these. Trackers and AniSkip key
    //      off these fields, so backfill them on the next scan.
    const needsFirstMatch = !s.posterMatchAttempted;
    const matchedButMissingAirDates =
      !!s.posterMatched &&
      (!Array.isArray(s.episodes) ||
        s.episodes.length === 0 ||
        !s.episodes.some((e) => !!e.airDate));
    const matchedButMissingTitles =
      !!s.posterMatched && s.titleRomaji === undefined && s.titleEnglish === undefined;
    const matchedButMissingProviderIds =
      !!s.posterMatched && s.anilistId === undefined && (s.malId === undefined || s.malId === null);
    // Movies whose first attempt failed: keep retrying on subsequent launches.
    // The first attempt for movies used basename(folderPath) = "Movies", which
    // never matched anything; the fixed code below uses the title. Cheap to
    // re-run for the few movies that exist in a library.
    const failedMovieNeedsRetry = s.type === 'movie' && !s.posterMatched;
    if (
      !needsFirstMatch &&
      !matchedButMissingAirDates &&
      !matchedButMissingTitles &&
      !matchedButMissingProviderIds &&
      !failedMovieNeedsRetry
    ) continue;
    // Series live in their own folder, so the folder name is the right
    // search input. Movies all share the "Movies" parent — searching for
    // "Movies" never matches anything, so use the cleaned movie title
    // (which the scanner derived from the file name) instead.
    const matchQuery = s.type === 'movie'
      ? (s.title ?? seriesId)
      : (s.folderPath ? basename(s.folderPath) : (s.title ?? seriesId));
    todo.push({ seriesId, folderName: matchQuery });
  }
  if (todo.length === 0) return;
  logger.info('metadata', `Matching ${todo.length} series (poster + air dates)`);
  for (const { seriesId, folderName } of todo) {
    await matchPosterForSeries(seriesId, folderName);
  }
}

async function ingestSubtree(dirPath: string): Promise<void> {
  // chokidar emits `addDir` when a new folder appears, but it does NOT
  // reliably emit `add` for files that were already inside the folder at
  // creation time (e.g. a torrent client renaming a temp dir into place).
  // Walk the new subtree ourselves and ingest every video we find.
  try {
    const { readdir, stat } = await import('fs/promises');
    const entries = await readdir(dirPath);
    for (const entry of entries) {
      const full = `${dirPath}/${entry}`;
      try {
        const s = await stat(full);
        if (s.isDirectory()) {
          await ingestSubtree(full);
        } else if (s.isFile() && /\.(mkv|mp4|avi|mov|webm|m4v|ts|wmv|flv)$/i.test(entry)) {
          void ingestSingleFile(full);
        }
      } catch { /* skip unreadable */ }
    }
  } catch (err) {
    logger.warn('watch', `ingestSubtree failed for ${dirPath}: ${(err as Error).message}`, { file: dirPath });
  }
}

function createWindow(): void {
  // Remove the application menu (File, Edit, View, etc.)
  Menu.setApplicationMenu(null);

  mainWindow = new BrowserWindow({
    width: 1920,
    height: 1080,
    minWidth: 800,
    minHeight: 600,
    webPreferences: {
      preload: join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      webSecurity: false, // Allow loading local files
    },
    autoHideMenuBar: true,
    backgroundColor: '#0a0a0f',
  });

  // Load the app using Vite's dev server URL (set by @electron-forge/plugin-vite)
  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL);
  } else {
    mainWindow.loadFile(join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`));
  }

  // Conditionally enable dev tools in dev mode only
  if (process.env.DEV_MODE === 'true') {
    mainWindow.webContents.openDevTools();
    mainWindow.webContents.on('before-input-event', (_event, input) => {
      if (input.control && input.shift && input.key.toLowerCase() === 'i') {
        mainWindow?.webContents.toggleDevTools();
      }
    });
  }

  mainWindow.webContents.once('did-finish-load', () => {
    logger.info('system', 'AniBeam ready');
  });

  // Log renderer errors
  mainWindow.webContents.on('did-fail-load', (_event, errorCode, errorDescription) => {
    console.error('Renderer failed to load:', errorCode, errorDescription);
  });
}

// Register custom protocol for serving local media files
protocol.registerSchemesAsPrivileged([
  {
    scheme: 'media',
    privileges: {
      secure: true,
      supportFetchAPI: true,
      stream: true,
      bypassCSP: true,
    },
  },
]);

app.whenReady().then(async () => {
  initMediaProgress(); // Debug progress bar initialization
  // Handle media:// protocol using net.fetch for proper streaming
  protocol.handle('media', async (request) => {
    // Parse the URL to extract the file path
    let filePath = request.url.replace(/^media:\/\//, '');
    filePath = decodeURIComponent(filePath);

    // Cross-platform path handling: ensure absolute paths
    if (!isAbsolute(filePath)) {
      // Only add leading / for Unix-like systems (Linux, macOS)
      if (process.platform !== 'win32') {
        filePath = '/' + filePath;
      } else {
        // On Windows, reject relative paths for security
        logger.error('system', `Rejected relative path: ${filePath}`, { file: filePath });
        return new Response('Invalid path: relative paths not allowed', { status: 403 });
      }
    }

    // SECURITY: Validate path is within allowed folder sources OR app's userData directory
    try {
      const normalizedPath = resolve(filePath);
      
      const userDataPath = app.getPath('userData');
      const normalizedUserData = resolve(userDataPath);
      const userDataRelative = relative(normalizedUserData, normalizedPath);
      const isInUserData = !userDataRelative.startsWith('..') && !userDataRelative.startsWith('/');
      
      const allowedSources = await configHandler.getFolderSources();
      const isInAllowedSource = allowedSources.some(source => {
        try {
          const normalizedSource = resolve(source);
          const relativePath = relative(normalizedSource, normalizedPath);
          return !relativePath.startsWith('..') && !relativePath.startsWith('/');
        } catch {
          return false;
        }
      });

      if (!isInUserData && !isInAllowedSource) {
        if (allowedSources.length === 0 && !isInUserData) {
          logger.error('system', 'Access denied: path not in userData and no folder sources configured');
          return new Response('Access denied: path not in allowed directories', { status: 403 });
        }
        logger.error('system', `Access denied for path: ${filePath}`, { file: filePath });
        return new Response('Access denied: path not in allowed directories', { status: 403 });
      }
    } catch (error) {
      logger.error('system', 'Error validating path');
      return new Response('Error validating path', { status: 500 });
    }

    updateMediaProgress(filePath); // Debug progress bar update

    // Check if file exists
    if (!existsSync(filePath)) {
      logger.error('system', `File not found: ${filePath}`, { file: filePath });
      return new Response('File not found', { status: 404 });
    }

    // Use net.fetch with file:// URL - Electron handles range requests automatically
    const fileUrl = pathToFileURL(filePath).toString();

    // Forward the request headers (including Range)
    const headers: Record<string, string> = {};
    request.headers.forEach((value, key) => {
      headers[key] = value;
    });

    try {
      const response = await net.fetch(fileUrl, {
        method: request.method,
        headers: headers,
      });

      // Get file extension for MIME type
      const ext = filePath.split('.').pop()?.toLowerCase() || '';
      const mimeTypes: Record<string, string> = {
        // Video formats
        'mp4': 'video/mp4',
        'mkv': 'video/x-matroska',
        'avi': 'video/x-msvideo',
        'webm': 'video/webm',
        'mov': 'video/quicktime',
        'wmv': 'video/x-ms-wmv',
        'flv': 'video/x-flv',
        'm4v': 'video/mp4',
        // Subtitle formats
        'srt': 'text/plain; charset=utf-8',
        'vtt': 'text/vtt; charset=utf-8',
        'ass': 'text/plain; charset=utf-8',
        'ssa': 'text/plain; charset=utf-8',
        // Image formats (for cached images)
        'jpg': 'image/jpeg',
        'jpeg': 'image/jpeg',
        'png': 'image/png',
        'gif': 'image/gif',
        'webp': 'image/webp',
        'avif': 'image/avif',
      };

      // Clone response with correct content type
      const newHeaders = new Headers(response.headers);
      if (mimeTypes[ext]) {
        newHeaders.set('Content-Type', mimeTypes[ext]);
      }

      return new Response(response.body, {
        status: response.status,
        statusText: response.statusText,
        headers: newHeaders,
      });
    } catch (error) {
      logger.error('system', 'Error fetching file');
      return new Response('Error loading file', { status: 500 });
    }
  });

  createWindow();
  videoProbeHandler.start(updateFileStatus);

  // Re-enqueue any files that were 'verifying' when the app last quit —
  // the in-memory probe queue doesn't survive restarts, and the watcher's
  // ignoreInitial:true means cold-start won't re-discover existing files.
  // Without this, a file caught mid-verification stays stuck forever.
  try {
    const meta = await metadataHandler.loadMetadata();
    let resumed = 0;
    for (const series of Object.values(meta)) {
      const s = series as { fileEpisodes?: Array<{ filePath: string; status?: string }> };
      if (!Array.isArray(s.fileEpisodes)) continue;
      for (const f of s.fileEpisodes) {
        if (f.status === 'verifying' || f.status === 'stalled') {
          videoProbeHandler.enqueue(f.filePath);
          resumed++;
        }
      }
    }
    if (resumed > 0) logger.info('probe', `Resumed ${resumed} unverified file(s) from prior session`);
  } catch (err) {
    logger.warn('probe', `Could not resume unverified files: ${(err as Error).message}`);
  }

  const initialRoots = await configHandler.getFolderSources();
  await fileWatcher.start(initialRoots, {
    onAdd: (path) => { void ingestSingleFile(path); },
    onAddDir: (dirPath) => { void ingestSubtree(dirPath); },
    onUnlink: (path) => { void handleUnlink(path); },
    // unlinkDir: chokidar emits unlink for each contained file too, so the
    // per-file handler covers actual cleanup. Just log; don't double-reconcile.
    onUnlinkDir: (dirPath) => { logger.info('watch', `Directory removed`, { file: dirPath }); },
  });

  // One-shot catch-up for files that landed while the app was closed
  // (chokidar's ignoreInitial:true skips them). NOT a periodic safety net —
  // ongoing detection is the watcher's job, period.
  void runStartupCatchUp();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// ==================== LOGGER IPC ====================

ipcMain.handle('log:get-buffer', () => logger.getBuffer());
ipcMain.handle('log:clear', () => {
  logger.clear();
});

// ==================== CONFIG IPC ====================

ipcMain.handle('get-folder-sources', async () => {
  try {
    return await configHandler.getFolderSources();
  } catch (error) {
    logger.error('system', 'Error getting folder sources');
    return [];
  }
});

ipcMain.handle('find-movie-folders', async (_event, rootPath: string) => {
  try {
    return await folderHandler.findMovieFolders(rootPath);
  } catch (error) {
    logger.error('folder', `Error finding movie folders: ${(error as Error).message}`);
    return [];
  }
});

ipcMain.handle('add-folder-source', async (_event, folderPath: string) => {
  try {
    const ok = await configHandler.addFolderSource(folderPath);
    if (ok) {
      const roots = await configHandler.getFolderSources();
      await fileWatcher.restart(roots);
      logger.info('folder', `Added library root: ${folderPath}`);
    }
    return ok;
  } catch (error) {
    logger.error('folder', `Error adding folder source: ${(error as Error).message}`);
    throw error;
  }
});

ipcMain.handle('remove-folder-source', async (_event, folderPath: string) => {
  try {
    const ok = await configHandler.removeFolderSource(folderPath);
    if (ok) {
      logger.info('folder', `Removed library root: ${folderPath}`);
      const activeRoots = await configHandler.getFolderSources();
      await metadataHandler.transaction(async (meta) => {
        const reconciled = await folderHandler.reconcileMetadata(meta, activeRoots);
        return { updated: reconciled };
      });
      await fileWatcher.restart(activeRoots);
    }
    return ok;
  } catch (error) {
    logger.error('folder', `Error removing folder source: ${(error as Error).message}`);
    throw error;
  }
});

// ==================== FOLDER IPC ====================

ipcMain.handle('select-folder', async () => {
  if (!mainWindow) return null;

  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ['openDirectory'],
    title: 'Select Anime Folder',
  });

  if (!result.canceled && result.filePaths.length > 0) {
    return result.filePaths[0];
  }
  return null;
});

ipcMain.handle('scan-folder', async (_event, folderPath: string) => {
  try {
    return await folderHandler.scanFolder(folderPath);
  } catch (error) {
    logger.error('folder', 'Error scanning folder');
    throw error;
  }
});

// Pure filesystem walk of every configured library root, joined with
// just the bits of metadata.json the renderer actually displays right
// now (poster URL/local path + match state). No episode-level metadata,
// no banners, no online thumbnails. The renderer renders directly off
// this — folder names verbatim, posters when matched, placeholder when
// not.
ipcMain.handle('library:walk', async () => {
  const roots = await configHandler.getFolderSources();
  const all: ScannedMedia[] = [];
  for (const root of roots) {
    try {
      const items = await folderHandler.scanFolder(root, [root]);
      all.push(...items);
    } catch (err) {
      logger.warn('folder', `library:walk failed for ${root}: ${(err as Error).message}`, { file: root });
    }
  }
  const meta = await metadataHandler.loadMetadata();
  return all.map((m) => {
    const stored = (meta[m.id] ?? {}) as {
      poster?: string | null;
      posterLocal?: string | null;
      posterMatched?: boolean;
      posterMatchAttempted?: boolean;
      matchSource?: 'mal' | 'anilist';
      matchedTitle?: string | null;
      titleRomaji?: string | null;
      titleEnglish?: string | null;
      status?: string | null;
      startDate?: string | null;
      totalEpisodes?: number | null;
      anilistId?: number;
      malId?: number | null;
      episodes?: Array<{ episodeNumber: number; airDate: string | null }>;
    };
    return {
      id: m.id,
      // Movies share the "Movies" parent folder, so basename(folderPath)
      // would be "Movies" for every movie. Use the scanner-derived title
      // (m.name) as the display fallback instead so the card shows
      // "Kimi no Na Wa" rather than "Movies".
      folderName: m.type === 'movie' ? m.name : basename(m.folderPath),
      folderPath: m.folderPath,
      type: m.type,
      poster: stored.poster ?? null,
      posterLocal: stored.posterLocal ?? null,
      posterMatched: stored.posterMatched ?? false,
      posterMatchAttempted: stored.posterMatchAttempted ?? false,
      matchSource: stored.matchSource ?? null,
      matchedTitle: stored.matchedTitle ?? null,
      titleRomaji: stored.titleRomaji ?? stored.matchedTitle ?? null,
      titleEnglish: stored.titleEnglish ?? null,
      status: stored.status ?? null,
      startDate: stored.startDate ?? null,
      totalEpisodes: stored.totalEpisodes ?? null,
      anilistId: stored.anilistId ?? null,
      malId: stored.malId ?? null,
      episodes: (stored.episodes ?? []).map((e) => ({
        episodeNumber: e.episodeNumber,
        airDate: e.airDate ?? null,
      })),
      files: m.files.map((f) => ({
        filename: f.filename,
        filePath: f.filePath,
        title: f.title,
        episodeNumber: f.episodeNumber,
        seasonNumber: f.seasonNumber,
        subtitlePath: f.subtitlePath,
        subtitlePaths: f.subtitlePaths,
        mtime: f.mtime,
      })),
    };
  });
});

ipcMain.handle('scan-all-folders', async () => {
  try {
    const folderSources = await configHandler.getFolderSources();
    if (folderSources.length === 0) {
      return [];
    }
    return await folderHandler.scanMultipleFolders(folderSources);
  } catch (error) {
    logger.error('folder', 'Error scanning all folders');
    throw error;
  }
});

// ==================== METADATA IPC ====================

ipcMain.handle('fetch-metadata', async (_event, searchName: string, seasonNumber?: number | null) => {
  // Best-match across MAL + AniList. No folderEpisodeCount on this path
  // (renderer-side refresh doesn't know it), so the strict-tier ep-count
  // filter is disabled — title similarity does the heavy lifting.
  const seasonInfo = seasonNumber !== null && seasonNumber !== undefined ? ` Season ${seasonNumber}` : '';
  logger.info('metadata', `Fetching metadata for: "${searchName}"${seasonInfo}`);

  const result = await findBestMatch(searchName, seasonNumber ?? null, null, undefined);
  if (result) {
    const meta = result.metadata as { title?: string };
    logger.info(
      'metadata',
      `Refresh match: ${result.source.toUpperCase()} "${meta.title ?? '?'}" (score ${result.score.toFixed(2)})`,
    );
    return result.metadata;
  }
  logger.warn('metadata', `No metadata found for: "${searchName}"${seasonInfo}`);
  return null;
});

ipcMain.handle('fetch-mal-metadata', async (_event, seriesName: string, seasonNumber?: number | null) => {
  try {
    return await malHandler.searchAndFetchMetadata(seriesName, seasonNumber);
  } catch (error) {
    logger.error('metadata', 'Error fetching MAL metadata');
    throw error;
  }
});

ipcMain.handle('fetch-anilist-metadata', async (_event, seriesName: string, seasonNumber?: number | null) => {
  try {
    return await anilistHandler.searchAndFetchMetadata(seriesName, seasonNumber);
  } catch (error) {
    logger.error('metadata', 'Error fetching AniList metadata');
    throw error;
  }
});

// Match-picker search: returns multiple AniList results for the user to choose
// from. Lighter than fetch-metadata — no episode fetch, no filtering, no
// fallback dance. Used by the metadata-override modal.
ipcMain.handle('anilist:search', async (_event, query: string, limit?: number) => {
  if (!query || typeof query !== 'string' || query.trim().length === 0) return [];
  try {
    return await anilistHandler.searchAnimeMultiple(query.trim(), typeof limit === 'number' ? limit : 12);
  } catch (error) {
    if (!isRateLimitError(error)) logger.error('metadata', 'Error searching AniList for picker');
    return [];
  }
});

// Override path: the user picked a specific AniList ID in the match modal.
// Run the same image-caching + local-thumbnail generation pipeline that
// scan-and-fetch uses, so episode thumbnails actually populate (raw AniList
// data has only a sparse `streamingEpisodes` set; everything else needs an
// ffmpeg fallback against the on-disk video files).
ipcMain.handle('metadata:apply-anilist-match', async (
  _event,
  seriesId: string,
  anilistId: number,
  seasonNumber: number | null = null,
) => {
  if (!seriesId || typeof anilistId !== 'number' || !Number.isFinite(anilistId)) {
    return { ok: false, reason: 'bad-args' };
  }

  // 1. Fetch the chosen media from AniList.
  const fetched = await anilistHandler.fetchMetadataById(anilistId, seasonNumber);
  if (!fetched) return { ok: false, reason: 'fetch-failed' };

  // 2. Pull the existing entry for fileEpisodes / folderPath / type — those
  //    describe local state and must be preserved across the override.
  const allMeta = await metadataHandler.loadMetadata();
  const existing = allMeta[seriesId] as {
    fileEpisodes?: Array<{ episodeNumber: number; seasonNumber?: number | null; filePath: string }>;
    folderPath?: string;
    type?: 'series' | 'movie';
  } | undefined;

  const fileEpisodes = existing?.fileEpisodes ?? [];

  // 3. Build a season+episode → filePath lookup so we can pick the right
  //    video for each AniList-listed episode that lacks an online thumbnail.
  const fileEpisodeMap = new Map<string, string>();
  for (const f of fileEpisodes) {
    const key = f.seasonNumber !== null && f.seasonNumber !== undefined
      ? `${f.seasonNumber}_${f.episodeNumber}`
      : `null_${f.episodeNumber}`;
    fileEpisodeMap.set(key, f.filePath);
  }

  // 4. Cache poster + banner + every online episode thumbnail in one pass.
  const imagesToCache: (string | null)[] = [fetched.poster, fetched.banner];
  for (const ep of fetched.episodes || []) {
    if (ep.thumbnail) imagesToCache.push(ep.thumbnail);
  }
  const cachedImages = await imageCacheHandler.cacheImages(imagesToCache);

  const posterLocal = fetched.poster ? cachedImages.get(fetched.poster) ?? null : null;
  const bannerLocal = fetched.banner ? cachedImages.get(fetched.banner) ?? null : null;

  // 5. For each AniList episode, prefer the cached online thumbnail; if there
  //    isn't one, generate a frame from the matching local video. Match by
  //    season+episode first, then any-season fallback (handles both libraries
  //    that flag seasons in folder names and ones that don't).
  const episodesWithLocalThumbs = [];
  let firstThumbnailInBatch = true;
  for (const ep of fetched.episodes || []) {
    let thumbnailLocal: string | null = null;
    if (ep.thumbnail) thumbnailLocal = cachedImages.get(ep.thumbnail) ?? null;

    if (!thumbnailLocal) {
      const epSeason = ep.seasonNumber ?? null;
      const exactKey = epSeason !== null ? `${epSeason}_${ep.episodeNumber}` : `null_${ep.episodeNumber}`;
      let videoPath = fileEpisodeMap.get(exactKey);
      if (!videoPath && epSeason !== null) videoPath = fileEpisodeMap.get(`null_${ep.episodeNumber}`);
      if (!videoPath) {
        for (const [mapKey, path] of fileEpisodeMap.entries()) {
          const parts = mapKey.split('_');
          const keyEpNum = parseFloat(parts[parts.length - 1]);
          if (keyEpNum === ep.episodeNumber) { videoPath = path; break; }
        }
      }
      if (videoPath) {
        try {
          thumbnailLocal = await thumbnailHandler.generateThumbnail(videoPath, 120, firstThumbnailInBatch);
          firstThumbnailInBatch = false;
        } catch (err) {
          logger.warn('thumbnail', `apply-match: failed to generate thumbnail for ep ${ep.episodeNumber}`, { file: videoPath });
        }
      }
    }
    episodesWithLocalThumbs.push({ ...ep, thumbnailLocal });
  }

  // 6. Merge into existing entry. Keep the original seriesId KEY in
  //    metadata.json (caller still uses it); also write the AniList one
  //    inside the entry so future code can resolve the chosen ID.
  const merged: Record<string, unknown> = {
    ...fetched,
    posterLocal,
    bannerLocal,
    episodes: episodesWithLocalThumbs,
    fileEpisodes,
    folderPath: existing?.folderPath,
    type: existing?.type,
    source: 'anilist',
  };

  await metadataHandler.updateSeriesMetadata(seriesId, merged);
  logger.info('metadata', `Override applied: ${seriesId} → AniList ${anilistId}`);
  return { ok: true };
});

ipcMain.handle('save-metadata', async (_event, metadata: Record<string, unknown>) => {
  try {
    return await metadataHandler.saveMetadata(metadata);
  } catch (error) {
    logger.error('metadata', 'Error saving metadata');
    throw error;
  }
});

ipcMain.handle('load-metadata', async () => {
  try {
    return await metadataHandler.loadMetadata();
  } catch (error) {
    logger.error('metadata', 'Error loading metadata');
    return {};
  }
});

ipcMain.handle('clear-metadata', async () => {
  try {
    return await metadataHandler.saveMetadata({});
  } catch (error) {
    logger.error('metadata', 'Error clearing metadata');
    throw error;
  }
});

ipcMain.handle('delete-series', async (_event, seriesId: string) => {
  try {
    // Get series metadata first to delete associated images
    const seriesData = await metadataHandler.getSeriesMetadata(seriesId);

    if (seriesData) {
      // Delete cached images for this series
      await imageCacheHandler.deleteSeriesImages(seriesData as {
        poster?: string | null;
        banner?: string | null;
        posterLocal?: string | null;
        bannerLocal?: string | null;
        episodes?: Array<{
          thumbnail?: string | null;
          thumbnailLocal?: string | null;
        }>;
      });
    }

    // Delete metadata entry
    await metadataHandler.deleteSeriesMetadata(seriesId);

    logger.info('metadata', `Deleted series: ${seriesId}`);
    return true;
  } catch (error) {
    logger.error('metadata', 'Error deleting series');
    throw error;
  }
});

// ==================== PER-SERIES METADATA PROCESSOR ====================
// PAUSED: metadata enrichment (network fetch, image cache, thumbnails) is
// currently disabled — the renderer reads directly from library:walk for
// display, and runScanAndFetch only does a filesystem-only fileEpisodes
// sync. processOneMedia is kept here to make re-enabling enrichment a
// one-liner (call it again from runScanAndFetch's slow pass) without
// rewriting the hundreds of lines of online-thumbnail / image-cache /
// MAL+AniList logic. Don't delete it.
void processOneMedia;

async function processOneMedia(
  media: ScannedMedia,
  existingMetadata: Record<string, unknown>,
): Promise<Record<string, unknown>> {
  const slice: Record<string, unknown> = {};
  const mediaId = media.id;

  // Start with existing metadata if available
  if (existingMetadata[mediaId]) {
    slice[mediaId] = { ...existingMetadata[mediaId] as Record<string, unknown> };
  }

  // Check if we already have metadata for this
  const existing = existingMetadata[mediaId] as Record<string, unknown> | undefined;
  if (existing?.title && existing?.posterLocal) {
    // Validate that cached metadata matches the current series name
    // This prevents using wrong metadata when series ID changes or files change
    const cachedTitle = (existing.title as string).toLowerCase().trim();
    const seriesNameLower = media.name.toLowerCase().trim();

    // Normalize titles for comparison (remove season info, special chars)
    const normalizeForComparison = (title: string): string => {
      return title
        .replace(/\s*\(season\s*\d+\)/gi, '')  // Remove (Season 1)
        .replace(/[^a-z0-9\s]/g, '')             // Remove special chars
        .replace(/\s+/g, ' ')                    // Normalize spaces
        .trim();
    };

    const normalizedCached = normalizeForComparison(cachedTitle);
    const normalizedSeries = normalizeForComparison(seriesNameLower);

    // Check if titles match (allowing for partial matches if series name is in cached title)
    // But be strict - if series name is substantial, require a good match
    const titlesMatch = normalizedCached === normalizedSeries ||
                       (normalizedSeries.length >= 5 && (
                         normalizedCached.includes(normalizedSeries) ||
                         normalizedSeries.includes(normalizedCached)
                       ));

    if (!titlesMatch && normalizedSeries.length >= 3) {
      // Titles don't match - metadata is likely wrong, re-fetch it
      logger.warn('metadata', `Cached metadata title "${cachedTitle}" doesn't match series name "${seriesNameLower}" — re-fetching`, { series: media.name });
      // Clear the cached metadata and fall through to fetch new metadata
      delete slice[mediaId];
      // Fall through to fetch new metadata
    } else if (titlesMatch) {
      logger.info('metadata', `Using cached metadata for: ${media.name}`, { series: media.name });

      // Ensure title includes season number if we have season-specific files
      let finalTitle = existing.title as string;
      if (media.seasonNumber !== null && media.seasonNumber !== undefined) {
        const seasonPattern = /\(Season\s*\d+\)/i;
        if (!seasonPattern.test(finalTitle)) {
          finalTitle = `${finalTitle} (Season ${media.seasonNumber})`;
        }
      }

      // Update file info but keep metadata
      slice[mediaId] = {
        ...existing,
        seriesId: mediaId,
        title: finalTitle,
        fileEpisodes: preserveStatus(mediaId, media.files.map(f => ({
          episodeNumber: f.episodeNumber,
          seasonNumber: f.seasonNumber,
          filePath: f.filePath,
          subtitlePath: f.subtitlePath,
          subtitlePaths: f.subtitlePaths,
          filename: f.filename,
          title: f.title,
          status: f.status,
          lastProbedAt: f.lastProbedAt,
        })), existingMetadata),
        folderPath: media.folderPath,
        type: media.type,
      };
      return slice;
    }
  }

  const seasonInfo = media.seasonNumber !== null ? ` Season ${media.seasonNumber}` : '';
  const partInfo = media.partNumber !== null ? ` Part ${media.partNumber}` : '';
  logger.info('metadata', `Processing ${media.name}${seasonInfo}${partInfo}`, { series: media.name });

  // Count only canonical episodes (exclude decimal episodes like 6.5, 7.5, 10.5)
  // Decimal episodes are stored as actual decimals: 6.5, 7.5, 10.5, etc.
  const canonicalEpisodes = media.files.filter(f => {
    // Skip decimal episodes: check if episodeNumber is not an integer
    return Number.isInteger(f.episodeNumber);
  });
  const canonicalEpisodeCount = canonicalEpisodes.length;

  logger.info('folder', `Folder has ${canonicalEpisodeCount} canonical episode${canonicalEpisodeCount !== 1 ? 's' : ''} (${media.files.length} total files including decimal episodes)`, { series: media.name });

  // Fetch new metadata. findBestMatch searches MAL + AniList in parallel,
  // scores every candidate against the folder name, and picks the best
  // (refusing if nothing clears the title-similarity threshold). Replaces
  // the old MAL-first AniList-fallback that mismatched fuzzy-similar shows.
  type FetchedShape = Record<string, unknown> & {
    title: string;
    poster?: string | null;
    banner?: string | null;
    episodes?: Array<{ episodeNumber: number; seasonNumber?: number | null; thumbnail?: string | null }>;
    totalEpisodes?: number | null;
  };
  const matchResult = await findBestMatch(media.name, media.seasonNumber, media.partNumber, canonicalEpisodeCount);
  let fetchedMetadata: FetchedShape | null = null;
  if (matchResult) {
    fetchedMetadata = matchResult.metadata as FetchedShape;
    logger.info(
      'metadata',
      `Found on ${matchResult.source.toUpperCase()}: ${fetchedMetadata.title} (${fetchedMetadata.totalEpisodes ?? 'unknown'} episodes, score ${matchResult.score.toFixed(2)})`,
      { series: fetchedMetadata.title },
    );
  } else {
    logger.warn('metadata', `No good match for ${media.name}${seasonInfo}`, { series: media.name });
  }

  if (fetchedMetadata) {
    // Cache images locally
    logger.info('image', `Caching images for: ${media.name}`, { series: media.name });

    // Collect all image URLs to cache
    const imagesToCache: (string | null)[] = [
      fetchedMetadata.poster ?? null,
      fetchedMetadata.banner ?? null,
    ];

    // Add episode thumbnails that exist online
    if (fetchedMetadata.episodes) {
      for (const ep of fetchedMetadata.episodes) {
        if (ep.thumbnail) {
          imagesToCache.push(ep.thumbnail);
        }
      }
    }

    // Cache all online images in parallel
    const cachedImages = await imageCacheHandler.cacheImages(imagesToCache);

    // Update metadata with local paths
    const posterLocal = fetchedMetadata.poster ? cachedImages.get(fetchedMetadata.poster) || null : null;
    const bannerLocal = fetchedMetadata.banner ? cachedImages.get(fetchedMetadata.banner) || null : null;

    // Create a map of file episodes by season and episode number for thumbnail generation
    // Use a composite key: "season_episode" or "null_episode" for episodes without season
    const fileEpisodeMap = new Map<string, string>();
    for (const f of media.files) {
      const key = f.seasonNumber !== null
        ? `${f.seasonNumber}_${f.episodeNumber}`
        : `null_${f.episodeNumber}`;
      fileEpisodeMap.set(key, f.filePath);
    }

    // Update episode thumbnails - use online if available, otherwise generate from video
    const episodesWithLocalThumbs = [];
    let firstThumbnailInBatch = true;
    for (const ep of fetchedMetadata.episodes || []) {
      let thumbnailLocal: string | null = null;

      if (ep.thumbnail) {
        // Use cached online thumbnail
        thumbnailLocal = cachedImages.get(ep.thumbnail) || null;
      }

      // If no online thumbnail, try to generate from video file
      // Match by season and episode number if available, otherwise by episode number only
      if (!thumbnailLocal) {
        const epSeason = ep.seasonNumber ?? null;
        const key = epSeason !== null
          ? `${epSeason}_${ep.episodeNumber}`
          : `null_${ep.episodeNumber}`;

        // Try exact match first (season + episode)
        let videoPath = fileEpisodeMap.get(key);

        // If no exact match and we have a season, try matching by episode number only
        // (in case the file has a different season number)
        if (!videoPath && epSeason !== null) {
          videoPath = fileEpisodeMap.get(`null_${ep.episodeNumber}`);
        }

        // If still no match, try any season with same episode number (including decimals)
        if (!videoPath) {
          for (const [mapKey, path] of fileEpisodeMap.entries()) {
            // Extract episode number from key (format: "season_episode" or "null_episode")
            const keyParts = mapKey.split('_');
            const keyEpisodeNum = parseFloat(keyParts[keyParts.length - 1]);
            // Match exact episode number (works for both integers and decimals)
            if (keyEpisodeNum === ep.episodeNumber) {
              videoPath = path;
              break;
            }
          }
        }

        if (videoPath) {
          thumbnailLocal = await thumbnailHandler.generateThumbnail(videoPath, 120, firstThumbnailInBatch);
          firstThumbnailInBatch = false;
        }
      }

      episodesWithLocalThumbs.push({
        ...ep,
        thumbnailLocal,
      });
    }

    // Generate thumbnails for decimal episodes (6.5, 7.5, etc.) that exist in files but not in metadata
    // These won't be in the metadata episodes list, so we need to handle them separately
    const processedEpisodeNumbers = new Set(episodesWithLocalThumbs.map(ep => ep.episodeNumber));
    for (const fileEp of media.files) {
      // Check if this is a decimal episode (not an integer) and not already processed
      if (!Number.isInteger(fileEp.episodeNumber) && !processedEpisodeNumbers.has(fileEp.episodeNumber)) {
        const epSeason = fileEp.seasonNumber ?? null;

        // Generate thumbnail for this decimal episode
        let thumbnailLocal: string | null = null;
        try {
          thumbnailLocal = await thumbnailHandler.generateThumbnail(fileEp.filePath, 120, false);
        } catch (err) {
          logger.warn('thumbnail', `Failed to generate thumbnail for decimal episode ${fileEp.episodeNumber}`, { file: fileEp.filePath });
        }

        // Add to episodes list (these won't have metadata, but will have file info)
        episodesWithLocalThumbs.push({
          episodeNumber: fileEp.episodeNumber,
          seasonNumber: epSeason ?? undefined,
          title: fileEp.episodeNumber === 0 ? 'Special' : `Episode ${fileEp.episodeNumber.toFixed(1)}`,
          description: null,
          airDate: null,
          thumbnail: null,
          thumbnailLocal,
        });
      }
    }

    // Ensure title includes season number if we have season-specific files
    let finalTitle = fetchedMetadata.title;
    if (media.seasonNumber !== null && media.seasonNumber !== undefined) {
      // Check if title already includes season info
      const seasonPattern = /\(Season\s*\d+\)/i;
      if (!seasonPattern.test(finalTitle)) {
        finalTitle = `${finalTitle} (Season ${media.seasonNumber})`;
      }
    }

    slice[mediaId] = {
      ...fetchedMetadata,
      seriesId: mediaId,
      title: finalTitle,
      posterLocal,
      bannerLocal,
      episodes: episodesWithLocalThumbs,
      fileEpisodes: preserveStatus(mediaId, media.files.map(f => ({
        episodeNumber: f.episodeNumber,
        seasonNumber: f.seasonNumber,
        filePath: f.filePath,
        subtitlePath: f.subtitlePath,
        subtitlePaths: f.subtitlePaths,
        filename: f.filename,
        title: f.title,
        status: f.status,
        lastProbedAt: f.lastProbedAt,
      })), existingMetadata),
      folderPath: media.folderPath,
      type: media.type,
    };
    logger.info('metadata', `Found: ${finalTitle}`, { series: finalTitle });
  } else {
    // No metadata found, use folder/file name
    logger.warn('metadata', `No online metadata, generating local thumbnails: ${media.name}`, { series: media.name });

    // Generate thumbnails from video files
    const localEpisodes = [];
    let firstThumbnail = true;
    for (const f of media.files) {
      const thumbnailLocal = await thumbnailHandler.generateThumbnail(f.filePath, 120, firstThumbnail);
      firstThumbnail = false;
      localEpisodes.push({
        episodeNumber: f.episodeNumber,
        seasonNumber: f.seasonNumber,
        title: f.title || (f.episodeNumber === 0 ? 'Special' : `Episode ${f.episodeNumber}`),
        description: null,
        airDate: null,
        thumbnail: null,
        thumbnailLocal,
      });
    }

    // Ensure title includes season number if we have season-specific files
    let localTitle = media.name;
    if (media.seasonNumber !== null && media.seasonNumber !== undefined) {
      const seasonPattern = /Season\s*\d+/i;
      if (!seasonPattern.test(localTitle)) {
        localTitle = `${localTitle} (Season ${media.seasonNumber})`;
      }
    }

    slice[mediaId] = {
      seriesId: mediaId,
      title: localTitle,
      description: '',
      genres: [],
      poster: null,
      posterLocal: null,
      banner: null,
      bannerLocal: null,
      episodes: localEpisodes,
      fileEpisodes: preserveStatus(mediaId, media.files.map(f => ({
        episodeNumber: f.episodeNumber,
        seasonNumber: f.seasonNumber,
        filePath: f.filePath,
        subtitlePath: f.subtitlePath,
        subtitlePaths: f.subtitlePaths,
        filename: f.filename,
        title: f.title,
        status: f.status,
        lastProbedAt: f.lastProbedAt,
      })), existingMetadata),
      folderPath: media.folderPath,
      type: media.type,
      source: 'local',
    };
  }

  // Small delay to avoid rate limiting
  await new Promise(resolve => setTimeout(resolve, 500));

  return slice;
}

// ==================== SCAN AND FETCH COMBINED ====================

async function runScanAndFetch(folderPath: string, activeRoots: string[]): Promise<{ success: boolean; count: number }> {
  logger.info('folder', `Starting scan and metadata fetch for: ${folderPath}`, { file: folderPath });

  // 1. Scan. Pass activeRoots so scanFolder treats a sub-folder (rescan-show)
  // as part of its containing library root, not as a fresh root itself.
  const scannedMedia = await folderHandler.scanFolder(folderPath, activeRoots);

  // 2. Reconcile against disk: drop fileEpisodes pointing at files that are
  // gone. Keeps metadata.json honest about what's on disk.
  await metadataHandler.transaction(async (raw) => {
    const reconciled = await folderHandler.reconcileMetadata(raw, activeRoots);
    return { updated: reconciled };
  });

  // 3. Filesystem-only pass: build/refresh fileEpisodes from disk for every
  // series. NO network, NO ffmpeg, NO thumbnails, NO posters. The renderer
  // doesn't need them — it walks the filesystem directly via library:walk
  // for display. metadata.json is kept in sync only because the player still
  // resolves files by seriesId. Metadata enrichment is paused.
  const scannedIds = new Set<string>();
  await metadataHandler.transaction(async (current) => {
    let touched = 0;
    for (const media of scannedMedia) {
      if (media.files.length === 0) continue;
      scannedIds.add(media.id);
      const existing = (current[media.id] ?? {}) as {
        fileEpisodes?: Array<{ filePath: string; status?: string; lastProbedAt?: number }>;
        title?: string;
        folderPath?: string;
      };

      const oldByPath = new Map(
        (existing.fileEpisodes ?? []).map((f) => [f.filePath, f]),
      );
      const newFileEpisodes = media.files.map((f) => {
        const old = oldByPath.get(f.filePath);
        return {
          episodeNumber: f.episodeNumber,
          seasonNumber: f.seasonNumber,
          filePath: f.filePath,
          subtitlePath: f.subtitlePath,
          subtitlePaths: f.subtitlePaths,
          filename: f.filename,
          title: f.title,
          status: old?.status ?? f.status,
          lastProbedAt: old?.lastProbedAt ?? f.lastProbedAt,
        };
      });

      const oldPaths = new Set(oldByPath.keys());
      const newPaths = new Set(newFileEpisodes.map((f) => f.filePath));
      const sameLength = oldPaths.size === newPaths.size;
      const sameContents = sameLength && [...newPaths].every((p) => oldPaths.has(p));
      if (sameContents && existing.folderPath === media.folderPath) continue;

      current[media.id] = {
        ...existing,
        seriesId: media.id,
        title: existing.title ?? media.name,
        fileEpisodes: newFileEpisodes,
        folderPath: media.folderPath,
        type: media.type,
      };
      touched++;
      logger.info('folder', `Sync: ${media.name} → ${newFileEpisodes.length} file(s)`, { series: media.name });
    }
    if (touched === 0) return { updated: null };
    return { updated: current };
  });
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send('metadata:file-status-changed', { filePath: '', status: 'ready' });
  }

  // 4. Cleanup: drop any pre-existing entries whose folderPath is under the
  // scanned folder but didn't show up in this scan — they're stale (deleted
  // series, orphan movie_* entries from earlier misclassifications, etc.).
  // Entries already replaced via the per-series writes above stay intact.
  const normalize = (p: string) => (p.endsWith('/') ? p.slice(0, -1) : p);
  const normalizedScan = normalize(folderPath);
  const scanPrefix = normalizedScan + '/';
  await metadataHandler.transaction(async (current) => {
    const merged: Record<string, unknown> = {};
    let dropped = 0;
    for (const [seriesId, seriesData] of Object.entries(current)) {
      const fp = (seriesData as { folderPath?: string })?.folderPath;
      const isUnderScan = fp && (normalize(fp) === normalizedScan || normalize(fp).startsWith(scanPrefix));
      if (!isUnderScan || scannedIds.has(seriesId)) {
        merged[seriesId] = seriesData;
      } else {
        dropped++;
      }
    }
    if (dropped === 0) return { updated: null };
    return { updated: merged };
  });

  logger.info('folder', `Scan complete! Found ${scannedMedia.length} items`);
  return { success: true, count: scannedMedia.length };
}

ipcMain.handle('scan-and-fetch-metadata', async (_event, folderPath: string) => {
  try {
    const activeRoots = await configHandler.getFolderSources();
    return await runScanAndFetch(folderPath, activeRoots);
  } catch (error) {
    logger.error('folder', 'Error in scan-and-fetch-metadata');
    throw error;
  }
});

ipcMain.handle('get-series-episodes', async (_event, seriesId: string) => {
  try {
    const metadata = await metadataHandler.loadMetadata();
    const series = metadata[seriesId] as { episodes?: unknown[] } | undefined;
    return series?.episodes || [];
  } catch (error) {
    logger.error('metadata', 'Error getting series episodes');
    return [];
  }
});

// ==================== IMAGE CACHE IPC ====================

ipcMain.handle('get-image-cache-stats', async () => {
  try {
    return await imageCacheHandler.getCacheStats();
  } catch (error) {
    logger.error('image', 'Error getting image cache stats');
    return { count: 0, sizeBytes: 0 };
  }
});

ipcMain.handle('clear-image-cache', async () => {
  try {
    await imageCacheHandler.clearCache();
    return true;
  } catch (error) {
    logger.error('image', 'Error clearing image cache');
    throw error;
  }
});

ipcMain.handle('get-image-cache-path', () => {
  return imageCacheHandler.getCachePath();
});

// ==================== VIDEO PROBE IPC ====================

ipcMain.handle('probe:retry', (_event, filePath: string) => {
  if (typeof filePath === 'string' && filePath.length > 0) {
    videoProbeHandler.retry(filePath);
  }
});

ipcMain.handle('subtitle:list-embedded', async (_event, videoPath: string) => {
  if (typeof videoPath !== 'string' || !videoPath) return [];
  return subtitleHandler.listEmbedded(videoPath);
});

ipcMain.handle('subtitle:extract', async (_event, videoPath: string, streamIndex: number, codec: string) => {
  if (typeof videoPath !== 'string' || !videoPath || typeof streamIndex !== 'number') return null;
  return subtitleHandler.extractEmbedded(videoPath, streamIndex, codec ?? '');
});

ipcMain.handle('aniskip:fetch', async (_event, seriesId: string, malId: number, episodeNumber: number, episodeLength: number) => {
  if (!seriesId || typeof malId !== 'number' || typeof episodeNumber !== 'number' || typeof episodeLength !== 'number') {
    return {};
  }
  return aniSkipHandler.fetchAndCache(seriesId, malId, episodeNumber, episodeLength);
});

// ----- Tracker (MAL + AniList progress sync) -----
function isProvider(v: unknown): v is TrackerProvider {
  return v === 'anilist' || v === 'mal';
}

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

function broadcastProgressChanged(): void {
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send('tracker:progress-changed');
  }
}

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

ipcMain.handle('tracker:get-main-provider', async () => {
  return trackerHandler.getMainProvider();
});

ipcMain.handle('tracker:set-main-provider', async (_event, provider: unknown) => {
  if (!isProvider(provider)) throw new Error('invalid provider');
  await trackerHandler.setMainProvider(provider);
  broadcastProgressChanged();
  return provider;
});

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

app.on('before-quit', () => {
  videoProbeHandler.stop();
  void fileWatcher.stop();
});
