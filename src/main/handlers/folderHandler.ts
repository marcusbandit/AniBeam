import { readdir, stat } from 'fs/promises';
import { existsSync } from 'fs';
import { join, extname, basename } from 'path';
import { logger } from '../services/logger';
import imageCacheHandler from './imageCacheHandler';
import thumbnailHandler from './thumbnailHandler';
import type { FileStatus } from '../../shared/fileStatus';

const VIDEO_EXTENSIONS = ['.mp4', '.mkv', '.avi', '.mov', '.wmv', '.flv', '.webm', '.m4v'];
const SUBTITLE_EXTENSIONS = ['.srt', '.vtt', '.ass', '.ssa'];

export interface VideoFile {
  filename: string;
  filePath: string;
  title: string;
  episodeNumber: number;
  seasonNumber: number | null;
  subtitlePath: string | null;
  subtitlePaths: string[];
  parentFolder: string;
  status: FileStatus;
  lastProbedAt?: number;
}

export interface ScannedMedia {
  id: string;
  name: string;           // Name to use for metadata lookup
  type: 'series' | 'movie';
  folderPath: string;
  files: VideoFile[];
  seasonNumber: number | null;  // Season number extracted from folder name
  partNumber: number | null;    // Part number extracted from folder name
}

function isVideoFile(filename: string): boolean {
  return VIDEO_EXTENSIONS.includes(extname(filename).toLowerCase());
}

function isSubtitleFile(filename: string): boolean {
  return SUBTITLE_EXTENSIONS.includes(extname(filename).toLowerCase());
}

function getBaseName(filename: string): string {
  return basename(filename, extname(filename));
}

function extractSeasonAndEpisode(filename: string): { season: number | null; episode: number } {
  // IMPORTANT: Work on base name WITHOUT extension to avoid ".mp4" → "4" bug
  const baseName = getBaseName(filename);

  // First, try to extract season and episode from S01E01 format
  const seasonEpisodeMatch = baseName.match(/\bS(\d+)E(\d+)\b/i);
  if (seasonEpisodeMatch) {
    return {
      season: parseInt(seasonEpisodeMatch[1], 10),
      episode: parseInt(seasonEpisodeMatch[2], 10),
    };
  }

  // Try to extract season from folder name patterns (Season 1, S01, etc.)
  // This will be handled separately when processing folders

  // Try various patterns to extract episode number
  const decimalEpisodeMatch = baseName.match(/Episode\s*(\d+)\.(\d+)/i);
  if (decimalEpisodeMatch) {
    const whole = parseInt(decimalEpisodeMatch[1], 10);
    const decimal = parseInt(decimalEpisodeMatch[2], 10);
    // Store as actual decimal: 6.5, 7.5, 10.5, etc.
    // This allows proper sorting (6, 6.5, 7, 7.5) and correct display
    return {
      season: null,
      episode: whole + decimal / 10,
    };
  }

  const patterns = [
    /Episode\s*(\d+)/i,           // "Episode 10" or "Episode10"
    /Ep\.?\s*(\d+)/i,             // "Ep 10" or "Ep.10"
    /E(\d{2,})/i,                 // "E10" (at least 2 digits)
    /\s-\s*(\d+)(?:\s|$)/,        // " - 10 " or " - 10" at end
    /\[(\d+)\]/,                  // "[10]"
    /\s(\d{1,3})(?:\s|$)/,        // " 10 " or " 10" at end (1-3 digit episode)
  ];

  for (const pattern of patterns) {
    const match = baseName.match(pattern);
    if (match) {
      return {
        season: null,
        episode: parseInt(match[1], 10),
      };
    }
  }

  // Fallback: find numbers in the base name (not extension!)
  const numbers = baseName.match(/\d+/g);
  if (numbers && numbers.length > 0) {
    // Filter out year-like numbers (1900-2099)
    const nonYearNumbers = numbers.filter(n => {
      const num = parseInt(n, 10);
      return num < 1900 || num > 2099;
    });

    if (nonYearNumbers.length > 0) {
      // Return the last non-year number
      return {
        season: null,
        episode: parseInt(nonYearNumbers[nonYearNumbers.length - 1], 10),
      };
    }
  }

  return { season: null, episode: 1 };
}

