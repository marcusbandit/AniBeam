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
  episodes: LibraryEpisodeAirDate[];
  files: LibraryFile[];
}

interface CacheStats {
  count: number;
  sizeBytes: number;
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
  
  // Metadata
  fetchMetadata: (seriesName: string) => Promise<unknown>;
  fetchAnilistMetadata: (seriesName: string) => Promise<unknown>;
  fetchMALMetadata: (seriesName: string) => Promise<unknown>;
  fetchTVDBMetadata: (seriesName: string) => Promise<unknown>;
  saveMetadata: (metadata: Record<string, unknown>) => Promise<boolean>;
  loadMetadata: () => Promise<Record<string, unknown>>;
  clearMetadata: () => Promise<boolean>;
  deleteSeries: (seriesId: string) => Promise<boolean>;
  getSeriesEpisodes: (seriesId: string) => Promise<unknown[]>;

  // Match picker
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
}

declare global {
  interface Window {
    electronAPI: ElectronAPI;
  }
}

export {};

