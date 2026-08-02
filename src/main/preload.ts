import { contextBridge, ipcRenderer } from 'electron';

export type { LogLevel, LogStage, LogEvent } from '../shared/logTypes';
import type { FranchiseGraph } from '../shared/franchise';
import type { LogEvent } from '../shared/logTypes';
export type { FileStatus } from '../shared/fileStatus';
import type { FileStatus } from '../shared/fileStatus';
export type { SubtitleState } from '../shared/subtitleSupport';
import type { SubtitleState } from '../shared/subtitleSupport';
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
  /** Incognito flag mirrored from metadata.json so every list page can filter
   *  without a separate metadata fetch. */
  hidden: boolean;
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
  saveMetadata: (metadata: Record<string, unknown>) => Promise<boolean>;
  setSeriesHidden: (seriesId: string, hidden: boolean) => Promise<boolean>;
  loadMetadata: () => Promise<Record<string, unknown>>;
  clearMetadata: () => Promise<boolean>;
  deleteSeries: (seriesId: string) => Promise<boolean>;
  getSeriesEpisodes: (seriesId: string) => Promise<unknown[]>;
  attachMissingSources: () => Promise<{ backfilled: number; matched: number; stillUnmatched: number }>;

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

  // Series-level transcode queue. getTranscodeQueueSnapshot() is the initial
  // pull; onTranscodeQueueChanged streams the full map on every change.
  getTranscodeQueueSnapshot: () => Promise<TranscodeQueueSnapshot>;
  onTranscodeQueueChanged: (handler: (snap: TranscodeQueueSnapshot) => void) => () => void;

  /**
   * Which encoder transcodes run on. `kind: 'libx264'` means no hardware
   * encoder was usable and every transcode is burning CPU; `reason` says why.
   */
  getTranscodeEncoder: () => Promise<TranscodeEncoderStatus>;

  /**
   * Re-pull air dates for one releasing series if they've gone stale, so an
   * open series page shows a current next-episode countdown. No-ops for
   * finished series and inside the refresh TTL.
   */
  refreshAiring: (seriesId: string) => Promise<{ ok: boolean; updated: boolean }>;

  // Embedded subtitles
  listEmbeddedSubtitles: (videoPath: string) => Promise<Array<{ streamIndex: number; codec: string; language: string | null; title: string | null }>>;
  extractEmbeddedSubtitle: (videoPath: string, streamIndex: number, codec: string) => Promise<{ path: string; format: 'ass' | 'vtt' } | null>;
  /** Warm the embedded-subtitle cache ahead of play time (fire-and-forget). */
  prewarmSubtitles: (videoPath: string) => Promise<void>;

  // Per-file subtitle availability for the episode-list marker.
  // evaluateSeriesSubtitles: cheap probe sweep over a series' files on open
  // (flags bitmap-only / unreadable subs). reportSubtitleState: authoritative
  // play-time outcome from the player. onSubtitleStateChanged: live push when
  // the play-time outcome lands while the series page is open.
  evaluateSeriesSubtitles: (filePaths: string[]) => Promise<Array<{ filePath: string; state: SubtitleState | null }>>;
  reportSubtitleState: (filePath: string, state: SubtitleState) => Promise<void>;
  onSubtitleStateChanged: (handler: (payload: { filePath: string; subtitleState: SubtitleState }) => void) => () => void;

  /**
   * Append a line to the subtitle debug log (userData/logs/subtitles.log).
   * Fire-and-forget; main prefixes the scope with `renderer/`. Debug-only,
   * never reaches the user-facing activity log.
   */
  subLog: (scope: string, message: string, data?: unknown) => void;

  // Open a video — main checks for a pre-transcoded cache entry, otherwise
  // returns the original file:// URL. The renderer hands the URL to <video>.
  openVideo: (filePath: string) => Promise<VideoOpenResult>;

  // Opening a series page calls this with all its file paths (episode order)
  // so every episode needing re-encode is probed and priority-queued at once,
  // instead of waiting for the user to click each one. Returns each file's
  // state; live progress then arrives via onTranscodeProgress.
  ensureSeriesTranscoded: (filePaths: string[]) => Promise<TranscodeEnsureResult[]>;

  // Stopping re-encodes. cancelTranscode kills the named file's ffmpeg (or
  // drops it from the queue); cancelAllTranscodes clears the lot. Both are
  // remembered so the automatic sweeps don't immediately re-queue what was
  // just stopped — opening the episode still forces a fresh encode.
  cancelTranscode: (filePath: string) => Promise<{ ok: boolean; stopped: boolean }>;
  cancelAllTranscodes: () => Promise<{ ok: boolean; stopped: number }>;
  resumeTranscode: (filePath: string) => Promise<{ ok: boolean }>;
  getTranscodeAuto: () => Promise<TranscodeAutoState>;
  setTranscodeAuto: (enabled: boolean) => Promise<{ auto: boolean; stopped: number; resumed: number }>;

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
  // Launch mpv on a library file. Resolves as soon as mpv is up, not when it
  // exits. `context` lets main attribute the session to an episode: without it
  // the file still plays, but nothing is recorded when it ends.
  openWithMpv: (filePath: string, context?: MpvLaunchContext) => Promise<boolean>;
  // Fires when an mpv session started by openWithMpv ends. Carries the final
  // playhead so the renderer can store the resume position (localStorage is
  // renderer-owned). View history and tracker updates are applied in main
  // before this fires — the renderer only handles the resume point.
  onMpvPlaybackEnded: (handler: (report: MpvPlaybackEnded) => void) => () => void;

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

  // Franchise graph
  getFranchiseGraph: (anilistId: number) => Promise<FranchiseGraph | null>;
  getFranchiseCrawlProgress: () => Promise<{ total: number; crawled: number }>;
  onFranchiseStoreUpdated: (handler: () => void) => () => void;
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