function extractSeasonNumber(folderName: string): number | null {
  // Try various patterns to extract season number from folder name
  const patterns = [
    /Season\s*(\d+)/i,           // "Season 1" or "Season1"
    /\bS(\d+)\b/i,               // "S01" or "S1"
    /Season\s*(\d+)/i,           // "Season 1"
  ];

  for (const pattern of patterns) {
    const match = folderName.match(pattern);
    if (match) {
      return parseInt(match[1], 10);
    }
  }

  // No season pattern found - return null
  // This correctly handles cases like "86" (show name) vs "Season 1" (season folder)
  return null;
}

function extractPartNumber(folderName: string): number | null {
  // Try various patterns to extract part number from folder name
  const patterns = [
    /Part\s*(\d+)/i,             // "Part 1" or "Part1"
    /\bP(\d+)\b/i,               // "P1" or "P01"
  ];

  for (const pattern of patterns) {
    const match = folderName.match(pattern);
    if (match) {
      return parseInt(match[1], 10);
    }
  }

  // No part pattern found - return null
  return null;
}

function extractSeriesNameFromFilenames(videoFiles: VideoFile[]): string {
  if (videoFiles.length === 0) {
    return '';
  }

  // Extract base names (without extension) and clean them
  const cleanedNames = videoFiles.map(video => {
    const baseName = getBaseName(video.filename);

    // Remove common patterns that indicate episode/season numbers
    let cleaned = baseName
      .replace(/\s*\[[^\]]*\]\s*/g, ' ')                          // Strip anything in [square brackets] (release groups, quality, hashes, etc.)
      .replace(/\s*\bS\d+E\d+(\.\d+)?\b.*$/i, '')                 // Remove S01E01, S1E1, S01E10.5, etc.
      .replace(/\s*\bE\d+(\.\d+)?\b.*$/i, '')                     // Remove E01, E1, E10.5 patterns (keep everything before)
      .replace(/\s*\bPart\s*\d+\b.*$/i, '')                       // Remove 'Part 1', 'Part 2', etc.
      .replace(/\s*\bSeason\s*\d+\b.*$/i, '')                     // Remove 'Season 1', 'Season 2', etc. NOT removing 'season' only if it has a number after it.
      .replace(/\s*Episode\s*\d+(\.\d+)?[ab]?\s*.*$/i, '')        // Remove 'Episode 10', 'Episode 10.5', 'Episode 12a', etc.
      .replace(/\s*Ep\.?\s*\d+(\.\d+)?[ab]?\s*.*$/i, '')          // Remove 'Ep 12', 'Ep. 10.5', etc.
      .replace(/\s*-\s*\d{1,4}(?:\.\d+)?[ab]?(?:\s|$).*$/, '')    // Remove episode at end like '- 01', '- 10.5', '- 12a' but not years
      .replace(/\s+\d{1,3}(?!\d)(?:\s|$)/g, '')                   // Remove standalone numbers (1-3 digits) unless part of larger numbers
      .replace(/\s*\(\d{4}\)\s*$/, '')                            // Remove trailing years (2020), (2021), etc
      .replace(/\s*\([^)]*\)\s*/g, ' ')                           // Remove other parentheses content
      .replace(/\./g, ' ')                                        // Replace dots with spaces
      .replace(/_/g, ' ')                                         // Replace underscores with spaces
      .replace(/\s+/g, ' ')                                       // Normalize whitespace
      .trim();

    return cleaned;
  });

  // Find the longest common prefix
  if (cleanedNames.length === 1) {
    return cleanedNames[0];
  }

  // Sort by length to find common patterns
  cleanedNames.sort((a, b) => a.length - b.length);
  const shortest = cleanedNames[0];

  // Find longest common prefix
  let commonPrefix = '';
  for (let i = 0; i < shortest.length; i++) {
    const char = shortest[i];
    if (cleanedNames.every(name => name[i] === char)) {
      commonPrefix += char;
    } else {
      break;
    }
  }

  // Clean up the common prefix (remove trailing dashes, spaces, etc)
  let result = commonPrefix
    .replace(/[-_\s]+$/, '')  // Remove trailing dashes, underscores, spaces
    .trim();

  // If common prefix is too short or empty, try finding common words
  if (result.length < 3) {
    // Extract first few words that appear in all filenames
    // Don't filter out numeric words - they might be part of the series name (e.g., "86" aka top 10 anime of all time. fight me.)
    const firstFileWords = cleanedNames[0].split(/\s+/).filter(w => w.length > 0);
    const commonWords: string[] = [];

    for (let i = 0; i < firstFileWords.length; i++) {
      const word = firstFileWords[i];

      if (cleanedNames.every(name => {
        const words = name.split(/\s+/).filter(w => w.length > 0);
        // Check if word exists at position i before comparing
        return i < words.length && words[i] === word;
      })) {
        commonWords.push(word);
      } else {
        break;
      }
    }

    result = commonWords.join(' ').trim();
  }

  // Fallback: if still empty, use the shortest cleaned name
  if (!result || result.length < 2) {
    result = cleanedNames[0];
  }

  // Final cleanup: remove any remaining trailing numbers or dashes
  result = result
    .replace(/\s*-\s*$/, '')
    .replace(/\s+\d+\s*$/, '')
    .trim();

  if (result.length <= 3 && /^\d+$/.test(result)) {
    return result;
  }

  return result;
}

