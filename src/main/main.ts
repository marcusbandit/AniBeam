import { app, BrowserWindow, ipcMain, dialog, Menu, protocol, net } from 'electron';
import { fileURLToPath, pathToFileURL } from 'url';
import { dirname, join, isAbsolute, resolve, relative } from 'path';
import { existsSync } from 'fs';
import axios from 'axios';
import folderHandler from './handlers/folderHandler';
import anilistHandler from './handlers/anilistHandler';
import malHandler from './handlers/malHandler';
import metadataHandler from './handlers/metadataHandler';
import configHandler from './handlers/configHandler';
import imageCacheHandler from './handlers/imageCacheHandler';
import thumbnailHandler from './handlers/thumbnailHandler';
import { initMediaProgress, updateMediaProgress } from './utils/debugUtils';
import { logger } from './services/logger';
import type { FileStatus } from '../shared/fileStatus';
import type { ScannedMedia } from './handlers/folderHandler';
import videoProbeHandler from './handlers/videoProbeHandler';
import subtitleHandler from './handlers/subtitleHandler';
import aniSkipHandler from './handlers/aniSkipHandler';
import { fileWatcher } from './services/watcher';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

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
    logger.info('metadata', `Fetching for new file`, { series: media.name, file: filePath });

    // Run network-bound metadata fetch OUTSIDE the lock — it can take seconds.
    // Read a snapshot now; we'll re-merge with the latest state inside the transaction.
    const snapshot = await metadataHandler.loadMetadata();
    const slice = await processOneMedia(media, snapshot);

    await metadataHandler.transaction(async (current) => {
      // Force this newly-discovered file to 'verifying' in the merged slice.
      for (const seriesValue of Object.values(slice)) {
        const s = seriesValue as { fileEpisodes?: Array<{ filePath: string; status?: string; lastProbedAt?: number }> };
        if (!Array.isArray(s.fileEpisodes)) continue;
        for (const f of s.fileEpisodes) {
          if (f.filePath === filePath) {
            f.status = 'verifying';
            f.lastProbedAt = Date.now();
          }
        }
      }
      // Merge slice over the FRESH state (not snapshot) so any concurrent
      // probe completions or other ingests don't get clobbered.
      return { updated: { ...current, ...slice } };
    });

    videoProbeHandler.enqueue(filePath);

    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:file-status-changed', { filePath, status: 'verifying' });
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
    onUnlink: (path) => { void handleUnlink(path); },
    // unlinkDir: chokidar emits unlink for each contained file too, so the
    // per-file handler covers actual cleanup. Just log; don't double-reconcile.
    onUnlinkDir: (dirPath) => { logger.info('watch', `Directory removed`, { file: dirPath }); },
  });

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
  // Try sources in priority order: MAL -> AniList
  const seasonInfo = seasonNumber !== null && seasonNumber !== undefined ? ` Season ${seasonNumber}` : '';
  logger.info('metadata', `Fetching metadata for: "${searchName}"${seasonInfo}`);

  try {
    const malData = await malHandler.searchAndFetchMetadata(searchName, seasonNumber);
    if (malData) {
      logger.info('metadata', `Found on MAL: ${malData.title}`, { series: malData.title });
      return { ...malData, source: 'mal' };
    } else {
      logger.warn('metadata', `MAL returned no results for "${searchName}"${seasonInfo}`);
    }
  } catch (error) {
    if (!isRateLimitError(error)) {
      logger.error('metadata', `MAL failed`);
    }
  }

  try {
    const anilistData = await anilistHandler.searchAndFetchMetadata(searchName, seasonNumber);
    if (anilistData) {
      logger.info('metadata', `Found on AniList (fallback): ${anilistData.title}`, { series: anilistData.title });
      return { ...anilistData, source: 'anilist' };
    } else {
      logger.warn('metadata', `AniList returned no results for "${searchName}"${seasonInfo}`);
    }
  } catch (error) {
    if (!isRateLimitError(error)) {
      logger.error('metadata', `AniList failed`);
    }
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

  // Fetch new metadata with season/part information
  // Try multiple sources in order: MAL -> AniList
  // Pass canonical episode count to validate search results
  let fetchedMetadata = null;

  try {
    fetchedMetadata = await malHandler.searchAndFetchMetadata(media.name, media.seasonNumber, media.partNumber, canonicalEpisodeCount);
    if (fetchedMetadata) {
      logger.info('metadata', `Found on MAL: ${fetchedMetadata.title} (${fetchedMetadata.totalEpisodes || 'unknown'} episodes)`, { series: fetchedMetadata.title });
      fetchedMetadata = { ...fetchedMetadata, source: 'mal' };
    } else {
      logger.warn('metadata', `MAL returned no results for ${media.name}${seasonInfo}`, { series: media.name });
    }
  } catch (err) {
    if (!isRateLimitError(err)) {
      logger.error('metadata', `MAL failed for ${media.name}${seasonInfo}`, { series: media.name });
    }
  }

  if (!fetchedMetadata) {
    try {
      fetchedMetadata = await anilistHandler.searchAndFetchMetadata(media.name, media.seasonNumber, media.partNumber, canonicalEpisodeCount);
      if (fetchedMetadata) {
        logger.info('metadata', `Found on AniList (fallback): ${fetchedMetadata.title} (${fetchedMetadata.totalEpisodes || 'unknown'} episodes)`, { series: fetchedMetadata.title });
        fetchedMetadata = { ...fetchedMetadata, source: 'anilist' };
      } else {
        logger.warn('metadata', `AniList returned no results for ${media.name}${seasonInfo}`, { series: media.name });
      }
    } catch (err) {
      if (!isRateLimitError(err)) {
        logger.error('metadata', `AniList failed for ${media.name}${seasonInfo}`, { series: media.name });
      }
    }
  }

  if (fetchedMetadata) {
    // Cache images locally
    logger.info('image', `Caching images for: ${media.name}`, { series: media.name });

    // Collect all image URLs to cache
    const imagesToCache: (string | null)[] = [
      fetchedMetadata.poster,
      fetchedMetadata.banner,
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
          title: `Episode ${fileEp.episodeNumber.toFixed(1)}`,
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
        title: f.title || `Episode ${f.episodeNumber}`,
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

ipcMain.handle('scan-and-fetch-metadata', async (_event, folderPath: string) => {
  try {
    logger.info('folder', `Starting scan and metadata fetch for: ${folderPath}`, { file: folderPath });

    const activeRoots = await configHandler.getFolderSources();

    // 1. Scan. Pass activeRoots so scanFolder treats a sub-folder (rescan-show)
    // as part of its containing library root, not as a fresh root itself.
    const scannedMedia = await folderHandler.scanFolder(folderPath, activeRoots);

    // 2. Reconcile against disk inside a transaction. Crash-recovery checkpoint:
    // if the per-series fetch loop below throws, on-disk state is the
    // reconciled state (deletions are not reverted).
    const existingMetadata = await metadataHandler.transaction<Record<string, unknown>>(async (raw) => {
      const reconciled = await folderHandler.reconcileMetadata(raw, activeRoots);
      return { result: reconciled, updated: reconciled };
    }) ?? {};

    // 3. Fetch metadata for each scanned item. Network-bound; runs outside the
    // lock — must not assume `existingMetadata` matches disk when we save.
    const newMetadata: Record<string, unknown> = {};
    for (const media of scannedMedia) {
      if (media.files.length === 0) {
        logger.warn('folder', `Skipping ${media.name} - no files found`, { series: media.name });
        continue;
      }
      const slice = await processOneMedia(media, existingMetadata);
      Object.assign(newMetadata, slice);
    }

    // 4. Merge into the FRESH on-disk state. Drop any existing entries whose
    // folderPath is the scanned folder OR a descendant of it — those are
    // either replaced by what's in newMetadata, or they're stale (e.g. orphan
    // movie_* entries from earlier misclassifications) and should not survive.
    const normalize = (p: string) => (p.endsWith('/') ? p.slice(0, -1) : p);
    const normalizedScan = normalize(folderPath);
    const scanPrefix = normalizedScan + '/';
    await metadataHandler.transaction(async (current) => {
      const merged: Record<string, unknown> = {};
      for (const [seriesId, seriesData] of Object.entries(current)) {
        const fp = (seriesData as { folderPath?: string })?.folderPath;
        const isUnderScan = fp && (normalize(fp) === normalizedScan || normalize(fp).startsWith(scanPrefix));
        if (!isUnderScan) merged[seriesId] = seriesData;
      }
      for (const [seriesId, seriesData] of Object.entries(newMetadata)) {
        merged[seriesId] = seriesData;
      }
      return { updated: merged };
    });

    logger.info('folder', `Scan complete! Found ${scannedMedia.length} items`);

    return { success: true, count: scannedMedia.length };
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

ipcMain.handle('subtitle:extract', async (_event, videoPath: string, streamIndex: number) => {
  if (typeof videoPath !== 'string' || !videoPath || typeof streamIndex !== 'number') return null;
  return subtitleHandler.extractEmbedded(videoPath, streamIndex);
});

ipcMain.handle('aniskip:fetch', async (_event, seriesId: string, malId: number, episodeNumber: number, episodeLength: number) => {
  if (!seriesId || typeof malId !== 'number' || typeof episodeNumber !== 'number' || typeof episodeLength !== 'number') {
    return {};
  }
  return aniSkipHandler.fetchAndCache(seriesId, malId, episodeNumber, episodeLength);
});

app.on('before-quit', () => {
  videoProbeHandler.stop();
  void fileWatcher.stop();
});
