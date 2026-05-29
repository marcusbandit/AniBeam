import { readdir, stat } from 'fs/promises';
import { existsSync } from 'fs';
import { join, extname, basename } from 'path';
import { logger } from '../services/logger';
import imageCacheHandler from './imageCacheHandler';
import thumbnailHandler from './thumbnailHandler';
import type { FileStatus } from '../../shared/fileStatus';
import { classifyFile, type EpisodeKind } from '../../shared/episodeClassifier';

const VIDEO_EXTENSIONS = ['.mp4', '.mkv', '.avi', '.mov', '.wmv', '.flv', '.webm', '.m4v', '.ts'];
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
  // Filesystem mtime in ms since epoch. Used by the feed as a fallback
  // "downloaded X ago" when API air dates aren't available.
  mtime: number;
  // Classifier output. Distinguishes real episodes from OP/ED/PV/SP/extras.
  // Consumers that count or list "real" episodes must filter on
  // kind === 'episode' (not just episodeNumber, which holds the extras index
  // for non-episode kinds so within-kind sorting works).
  kind: EpisodeKind;
  extraIndex: number | null;
  extraVariant: string | null;
  rawLabel: string | null;
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

/**
 * Single source of truth for filename cleaning. EVERY function that parses
 * episode/season numbers, series names, or display titles MUST run a filename
 * through this first. Strips:
 *   - the file extension
 *   - anything inside [square brackets] — release groups, quality, CRC32 hashes
 *     like [F1E24928] that would otherwise match /E(\d{2,})/
 *   - leftover separator noise and collapsed whitespace
 */
function stripFilename(filename: string): string {
  return basename(filename, extname(filename))
    .replace(/\s*\[[^\]]*\]\s*/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

function cleanEpisodeTitle(filename: string): string {
  return stripFilename(filename).replace(/\s*[-_]\s*$/g, '').trim();
}

// Episode/season extraction is now handled by classifyFile() in
// src/shared/episodeClassifier.ts. That module also distinguishes real
// episodes from OP/ED/PV/SP/extras tokens, so the old digit-fallback no
// longer collapses every "<show>_ED1_.mkv" onto episode 1.

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

// Series name = folder name, verbatim. We deliberately do NOT strip
// "Season N" / "Part N" / trailing year — folder strings go to MAL and
// AniList unchanged. extractSeasonNumber / extractPartNumber pull season
// info out separately when present. Cleaning still happens at the file
// level (stripFilename, cleanEpisodeTitle).
function extractSeriesNameFromFolder(folderName: string): string {
  return folderName.trim();
}

// Strip release-group brackets and episode-range / END suffixes from a
// folder NAME (not a filename — no extension). Used when deriving a series
// name for a wrapper subfolder, where the user wants a clean canonical
// title rather than the raw release-tagged folder name.
function cleanFolderTitle(name: string): string {
  return name
    .replace(/\s*\[[^\]]*\]\s*/g, ' ')                         // [Erai-raws], [1080p], CRC tags
    .replace(/\s+-\s+\d+\s*[~–-]\s*\d+(\s+END)?\s*$/i, '') // " - 01 ~ 12" / " - 01-12 END"
    .replace(/\s+-?\s*END\s*$/i, '')                           // trailing END
    .replace(/\s+/g, ' ')
    .trim();
}