function generateSeriesId(seriesName: string, folderName: string, seasonNumber: number | null, partNumber: number | null): string {
  let baseId: string;
  
  if (seriesName && seriesName.length >= 2) {
    baseId = seriesName
      .replace(/[^a-zA-Z0-9\s]/g, '')
      .replace(/\s+/g, '_')
      .toLowerCase()
      .substring(0, 50);
  } else {
    // Fallback to folder name if series name is too short or empty
    baseId = folderName.replace(/[^a-zA-Z0-9]/g, '_').toLowerCase();
  }
  
  if (partNumber !== null) {
    return `${baseId}_part${partNumber.toString().padStart(2, '0')}`;
  }
  
  if (seasonNumber !== null) {
    return `${baseId}_s${seasonNumber.toString().padStart(2, '0')}`;
  }
  
  return baseId;
}

function cleanMovieTitle(filename: string): string {
  // Clean movie filename for metadata search
  const baseName = getBaseName(filename);
  return baseName
    .replace(/\s*\(.*?\)\s*/g, '')     // Remove (2020), etc
    .replace(/\s*\[.*?\]\s*/g, '')     // Remove [1080p], etc
    .replace(/\.\d{4}\./g, ' ')        // Remove .2018.
    .replace(/\./g, ' ')               // Replace dots with spaces
    .replace(/_/g, ' ')                // Replace underscores with spaces
    .replace(/\s+/g, ' ')              // Normalize whitespace
    .trim();
}

