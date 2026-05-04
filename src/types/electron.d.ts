interface ScanResult {
  success: boolean;
  count: number;
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
  fetchAnilistById: (id: number, seasonNumber?: number | null) => Promise<unknown>;
  
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

