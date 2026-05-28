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
export type { MarkResult as TrackerMarkResult, ScoreResult as TrackerScoreResult } from './handlers/trackerHandler';
import type { MarkResult as TrackerMarkResult, ScoreResult as TrackerScoreResult } from './handlers/trackerHandler';
export type { AnilistWatchingEntry, WatchingListResult } from './handlers/trackerHandler';
import type { WatchingListResult } from './handlers/trackerHandler';

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
  /**
   * Discriminator for what this file actually is. `episode` is a real numbered
   * episode; the other values are bonus content extracted from release-group
   * naming. Renderer code that displays the canonical episode list MUST filter
   * on `kind === 'episode'` — non-episode kinds also carry an `episodeNumber`
   * (set to their extras index for sorting within a group) and would otherwise
   * collide on whatever digit their label happened to end with.
   *
   * Optional only for backward compatibility with library entries persisted
   * before the classifier landed; treat a missing `kind` as 'episode'.
   */
  kind?: 'episode' | 'op' | 'ed' | 'pv' | 'sp' | 'other';
  /** Numeric index lifted from the extras token (ED1 → 1, OP4a → 4). Null for episodes. */
  extraIndex?: number | null;
  /** Letter suffix on the extras token (OP4a → "a", OP3 → null). */
  extraVariant?: string | null;
  /** The matched extras token verbatim ("OP4a", "ED1", "PV12"). Null for episodes. */
  rawLabel?: string | null;
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
  /** Raw score from the matched metadata source. AniList is 0-100, MAL is 0-10. */
  averageScore: number | null;
  /** Where the metadata was fetched from — controls how `averageScore` is normalised. */
  source: string | null;
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

  // Live transcode progress (emitted while ffmpeg is re-encoding a file
  // to the cached browser-playable MP4).
  onTranscodeProgress: (handler: (payload: TranscodeProgressPayload) => void) => () => void;

  // Embedded subtitles
  listEmbeddedSubtitles: (videoPath: string) => Promise<Array<{ streamIndex: number; codec: string; language: string | null; title: string | null }>>;
  extractEmbeddedSubtitle: (videoPath: string, streamIndex: number, codec: string) => Promise<{ path: string; format: 'ass' | 'vtt' } | null>;

  // Open a video — main checks for a pre-transcoded cache entry, otherwise
  // returns the original file:// URL. The renderer hands the URL to <video>.
  openVideo: (filePath: string) => Promise<VideoOpenResult>;

  // View history — per-series record of the most recent playback session,
  // backing the Library "Last viewed" sort. Renderer marks an episode after
  // it has accumulated ~30s of playtime (one mark per player mount).
  markEpisodeViewed: (payload: { seriesId: string; episodeNumber: number; ts?: number }) => Promise<boolean>;
  getViewHistory: () => Promise<Record<string, ViewHistoryEntry>>;
  onViewHistoryChanged: (handler: () => void) => () => void;

  // Skip times — chapter markers first, AniSkip community DB as fallback.
  fetchSkipTimes: (seriesId: string, malId: number, episodeNumber: number, episodeLength: number, filePath?: string) => Promise<{ op?: { start: number; end: number }; ed?: { start: number; end: number }; source?: 'chapters' | 'aniskip' }>;

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
  trackerSetScore: (
    provider: TrackerProvider,
    mediaId: number,
    score: number,
    totalEpisodes: number | null,
  ) => Promise<TrackerScoreResult>;
  /** Set watched progress to an exact value (can decrease — corrects over-counts). */
  trackerSetProgress: (
    provider: TrackerProvider,
    mediaId: number,
    progress: number,
    totalEpisodes: number | null,
  ) => Promise<TrackerMarkResult>;
  trackerGetProgress: () => Promise<TrackerProgressSnapshot>;
  trackerRefreshProgress: (provider?: TrackerProvider) => Promise<TrackerProgressSnapshot>;
  trackerGetMainProvider: () => Promise<TrackerProvider>;
  trackerSetMainProvider: (provider: TrackerProvider) => Promise<TrackerProvider>;
  /** AniList "Currently Watching" + "Rewatching" list, with media metadata. */
  trackerGetWatchingList: () => Promise<WatchingListResult>;
  onTrackerProgressChanged: (handler: () => void) => () => void;

  // Subscriptions (anirss feed list)
  listSubscriptions: () => Promise<SubscriptionsResult>;
}