async function scanFolderForVideos(folderPath: string, folderSeason: number | null = null): Promise<{ videos: VideoFile[], subtitles: Map<string, string[]> }> {
  const videos: VideoFile[] = [];
  const subtitles = new Map<string, string[]>();
  const folderName = basename(folderPath);

  // Extract season from folder name if not provided
  const seasonFromFolder = folderSeason ?? extractSeasonNumber(folderName);

  try {
    const entries = await readdir(folderPath);

    for (const entry of entries) {
      const fullPath = join(folderPath, entry);

      try {
        const stats = await stat(fullPath);

        if (stats.isFile()) {
          if (isVideoFile(entry)) {
            const { season, episode } = extractSeasonAndEpisode(entry);
            // Use season from filename if available, otherwise from folder
            const finalSeason = season ?? seasonFromFolder;

            videos.push({
              filename: entry,
              filePath: fullPath,
              title: getBaseName(entry),
              episodeNumber: episode,
              seasonNumber: finalSeason,
              subtitlePath: null,
              subtitlePaths: [],
              parentFolder: folderName,
              status: 'ready',
            });
          } else if (isSubtitleFile(entry)) {
            const baseName = getBaseName(entry);
            const existing = subtitles.get(baseName) || [];
            existing.push(fullPath);
            subtitles.set(baseName, existing);
          }
        }
      } catch (err) {
        // Skip files we can't stat
        logger.warn('folder', `Could not stat: ${fullPath}`, { file: fullPath });
      }
    }

    // Match subtitles to videos
    for (const video of videos) {
      const videoBase = getBaseName(video.filename);
      const matchingSubs = subtitles.get(videoBase) || [];
      if (matchingSubs.length > 0) {
        video.subtitlePath = matchingSubs[0];
        video.subtitlePaths = matchingSubs;
      }
    }

    // Deduplicate videos by file path (in case of duplicate entries)
    const seenPaths = new Set<string>();
    const uniqueVideos = videos.filter(video => {
      if (seenPaths.has(video.filePath)) {
        logger.warn('folder', `Duplicate video file detected: ${video.filePath}`, { file: video.filePath });
        return false;
      }
      seenPaths.add(video.filePath);
      return true;
    });

    return { videos: uniqueVideos, subtitles };
  } catch (error) {
    logger.error('folder', `Error scanning folder ${folderPath}`, { file: folderPath });
  }

  return { videos: [], subtitles };
}

