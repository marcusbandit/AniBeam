import { contextBridge, ipcRenderer } from 'electron';

export type { LogLevel, LogStage, LogEvent } from '../shared/logTypes';
import type { LogEvent } from '../shared/logTypes';
export type { FileStatus } from '../shared/fileStatus';
import type { FileStatus } from '../shared/fileStatus';
// Source-of-truth tracker types live with the store/handler code in main.
// Re-export from preload so renderer code only has to import from one place.
export type {
  TrackerProvider,
  TrackerStatus,
  ProgressSnapshot as TrackerProgressSnapshot,
  ProgressEntry as TrackerProgressEntry,
  ListStatus as TrackerListStatus,
} from './services/trackerStore';
import type { TrackerProvider, TrackerStatus, ProgressSnapshot as TrackerProgressSnapshot } from './services/trackerStore';
export type { MarkResult as TrackerMarkResult } from './handlers/trackerHandler';
import type { MarkResult as TrackerMarkResult } from './handlers/trackerHandler';

interface ScanResult {
  success: boolean;
  count: number;
}

export interface LibraryFile {
  filename: string;
  filePath: string;
  title: string;
  episodeNumber: number;
  seasonNumber: number | null;
  subtitlePath: string | null;
  subtitlePaths: string[];
  /** Filesystem mtime in ms since epoch. */
  mtime: number;
}

export interface LibraryEpisodeAirDate {
  episodeNumber: number;
  airDate: string | null;
}

export interface LibraryItem {
  id: string;
  folderName: string;
  folderPath: string;
  type: 'series' | 'movie';
  poster: string | null;
  posterLocal: string | null;
  posterMatched: boolean;
  posterMatchAttempted: boolean;
  matchSource: 'mal' | 'anilist' | null;
  matchedTitle: string | null;
  titleRomaji: string | null;
  titleEnglish: string | null;
  status: string | null;
  startDate: string | null;
  totalEpisodes: number | null;
  anilistId: number | null;
  malId: number | null;
  episodes: LibraryEpisodeAirDate[];
  files: LibraryFile[];
}

interface CacheStats {
  count: number;
  sizeBytes: number;
}

export interface ElectronAPI {
  // Config
  getFolderSources: () => Promise<string[]>;
  addFolderSource: (folderPath: string) => Promise<boolean>;
  removeFolderSource: (folderPath: string) => Promise<boolean>;
  
  // Folder scanning
  selectFolder: () => Promise<string | null>;
  scanFolder: (folderPath: string) => Promise<unknown>;
  scanAllFolders: () => Promise<unknown>;
  scanAndFetchMetadata: (folderPath: string) => Promise<ScanResult>;
  libraryWalk: () => Promise<LibraryItem[]>;
  findMovieFolders: (rootPath: string) => Promise<string[]>;
  
  // Metadata
  fetchMetadata: (seriesName: string) => Promise<unknown>;
  fetchAnilistMetadata: (seriesName: string) => Promise<unknown>;
  fetchMALMetadata: (seriesName: string) => Promise<unknown>;
  saveMetadata: (metadata: Record<string, unknown>) => Promise<boolean>;
  loadMetadata: () => Promise<Record<string, unknown>>;
  clearMetadata: () => Promise<boolean>;
  deleteSeries: (seriesId: string) => Promise<boolean>;
  getSeriesEpisodes: (seriesId: string) => Promise<unknown[]>;

  // Match picker (override metadata for a series)
  searchAnilist: (query: string, limit?: number) => Promise<AnilistSearchResult[]>;
  applyAnilistMatch: (
    seriesId: string,
    anilistId: number,
    seasonNumber?: number | null,
  ) => Promise<{ ok: boolean; reason?: string }>;
  
  // Image cache
  getImageCacheStats: () => Promise<CacheStats>;
  clearImageCache: () => Promise<boolean>;
  getImageCachePath: () => Promise<string>;

  // Activity log
  onLogEvent: (handler: (event: LogEvent) => void) => () => void;
  getLogBuffer: () => Promise<LogEvent[]>;
  clearLog: () => Promise<void>;

  // Video probe
  probeRetry: (filePath: string) => Promise<void>;
  onMetadataFileStatusChanged: (handler: (payload: { filePath: string; status: FileStatus }) => void) => () => void;

  // Embedded subtitles
  listEmbeddedSubtitles: (videoPath: string) => Promise<Array<{ streamIndex: number; codec: string; language: string | null; title: string | null }>>;
  extractEmbeddedSubtitle: (videoPath: string, streamIndex: number, codec: string) => Promise<{ path: string; format: 'ass' | 'vtt' } | null>;

