import { contextBridge, ipcRenderer } from 'electron';

export type { LogLevel, LogStage, LogEvent } from '../shared/logTypes';
import type { LogEvent } from '../shared/logTypes';
export type { FileStatus } from '../shared/fileStatus';
import type { FileStatus } from '../shared/fileStatus';

interface ScanResult {
  success: boolean;
  count: number;
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
  fetchAnilistById: (id: number, seasonNumber?: number | null) => Promise<unknown>;
  
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

  // AniSkip — intro/outro skip times
  fetchSkipTimes: (seriesId: string, malId: number, episodeNumber: number, episodeLength: number) => Promise<{ op?: { start: number; end: number }; ed?: { start: number; end: number } }>;

  // Shell — open a URL in the user's default browser, not an Electron window.
  openExternal: (url: string) => Promise<boolean>;

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
}

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

export type TrackerProvider = 'anilist' | 'mal';
export interface TrackerStatus {
  connected: boolean;
  username: string | null;
  expiresAt: number | null;
  lastSync: number | null;
  clientId: string;
  hasClientSecret: boolean;
  cipherEncrypted: boolean;
}
export interface TrackerMarkResult {
  ok: boolean;
  provider: TrackerProvider;
  newProgress: number | null;
  previousProgress: number | null;
  reason?: 'no-account' | 'no-id' | 'not-newer' | 'error';
  message?: string;
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
  fetchAnilistById: (id: number, seasonNumber?: number | null) => ipcRenderer.invoke('anilist:fetch-by-id', id, seasonNumber ?? null),
  
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

  // AniSkip
  fetchSkipTimes: (seriesId: string, malId: number, episodeNumber: number, episodeLength: number) => ipcRenderer.invoke('aniskip:fetch', seriesId, malId, episodeNumber, episodeLength),

  // Shell
  openExternal: (url: string) => ipcRenderer.invoke('shell:open-external', url),

  // Trackers
  trackerStatus: (provider: TrackerProvider) => ipcRenderer.invoke('tracker:status', provider),
  trackerSetClientId: (provider: TrackerProvider, clientId: string) => ipcRenderer.invoke('tracker:set-client-id', provider, clientId),
  trackerGetClientId: (provider: TrackerProvider) => ipcRenderer.invoke('tracker:get-client-id', provider),
  trackerConnect: (provider: TrackerProvider, clientId: string, clientSecret?: string) => ipcRenderer.invoke('tracker:connect', provider, clientId, clientSecret ?? ''),
  trackerCancelConnect: () => ipcRenderer.invoke('tracker:cancel-connect'),
  trackerDisconnect: (provider: TrackerProvider) => ipcRenderer.invoke('tracker:disconnect', provider),
  trackerMarkEpisode: (provider: TrackerProvider, mediaId: number, episodeNumber: number, totalEpisodes: number | null) =>
    ipcRenderer.invoke('tracker:mark-episode', provider, mediaId, episodeNumber, totalEpisodes),
});