// When recursing into a "franchise wrapper" subfolder, derive a clean series
// name anchored on the WRAPPER folder name (which the user named themselves
// and is treated as canonical). The cleaned subfolder name is searched for
// the wrapper name; whatever follows is kept as a suffix. A pure trailing
// digit ("Karakai... 2") is also returned as a season hint.
function deriveSubfolderSeriesName(
  subfolderName: string,
  wrapperName: string,
): { name: string; seasonHint: number | null } {
  const cleanedSub = cleanFolderTitle(subfolderName);
  const cleanedWrapper = wrapperName.trim();
  const idx = cleanedSub.toLowerCase().indexOf(cleanedWrapper.toLowerCase());
  if (idx >= 0) {
    const suffix = cleanedSub.slice(idx + cleanedWrapper.length).trim();
    if (!suffix) return { name: cleanedWrapper, seasonHint: null };
    const numMatch = suffix.match(/^(\d+)$/);
    if (numMatch) {
      const n = parseInt(numMatch[1], 10);
      return { name: `${cleanedWrapper} ${n}`, seasonHint: n };
    }
    return { name: `${cleanedWrapper} ${suffix}`, seasonHint: null };
  }
  // Wrapper name not found in cleaned subfolder — fall back to whatever
  // cleanup produced, or the wrapper name itself if cleanup left nothing.
  return { name: cleanedSub || cleanedWrapper, seasonHint: null };
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
  // Brackets are already stripped by stripFilename — handle parens and dots here.
  return stripFilename(filename)
    .replace(/\s*\(.*?\)\s*/g, '')     // Remove (2020), etc
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
            const classified = classifyFile(entry);
            // Use season from filename if classifier found one, else from folder.
            const finalSeason = classified.seasonNumber ?? seasonFromFolder;

            videos.push({
              filename: entry,
              filePath: fullPath,
              title: cleanEpisodeTitle(entry),
              episodeNumber: classified.episodeNumber,
              seasonNumber: finalSeason,
              subtitlePath: null,
              subtitlePaths: [],
              parentFolder: folderName,
              status: 'ready',
              mtime: stats.mtimeMs,
              kind: classified.kind,
              extraIndex: classified.extraIndex,
              extraVariant: classified.extraVariant,
              rawLabel: classified.rawLabel,
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

// A folder named "Movies" (case-insensitive) is a movie collection — never a
// series. Its videos are individual movies, and so are its direct subfolders
// (each subfolder is one movie even if it bundles a single video + subs).
function isMoviesFolderName(name: string): boolean {
  return name.toLowerCase() === 'movies';
}

// Emit one movie ScannedMedia for a single file. Caller decides whether to
// derive the title from the folder name (cleaner when files are buried under
// release-tag-laden filenames) or from the filename itself.
async function emitMovieFile(
  fileName: string,
  containingFolder: string,
  useFolderNameAsTitle: boolean,
  results: ScannedMedia[],
): Promise<void> {
  const filePath = join(containingFolder, fileName);
  const folderName = basename(containingFolder);
  const movieTitle = useFolderNameAsTitle ? cleanMovieTitle(folderName) : cleanMovieTitle(fileName);
  const movieId = movieTitle.replace(/[^a-zA-Z0-9]/g, '_').toLowerCase();
  const classified = classifyFile(fileName);
  let mtime = 0;
  try {
    mtime = (await stat(filePath)).mtimeMs;
  } catch { /* best-effort; mtime stays 0 → ignored by feed fallback */ }

  results.push({
    id: `movie_${movieId}`,
    name: movieTitle,
    type: 'movie',
    folderPath: containingFolder,
    files: [{
      filename: fileName,
      filePath,
      title: cleanEpisodeTitle(fileName),
      episodeNumber: classified.episodeNumber,
      seasonNumber: null,
      subtitlePath: null,
      subtitlePaths: [],
      parentFolder: folderName,
      status: 'ready',
      mtime,
      kind: classified.kind,
      extraIndex: classified.extraIndex,
      extraVariant: classified.extraVariant,
      rawLabel: classified.rawLabel,
    }],
    seasonNumber: null,
    partNumber: null,
  });
}

// Walk a subtree and gather every video file. Used by series folders that
// nest their episodes one or more levels deep — e.g.
// "Series/[release group]/ep01.mkv" or "Series/Season 1/ep01.mkv". The whole
// subtree is treated as one logical series and the outer folder's name wins.
// Per-folder season hints (a "Season 2" subfolder) propagate to the videos
// inside that branch via scanFolderForVideos's `folderSeason`.
async function collectVideosInSubtree(
  rootPath: string,
  inheritedSeason: number | null,
): Promise<VideoFile[]> {
  const out: VideoFile[] = [];
  const visit = async (dir: string, parentSeason: number | null): Promise<void> => {
    const seasonHere = extractSeasonNumber(basename(dir)) ?? parentSeason;
    const { videos } = await scanFolderForVideos(dir, seasonHere);
    out.push(...videos);
    let entries: string[];
    try {
      entries = await readdir(dir);
    } catch { return; }
    for (const entry of entries) {
      const sub = join(dir, entry);
      try {
        const s = await stat(sub);
        if (s.isDirectory()) await visit(sub, seasonHere);
      } catch { /* skip unreadable */ }
    }
  };
  await visit(rootPath, inheritedSeason);
  return out;
}

// Shallow video-presence check: true if `dir` contains 1+ video files
// directly OR has 1+ child subfolders that each contain 1+ video files
// directly. Used by classifyFolder to detect "franchise wrapper" folders
// where each subfolder is its own logical series. One level of nesting
// covers the common `[release-group]/ep01.mkv` wrapping convention without
// triggering a full subtree walk.
async function hasVideosShallow(dir: string): Promise<boolean> {
  let entries: string[];
  try {
    entries = await readdir(dir);
  } catch {
    return false;
  }
  const childDirs: string[] = [];
  for (const entry of entries) {
    const fullPath = join(dir, entry);
    try {
      const s = await stat(fullPath);
      if (s.isFile() && isVideoFile(entry)) return true;
      if (s.isDirectory()) childDirs.push(fullPath);
    } catch { /* skip unreadable */ }
  }
  for (const child of childDirs) {
    let childEntries: string[];
    try {
      childEntries = await readdir(child);
    } catch { continue; }
    for (const e of childEntries) {
      try {
        const s = await stat(join(child, e));
        if (s.isFile() && isVideoFile(e)) return true;
      } catch { /* skip unreadable */ }
    }
  }
  return false;
}

// Decide whether a non-root, non-Movies folder is a single series ('series')
// or a "franchise wrapper" ('wrapper') that holds multiple distinct shows
// (and possibly a loose movie file) — e.g. an "Anime/Show Title/" folder
// containing "Show S1/", "Show S2/", "Show S3/", "Show - Movie.mkv".
//
// Rule (shallow, no full subtree walk):
//   - 2+ video-bearing subfolders                            → wrapper
//   - 1+ video-bearing subfolder AND 1+ loose video at top   → wrapper
//   - otherwise                                              → series
//
// Returns the wrapper's video-bearing subfolders so the caller can skip
// non-video subdirs (`screenshots/`, empty `Extras/`) cleanly.
async function classifyFolder(
  looseVideoCount: number,
  subDirs: string[],
  folderPath: string,
): Promise<{ kind: 'series' } | { kind: 'wrapper'; videoBearingSubs: string[] }> {
  const videoBearing: string[] = [];
  for (const sub of subDirs) {
    if (await hasVideosShallow(join(folderPath, sub))) {
      videoBearing.push(sub);
    }
  }
  if (videoBearing.length >= 2) return { kind: 'wrapper', videoBearingSubs: videoBearing };
  if (videoBearing.length >= 1 && looseVideoCount >= 1) {
    return { kind: 'wrapper', videoBearingSubs: videoBearing };
  }
  return { kind: 'series' };
}

// Walk a folder tree and emit ScannedMedia.
//
// Three contexts produce different shapes:
//
//   1. Library root: not a series itself. Loose video files at the root are
//      treated as individual movies. Subfolders recurse normally.
//
//   2. Movies container (a folder named "Movies", or anything beneath one):
//      every video file is a separate movie entry. Subfolders that hold one
//      movie inherit their folder name as the title (cleaner than filenames
//      laden with release tags). No series are ever produced here.
//
//   3. Anything else (non-root, non-Movies): classifyFolder decides whether
//      this is a single series (whose subtree is collapsed into one entry,
//      with intermediate folders treated transparently) or a "franchise
//      wrapper" — multiple distinct shows under one parent folder, where
//      each subfolder becomes its own series and loose top-level videos
//      become individual movies.
async function collectMediaRecursive(
  folderPath: string,
  results: ScannedMedia[],
  isLibraryRoot: boolean,
  inMoviesContext: boolean = false,
  // Only set when this folder is a video-bearing subdir of a franchise
  // wrapper. Carries the canonical series name + optional season hint
  // derived from the wrapper's folder name. Consumed exactly once, in the
  // single-series branch — never propagated deeper.
  wrapperContext?: { name: string; seasonHint: number | null },
): Promise<void> {
  const folderName = basename(folderPath);
  const isMoviesContainer = isMoviesFolderName(folderName);
  const moviesContext = inMoviesContext || isMoviesContainer;

  let entries: string[];
  try {
    entries = await readdir(folderPath);
  } catch {
    logger.warn('folder', `Could not read ${folderPath}`, { file: folderPath });
    return;
  }

  const subDirs: string[] = [];
  const videoFilenames: string[] = [];

  for (const entry of entries) {
    const fullPath = join(folderPath, entry);
    try {
      const stats = await stat(fullPath);
      if (stats.isDirectory()) {
        subDirs.push(entry);
      } else if (stats.isFile() && isVideoFile(entry)) {
        videoFilenames.push(entry);
      }
    } catch {
      logger.warn('folder', `Could not stat ${fullPath}`, { file: fullPath });
    }
  }

  // 1. Library root: loose videos are loose movies; subfolders recurse.
  if (isLibraryRoot) {
    for (const entry of videoFilenames) {
      await emitMovieFile(entry, folderPath, false, results);
    }
    for (const subDir of subDirs) {
      await collectMediaRecursive(join(folderPath, subDir), results, false, moviesContext);
    }
    return;
  }

  // 2. Movies context: each video → its own movie; each subdir recurses with
  //    the same context so nested movie folders still produce movies.
  if (moviesContext) {
    if (videoFilenames.length > 0) {
      // Folder name is the cleaner title when the folder holds exactly one
      // video and isn't the "Movies" container itself.
      const useFolderTitle =
        !isMoviesContainer && videoFilenames.length === 1 && subDirs.length === 0;
      for (const entry of videoFilenames) {
        await emitMovieFile(entry, folderPath, useFolderTitle, results);
      }
    }
    for (const subDir of subDirs) {
      await collectMediaRecursive(join(folderPath, subDir), results, false, true);
    }
    return;
  }

  // 3. Series or franchise wrapper — see classifyFolder.
  const classification = await classifyFolder(videoFilenames.length, subDirs, folderPath);

  if (classification.kind === 'wrapper') {
    // Loose videos at the wrapper level → individual movies. Use the
    // cleaned filename for the title; the wrapper folder isn't the movie.
    for (const v of videoFilenames) {
      await emitMovieFile(v, folderPath, false, results);
    }
    // Each video-bearing subdir gets its own series scan, in a fresh
    // non-Movies series context. Non-video-bearing subdirs are skipped.
    // Pass a wrapper context so the child series uses the user-canonical
    // wrapper name (+ trailing digit as season hint) instead of the raw
    // release-tagged subfolder name.
    for (const subDir of classification.videoBearingSubs) {
      const derived = deriveSubfolderSeriesName(subDir, folderName);
      await collectMediaRecursive(
        join(folderPath, subDir), results, false, false, derived,
      );
    }
    return;
  }

  // Single-series case: walk the whole subtree as one series. Outer folder
  // name wins; intermediate folders are transparent except for season hints.
  // wrapperContext (set only when this is the immediate child of a franchise
  // wrapper) overrides the verbatim folder name and contributes a season hint.
  const seasonFromFolder =
    extractSeasonNumber(folderName) ?? wrapperContext?.seasonHint ?? null;
  const partFromFolder = extractPartNumber(folderName);
  const allVideos = await collectVideosInSubtree(folderPath, seasonFromFolder);
  if (allVideos.length === 0) return;

  const seriesName = wrapperContext?.name ?? extractSeriesNameFromFolder(folderName);
  const seriesId = generateSeriesId(seriesName, folderName, seasonFromFolder, partFromFolder);

  results.push({
    id: seriesId,
    name: seriesName,
    type: 'series',
    folderPath,
    files: allVideos.sort((a, b) => {
      const seasonA = a.seasonNumber ?? 0;
      const seasonB = b.seasonNumber ?? 0;
      if (seasonA !== seasonB) return seasonA - seasonB;
      return a.episodeNumber - b.episodeNumber;
    }),
    seasonNumber: seasonFromFolder,
    partNumber: partFromFolder,
  });
}

async function scanDirectory(rootPath: string): Promise<ScannedMedia[]> {
  const results: ScannedMedia[] = [];

  try {
    await collectMediaRecursive(rootPath, results, true);
  } catch (error) {
    logger.error('folder', `Error scanning root directory ${rootPath}`, { file: rootPath });
    throw error;
  }

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
  /**
   * Scan a folder for media. `folderPath` may be either a configured library
   * root (full library scan) or a sub-folder beneath one (rescan a single
   * show). When called with a sub-folder, `activeRoots` MUST be passed so we
   * can locate the containing root and scan from there — otherwise scanDirectory
   * would treat the series folder as a root and misclassify every episode
   * directly inside it as a "movie at root."
   */
  async scanFolder(folderPath: string, activeRoots?: string[]): Promise<ScannedMedia[]> {
    if (!folderPath) {
      throw new Error('Folder path is required');
    }

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

    const normalize = (p: string) => (p.endsWith('/') ? p.slice(0, -1) : p);
    const normalizedFolder = normalize(folderPath);

    // No roots provided, or the path itself is one of the roots → scan directly.
    const isItselfARoot = activeRoots?.some((r) => normalize(r) === normalizedFolder);
    if (!activeRoots || activeRoots.length === 0 || isItselfARoot) {
      return await scanDirectory(folderPath);
    }

    // Sub-folder of a library root → scan from the root and filter to results
    // whose folderPath is the requested folder OR a descendant of it.
    const matchingRoot = activeRoots
      .filter((root) => {
        const nr = normalize(root);
        return normalizedFolder === nr || normalizedFolder.startsWith(nr + '/');
      })
      .sort((a, b) => b.length - a.length)[0];
    if (!matchingRoot) {
      logger.warn('folder', `${folderPath} is not under any active library root — falling back to direct scan`);
      return await scanDirectory(folderPath);
    }
    const all = await scanDirectory(matchingRoot);
    const prefix = normalizedFolder + '/';
    return all.filter((media) => {
      const mp = normalize(media.folderPath);
      return mp === normalizedFolder || mp.startsWith(prefix);
    });
  },

  /**
   * Walk a library root and return paths of every folder named "Movies"
   * (case-insensitive). Used by Settings to list detected movie containers
   * under each library root. Skips dotfiles.
   */
  async findMovieFolders(rootPath: string): Promise<string[]> {
    const found: string[] = [];

    async function walk(dir: string): Promise<void> {
      let entries: string[];
      try {
        entries = await readdir(dir);
      } catch {
        return;
      }
      for (const entry of entries) {
        if (entry.startsWith('.')) continue;
        const fullPath = join(dir, entry);
        let isDir = false;
        try {
          isDir = (await stat(fullPath)).isDirectory();
        } catch {
          continue;
        }
        if (!isDir) continue;
        if (isMoviesFolderName(entry)) {
          found.push(fullPath);
          // Don't recurse into a Movies folder — its subfolders are individual
          // movies, not nested Movies containers.
          continue;
        }
        await walk(fullPath);
      }
    }

    try {
      const stats = await stat(rootPath);
      if (!stats.isDirectory()) return [];
    } catch {
      return [];
    }
    await walk(rootPath);
    return found;
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

  /**
   * Scan just enough of the library to classify a single file the same way a
   * full scan would. Must be passed the active library roots — without them,
   * we'd treat the file's parent as a library root and misclassify every file
   * directly inside a series folder as a "movie at root".
   */
  async scanSingleFile(
    filePath: string,
    activeRoots: string[],
  ): Promise<{ media: ScannedMedia; file: VideoFile } | null> {
    if (!existsSync(filePath)) return null;
    // Find the library root that contains this file (longest matching prefix
    // wins, in case roots are nested).
    const matchingRoot = activeRoots
      .filter((root) => filePath === root || filePath.startsWith(root.endsWith('/') ? root : root + '/'))
      .sort((a, b) => b.length - a.length)[0];
    if (!matchingRoot) {
      logger.warn('folder', `File is not under any active library root`, { file: filePath });
      return null;
    }
    const directoryResults = await scanDirectory(matchingRoot);
    for (const media of directoryResults) {
      const file = media.files.find((f) => f.filePath === filePath);
      if (file) {
        file.status = 'verifying';
        return { media, file };
      }
    }
    return null;
  },

  reconcileMetadata,
};

export default folderHandler;