  // Open a video — main checks for a pre-transcoded cache entry, otherwise
  // returns the original file:// URL. The renderer hands the URL to <video>.
  openVideo: (filePath: string) => Promise<VideoOpenResult>;

  // AniSkip — intro/outro skip times
  fetchSkipTimes: (seriesId: string, malId: number, episodeNumber: number, episodeLength: number) => Promise<{ op?: { start: number; end: number }; ed?: { start: number; end: number } }>;

  // Shell — open a URL in the user's default browser, not an Electron window.
  openExternal: (url: string) => Promise<boolean>;
  openWithMpv: (filePath: string) => Promise<boolean>;

  // Trackers (MAL + AniList progress sync)
  trackerStatus: (provider: TrackerProvider) => Promise<TrackerStatus>;
  trackerSetClientId: (provider: TrackerProvider, clientId: string) => Promise<TrackerStatus>;
  trackerGetClientId: (provider: TrackerProvider) => Promise<string>;
  trackerConnect: (provider: TrackerProvider, clientId: string, clientSecret?: string) => Promise<TrackerStatus>;
  trackerCancelConnect: () => Promise<boolean>;
  trackerDisconnect: (provider: TrackerProvider) => Promise<TrackerStatus>;
  trackerMarkEpisode: (
    provider: TrackerProvider,
    mediaId: number,
    episodeNumber: number,
    totalEpisodes: number | null,
  ) => Promise<TrackerMarkResult>;
  trackerGetProgress: () => Promise<TrackerProgressSnapshot>;
  trackerRefreshProgress: (provider?: TrackerProvider) => Promise<TrackerProgressSnapshot>;
  trackerGetMainProvider: () => Promise<TrackerProvider>;
  trackerSetMainProvider: (provider: TrackerProvider) => Promise<TrackerProvider>;
  onTrackerProgressChanged: (handler: () => void) => () => void;

  // Subscriptions (anirss feed list)
  listSubscriptions: () => Promise<SubscriptionsResult>;
}

export type VideoOpenResult =
  | { kind: 'direct'; url: string }
  | { kind: 'unsupported'; vCodec: string; aCodec: string };

export interface SubscriptionFeed {
  name: string;
  feedUrl: string;
  savePath: string;
  ruleEnabled: boolean;
  torrentCount: number;
}
export type SubscriptionsResult =
  | { ok: true; items: SubscriptionFeed[] }
  | { ok: false; error: string; needsAuth?: boolean };

export interface AnilistSearchResult {
  id: number;
  title: { romaji: string; english: string | null; native: string };
  coverImage: { large: string; extraLarge: string } | null;
  bannerImage: string | null;
  format: string;
  status: string;
  episodes: number | null;
  season: string | null;
  seasonYear: number | null;
}

declare global {
  interface Window {
    electronAPI: ElectronAPI;
  }
}