async function scanDirectory(rootPath: string): Promise<ScannedMedia[]> {
  const results: ScannedMedia[] = [];

  logger.info('folder', `Scanning: ${rootPath}`, { file: rootPath });

  try {
    const entries = await readdir(rootPath);

    for (const entry of entries) {
      const entryPath = join(rootPath, entry);

      try {
        const stats = await stat(entryPath);

        if (stats.isDirectory()) {
          // This is a subfolder - could be a category (Movies, Series) or a series folder
          const { videos: subVideos } = await scanFolderForVideos(entryPath);

          // Check for nested subfolders (like Series/ShowName/)
          const subEntries = await readdir(entryPath);
          const subDirs: string[] = [];

          for (const subEntry of subEntries) {
            const subPath = join(entryPath, subEntry);
            try {
              const subStats = await stat(subPath);
              if (subStats.isDirectory()) {
                subDirs.push(subEntry);
              }
            } catch {
              // Skip
            }
          }

          if (subDirs.length > 0 && subVideos.length === 0) {
            // This is a CATEGORY folder (like "Series") containing series subfolders
            logger.info('folder', `Category folder: ${entry}`);

            for (const subDir of subDirs) {
              const seriesPath = join(entryPath, subDir);
              const seasonFromFolder = extractSeasonNumber(subDir);
              const partFromFolder = extractPartNumber(subDir);
              const { videos } = await scanFolderForVideos(seriesPath, seasonFromFolder);

              if (videos.length > 0) {
                const seriesName = extractSeriesNameFromFilenames(videos);
                const seriesId = generateSeriesId(seriesName, subDir, seasonFromFolder, partFromFolder);

                const partInfo = partFromFolder ? `, Part ${partFromFolder}` : '';
                const seasonInfo = seasonFromFolder ? `, Season ${seasonFromFolder}` : '';
                logger.info('folder', `Series: ${subDir} (${videos.length} episodes${seasonInfo}${partInfo}) → search: "${seriesName}" → ID: "${seriesId}"`, { series: seriesName });

                results.push({
                  id: seriesId,
                  name: seriesName,
                  type: 'series',
                  folderPath: seriesPath,
                  files: videos.sort((a, b) => {
                    // Sort by season first, then episode
                    const seasonA = a.seasonNumber ?? 0;
                    const seasonB = b.seasonNumber ?? 0;
                    if (seasonA !== seasonB) return seasonA - seasonB;
                    return a.episodeNumber - b.episodeNumber;
                  }),
                  seasonNumber: seasonFromFolder,
                  partNumber: partFromFolder,
                });
              }
            }

            // Also check for loose video files in category (like Movies/movie.mp4)
            if (subVideos.length > 0) {
              logger.info('folder', `Loose videos in ${entry}: ${subVideos.length}`);

              for (const video of subVideos) {
                const movieTitle = cleanMovieTitle(video.filename);
                const movieId = movieTitle.replace(/[^a-zA-Z0-9]/g, '_').toLowerCase();

                logger.info('folder', `Movie: ${video.filename} → search: "${movieTitle}"`, { series: movieTitle });

                results.push({
                  id: `movie_${movieId}`,
                  name: movieTitle,
                  type: 'movie',
                  folderPath: entryPath,
                  files: [video],
                  seasonNumber: null,
                  partNumber: null,
                });
              }
            }
          } else if (subVideos.length > 0) {
            // This folder directly contains videos - treat as series (folder name is series name)
            // Don't assume single video = movie - user may just have 1 episode downloaded
            const seriesName = extractSeriesNameFromFilenames(subVideos);
            const seasonFromFolder = extractSeasonNumber(entry);
            const partFromFolder = extractPartNumber(entry);
            const seriesId = generateSeriesId(seriesName, entry, seasonFromFolder, partFromFolder);

            const partInfo = partFromFolder ? `, Part ${partFromFolder}` : '';
            const seasonInfo = seasonFromFolder ? `, Season ${seasonFromFolder}` : '';
            logger.info('folder', `Series: ${entry} (${subVideos.length} episode${subVideos.length > 1 ? 's' : ''}${seasonInfo}${partInfo}) → search: "${seriesName}" → ID: "${seriesId}"`, { series: seriesName });

            results.push({
              id: seriesId,
              name: seriesName,
              type: 'series',
              folderPath: entryPath,
              files: subVideos.sort((a, b) => {
                // Sort by season first, then episode
                const seasonA = a.seasonNumber ?? 0;
                const seasonB = b.seasonNumber ?? 0;
                if (seasonA !== seasonB) return seasonA - seasonB;
                return a.episodeNumber - b.episodeNumber;
              }),
              seasonNumber: seasonFromFolder,
              partNumber: partFromFolder,
            });
          }
        } else if (stats.isFile() && isVideoFile(entry)) {
          // Video file directly in root - treat as standalone movie
          const movieTitle = cleanMovieTitle(entry);
          const movieId = movieTitle.replace(/[^a-zA-Z0-9]/g, '_').toLowerCase();

          logger.info('folder', `Movie (root): ${entry} → search: "${movieTitle}"`, { series: movieTitle });

          const { episode: movieEpisode } = extractSeasonAndEpisode(entry);
          results.push({
            id: `movie_${movieId}`,
            name: movieTitle,
            type: 'movie',
            folderPath: rootPath,
            files: [{
              filename: entry,
              filePath: entryPath,
              title: getBaseName(entry),
              episodeNumber: movieEpisode,
              seasonNumber: null,
              subtitlePath: null,
              subtitlePaths: [],
              parentFolder: basename(rootPath),
              status: 'ready',
            }],
            seasonNumber: null,
            partNumber: null,
          });
        }
      } catch (err) {
        logger.warn('folder', `Could not process ${entryPath}`, { file: entryPath });
      }
    }
  } catch (error) {
    logger.error('folder', `Error scanning root directory ${rootPath}`, { file: rootPath });
    throw error;
  }

  logger.info('folder', `Found ${results.length} media items in ${rootPath}`, { file: rootPath });
  return results;
}

interface FileEpisodeEntry {
  filePath: string;
  episodeNumber?: number;
  seasonNumber?: number | null;
  subtitlePath?: string | null;
  subtitlePaths?: string[];
  filename?: string;
  title?: string;
  status?: FileStatus;
  lastProbedAt?: number;
}

interface SeriesEntry {
  fileEpisodes?: FileEpisodeEntry[];
  poster?: string | null;
  banner?: string | null;
  posterLocal?: string | null;
  bannerLocal?: string | null;
  episodes?: Array<{ thumbnail?: string | null; thumbnailLocal?: string | null }>;
  [k: string]: unknown;
}