// Series-level view of the transcode queue. Keyed by seriesId; a series is
// 'encoding' if one of its episodes is the active ffmpeg job, else 'queued'
// if any of its episodes are waiting. 'encoding' wins when both apply.
export type TranscodeQueueStatus = 'encoding' | 'queued';
export type TranscodeQueueSnapshot = Record<string, TranscodeQueueStatus>;

// Which encoder the transcode pipeline resolved to. 'vaapi' / 'nvenc' are
// hardware; 'libx264' is the CPU fallback, which saturates every core for
// the length of an encode. `reason` is non-null only for that fallback and
// explains why hardware was unusable.
export interface TranscodeEncoderStatus {
  kind: 'vaapi' | 'nvenc' | 'libx264';
  reason: string | null;
}

// Per-file classification returned by ensureSeriesTranscoded:
//   'cached'  — a usable transcode already exists on disk (shows "Re-encoded").
//   'pending' — needs transcoding; it has been priority-queued, so live
//               progress events will follow on the transcode-progress channel.
//   'none'    — browser-playable as-is (or missing); nothing to do.
//   'stopped' — needs transcoding, but the user stopped this file (or turned
//               automatic re-encoding off), so nothing was queued.
export interface TranscodeEnsureResult {
  filePath: string;
  state: 'cached' | 'pending' | 'none' | 'stopped';
}

// Whether the automatic re-encode sweeps may queue work, and how many files
// the user has individually stopped.
export interface TranscodeAutoState {
  auto: boolean;
  optedOutCount: number;
}

// What the renderer tells main about an mpv launch. Everything is optional:
// a launch with no context still plays, it just can't be attributed to an
// episode when it ends.
export interface MpvLaunchContext {
  seriesId?: string | null;
  episodeNumber?: number | null;
  /** OP/ED/PV/SP — shares an episodeNumber with a real episode, so it must
   *  never move the tracker or the view history. */
  isExtra?: boolean;
  /** Resume point in seconds, passed to mpv as --start. */
  startSec?: number;
}