contextBridge.exposeInMainWorld('electronAPI', {
  // Config
  getFolderSources: () => ipcRenderer.invoke('get-folder-sources'),
  addFolderSource: (folderPath: string) => ipcRenderer.invoke('add-folder-source', folderPath),
  removeFolderSource: (folderPath: string) => ipcRenderer.invoke('remove-folder-source', folderPath),
  
  // Folder scanning
  selectFolder: () => ipcRenderer.invoke('select-folder'),
  scanFolder: (folderPath: string) => ipcRenderer.invoke('scan-folder', folderPath),
  scanAllFolders: () => ipcRenderer.invoke('scan-all-folders'),
  scanAndFetchMetadata: (folderPath: string) => ipcRenderer.invoke('scan-and-fetch-metadata', folderPath),
  libraryWalk: () => ipcRenderer.invoke('library:walk'),
  findMovieFolders: (rootPath: string) => ipcRenderer.invoke('find-movie-folders', rootPath),
  
  // Metadata
  fetchMetadata: (seriesName: string) => ipcRenderer.invoke('fetch-metadata', seriesName),
  fetchAnilistMetadata: (seriesName: string) => ipcRenderer.invoke('fetch-anilist-metadata', seriesName),
  fetchMALMetadata: (seriesName: string) => ipcRenderer.invoke('fetch-mal-metadata', seriesName),
  saveMetadata: (metadata: Record<string, unknown>) => ipcRenderer.invoke('save-metadata', metadata),
  loadMetadata: () => ipcRenderer.invoke('load-metadata'),
  clearMetadata: () => ipcRenderer.invoke('clear-metadata'),
  deleteSeries: (seriesId: string) => ipcRenderer.invoke('delete-series', seriesId),
  getSeriesEpisodes: (seriesId: string) => ipcRenderer.invoke('get-series-episodes', seriesId),

  // Match picker
  searchAnilist: (query: string, limit?: number) => ipcRenderer.invoke('anilist:search', query, limit),
  applyAnilistMatch: (seriesId: string, anilistId: number, seasonNumber?: number | null) =>
    ipcRenderer.invoke('metadata:apply-anilist-match', seriesId, anilistId, seasonNumber ?? null),
  
  // Image cache
  getImageCacheStats: () => ipcRenderer.invoke('get-image-cache-stats'),
  clearImageCache: () => ipcRenderer.invoke('clear-image-cache'),
  getImageCachePath: () => ipcRenderer.invoke('get-image-cache-path'),

  // Activity log
  onLogEvent: (handler: (event: LogEvent) => void) => {
    const listener = (_e: unknown, event: LogEvent) => handler(event);
    ipcRenderer.on('log:event', listener);
    return () => ipcRenderer.removeListener('log:event', listener);
  },
  getLogBuffer: () => ipcRenderer.invoke('log:get-buffer'),
  clearLog: () => ipcRenderer.invoke('log:clear'),

  // Video probe
  probeRetry: (filePath: string) => ipcRenderer.invoke('probe:retry', filePath),
  onMetadataFileStatusChanged: (handler: (payload: { filePath: string; status: FileStatus }) => void) => {
    const listener = (_e: unknown, payload: { filePath: string; status: FileStatus }) => handler(payload);
    ipcRenderer.on('metadata:file-status-changed', listener);
    return () => ipcRenderer.removeListener('metadata:file-status-changed', listener);
  },

  // Embedded subtitles
  listEmbeddedSubtitles: (videoPath: string) => ipcRenderer.invoke('subtitle:list-embedded', videoPath),
  extractEmbeddedSubtitle: (videoPath: string, streamIndex: number, codec: string) => ipcRenderer.invoke('subtitle:extract', videoPath, streamIndex, codec),

  // Video open
  openVideo: (filePath: string) => ipcRenderer.invoke('video:open', filePath),

  // AniSkip
  fetchSkipTimes: (seriesId: string, malId: number, episodeNumber: number, episodeLength: number) => ipcRenderer.invoke('aniskip:fetch', seriesId, malId, episodeNumber, episodeLength),

  // Shell
  openExternal: (url: string) => ipcRenderer.invoke('shell:open-external', url),
  openWithMpv: (filePath: string) => ipcRenderer.invoke('shell:open-with-mpv', filePath),

  // Trackers
  trackerStatus: (provider: TrackerProvider) => ipcRenderer.invoke('tracker:status', provider),
  trackerSetClientId: (provider: TrackerProvider, clientId: string) => ipcRenderer.invoke('tracker:set-client-id', provider, clientId),
  trackerGetClientId: (provider: TrackerProvider) => ipcRenderer.invoke('tracker:get-client-id', provider),
  trackerConnect: (provider: TrackerProvider, clientId: string, clientSecret?: string) => ipcRenderer.invoke('tracker:connect', provider, clientId, clientSecret ?? ''),
  trackerCancelConnect: () => ipcRenderer.invoke('tracker:cancel-connect'),
  trackerDisconnect: (provider: TrackerProvider) => ipcRenderer.invoke('tracker:disconnect', provider),
  trackerMarkEpisode: (provider: TrackerProvider, mediaId: number, episodeNumber: number, totalEpisodes: number | null) =>
    ipcRenderer.invoke('tracker:mark-episode', provider, mediaId, episodeNumber, totalEpisodes),
  trackerGetProgress: () => ipcRenderer.invoke('tracker:get-progress'),
  trackerRefreshProgress: (provider?: TrackerProvider) => ipcRenderer.invoke('tracker:refresh-progress', provider ?? null),
  trackerGetMainProvider: () => ipcRenderer.invoke('tracker:get-main-provider'),
  trackerSetMainProvider: (provider: TrackerProvider) => ipcRenderer.invoke('tracker:set-main-provider', provider),
  onTrackerProgressChanged: (handler: () => void) => {
    const listener = () => handler();
    ipcRenderer.on('tracker:progress-changed', listener);
    return () => ipcRenderer.removeListener('tracker:progress-changed', listener);
  },

  // Subscriptions
  listSubscriptions: () => ipcRenderer.invoke('subscriptions:list'),
});
