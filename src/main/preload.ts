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
  extractEmbeddedSubtitle: (videoPath: string, streamIndex: number) => Promise<string | null>;

  // AniSkip — intro/outro skip times
  fetchSkipTimes: (seriesId: string, malId: number, episodeNumber: number, episodeLength: number) => Promise<{ op?: { start: number; end: number }; ed?: { start: number; end: number } }>;
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
  extractEmbeddedSubtitle: (videoPath: string, streamIndex: number) => ipcRenderer.invoke('subtitle:extract', videoPath, streamIndex),

  // AniSkip
  fetchSkipTimes: (seriesId: string, malId: number, episodeNumber: number, episodeLength: number) => ipcRenderer.invoke('aniskip:fetch', seriesId, malId, episodeNumber, episodeLength),
});