// Final state of an mpv session, pushed when the window closes.
export interface MpvPlaybackEnded {
  filePath: string;
  seriesId: string | null;
  episodeNumber: number | null;
  isExtra: boolean;
  /** Last observed playhead in seconds. */
  position: number;
  /** Media duration in seconds; 0 when mpv never reported one. */
  duration: number;
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
  saveMetadata: (metadata: Record<string, unknown>) => ipcRenderer.invoke('save-metadata', metadata),
  setSeriesHidden: (seriesId: string, hidden: boolean) =>
    ipcRenderer.invoke('metadata:set-hidden', seriesId, hidden),
  loadMetadata: () => ipcRenderer.invoke('load-metadata'),
  clearMetadata: () => ipcRenderer.invoke('clear-metadata'),
  deleteSeries: (seriesId: string) => ipcRenderer.invoke('delete-series', seriesId),
  getSeriesEpisodes: (seriesId: string) => ipcRenderer.invoke('get-series-episodes', seriesId),
  attachMissingSources: () => ipcRenderer.invoke('metadata:attach-missing-sources'),

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
  getTranscodeQueueSnapshot: () => ipcRenderer.invoke('transcode:queue-snapshot'),
  getTranscodeEncoder: () => ipcRenderer.invoke('transcode:encoder'),
  refreshAiring: (seriesId: string) => ipcRenderer.invoke('metadata:refresh-airing', seriesId),
  onTranscodeQueueChanged: (handler: (snap: TranscodeQueueSnapshot) => void) => {
    const listener = (_e: unknown, snap: TranscodeQueueSnapshot) => handler(snap);
    ipcRenderer.on('transcode:queue-changed', listener);
    return () => ipcRenderer.removeListener('transcode:queue-changed', listener);
  },

  // Embedded subtitles
  listEmbeddedSubtitles: (videoPath: string) => ipcRenderer.invoke('subtitle:list-embedded', videoPath),
  extractEmbeddedSubtitle: (videoPath: string, streamIndex: number, codec: string) => ipcRenderer.invoke('subtitle:extract', videoPath, streamIndex, codec),
  prewarmSubtitles: (videoPath: string) => ipcRenderer.invoke('subtitle:prewarm', videoPath),
  evaluateSeriesSubtitles: (filePaths: string[]) => ipcRenderer.invoke('subtitle:evaluate-series', filePaths),
  reportSubtitleState: (filePath: string, state: SubtitleState) => ipcRenderer.invoke('subtitle:report-state', filePath, state),
  onSubtitleStateChanged: (handler: (payload: { filePath: string; subtitleState: SubtitleState }) => void) => {
    const listener = (_e: unknown, payload: { filePath: string; subtitleState: SubtitleState }) => handler(payload);
    ipcRenderer.on('metadata:subtitle-state-changed', listener);
    return () => ipcRenderer.removeListener('metadata:subtitle-state-changed', listener);
  },
  subLog: (scope: string, message: string, data?: unknown) => ipcRenderer.send('subtitle:log', scope, message, data),

  // Video open
  openVideo: (filePath: string) => ipcRenderer.invoke('video:open', filePath),
  ensureSeriesTranscoded: (filePaths: string[]) => ipcRenderer.invoke('transcode:ensure-series', filePaths),
  cancelTranscode: (filePath: string) => ipcRenderer.invoke('transcode:cancel', filePath),
  cancelAllTranscodes: () => ipcRenderer.invoke('transcode:cancel-all'),
  resumeTranscode: (filePath: string) => ipcRenderer.invoke('transcode:resume', filePath),
  getTranscodeAuto: () => ipcRenderer.invoke('transcode:get-auto'),
  setTranscodeAuto: (enabled: boolean) => ipcRenderer.invoke('transcode:set-auto', enabled),

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
  openWithMpv: (filePath: string, context?: MpvLaunchContext) =>
    ipcRenderer.invoke('shell:open-with-mpv', filePath, context ?? null),
  onMpvPlaybackEnded: (handler: (report: MpvPlaybackEnded) => void) => {
    const listener = (_e: unknown, report: MpvPlaybackEnded) => handler(report);
    ipcRenderer.on('playback:mpv-ended', listener);
    return () => ipcRenderer.removeListener('playback:mpv-ended', listener);
  },

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

  // Franchise graph
  getFranchiseGraph: (anilistId: number) => ipcRenderer.invoke('franchise:graph', anilistId),
  getFranchiseCrawlProgress: () => ipcRenderer.invoke('franchise:crawl-progress'),
  onFranchiseStoreUpdated: (handler: () => void) => {
    const listener = () => handler();
    ipcRenderer.on('franchise:store-updated', listener);
    return () => ipcRenderer.removeListener('franchise:store-updated', listener);
  },
});