export interface ViewHistoryEntry {
  /** ms-since-epoch the user crossed the watched-threshold for the session. */
  lastViewedAt: number;
  /** Episode number of that session. */
  lastEpisode: number;
}

export type VideoOpenResult =
  | { kind: 'direct'; url: string }
  | { kind: 'transcoding'; vCodec: string; aCodec: string }
  | { kind: 'unsupported'; vCodec: string; aCodec: string };

export interface TranscodeProgressPayload {
  filePath: string;
  currentSec: number;
  totalSec: number;
  fraction: number;
  speed: number | null;
  etaSec: number | null;
}

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
  onTranscodeProgress: (handler: (payload: TranscodeProgressPayload) => void) => {
    const listener = (_e: unknown, payload: TranscodeProgressPayload) => handler(payload);
    ipcRenderer.on('metadata:transcode-progress', listener);
    return () => ipcRenderer.removeListener('metadata:transcode-progress', listener);
  },

  // Embedded subtitles
  listEmbeddedSubtitles: (videoPath: string) => ipcRenderer.invoke('subtitle:list-embedded', videoPath),
  extractEmbeddedSubtitle: (videoPath: string, streamIndex: number, codec: string) => ipcRenderer.invoke('subtitle:extract', videoPath, streamIndex, codec),

  // Video open
  openVideo: (filePath: string) => ipcRenderer.invoke('video:open', filePath),

  // View history
  markEpisodeViewed: (payload: { seriesId: string; episodeNumber: number; ts?: number }) =>
    ipcRenderer.invoke('playback:viewed', payload),
  getViewHistory: () => ipcRenderer.invoke('playback:get-view-history'),
  onViewHistoryChanged: (handler: () => void) => {
    const listener = () => handler();
    ipcRenderer.on('playback:view-history-changed', listener);
    return () => ipcRenderer.removeListener('playback:view-history-changed', listener);
  },

  // AniSkip
  fetchSkipTimes: (seriesId: string, malId: number, episodeNumber: number, episodeLength: number, filePath?: string) => ipcRenderer.invoke('aniskip:fetch', seriesId, malId, episodeNumber, episodeLength, filePath),

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
  trackerSetScore: (provider: TrackerProvider, mediaId: number, score: number, totalEpisodes: number | null) =>
    ipcRenderer.invoke('tracker:set-score', provider, mediaId, score, totalEpisodes),
  trackerSetProgress: (provider: TrackerProvider, mediaId: number, progress: number, totalEpisodes: number | null) =>
    ipcRenderer.invoke('tracker:set-progress', provider, mediaId, progress, totalEpisodes),
  trackerGetProgress: () => ipcRenderer.invoke('tracker:get-progress'),
  trackerRefreshProgress: (provider?: TrackerProvider) => ipcRenderer.invoke('tracker:refresh-progress', provider ?? null),
  trackerGetMainProvider: () => ipcRenderer.invoke('tracker:get-main-provider'),
  trackerSetMainProvider: (provider: TrackerProvider) => ipcRenderer.invoke('tracker:set-main-provider', provider),
  trackerGetWatchingList: () => ipcRenderer.invoke('tracker:get-watching-list'),
  onTrackerProgressChanged: (handler: () => void) => {
    const listener = () => handler();
    ipcRenderer.on('tracker:progress-changed', listener);
    return () => ipcRenderer.removeListener('tracker:progress-changed', listener);
  },

  // Subscriptions
  listSubscriptions: () => ipcRenderer.invoke('subscriptions:list'),
});
