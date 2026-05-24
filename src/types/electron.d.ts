// Renderer-side ambient declaration. The canonical ElectronAPI shape — and
// every domain type the renderer touches — lives in src/main/preload.ts
// (next to the contextBridge.exposeInMainWorld call that defines it). This
// file re-exports those types so renderer code can import from one stable
// path and also makes `window.electronAPI` resolve.
import type { ElectronAPI } from '../main/preload';

export type {
  ElectronAPI,
  LibraryFile,
  LibraryEpisodeAirDate,
  LibraryItem,
  AnilistSearchResult,
  VideoOpenResult,
  SubscriptionFeed,
  SubscriptionsResult,
  TrackerProvider,
  TrackerStatus,
  TrackerMarkResult,
  TrackerProgressSnapshot,
  TrackerProgressEntry,
  TrackerListStatus,
  LogEvent,
  LogLevel,
  LogStage,
  FileStatus,
  ViewHistoryEntry,
} from '../main/preload';

declare global {
  interface Window {
    electronAPI: ElectronAPI;
  }
}
