import { useState, useEffect, useCallback, useRef } from 'react';
import type { FileStatus } from '../../shared/fileStatus';
import type { EpisodeKind } from '../../shared/episodeClassifier';
import type { SubtitleState } from '../../shared/subtitleSupport';

export interface SeriesMetadata {
  seriesId?: string;
  title?: string;
  titleRomaji?: string;
  titleEnglish?: string | null;
  titleNative?: string;
  genres?: string[];
  description?: string;
  poster?: string | null;
  posterLocal?: string | null;
  banner?: string | null;
  bannerLocal?: string | null;
  episodes?: EpisodeMetadata[];
  fileEpisodes?: FileEpisode[];
  folderPath?: string;
  source?: string;
  totalEpisodes?: number | null;
  duration?: number | null;
  season?: string | null;
  seasonYear?: number | null;
  averageScore?: number | null;
  status?: string;
  format?: string;
  type?: 'series' | 'movie';
  studios?: string[];
  startDate?: string | null;
  endDate?: string | null;
  anilistId?: number;
  malId?: number | null;
  relations?: Relation[];
  tags?: Tag[];
  characters?: Character[];
  recommendations?: Recommendation[];
  /** Incognito flag. When true, the series never syncs to external trackers
   *  and is hidden from all list pages unless the session "Show hidden" toggle
   *  is on. Absent / false = visible. */
  hidden?: boolean;
  /** Main animation studio name when known. Populated alongside `studios`
   *  from AniList's studios edge (isMain + isAnimationStudio). Falls back
   *  to the first entry of `studios[]` when AniList doesn't flag one. */
  animationStudio?: string | null;
  [key: string]: unknown;
}

export interface Tag {
  name: string;
  /** AniList tag rank (0–100). Higher = more characteristic of this show. */
  rank: number | null;
  /** Spoils a specific plot point of this show. Hidden unless the user opts in. */
  isMediaSpoiler: boolean;
  /** Spoils a general anime trope (still spoilery, treat as spoiler). */
  isGeneralSpoiler: boolean;
  /** 18+ tag — hidden alongside spoilers behind the same toggle. */
  isAdult: boolean;
  category: string | null;
}

export interface Character {
  anilistId: number;
  name: string | null;
  /** AniList role: MAIN / SUPPORTING / BACKGROUND. */
  role: string | null;
  image: string | null;
  siteUrl: string | null;
}

export interface Recommendation {
  /** Net community recommendation score from AniList. Higher = stronger pick. */
  rating: number | null;
  anilistId: number;
  malId: number | null;
  type: 'ANIME' | 'MANGA' | null;
  format: string | null;
  status: string | null;
  seasonYear: number | null;
  siteUrl: string | null;
  titleRomaji: string | null;
  titleEnglish: string | null;
  poster: string | null;
}

export interface Relation {
  /** AniList relation type — SEQUEL, PREQUEL, SIDE_STORY, ADAPTATION, … */
  relationType: string;
  anilistId: number;
  malId: number | null;
  /** AniList media type — drives whether we navigate in-app (ANIME found
   *  in user's library) or open AniList externally (MANGA, or absent). */
  type: 'ANIME' | 'MANGA' | null;
  /** Sub-format — TV, MOVIE, OVA, MANGA, LIGHT_NOVEL, … */
  format: string | null;
  status: string | null;
  seasonYear: number | null;
  /** Canonical AniList URL for the related entry — used as the click
   *  target for external-only relations. */
  siteUrl: string | null;
  titleRomaji: string | null;
  titleEnglish: string | null;
  /** Remote cover image URL. Not locally cached for v1 — relation
   *  posters are render-on-demand from AniList's CDN. */
  poster: string | null;
}

export interface EpisodeMetadata {
  episodeNumber: number;
  seasonNumber?: number | null;
  title?: string;
  description?: string | null;
  airDate?: string | null;
  thumbnail?: string | null;
  thumbnailLocal?: string | null;
  filePath?: string;
  subtitlePath?: string | null;
  subtitlePaths?: string[];
  status?: FileStatus;
  lastProbedAt?: number;
  // Skip times — populated lazily on first play. Source records whether
  // the values came from the file's embedded chapter markers or the
  // AniSkip community DB, so we know if a re-probe is worthwhile (only
  // when source is not yet 'chapters').
  opStart?: number;
  opEnd?: number;
  edStart?: number;
  edEnd?: number;
  skipFetched?: boolean;
  skipSource?: 'chapters' | 'aniskip';
}

export interface FileEpisode {
  episodeNumber: number;
  seasonNumber?: number | null;
  filePath: string;
  subtitlePath: string | null;
  subtitlePaths: string[];
  filename: string;
  title?: string;
  status?: FileStatus;
  lastProbedAt?: number;
  // Absolute path to a pre-transcoded .mp4 under userData/transcode-cache/,
  // present once the file has been converted from a Chromium-incompatible
  // codec (HEVC etc.) so the <video> element can play it natively. Cached
  // across launches and validated on startup.
  transcodedPath?: string | null;
  // Classifier output for bonus content (OP/ED/PV/SP/extras). Absent on real
  // episodes and on entries persisted before the classifier landed. Used by the
  // player header to label an extra by its own identity instead of the
  // colliding episode number it shares with a real episode.
  kind?: EpisodeKind;
  extraIndex?: number | null;
  extraVariant?: string | null;
  rawLabel?: string | null;
  // Subtitle availability marker (bitmap-only / failed-to-load). Persisted by
  // main; drives the episode-row "no subs" marker. See shared/subtitleSupport.ts.
  subtitleState?: SubtitleState | null;
  subtitleCheckedAt?: number;
}

const hasElectronAPI = typeof window !== 'undefined' && window.electronAPI;

export function useMetadata() {
  const [metadata, setMetadata] = useState<Record<string, SeriesMetadata>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  // Only the very first load should flip `loading` on — every subsequent
  // reload (file-status ping, refresh button, match-modal apply, delete)
  // silently swaps the data so MetadataTab's table stays mounted and the
  // user's scroll position is preserved.
  const hasLoadedRef = useRef(false);

  const loadMetadata = useCallback(async () => {
    const isInitial = !hasLoadedRef.current;
    try {
      if (isInitial) setLoading(true);
      if (hasElectronAPI) {
        const data = await window.electronAPI.loadMetadata();
        setMetadata((data || {}) as Record<string, SeriesMetadata>);
      } else {
        console.warn('electronAPI not available, using empty metadata');
        setMetadata({});
      }
      setError(null);
      hasLoadedRef.current = true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      console.error('Error loading metadata:', err);
    } finally {
      if (isInitial) setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadMetadata();
  }, [loadMetadata]);

  const saveMetadata = async (newMetadata: Record<string, SeriesMetadata>): Promise<void> => {
    try {
      if (hasElectronAPI) {
        await window.electronAPI.saveMetadata(newMetadata as Record<string, unknown>);
        await loadMetadata();
      } else {
        console.warn('electronAPI not available, cannot save metadata');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      throw err;
    }
  };

  const updateSeriesMetadata = async (seriesId: string, seriesData: Partial<SeriesMetadata>): Promise<void> => {
    try {
      const updatedMetadata = {
        ...metadata,
        [seriesId]: {
          ...metadata[seriesId],
          ...seriesData,
        },
      };
      await saveMetadata(updatedMetadata);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      throw err;
    }
  };

  return {
    metadata,
    loading,
    error,
    loadMetadata,
    saveMetadata,
    updateSeriesMetadata,
  };
}