/**
 * Drop file entries whose absolute path is gone from disk OR not under
 * any active library root. If a series ends up with zero files, drop the
 * whole series and purge its cached posters/banners and episode thumbnails.
 *
 * @param activeRoots - Optional list of currently-active library root paths.
 *                     If omitted (`undefined`), the root-reachability check is
 *                     skipped — only disk presence matters. Pass `[]` explicitly
 *                     to mean "no roots configured" — current semantics treat
 *                     that the same as `undefined` (preserve everything that's
 *                     still on disk). Path comparison is Linux-only by design;
 *                     no normalization or symlink resolution is performed.
 * @returns the reconciled metadata object (may share references with the input).
 *
 * Synchronous `existsSync` is used on purpose — typical libraries have <10k
 * files and reconcile runs at most once per scan or unlink-burst, so the
 * sync stat is cheaper than the async overhead.
 */
async function reconcileMetadata(
  metadata: Record<string, unknown>,
  activeRoots?: string[],
): Promise<Record<string, unknown>> {
  const out: Record<string, unknown> = {};
  let removedFiles = 0;
  let removedSeries = 0;

  const isUnderActiveRoot = (filePath: string): boolean => {
    // No roots configured (or none passed) → skip the reachability check.
    if (!activeRoots || activeRoots.length === 0) return true;
    return activeRoots.some(
      (root) => filePath === root || filePath.startsWith(root.endsWith('/') ? root : root + '/'),
    );
  };

  for (const [seriesId, raw] of Object.entries(metadata)) {
    const series = raw as SeriesEntry;
    const files = Array.isArray(series.fileEpisodes) ? series.fileEpisodes : [];
    const kept = files.filter((f) => {
      if (!f?.filePath) return false;
      const reachable = isUnderActiveRoot(f.filePath);
      const present = reachable && existsSync(f.filePath);
      if (!present) removedFiles++;
      return present;
    });

    if (kept.length === 0 && files.length > 0) {
      removedSeries++;
      logger.info('folder', `Reconcile: dropping series`, { series: String(series.title ?? seriesId) });
      try {
        await imageCacheHandler.deleteSeriesImages({
          poster: series.poster ?? null,
          banner: series.banner ?? null,
          posterLocal: series.posterLocal ?? null,
          bannerLocal: series.bannerLocal ?? null,
          episodes: series.episodes ?? [],
        });
      } catch (err) {
        logger.warn('image', `Reconcile: image cleanup failed: ${(err as Error).message}`, { series: String(series.title ?? seriesId) });
      }
      try {
        await thumbnailHandler.deleteSeriesThumbnails(files.map((f) => f.filePath).filter(Boolean) as string[]);
      } catch (err) {
        logger.warn('thumbnail', `Reconcile: thumbnail cleanup failed: ${(err as Error).message}`, { series: String(series.title ?? seriesId) });
      }
      continue;
    }

    if (kept.length !== files.length) {
      logger.info('folder', `Reconcile: dropped ${files.length - kept.length} file(s)`, { series: String(series.title ?? seriesId) });
      out[seriesId] = { ...series, fileEpisodes: kept };
    } else {
      out[seriesId] = series;
    }
  }

  if (removedFiles || removedSeries) {
    logger.info('folder', `Reconcile complete: ${removedFiles} file(s), ${removedSeries} series removed`);
  }
  return out;
}

const folderHandler = {
  async scanFolder(folderPath: string): Promise<ScannedMedia[]> {
    if (!folderPath) {
      throw new Error('Folder path is required');
    }

    // Validate path exists and is accessible
    try {
      const stats = await stat(folderPath);
      if (!stats.isDirectory()) {
        throw new Error(`Path is not a directory: ${folderPath}`);
      }
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
        throw new Error(`Folder does not exist: ${folderPath}`);
      }
      throw error;
    }

    return await scanDirectory(folderPath);
  },

  async scanMultipleFolders(folderPaths: string[]): Promise<ScannedMedia[]> {
    const allResults: ScannedMedia[] = [];

    for (const folderPath of folderPaths) {
      try {
        const results = await scanDirectory(folderPath);
        allResults.push(...results);
      } catch (error) {
        logger.error('folder', `Error scanning ${folderPath}`, { file: folderPath });
      }
    }

    return allResults;
  },

  reconcileMetadata,
};

export default folderHandler;
