import { app, BrowserWindow, ipcMain, Menu, protocol, net } from 'electron';
import { fileURLToPath, pathToFileURL } from 'url';
import { dirname, join, isAbsolute, resolve, relative } from 'path';
import { existsSync } from 'fs';
import axios from 'axios';
import folderHandler, { type ScannedMedia } from './handlers/folderHandler';
import anilistHandler from './handlers/anilistHandler';
import malHandler from './handlers/malHandler';
import metadataHandler from './handlers/metadataHandler';
import configHandler from './handlers/configHandler';
import imageCacheHandler from './handlers/imageCacheHandler';
import thumbnailHandler from './handlers/thumbnailHandler';
import { initMediaProgress, updateMediaProgress } from './utils/debugUtils';
import { findBestMatch } from './utils/metadataMatcher';
import { findShowMatch, fetchEpisodeAirDates } from './utils/posterMatch';
import { logger } from './services/logger';
import type { FileStatus } from '../shared/fileStatus';
import { findFileEpisode, type FileEpisodeEntry } from '../shared/fileEpisode';
import videoProbeHandler from './handlers/videoProbeHandler';
import transcodeCacheHandler from './handlers/transcodeCacheHandler';
import { fileWatcher } from './services/watcher';
import { getFranchiseGraph, getFranchiseCrawlProgress } from './services/franchiseGraph';
// IPC modules — each registers its own handlers at app-ready time.
import { registerLogIpc } from './ipc/log';
import { registerConfigIpc } from './ipc/config';
import { registerFolderIpc } from './ipc/folder';
import { registerImageCacheIpc } from './ipc/imageCache';
import { registerMediaPlaybackIpc } from './ipc/mediaPlayback';
import { registerTrackerIpc } from './ipc/tracker';
import { registerShellIpc } from './ipc/shell';
import { registerSubscriptionsIpc } from './ipc/subscriptions';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Castlabs Electron ships Chromium with HEVC compiled in, but actually
// USING the HEVC decoder is gated on a feature flag (and so is platform
// HW video decode on Linux). Without these, an HEVC <video> stream
// reaches the element, decodes audio, but produces no frames — exactly
// the "audio plays, screen is black" symptom we hit on Steins;Gate.
// These must be set BEFORE app.whenReady(); Chromium reads them once
// at GPU process init.
app.commandLine.appendSwitch(
  'enable-features',
  [
    'PlatformHEVCDecoderSupport',
    // VAAPI / NVDEC hardware decode on Linux. Harmless on systems where
    // the platform decoder is unavailable — Chromium just falls back to
    // software decode (which the codec build above provides for HEVC).
    'VaapiVideoDecoder',
    'VaapiVideoDecodeLinuxGL',
    'VaapiIgnoreDriverChecks',
  ].join(','),
);
// Don't let Chromium's GPU blocklist veto hardware decode on this box
// — Linux NVIDIA in particular is conservatively blocklisted by default.
app.commandLine.appendSwitch('ignore-gpu-blocklist');

// When the app is launched from a .desktop entry, stdout/stderr aren't
// connected to anything — the first write that hits a closed pipe takes
// down the main process with EPIPE. Swallow EPIPE specifically so any
// stray console.log can't crash us; surface other errors normally.
for (const stream of [process.stdout, process.stderr]) {
  stream.on('error', (err: NodeJS.ErrnoException) => {
    if (err.code !== 'EPIPE') throw err;
  });
}

// Helper to check if error is a rate limit (already logged by handlers)
function isRateLimitError(error: unknown): boolean {
  if (axios.isAxiosError(error)) {
    return error.response?.status === 429;
  }
  if (error && typeof error === 'object') {
    const err = error as { response?: { status?: number }; statusCode?: number; message?: string };
    if (err.response?.status === 429 || err.statusCode === 429) {
      return true;
    }
    if (err.message && /rate.?limit/i.test(err.message)) {
      return true;
    }
  }
  return false;
}

function preserveStatus(
  seriesId: string,
  files: Array<{ filePath: string; status?: string; lastProbedAt?: number }>,
  existingMetadata: Record<string, unknown>,
): typeof files {
  const prior = existingMetadata[seriesId] as
    | { fileEpisodes?: Array<{ filePath: string; status?: string; lastProbedAt?: number }> }
    | undefined;
  if (!prior?.fileEpisodes) return files;
  const byPath = new Map(prior.fileEpisodes.map((f) => [f.filePath, f]));
  return files.map((f) => {
    const old = byPath.get(f.filePath);
    if (!old) return f;
    return { ...f, status: old.status ?? f.status, lastProbedAt: old.lastProbedAt ?? f.lastProbedAt };
  });
}

// Called when the videoProbeHandler finishes verifying a file (or when
// transcodeCacheHandler completes/changes a transcode). For files that
// just became 'ready', we additionally check whether the codec is one
// Chromium can't decode (HEVC etc.) and, if so, enqueue a pre-transcode.
// The renderer sees 'transcoding' status while the encode runs and
// 'ready' once a cached .mp4 is published.
async function maybeEnqueueTranscode(filePath: string): Promise<void> {
  try {
    // Skip files we've already cached. Saves a redundant ffprobe.
    const meta = await metadataHandler.loadMetadata();
    const hit = findFileEpisode(meta, filePath);
    if (hit?.transcodedPath && existsSync(hit.transcodedPath)) return;
    const needs = await transcodeCacheHandler.shouldTranscode(filePath);
    if (!needs) return;
    void transcodeCacheHandler.enqueue(filePath);
  } catch (err) {
    logger.warn('system', `maybeEnqueueTranscode failed: ${(err as Error).message}`, { file: filePath });
  }
}

async function updateFileStatus(filePath: string, status: FileStatus): Promise<void> {
  const touched = await metadataHandler.transaction<boolean>(async (meta) => {
    let changed = false;
    for (const series of Object.values(meta)) {
      const s = series as { fileEpisodes?: FileEpisodeEntry[] };
      if (!Array.isArray(s.fileEpisodes)) continue;
      for (const file of s.fileEpisodes) {
        if (file.filePath === filePath) {
          file.status = status;
          file.lastProbedAt = Date.now();
          changed = true;
        }
      }
    }
    return { result: changed, updated: changed ? meta : null };
  });
  if (touched && mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send('metadata:file-status-changed', { filePath, status });
  }
}

// Debounced auto-fetch. When a new series lands — a whole folder dropped in,
// or episodes copied one at a time — chokidar fires a burst of `add` events.
// Matching metadata off the FIRST file races the rest of the folder settling:
// the match can run against a half-populated folder or a still-renaming temp
// name, fail, and (because matchPosterForSeries stamps posterMatchAttempted)
// never get retried automatically — which is exactly why a manual refresh was
// needed. So we wait for the folder to go quiet: each new file for a
// not-yet-matched series re-arms a short timer, and the fetch only fires once
// no new file has arrived for AUTO_FETCH_SETTLE_MS. Already-matched series are
// never re-armed, so a weekly new episode doesn't re-fetch.
const AUTO_FETCH_SETTLE_MS = 4000;
const autoFetchTimers = new Map<string, NodeJS.Timeout>();
const autoFetchInFlight = new Set<string>();

function scheduleAutoFetch(seriesId: string, folderName: string): void {
  if (autoFetchInFlight.has(seriesId)) return;
  const prev = autoFetchTimers.get(seriesId);
  if (prev) clearTimeout(prev);
  autoFetchTimers.set(seriesId, setTimeout(() => {
    autoFetchTimers.delete(seriesId);
    autoFetchInFlight.add(seriesId);
    logger.info('watch', `Folder settled — fetching metadata for ${folderName}`, { series: folderName });
    void matchPosterForSeries(seriesId, folderName).finally(() => {
      autoFetchInFlight.delete(seriesId);
    });
  }, AUTO_FETCH_SETTLE_MS));
}

async function ingestSingleFile(filePath: string): Promise<void> {
  try {
    const activeRoots = await configHandler.getFolderSources();
    const result = await folderHandler.scanSingleFile(filePath, activeRoots);
    if (!result) {
      logger.warn('watch', `scanSingleFile returned null`, { file: filePath });
      return;
    }
    const { media } = result;

    // Filesystem-only ingest. Splice the new file into the series's
    // fileEpisodes (creating a minimal entry if the series is brand new).
    // No network, no thumbnails, no metadata fetch — metadata is paused.
    let everAttempted = false;
    await metadataHandler.transaction(async (current) => {
      const existing = (current[media.id] ?? {}) as {
        fileEpisodes?: FileEpisodeEntry[];
        title?: string;
        posterMatchAttempted?: boolean;
      };
      // A series that's never been match-attempted is either brand new or
      // still being populated (its deferred fetch hasn't fired yet) — both
      // should (re-)arm the settle timer below.
      everAttempted = !!existing.posterMatchAttempted;
      const byPath = new Map((existing.fileEpisodes ?? []).map((f) => [f.filePath, f]));
      const newFileEpisodes = media.files.map((f) => {
        const old = byPath.get(f.filePath);
        const isThisFile = f.filePath === filePath;
        // Spread the old entry FIRST so persistent fields the scanner doesn't
        // know about (transcodedPath, anything added later) survive an add
        // event for a sibling file. Then overlay scanner-derived fields and
        // re-stamp status/lastProbedAt — for the just-added file only.
        return {
          ...(old ?? {}),
          episodeNumber: f.episodeNumber,
          seasonNumber: f.seasonNumber,
          filePath: f.filePath,
          subtitlePath: f.subtitlePath,
          subtitlePaths: f.subtitlePaths,
          filename: f.filename,
          title: f.title,
          status: isThisFile ? 'verifying' : (old?.status ?? f.status),
          lastProbedAt: isThisFile ? Date.now() : (old?.lastProbedAt ?? f.lastProbedAt),
        };
      });
      current[media.id] = {
        ...existing,
        seriesId: media.id,
        title: existing.title ?? media.name,
        fileEpisodes: newFileEpisodes,
        folderPath: media.folderPath,
        type: media.type,
      };
      logger.info('watch', `Ingest: ${media.name} (${newFileEpisodes.length} files)`, { series: media.name, file: filePath });
      return { updated: current };
    });

    videoProbeHandler.enqueue(filePath);

    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:file-status-changed', { filePath, status: 'verifying' });
    }

    // Not yet matched → schedule a debounced metadata match once the folder
    // stops changing (see scheduleAutoFetch). media.name is the right search
    // input for both movies and series: for movies it's the cleaned
    // filename-derived title, for series it's the cleaned wrapper-derived or
    // folder name. The earlier code used basename(media.folderPath) for
    // series, which broke franchise-wrapper subfolders whose folder names are
    // release-tagged (e.g. "[Erai-raws] Karakai... - 01 ~ 12 [1080p]").
    if (!everAttempted) {
      scheduleAutoFetch(media.id, media.name);
    }
  } catch (err) {
    logger.error('watch', `Failed to ingest new file: ${(err as Error).message}`, { file: filePath });
  }
}

async function handleUnlink(filePath: string): Promise<void> {
  // Capture transcoded-cache path BEFORE the reconcile drops the entry,
  // so we can delete the cached .mp4 too. Otherwise it stays orphaned
  // forever (and the next "rip the same episode" would re-transcode to
  // a fresh hash and leak the old one).
  let orphanedCachePath: string | null = null;
  try {
    const meta = await metadataHandler.loadMetadata();
    orphanedCachePath = findFileEpisode(meta, filePath)?.transcodedPath ?? null;
  } catch { /* best-effort */ }

  const changed = await metadataHandler.transaction<boolean>(async (meta) => {
    const activeRoots = await configHandler.getFolderSources();
    const reconciled = await folderHandler.reconcileMetadata(meta, activeRoots);
    // reconcileMetadata always returns a new outer object, so reference
    // equality is unreliable. Compare keys + per-series file counts to detect
    // actual changes and avoid pointless writes.
    const sameShape =
      Object.keys(reconciled).length === Object.keys(meta).length &&
      Object.entries(reconciled).every(([id, value]) => {
        const before = meta[id] as { fileEpisodes?: unknown[] } | undefined;
        const after = value as { fileEpisodes?: unknown[] };
        return before && (before.fileEpisodes?.length ?? 0) === (after.fileEpisodes?.length ?? 0);
      });
    return { result: !sameShape, updated: sameShape ? null : reconciled };
  });
  if (changed && mainWindow && !mainWindow.isDestroyed()) {
    // Reuse the file-status-changed channel — renderers treat any payload as
    // "metadata changed, reload." Status field is a placeholder; the entry
    // for `filePath` is gone after a reconcile.
    mainWindow.webContents.send('metadata:file-status-changed', { filePath, status: 'ready' });
  }

  if (orphanedCachePath && existsSync(orphanedCachePath)) {
    try {
      const { unlink: fsUnlink } = await import('node:fs/promises');
      await fsUnlink(orphanedCachePath);
      logger.info('system', `Removed cached transcode for deleted file`, { file: orphanedCachePath });
    } catch (err) {
      logger.warn('system', `Failed to remove orphaned cache: ${(err as Error).message}`, { file: orphanedCachePath });
    }
  }
}

// Vite env variables (injected by @electron-forge/plugin-vite at build time)
declare const MAIN_WINDOW_VITE_DEV_SERVER_URL: string | undefined;
declare const MAIN_WINDOW_VITE_NAME: string;

let mainWindow: BrowserWindow | null = null;

let startupCatchUpInFlight = false;

async function runStartupCatchUp(): Promise<void> {
  if (startupCatchUpInFlight) return;
  startupCatchUpInFlight = true;
  try {
    const activeRoots = await configHandler.getFolderSources();
    if (activeRoots.length === 0) return;
    logger.info('watch', `Startup catch-up: ${activeRoots.length} root(s)`);
    for (const root of activeRoots) {
      try {
        await runScanAndFetch(root, activeRoots);
      } catch (err) {
        logger.warn('folder', `Startup catch-up failed for ${root}: ${(err as Error).message}`, { file: root });
      }
    }
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:file-status-changed', { filePath: '', status: 'ready' });
    }
    // Background: kick off poster matching for series that haven't been
    // tried yet. Doesn't block startup. Renderer reloads on each match
    // via the file-status-changed ping inside matchPosterForSeries.
    void matchPostersForLibrary();
    // Parallel fast-path: backfill franchise relations for already-matched
    // series that predate the related-strip feature. One AniList query per
    // series instead of redoing the full search; runs alongside the heavier
    // poster matcher so users see relations populate within seconds.
    void backfillRelationsForLibrary();
  } finally {
    startupCatchUpInFlight = false;
  }
}

// Background match for one series: poster + status + per-episode air
// dates (MAL only — AniList airing-schedule not wired yet). Idempotent:
// always writes `posterMatchAttempted: true` so we don't re-hammer MAL
// on every restart. Misses leave the placeholder; future "human in the
// loop" UI will let the user pick manually.
async function matchPosterForSeries(seriesId: string, folderName: string): Promise<void> {
  try {
    const match = await findShowMatch(folderName);
    if (!match) {
      await metadataHandler.transaction(async (current) => {
        const existing = (current[seriesId] ?? {}) as Record<string, unknown>;
        if (existing.posterMatchAttempted) return { updated: null };
        current[seriesId] = { ...existing, posterMatchAttempted: true, posterMatched: false };
        return { updated: current };
      });
      return;
    }
    // findShowMatch guarantees the id matching `source` is set; the other
    // is best-effort (cross-resolved or absent).
    const primaryId = match.source === 'anilist' ? match.anilistId! : match.malId!;
    // Relations come from AniList only (MAL has related_anime but no
    // cross-media graph), so we query by anilistId when we have it and
    // fall back to AniList's idMal filter for MAL-primary matches.
    const enrichmentOpts = match.anilistId
      ? { anilistId: match.anilistId }
      : match.malId
        ? { malId: match.malId }
        : {};
    const [cached, episodeDates, enrichment] = await Promise.all([
      imageCacheHandler.cacheImages([match.posterUrl]),
      fetchEpisodeAirDates(match.source, primaryId, match.totalEpisodes, match.malId),
      anilistHandler.getEnrichment(enrichmentOpts),
    ]);
    const posterLocal = cached.get(match.posterUrl) ?? null;
    await metadataHandler.transaction(async (current) => {
      const existing = (current[seriesId] ?? {}) as Record<string, unknown>;
      // Title priority: AniList streamingEpisodes (the "Watch" tab on
      // anilist.co) > MAL/Jikan. AniList is the canonical source for episode
      // names — Jikan often returns generic "Episode N" placeholders. The
      // enrichment query already pulled streamingEpisodes; we index them by
      // episode number for an O(1) merge with the air-date list.
      const anilistTitleByEp = new Map(
        enrichment.episodeTitles.map((e) => [e.episodeNumber, e.title]),
      );
      const slimEpisodes = episodeDates.map((e) => {
        const title = anilistTitleByEp.get(e.episodeNumber) ?? e.title ?? null;
        return {
          episodeNumber: e.episodeNumber,
          airDate: e.airDate,
          ...(title ? { title } : {}),
        };
      });
      // AniList sometimes lists episodes in streamingEpisodes that the
      // airing schedule doesn't (older completed shows). Splice those in
      // so the UI can render their names too, even with a null airDate.
      for (const [ep, title] of anilistTitleByEp) {
        if (!slimEpisodes.some((s) => s.episodeNumber === ep)) {
          slimEpisodes.push({ episodeNumber: ep, airDate: null, title });
        }
      }
      slimEpisodes.sort((a, b) => a.episodeNumber - b.episodeNumber);
      // Slim relations — drop CHARACTER (almost never meaningful for the
      // user's intent) but keep everything else verbatim. The renderer
      // groups & filters at display time so we don't bake UX decisions
      // into the persisted shape.
      const slimRelations = enrichment.relations
        .filter((r) => r.relationType !== 'CHARACTER')
        .map((r) => ({
          relationType: r.relationType,
          anilistId: r.anilistId,
          malId: r.malId,
          type: r.type,
          format: r.format,
          status: r.status,
          seasonYear: r.seasonYear,
          siteUrl: r.siteUrl,
          titleRomaji: r.titleRomaji,
          titleEnglish: r.titleEnglish,
          poster: r.poster,
        }));
      // AniList tags arrive sorted by their `rank` (community-voted weight).
      // We persist everything — the renderer filters spoilers/adult and
      // caps the visible count.
      const slimTags = enrichment.tags;
      // Characters — keep MAIN role only by default plus top supporting if
      // few mains exist. AniList already sorts ROLE → RELEVANCE so the
      // first N rows are the meaningful ones; cap at 10 for storage.
      const slimCharacters = enrichment.characters.slice(0, 10);
      // Recommendations — cap at 8 picks ordered by AniList rating.
      const slimRecommendations = enrichment.recommendations.slice(0, 8);
      // Pick the main animation studio: prefer the entry flagged both
      // isMain + isAnimationStudio. Fall back to the first animation studio,
      // then to the first studio of any kind. Persist a flat string for
      // simple consumption; persist the full studio array too so the UI
      // can show secondary studios later without another fetch.
      const studiosByPriority = [...enrichment.studios].sort((a, b) => {
        const score = (s: typeof a) =>
          (s.isMain && s.isAnimationStudio ? 0 : s.isAnimationStudio ? 1 : s.isMain ? 2 : 3);
        return score(a) - score(b);
      });
      const animationStudio =
        studiosByPriority.find((s) => s.isMain && s.isAnimationStudio)?.name
        ?? studiosByPriority.find((s) => s.isAnimationStudio)?.name
        ?? studiosByPriority[0]?.name
        ?? null;
      const studioNames = studiosByPriority.map((s) => s.name);
      // Persist BOTH provider ids when we have them. Trackers, AniSkip, and
      // the rest of the renderer key off `anilistId`/`malId` directly —
      // matchSource alone isn't enough.
      current[seriesId] = {
        ...existing,
        poster: match.posterUrl,
        posterLocal,
        posterMatchAttempted: true,
        posterMatched: true,
        matchSource: match.source,
        anilistId: match.anilistId ?? undefined,
        malId: match.malId ?? null,
        matchedTitle: match.matchedTitle,
        titleRomaji: match.titleRomaji,
        titleEnglish: match.titleEnglish,
        matchScore: match.score,
        status: match.status,
        startDate: match.startDate,
        totalEpisodes: match.totalEpisodes,
        episodes: slimEpisodes,
        relations: slimRelations,
        tags: slimTags,
        characters: slimCharacters,
        recommendations: slimRecommendations,
        studios: studioNames,
        animationStudio,
      };
      return { updated: current };
    });
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:file-status-changed', { filePath: '', status: 'ready' });
    }
  } catch (err) {
    logger.warn('metadata', `Match failed for ${folderName}: ${(err as Error).message}`, { series: folderName });
  }
}

// Walk every series in metadata.json and match a poster for any that
// haven't been attempted yet. Sequential so we don't blow through the
// MAL/AniList rate limiters in bursts. Designed to run in the background
// — never throw, never block the caller.
//
// Launch-time matching is intentionally narrow: ONLY entries that have
// never been attempted. After one attempt (success or fail) the entry is
// left alone — no periodic re-matches for ongoing shows, no legacy
// field backfills via the matcher, no auto-retries for failures. Manual
// recovery stays available via the per-row Refresh button and the Match
// modal in the Metadata tab. (Aligns with the "no periodic rescans"
// design: the user opts in to expensive work, the app never re-does it
// behind their back.)
async function matchPostersForLibrary(): Promise<void> {
  const meta = await metadataHandler.loadMetadata();
  const todo: Array<{ seriesId: string; folderName: string }> = [];
  for (const [seriesId, raw] of Object.entries(meta)) {
    const s = raw as { posterMatchAttempted?: boolean; title?: string };
    if (s.posterMatchAttempted) continue;
    // Use the cleaned title for the lookup. The scanner sets `title` to the
    // user-canonical (wrapper-derived for franchise subfolders, folder name
    // for root-level series, cleaned filename for movies) string — which is
    // what MAL/AniList actually expects to match against.
    const matchQuery = s.title ?? seriesId;
    todo.push({ seriesId, folderName: matchQuery });
  }
  if (todo.length === 0) return;
  logger.info('metadata', `Matching ${todo.length} new series (poster + air dates)`);
  for (const { seriesId, folderName } of todo) {
    await matchPosterForSeries(seriesId, folderName);
  }
}

// Dedicated fast-path backfill for the enrichment bundle (relations, tags,
// characters, recommendations, studio) on series that are already matched
// but predate one of those fields. Skips the entire findShowMatch round-trip
// (which would re-do MAL search + AniList fallback per series), and just
// calls AniList getEnrichment with the stored ids. One AniList request per
// series → completes in seconds for libraries that would otherwise spend
// minutes redoing search queries they don't need.
//
// Also backfills episode titles via MAL/Jikan when malId is known and the
// existing episodes array lacks them — feature was added after the slim
// schedule was already cached for most users.
async function backfillRelationsForLibrary(): Promise<void> {
  const meta = await metadataHandler.loadMetadata();
  const todo: Array<{
    seriesId: string;
    anilistId?: number;
    malId?: number;
    title: string;
    needsEnrichment: boolean;
    needsEpisodeTitles: boolean;
  }> = [];
  for (const [seriesId, raw] of Object.entries(meta)) {
    const s = raw as {
      posterMatched?: boolean;
      anilistId?: number;
      malId?: number | null;
      relations?: unknown;
      tags?: unknown;
      characters?: unknown;
      recommendations?: unknown;
      animationStudio?: unknown;
      episodes?: Array<{ episodeNumber?: number; title?: string }>;
      totalEpisodes?: number | null;
      title?: string;
    };
    // Only matched entries are candidates.
    if (!s.posterMatched) continue;
    if (s.anilistId == null && (s.malId == null)) continue;
    const hasRelations = Array.isArray(s.relations);
    const hasTags = Array.isArray(s.tags);
    const hasCharacters = Array.isArray(s.characters);
    const hasRecommendations = Array.isArray(s.recommendations);
    const hasStudio = typeof s.animationStudio === 'string' || s.animationStudio === null;
    const needsEnrichment = !hasRelations || !hasTags || !hasCharacters || !hasRecommendations || !hasStudio;
    const episodes = Array.isArray(s.episodes) ? s.episodes : [];
    // Backfill episode titles whenever any are missing — we now have two
    // sources (AniList streamingEpisodes + MAL/Jikan) so a series that
    // failed one source might succeed on the other.
    const needsEpisodeTitles = episodes.length > 0 && !episodes.some((e) => typeof e.title === 'string' && e.title.length > 0);
    if (!needsEnrichment && !needsEpisodeTitles) continue;
    todo.push({
      seriesId,
      anilistId: s.anilistId ?? undefined,
      malId: s.malId ?? undefined,
      title: s.title ?? seriesId,
      needsEnrichment,
      needsEpisodeTitles,
    });
  }
  if (todo.length === 0) return;
  logger.info('metadata', `Backfilling enrichment for ${todo.length} series`);
  for (const { seriesId, anilistId, malId, title, needsEnrichment, needsEpisodeTitles } of todo) {
    try {
      // Always pull the enrichment when titles are missing — its
      // streamingEpisodes are the canonical title source. Otherwise only
      // when other enrichment fields are missing.
      const enrichment = (needsEnrichment || needsEpisodeTitles)
        ? await anilistHandler.getEnrichment(anilistId ? { anilistId } : { malId: malId! })
        : null;
      // Build the merged title map: AniList streamingEpisodes first, then
      // MAL/Jikan fills any gaps. Skip MAL entirely if AniList already
      // covered every episode we have on disk.
      const titleMap = new Map<number, string>();
      if (enrichment) {
        for (const e of enrichment.episodeTitles) titleMap.set(e.episodeNumber, e.title);
      }
      if (needsEpisodeTitles && malId != null) {
        try {
          const eps = await malHandler.getEpisodes(malId, null);
          for (const e of eps) {
            if (!e.title || /^Episode\s+\d+$/i.test(e.title)) continue;
            if (!titleMap.has(e.episodeNumber)) titleMap.set(e.episodeNumber, e.title);
          }
        } catch (err) {
          logger.warn('metadata', `MAL episode-title backfill failed for ${title}: ${(err as Error).message}`, { series: title });
        }
      }
      const episodeTitles = titleMap;
      await metadataHandler.transaction(async (current) => {
        const existing = (current[seriesId] ?? {}) as Record<string, unknown> & {
          episodes?: Array<{ episodeNumber: number; airDate?: string | null; title?: string | null }>;
        };
        const patch: Record<string, unknown> = {};
        if (enrichment) {
          patch.relations = enrichment.relations
            .filter((r) => r.relationType !== 'CHARACTER')
            .map((r) => ({
              relationType: r.relationType,
              anilistId: r.anilistId,
              malId: r.malId,
              type: r.type,
              format: r.format,
              status: r.status,
              seasonYear: r.seasonYear,
              siteUrl: r.siteUrl,
              titleRomaji: r.titleRomaji,
              titleEnglish: r.titleEnglish,
              poster: r.poster,
            }));
          patch.tags = enrichment.tags;
          patch.characters = enrichment.characters.slice(0, 10);
          patch.recommendations = enrichment.recommendations.slice(0, 8);
          const studiosByPriority = [...enrichment.studios].sort((a, b) => {
            const score = (s: typeof a) =>
              (s.isMain && s.isAnimationStudio ? 0 : s.isAnimationStudio ? 1 : s.isMain ? 2 : 3);
            return score(a) - score(b);
          });
          patch.animationStudio =
            studiosByPriority.find((s) => s.isMain && s.isAnimationStudio)?.name
            ?? studiosByPriority.find((s) => s.isAnimationStudio)?.name
            ?? studiosByPriority[0]?.name
            ?? null;
          patch.studios = studiosByPriority.map((s) => s.name);
        }
        if (episodeTitles && Array.isArray(existing.episodes)) {
          patch.episodes = existing.episodes.map((e) => {
            const t = episodeTitles.get(e.episodeNumber);
            return t ? { ...e, title: t } : e;
          });
        }
        current[seriesId] = { ...existing, ...patch };
        return { updated: current };
      });
      // Ping the renderer so an open series-detail page picks up the new
      // data without a manual reload. Single ping per series matches the
      // existing match flow.
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send('metadata:file-status-changed', { filePath: '', status: 'ready' });
      }
    } catch (err) {
      logger.warn('metadata', `Enrichment backfill failed for ${title}: ${(err as Error).message}`, { series: title });
    }
  }
  logger.info('metadata', `Enrichment backfill complete`);
}

async function ingestSubtree(dirPath: string): Promise<void> {
  // chokidar emits `addDir` when a new folder appears, but it does NOT
  // reliably emit `add` for files that were already inside the folder at
  // creation time (e.g. a torrent client renaming a temp dir into place).
  // Walk the new subtree ourselves and ingest every video we find.
  try {
    const { readdir, stat } = await import('fs/promises');
    const entries = await readdir(dirPath);
    for (const entry of entries) {
      const full = `${dirPath}/${entry}`;
      try {
        const s = await stat(full);
        if (s.isDirectory()) {
          await ingestSubtree(full);
        } else if (s.isFile() && /\.(mkv|mp4|avi|mov|webm|m4v|ts|wmv|flv)$/i.test(entry)) {
          void ingestSingleFile(full);
        }
      } catch { /* skip unreadable */ }
    }
  } catch (err) {
    logger.warn('watch', `ingestSubtree failed for ${dirPath}: ${(err as Error).message}`, { file: dirPath });
  }
}

function createWindow(): void {
  // Remove the application menu (File, Edit, View, etc.)
  Menu.setApplicationMenu(null);

  mainWindow = new BrowserWindow({
    width: 1920,
    height: 1080,
    minWidth: 800,
    minHeight: 600,
    webPreferences: {
      preload: join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      // Dev only: the Vite dev server (http://localhost:5173) needs to
      // fetch local images via media:// during HMR, and Chromium would
      // otherwise block the cross-origin request even though media:// has
      // corsEnabled. In packaged builds the renderer loads from file://
      // so Chromium's same-origin checks are already a no-op for our
      // local paths — keep the security on.
      webSecurity: process.env.DEV_MODE !== 'true',
    },
    autoHideMenuBar: true,
    backgroundColor: '#0a0a0f',
  });

  // Load the app using Vite's dev server URL (set by @electron-forge/plugin-vite)
  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL);
  } else {
    mainWindow.loadFile(join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`));
  }

  // Conditionally enable dev tools in dev mode only
  if (process.env.DEV_MODE === 'true') {
    mainWindow.webContents.openDevTools();
    mainWindow.webContents.on('before-input-event', (_event, input) => {
      if (input.control && input.shift && input.key.toLowerCase() === 'i') {
        mainWindow?.webContents.toggleDevTools();
      }
    });
  }

  mainWindow.webContents.once('did-finish-load', () => {
    logger.info('system', 'AniBeam ready');
  });

  // Log renderer errors
  mainWindow.webContents.on('did-fail-load', (_event, errorCode, errorDescription) => {
    console.error('Renderer failed to load:', errorCode, errorDescription);
  });
}

// Register custom protocol for serving local media files. corsEnabled lets
// the dev-server renderer (http://localhost:5173) fetch through it without
// tripping Chromium's CORS check.
protocol.registerSchemesAsPrivileged([
  {
    scheme: 'media',
    privileges: {
      secure: true,
      supportFetchAPI: true,
      stream: true,
      bypassCSP: true,
      corsEnabled: true,
    },
  },
]);

app.whenReady().then(async () => {
  initMediaProgress(); // Debug progress bar initialization
  // Handle media:// protocol using net.fetch for proper streaming
  protocol.handle('media', async (request) => {
    // Parse the URL to extract the file path
    let filePath = request.url.replace(/^media:\/\//, '');
    filePath = decodeURIComponent(filePath);

    // Cross-platform path handling: ensure absolute paths
    if (!isAbsolute(filePath)) {
      // Only add leading / for Unix-like systems (Linux, macOS)
      if (process.platform !== 'win32') {
        filePath = '/' + filePath;
      } else {
        // On Windows, reject relative paths for security
        logger.error('system', `Rejected relative path: ${filePath}`, { file: filePath });
        return new Response('Invalid path: relative paths not allowed', { status: 403 });
      }
    }

    // SECURITY: Validate path is within allowed folder sources OR app's userData directory
    try {
      const normalizedPath = resolve(filePath);
      
      const userDataPath = app.getPath('userData');
      const normalizedUserData = resolve(userDataPath);
      const userDataRelative = relative(normalizedUserData, normalizedPath);
      const isInUserData = !userDataRelative.startsWith('..') && !userDataRelative.startsWith('/');
      
      const allowedSources = await configHandler.getFolderSources();
      const isInAllowedSource = allowedSources.some(source => {
        try {
          const normalizedSource = resolve(source);
          const relativePath = relative(normalizedSource, normalizedPath);
          return !relativePath.startsWith('..') && !relativePath.startsWith('/');
        } catch {
          return false;
        }
      });

      if (!isInUserData && !isInAllowedSource) {
        if (allowedSources.length === 0 && !isInUserData) {
          logger.error('system', 'Access denied: path not in userData and no folder sources configured');
          return new Response('Access denied: path not in allowed directories', { status: 403 });
        }
        logger.error('system', `Access denied for path: ${filePath}`, { file: filePath });
        return new Response('Access denied: path not in allowed directories', { status: 403 });
      }
    } catch (error) {
      logger.error('system', 'Error validating path');
      return new Response('Error validating path', { status: 500 });
    }

    updateMediaProgress(filePath); // Debug progress bar update

    // Check if file exists
    if (!existsSync(filePath)) {
      logger.error('system', `File not found: ${filePath}`, { file: filePath });
      return new Response('File not found', { status: 404 });
    }

    // Use net.fetch with file:// URL - Electron handles range requests automatically
    const fileUrl = pathToFileURL(filePath).toString();

    // Forward the request headers (including Range)
    const headers: Record<string, string> = {};
    request.headers.forEach((value, key) => {
      headers[key] = value;
    });

    try {
      const response = await net.fetch(fileUrl, {
        method: request.method,
        headers: headers,
      });

      // Get file extension for MIME type
      const ext = filePath.split('.').pop()?.toLowerCase() || '';
      const mimeTypes: Record<string, string> = {
        // Video formats
        'mp4': 'video/mp4',
        'mkv': 'video/x-matroska',
        'avi': 'video/x-msvideo',
        'webm': 'video/webm',
        'mov': 'video/quicktime',
        'wmv': 'video/x-ms-wmv',
        'flv': 'video/x-flv',
        'm4v': 'video/mp4',
        // Subtitle formats
        'srt': 'text/plain; charset=utf-8',
        'vtt': 'text/vtt; charset=utf-8',
        'ass': 'text/plain; charset=utf-8',
        'ssa': 'text/plain; charset=utf-8',
        // Image formats (for cached images)
        'jpg': 'image/jpeg',
        'jpeg': 'image/jpeg',
        'png': 'image/png',
        'gif': 'image/gif',
        'webp': 'image/webp',
        'avif': 'image/avif',
      };

      // Clone response with correct content type
      const newHeaders = new Headers(response.headers);
      if (mimeTypes[ext]) {
        newHeaders.set('Content-Type', mimeTypes[ext]);
      }

      return new Response(response.body, {
        status: response.status,
        statusText: response.statusText,
        headers: newHeaders,
      });
    } catch (error) {
      logger.error('system', 'Error fetching file');
      return new Response('Error loading file', { status: 500 });
    }
  });

  createWindow();

  // Wire IPC modules. Each register() takes a window getter (or nothing
  // when handlers don't need to send to the renderer) and adds its
  // handlers to ipcMain. Order doesn't matter — they're independent.
  const getMainWindow = (): BrowserWindow | null => mainWindow;
  registerLogIpc();
  registerConfigIpc();
  registerFolderIpc(getMainWindow);
  registerImageCacheIpc();
  registerMediaPlaybackIpc(getMainWindow);
  registerTrackerIpc(getMainWindow);
  registerShellIpc();
  registerSubscriptionsIpc();

  // Probe-ready and transcode events share the same status update plumbing.
  // The probe callback also tees into maybeEnqueueTranscode whenever a file
  // becomes 'ready' so we can decide if it needs a follow-up transcode.
  videoProbeHandler.start(async (filePath, status) => {
    await updateFileStatus(filePath, status);
    if (status === 'ready') await maybeEnqueueTranscode(filePath);
  });
  transcodeCacheHandler.start(updateFileStatus, undefined, (progress) => {
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('metadata:transcode-progress', progress);
    }
  });

  // Re-enqueue any files that were 'verifying' when the app last quit —
  // the in-memory probe queue doesn't survive restarts, and the watcher's
  // ignoreInitial:true means cold-start won't re-discover existing files.
  // Without this, a file caught mid-verification stays stuck forever.
  try {
    const meta = await metadataHandler.loadMetadata();
    let resumed = 0;
    for (const series of Object.values(meta)) {
      const s = series as { fileEpisodes?: FileEpisodeEntry[] };
      if (!Array.isArray(s.fileEpisodes)) continue;
      for (const f of s.fileEpisodes) {
        if (f.status === 'verifying' || f.status === 'stalled') {
          videoProbeHandler.enqueue(f.filePath);
          resumed++;
        }
      }
    }
    if (resumed > 0) logger.info('probe', `Resumed ${resumed} unverified file(s) from prior session`);
  } catch (err) {
    logger.warn('probe', `Could not resume unverified files: ${(err as Error).message}`);
  }

  // Validate the transcode cache and resume any interrupted transcodes.
  //   - 'transcoding' rows: the previous run was killed before ffmpeg
  //     finished. Re-enqueue so the encode picks back up.
  //   - transcodedPath set but file missing: cache was nuked externally
  //     (rm -rf, disk cleanup, etc.). Drop the field and re-enqueue.
  //   - transcodedPath set and file present: nothing to do — the player
  //     will pick it up on first openVideo.
  try {
    let resumedT = 0;
    let invalidated = 0;
    await metadataHandler.transaction<boolean>(async (meta) => {
      let changed = false;
      for (const series of Object.values(meta)) {
        const s = series as { fileEpisodes?: FileEpisodeEntry[] };
        if (!Array.isArray(s.fileEpisodes)) continue;
        for (const f of s.fileEpisodes) {
          if (f.transcodedPath && !existsSync(f.transcodedPath)) {
            f.transcodedPath = null;
            invalidated++;
            changed = true;
            if (existsSync(f.filePath)) void transcodeCacheHandler.enqueue(f.filePath);
          } else if (f.status === 'transcoding' && existsSync(f.filePath)) {
            if (!f.transcodedPath) {
              void transcodeCacheHandler.enqueue(f.filePath);
              resumedT++;
            }
          }
        }
      }
      return { result: changed, updated: changed ? meta : null };
    });
    if (resumedT > 0) logger.info('system', `Resumed ${resumedT} interrupted transcode(s)`);
    if (invalidated > 0) logger.info('system', `Re-queued ${invalidated} file(s) with missing cache`);
  } catch (err) {
    logger.warn('system', `Transcode cache resume failed: ${(err as Error).message}`);
  }

  // Catch-up for pre-existing library: files that were 'ready' before
  // this transcode pipeline existed have no transcodedPath. Probe each
  // one's codec; if it needs transcoding, enqueue. Runs in the
  // background (the enqueue method is async) so we don't block startup
  // — files appear in 'transcoding' status to the renderer as each
  // probe lands.
  void (async () => {
    try {
      const meta = await metadataHandler.loadMetadata();
      let scheduled = 0;
      for (const series of Object.values(meta)) {
        const s = series as { fileEpisodes?: FileEpisodeEntry[] };
        if (!Array.isArray(s.fileEpisodes)) continue;
        for (const f of s.fileEpisodes) {
          if (f.transcodedPath) continue;
          if (!existsSync(f.filePath)) continue;
          if (await transcodeCacheHandler.shouldTranscode(f.filePath)) {
            void transcodeCacheHandler.enqueue(f.filePath);
            scheduled++;
          }
        }
      }
      if (scheduled > 0) logger.info('system', `Scheduled ${scheduled} pre-existing file(s) for transcode`);
    } catch (err) {
      logger.warn('system', `Library transcode sweep failed: ${(err as Error).message}`);
    }
  })();

  const initialRoots = await configHandler.getFolderSources();
  await fileWatcher.start(initialRoots, {
    onAdd: (path) => { void ingestSingleFile(path); },
    onAddDir: (dirPath) => { void ingestSubtree(dirPath); },
    onUnlink: (path) => { void handleUnlink(path); },
    // unlinkDir: chokidar emits unlink for each contained file too, so the
    // per-file handler covers actual cleanup. Just log; don't double-reconcile.
    onUnlinkDir: (dirPath) => { logger.info('watch', `Directory removed`, { file: dirPath }); },
  });

  // One-shot catch-up for files that landed while the app was closed
  // (chokidar's ignoreInitial:true skips them). NOT a periodic safety net —
  // ongoing detection is the watcher's job, period.
  void runStartupCatchUp();

  // One-shot maintenance: drop expired image-cache entries and orphaned /
  // over-quota transcode-cache files. Best-effort, never throws.
  void imageCacheHandler.pruneIndexNow();
  void transcodeCacheHandler.pruneCacheNow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// ==================== METADATA IPC ====================

ipcMain.handle('fetch-metadata', async (_event, searchName: string, seasonNumber?: number | null) => {
  // Best-match across MAL + AniList. No folderEpisodeCount on this path
  // (renderer-side refresh doesn't know it), so the strict-tier ep-count
  // filter is disabled — title similarity does the heavy lifting.
  const seasonInfo = seasonNumber !== null && seasonNumber !== undefined ? ` Season ${seasonNumber}` : '';
  logger.info('metadata', `Fetching metadata for: "${searchName}"${seasonInfo}`);

  const result = await findBestMatch(searchName, seasonNumber ?? null, null, undefined);
  if (result) {
    const meta = result.metadata as { title?: string };
    logger.info(
      'metadata',
      `Refresh match: ${result.source.toUpperCase()} "${meta.title ?? '?'}" (score ${result.score.toFixed(2)})`,
    );
    return result.metadata;
  }
  logger.warn('metadata', `No metadata found for: "${searchName}"${seasonInfo}`);
  return null;
});

ipcMain.handle('fetch-mal-metadata', async (_event, seriesName: string, seasonNumber?: number | null) => {
  try {
    return await malHandler.searchAndFetchMetadata(seriesName, seasonNumber);
  } catch (error) {
    logger.error('metadata', 'Error fetching MAL metadata');
    throw error;
  }
});

ipcMain.handle('fetch-anilist-metadata', async (_event, seriesName: string, seasonNumber?: number | null) => {
  try {
    return await anilistHandler.searchAndFetchMetadata(seriesName, seasonNumber);
  } catch (error) {
    logger.error('metadata', 'Error fetching AniList metadata');
    throw error;
  }
});

// Match-picker search: returns multiple AniList results for the user to choose
// from. Lighter than fetch-metadata — no episode fetch, no filtering, no
// fallback dance. Used by the metadata-override modal.
ipcMain.handle('anilist:search', async (_event, query: string, limit?: number) => {
  if (!query || typeof query !== 'string' || query.trim().length === 0) return [];
  try {
    return await anilistHandler.searchAnimeMultiple(query.trim(), typeof limit === 'number' ? limit : 12);
  } catch (error) {
    if (!isRateLimitError(error)) logger.error('metadata', 'Error searching AniList for picker');
    return [];
  }
});

ipcMain.handle('franchise:graph', async (_event, anilistId: number) => {
  if (typeof anilistId !== 'number' || !Number.isFinite(anilistId)) return null;
  try {
    return await getFranchiseGraph(anilistId);
  } catch (error) {
    logger.error('metadata', `franchise:graph failed: ${(error as Error).message}`);
    return null;
  }
});

ipcMain.handle('franchise:crawl-progress', async () => {
  try { return await getFranchiseCrawlProgress(); }
  catch (e) {
    logger.error('metadata', `franchise:crawl-progress failed: ${(e as Error).message}`);
    return { total: 0, crawled: 0 };
  }
});

// Override path: the user picked a specific AniList ID in the match modal.
// Run the same image-caching + local-thumbnail generation pipeline that
// scan-and-fetch uses, so episode thumbnails actually populate (raw AniList
// data has only a sparse `streamingEpisodes` set; everything else needs an
// ffmpeg fallback against the on-disk video files).
ipcMain.handle('metadata:apply-anilist-match', async (
  _event,
  seriesId: string,
  anilistId: number,
  seasonNumber: number | null = null,
) => {
  if (!seriesId || typeof anilistId !== 'number' || !Number.isFinite(anilistId)) {
    return { ok: false, reason: 'bad-args' };
  }

  // 1. Fetch the chosen media from AniList.
  const fetched = await anilistHandler.fetchMetadataById(anilistId, seasonNumber);
  if (!fetched) return { ok: false, reason: 'fetch-failed' };

  // 2. Pull the existing entry for fileEpisodes / folderPath / type — those
  //    describe local state and must be preserved across the override.
  const allMeta = await metadataHandler.loadMetadata();
  const existing = allMeta[seriesId] as {
    fileEpisodes?: Array<{ episodeNumber: number; seasonNumber?: number | null; filePath: string }>;
    folderPath?: string;
    type?: 'series' | 'movie';
  } | undefined;

  const fileEpisodes = existing?.fileEpisodes ?? [];

  // 3. Build a season+episode → filePath lookup so we can pick the right
  //    video for each AniList-listed episode that lacks an online thumbnail.
  const fileEpisodeMap = new Map<string, string>();
  for (const f of fileEpisodes) {
    const key = f.seasonNumber !== null && f.seasonNumber !== undefined
      ? `${f.seasonNumber}_${f.episodeNumber}`
      : `null_${f.episodeNumber}`;
    fileEpisodeMap.set(key, f.filePath);
  }

  // 4. Cache poster + banner + every online episode thumbnail in one pass.
  const imagesToCache: (string | null)[] = [fetched.poster, fetched.banner];
  for (const ep of fetched.episodes || []) {
    if (ep.thumbnail) imagesToCache.push(ep.thumbnail);
  }
  const cachedImages = await imageCacheHandler.cacheImages(imagesToCache);

  const posterLocal = fetched.poster ? cachedImages.get(fetched.poster) ?? null : null;
  const bannerLocal = fetched.banner ? cachedImages.get(fetched.banner) ?? null : null;

  // 5. For each AniList episode, prefer the cached online thumbnail; if there
  //    isn't one, generate a frame from the matching local video. Match by
  //    season+episode first, then any-season fallback (handles both libraries
  //    that flag seasons in folder names and ones that don't).
  //
  //    Thumbnails are generated with bounded concurrency — one ffmpeg per
  //    episode used to run sequentially, which made a 24-ep apply take ~30s.
  //    THUMBNAIL_CONCURRENCY=4 stays well under typical core counts and
  //    keeps the user-perceived wait closer to a few seconds.
  const fetchedEpisodes = fetched.episodes ?? [];
  const resolveVideoPath = (ep: { episodeNumber: number; seasonNumber?: number | null }): string | undefined => {
    const epSeason = ep.seasonNumber ?? null;
    const exactKey = epSeason !== null ? `${epSeason}_${ep.episodeNumber}` : `null_${ep.episodeNumber}`;
    let videoPath = fileEpisodeMap.get(exactKey);
    if (!videoPath && epSeason !== null) videoPath = fileEpisodeMap.get(`null_${ep.episodeNumber}`);
    if (videoPath) return videoPath;
    for (const [mapKey, path] of fileEpisodeMap.entries()) {
      const parts = mapKey.split('_');
      const keyEpNum = parseFloat(parts[parts.length - 1]);
      if (keyEpNum === ep.episodeNumber) return path;
    }
    return undefined;
  };

  const thumbnailJobs = fetchedEpisodes.map((ep, idx) => {
    let thumbnailLocal: string | null = null;
    if (ep.thumbnail) thumbnailLocal = cachedImages.get(ep.thumbnail) ?? null;
    const videoPath = thumbnailLocal ? undefined : resolveVideoPath(ep);
    return { idx, ep, thumbnailLocal, videoPath };
  });

  const THUMBNAIL_CONCURRENCY = 4;
  const generated = new Array<string | null>(thumbnailJobs.length).fill(null);
  for (let i = 0; i < thumbnailJobs.length; i += THUMBNAIL_CONCURRENCY) {
    const batch = thumbnailJobs.slice(i, i + THUMBNAIL_CONCURRENCY);
    await Promise.allSettled(batch.map(async (job) => {
      if (job.thumbnailLocal) { generated[job.idx] = job.thumbnailLocal; return; }
      if (!job.videoPath) return;
      try {
        // resetProgressBar=true on the very first job in the apply batch so
        // the progress bar zeroes out; subsequent jobs append.
        generated[job.idx] = await thumbnailHandler.generateThumbnail(job.videoPath, 120, i === 0 && job === batch[0]);
      } catch {
        logger.warn('thumbnail', `apply-match: failed to generate thumbnail for ep ${job.ep.episodeNumber}`, { file: job.videoPath });
      }
    }));
  }
  const episodesWithLocalThumbs = thumbnailJobs.map((job) => ({ ...job.ep, thumbnailLocal: generated[job.idx] }));

  // 6. Merge into existing entry. Keep the original seriesId KEY in
  //    metadata.json (caller still uses it); also write the AniList one
  //    inside the entry so future code can resolve the chosen ID.
  const merged: Record<string, unknown> = {
    ...fetched,
    posterLocal,
    bannerLocal,
    episodes: episodesWithLocalThumbs,
    fileEpisodes,
    folderPath: existing?.folderPath,
    type: existing?.type,
    source: 'anilist',
  };

  await metadataHandler.updateSeriesMetadata(seriesId, merged);
  logger.info('metadata', `Override applied: ${seriesId} → AniList ${anilistId}`);
  return { ok: true };
});

ipcMain.handle('save-metadata', async (_event, metadata: Record<string, unknown>) => {
  try {
    return await metadataHandler.saveMetadata(metadata);
  } catch (error) {
    logger.error('metadata', 'Error saving metadata');
    throw error;
  }
});

ipcMain.handle('load-metadata', async () => {
  try {
    return await metadataHandler.loadMetadata();
  } catch (error) {
    logger.error('metadata', 'Error loading metadata');
    return {};
  }
});

ipcMain.handle('clear-metadata', async () => {
  try {
    return await metadataHandler.saveMetadata({});
  } catch (error) {
    logger.error('metadata', 'Error clearing metadata');
    throw error;
  }
});

ipcMain.handle('delete-series', async (_event, seriesId: string) => {
  try {
    // Get series metadata first to delete associated images
    const seriesData = await metadataHandler.getSeriesMetadata(seriesId);

    if (seriesData) {
      // Delete cached images for this series
      await imageCacheHandler.deleteSeriesImages(seriesData as {
        poster?: string | null;
        banner?: string | null;
        posterLocal?: string | null;
        bannerLocal?: string | null;
        episodes?: Array<{
          thumbnail?: string | null;
          thumbnailLocal?: string | null;
        }>;
      });
    }

    // Delete metadata entry
    await metadataHandler.deleteSeriesMetadata(seriesId);

    logger.info('metadata', `Deleted series: ${seriesId}`);
    return true;
  } catch (error) {
    logger.error('metadata', 'Error deleting series');
    throw error;
  }
});

// ==================== PER-SERIES METADATA PROCESSOR ====================
// PAUSED: metadata enrichment (network fetch, image cache, thumbnails) is
// currently disabled — the renderer reads directly from library:walk for
// display, and runScanAndFetch only does a filesystem-only fileEpisodes
// sync. processOneMedia is kept here to make re-enabling enrichment a
// one-liner (call it again from runScanAndFetch's slow pass) without
// rewriting the hundreds of lines of online-thumbnail / image-cache /
// MAL+AniList logic. Don't delete it.
void processOneMedia;

async function processOneMedia(
  media: ScannedMedia,
  existingMetadata: Record<string, unknown>,
): Promise<Record<string, unknown>> {
  const slice: Record<string, unknown> = {};
  const mediaId = media.id;

  // Start with existing metadata if available
  if (existingMetadata[mediaId]) {
    slice[mediaId] = { ...existingMetadata[mediaId] as Record<string, unknown> };
  }

  // Check if we already have metadata for this
  const existing = existingMetadata[mediaId] as Record<string, unknown> | undefined;
  if (existing?.title && existing?.posterLocal) {
    // Validate that cached metadata matches the current series name
    // This prevents using wrong metadata when series ID changes or files change
    const cachedTitle = (existing.title as string).toLowerCase().trim();
    const seriesNameLower = media.name.toLowerCase().trim();

    // Normalize titles for comparison (remove season info, special chars)
    const normalizeForComparison = (title: string): string => {
      return title
        .replace(/\s*\(season\s*\d+\)/gi, '')  // Remove (Season 1)
        .replace(/[^a-z0-9\s]/g, '')             // Remove special chars
        .replace(/\s+/g, ' ')                    // Normalize spaces
        .trim();
    };

    const normalizedCached = normalizeForComparison(cachedTitle);
    const normalizedSeries = normalizeForComparison(seriesNameLower);

    // Check if titles match (allowing for partial matches if series name is in cached title)
    // But be strict - if series name is substantial, require a good match
    const titlesMatch = normalizedCached === normalizedSeries ||
                       (normalizedSeries.length >= 5 && (
                         normalizedCached.includes(normalizedSeries) ||
                         normalizedSeries.includes(normalizedCached)
                       ));

    if (!titlesMatch && normalizedSeries.length >= 3) {
      // Titles don't match - metadata is likely wrong, re-fetch it
      logger.warn('metadata', `Cached metadata title "${cachedTitle}" doesn't match series name "${seriesNameLower}" — re-fetching`, { series: media.name });
      // Clear the cached metadata and fall through to fetch new metadata
      delete slice[mediaId];
      // Fall through to fetch new metadata
    } else if (titlesMatch) {
      logger.info('metadata', `Using cached metadata for: ${media.name}`, { series: media.name });

      // Ensure title includes season number if we have season-specific files
      let finalTitle = existing.title as string;
      if (media.seasonNumber !== null && media.seasonNumber !== undefined) {
        const seasonPattern = /\(Season\s*\d+\)/i;
        if (!seasonPattern.test(finalTitle)) {
          finalTitle = `${finalTitle} (Season ${media.seasonNumber})`;
        }
      }

      // Update file info but keep metadata
      slice[mediaId] = {
        ...existing,
        seriesId: mediaId,
        title: finalTitle,
        fileEpisodes: preserveStatus(mediaId, media.files.map(f => ({
          episodeNumber: f.episodeNumber,
          seasonNumber: f.seasonNumber,
          filePath: f.filePath,
          subtitlePath: f.subtitlePath,
          subtitlePaths: f.subtitlePaths,
          filename: f.filename,
          title: f.title,
          status: f.status,
          lastProbedAt: f.lastProbedAt,
        })), existingMetadata),
        folderPath: media.folderPath,
        type: media.type,
      };
      return slice;
    }
  }

  const seasonInfo = media.seasonNumber !== null ? ` Season ${media.seasonNumber}` : '';
  const partInfo = media.partNumber !== null ? ` Part ${media.partNumber}` : '';
  logger.info('metadata', `Processing ${media.name}${seasonInfo}${partInfo}`, { series: media.name });

  // Count only canonical episodes (exclude decimal episodes like 6.5, 7.5, 10.5)
  // Decimal episodes are stored as actual decimals: 6.5, 7.5, 10.5, etc.
  const canonicalEpisodes = media.files.filter(f => {
    // Skip decimal episodes: check if episodeNumber is not an integer
    return Number.isInteger(f.episodeNumber);
  });
  const canonicalEpisodeCount = canonicalEpisodes.length;

  logger.info('folder', `Folder has ${canonicalEpisodeCount} canonical episode${canonicalEpisodeCount !== 1 ? 's' : ''} (${media.files.length} total files including decimal episodes)`, { series: media.name });

  // Fetch new metadata. findBestMatch searches MAL + AniList in parallel,
  // scores every candidate against the folder name, and picks the best
  // (refusing if nothing clears the title-similarity threshold). Replaces
  // the old MAL-first AniList-fallback that mismatched fuzzy-similar shows.
  type FetchedShape = Record<string, unknown> & {
    title: string;
    poster?: string | null;
    banner?: string | null;
    episodes?: Array<{ episodeNumber: number; seasonNumber?: number | null; thumbnail?: string | null }>;
    totalEpisodes?: number | null;
  };
  const matchResult = await findBestMatch(media.name, media.seasonNumber, media.partNumber, canonicalEpisodeCount);
  let fetchedMetadata: FetchedShape | null = null;
  if (matchResult) {
    fetchedMetadata = matchResult.metadata as FetchedShape;
    logger.info(
      'metadata',
      `Found on ${matchResult.source.toUpperCase()}: ${fetchedMetadata.title} (${fetchedMetadata.totalEpisodes ?? 'unknown'} episodes, score ${matchResult.score.toFixed(2)})`,
      { series: fetchedMetadata.title },
    );
  } else {
    logger.warn('metadata', `No good match for ${media.name}${seasonInfo}`, { series: media.name });
  }

  if (fetchedMetadata) {
    // Cache images locally
    logger.info('image', `Caching images for: ${media.name}`, { series: media.name });

    // Collect all image URLs to cache
    const imagesToCache: (string | null)[] = [
      fetchedMetadata.poster ?? null,
      fetchedMetadata.banner ?? null,
    ];

    // Add episode thumbnails that exist online
    if (fetchedMetadata.episodes) {
      for (const ep of fetchedMetadata.episodes) {
        if (ep.thumbnail) {
          imagesToCache.push(ep.thumbnail);
        }
      }
    }

    // Cache all online images in parallel
    const cachedImages = await imageCacheHandler.cacheImages(imagesToCache);

    // Update metadata with local paths
    const posterLocal = fetchedMetadata.poster ? cachedImages.get(fetchedMetadata.poster) || null : null;
    const bannerLocal = fetchedMetadata.banner ? cachedImages.get(fetchedMetadata.banner) || null : null;

    // Create a map of file episodes by season and episode number for thumbnail generation
    // Use a composite key: "season_episode" or "null_episode" for episodes without season
    const fileEpisodeMap = new Map<string, string>();
    for (const f of media.files) {
      const key = f.seasonNumber !== null
        ? `${f.seasonNumber}_${f.episodeNumber}`
        : `null_${f.episodeNumber}`;
      fileEpisodeMap.set(key, f.filePath);
    }

    // Update episode thumbnails - use online if available, otherwise generate from video
    const episodesWithLocalThumbs = [];
    let firstThumbnailInBatch = true;
    for (const ep of fetchedMetadata.episodes || []) {
      let thumbnailLocal: string | null = null;

      if (ep.thumbnail) {
        // Use cached online thumbnail
        thumbnailLocal = cachedImages.get(ep.thumbnail) || null;
      }

      // If no online thumbnail, try to generate from video file
      // Match by season and episode number if available, otherwise by episode number only
      if (!thumbnailLocal) {
        const epSeason = ep.seasonNumber ?? null;
        const key = epSeason !== null
          ? `${epSeason}_${ep.episodeNumber}`
          : `null_${ep.episodeNumber}`;

        // Try exact match first (season + episode)
        let videoPath = fileEpisodeMap.get(key);

        // If no exact match and we have a season, try matching by episode number only
        // (in case the file has a different season number)
        if (!videoPath && epSeason !== null) {
          videoPath = fileEpisodeMap.get(`null_${ep.episodeNumber}`);
        }

        // If still no match, try any season with same episode number (including decimals)
        if (!videoPath) {
          for (const [mapKey, path] of fileEpisodeMap.entries()) {
            // Extract episode number from key (format: "season_episode" or "null_episode")
            const keyParts = mapKey.split('_');
            const keyEpisodeNum = parseFloat(keyParts[keyParts.length - 1]);
            // Match exact episode number (works for both integers and decimals)
            if (keyEpisodeNum === ep.episodeNumber) {
              videoPath = path;
              break;
            }
          }
        }

        if (videoPath) {
          thumbnailLocal = await thumbnailHandler.generateThumbnail(videoPath, 120, firstThumbnailInBatch);
          firstThumbnailInBatch = false;
        }
      }

      episodesWithLocalThumbs.push({
        ...ep,
        thumbnailLocal,
      });
    }

    // Generate thumbnails for decimal episodes (6.5, 7.5, etc.) that exist in files but not in metadata
    // These won't be in the metadata episodes list, so we need to handle them separately
    const processedEpisodeNumbers = new Set(episodesWithLocalThumbs.map(ep => ep.episodeNumber));
    for (const fileEp of media.files) {
      // Check if this is a decimal episode (not an integer) and not already processed
      if (!Number.isInteger(fileEp.episodeNumber) && !processedEpisodeNumbers.has(fileEp.episodeNumber)) {
        const epSeason = fileEp.seasonNumber ?? null;

        // Generate thumbnail for this decimal episode
        let thumbnailLocal: string | null = null;
        try {
          thumbnailLocal = await thumbnailHandler.generateThumbnail(fileEp.filePath, 120, false);
        } catch (err) {
          logger.warn('thumbnail', `Failed to generate thumbnail for decimal episode ${fileEp.episodeNumber}`, { file: fileEp.filePath });
        }

        // Add to episodes list (these won't have metadata, but will have file info)
        episodesWithLocalThumbs.push({
          episodeNumber: fileEp.episodeNumber,
          seasonNumber: epSeason ?? undefined,
          title: fileEp.episodeNumber === 0 ? 'Special' : `Episode ${fileEp.episodeNumber.toFixed(1)}`,
          description: null,
          airDate: null,
          thumbnail: null,
          thumbnailLocal,
        });
      }
    }

    // Ensure title includes season number if we have season-specific files
    let finalTitle = fetchedMetadata.title;
    if (media.seasonNumber !== null && media.seasonNumber !== undefined) {
      // Check if title already includes season info
      const seasonPattern = /\(Season\s*\d+\)/i;
      if (!seasonPattern.test(finalTitle)) {
        finalTitle = `${finalTitle} (Season ${media.seasonNumber})`;
      }
    }

    slice[mediaId] = {
      ...fetchedMetadata,
      seriesId: mediaId,
      title: finalTitle,
      posterLocal,
      bannerLocal,
      episodes: episodesWithLocalThumbs,
      fileEpisodes: preserveStatus(mediaId, media.files.map(f => ({
        episodeNumber: f.episodeNumber,
        seasonNumber: f.seasonNumber,
        filePath: f.filePath,
        subtitlePath: f.subtitlePath,
        subtitlePaths: f.subtitlePaths,
        filename: f.filename,
        title: f.title,
        status: f.status,
        lastProbedAt: f.lastProbedAt,
      })), existingMetadata),
      folderPath: media.folderPath,
      type: media.type,
    };
    logger.info('metadata', `Found: ${finalTitle}`, { series: finalTitle });
  } else {
    // No metadata found, use folder/file name
    logger.warn('metadata', `No online metadata, generating local thumbnails: ${media.name}`, { series: media.name });

    // Generate thumbnails from video files
    const localEpisodes = [];
    let firstThumbnail = true;
    for (const f of media.files) {
      const thumbnailLocal = await thumbnailHandler.generateThumbnail(f.filePath, 120, firstThumbnail);
      firstThumbnail = false;
      localEpisodes.push({
        episodeNumber: f.episodeNumber,
        seasonNumber: f.seasonNumber,
        title: f.title || (f.episodeNumber === 0 ? 'Special' : `Episode ${f.episodeNumber}`),
        description: null,
        airDate: null,
        thumbnail: null,
        thumbnailLocal,
      });
    }

    // Ensure title includes season number if we have season-specific files
    let localTitle = media.name;
    if (media.seasonNumber !== null && media.seasonNumber !== undefined) {
      const seasonPattern = /Season\s*\d+/i;
      if (!seasonPattern.test(localTitle)) {
        localTitle = `${localTitle} (Season ${media.seasonNumber})`;
      }
    }

    slice[mediaId] = {
      seriesId: mediaId,
      title: localTitle,
      description: '',
      genres: [],
      poster: null,
      posterLocal: null,
      banner: null,
      bannerLocal: null,
      episodes: localEpisodes,
      fileEpisodes: preserveStatus(mediaId, media.files.map(f => ({
        episodeNumber: f.episodeNumber,
        seasonNumber: f.seasonNumber,
        filePath: f.filePath,
        subtitlePath: f.subtitlePath,
        subtitlePaths: f.subtitlePaths,
        filename: f.filename,
        title: f.title,
        status: f.status,
        lastProbedAt: f.lastProbedAt,
      })), existingMetadata),
      folderPath: media.folderPath,
      type: media.type,
      source: 'local',
    };
  }

  // Small delay to avoid rate limiting
  await new Promise(resolve => setTimeout(resolve, 500));

  return slice;
}

// ==================== SCAN AND FETCH COMBINED ====================

async function runScanAndFetch(folderPath: string, activeRoots: string[]): Promise<{ success: boolean; count: number }> {
  logger.info('folder', `Starting scan and metadata fetch for: ${folderPath}`, { file: folderPath });

  // 1. Scan. Pass activeRoots so scanFolder treats a sub-folder (rescan-show)
  // as part of its containing library root, not as a fresh root itself.
  const scannedMedia = await folderHandler.scanFolder(folderPath, activeRoots);

  // 2. Reconcile against disk: drop fileEpisodes pointing at files that are
  // gone. Keeps metadata.json honest about what's on disk.
  await metadataHandler.transaction(async (raw) => {
    const reconciled = await folderHandler.reconcileMetadata(raw, activeRoots);
    return { updated: reconciled };
  });

  // 3. Filesystem-only pass: build/refresh fileEpisodes from disk for every
  // series. NO network, NO ffmpeg, NO thumbnails, NO posters. The renderer
  // doesn't need them — it walks the filesystem directly via library:walk
  // for display. metadata.json is kept in sync only because the player still
  // resolves files by seriesId. Metadata enrichment is paused.
  const scannedIds = new Set<string>();
  await metadataHandler.transaction(async (current) => {
    let touched = 0;
    for (const media of scannedMedia) {
      if (media.files.length === 0) continue;
      scannedIds.add(media.id);
      const existing = (current[media.id] ?? {}) as {
        fileEpisodes?: Array<{ filePath: string; status?: string; lastProbedAt?: number }>;
        title?: string;
        folderPath?: string;
      };

      const oldByPath = new Map(
        (existing.fileEpisodes ?? []).map((f) => [f.filePath, f]),
      );
      const newFileEpisodes = media.files.map((f) => {
        const old = oldByPath.get(f.filePath);
        return {
          episodeNumber: f.episodeNumber,
          seasonNumber: f.seasonNumber,
          filePath: f.filePath,
          subtitlePath: f.subtitlePath,
          subtitlePaths: f.subtitlePaths,
          filename: f.filename,
          title: f.title,
          status: old?.status ?? f.status,
          lastProbedAt: old?.lastProbedAt ?? f.lastProbedAt,
        };
      });

      const oldPaths = new Set(oldByPath.keys());
      const newPaths = new Set(newFileEpisodes.map((f) => f.filePath));
      const sameLength = oldPaths.size === newPaths.size;
      const sameContents = sameLength && [...newPaths].every((p) => oldPaths.has(p));
      if (sameContents && existing.folderPath === media.folderPath) continue;

      current[media.id] = {
        ...existing,
        seriesId: media.id,
        title: existing.title ?? media.name,
        fileEpisodes: newFileEpisodes,
        folderPath: media.folderPath,
        type: media.type,
      };
      touched++;
      logger.info('folder', `Sync: ${media.name} → ${newFileEpisodes.length} file(s)`, { series: media.name });
    }
    if (touched === 0) return { updated: null };
    return { updated: current };
  });
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send('metadata:file-status-changed', { filePath: '', status: 'ready' });
  }

  // 4. Cleanup: drop any pre-existing entries whose folderPath is under the
  // scanned folder but didn't show up in this scan — they're stale (deleted
  // series, orphan movie_* entries from earlier misclassifications, etc.).
  // Entries already replaced via the per-series writes above stay intact.
  const normalize = (p: string) => (p.endsWith('/') ? p.slice(0, -1) : p);
  const normalizedScan = normalize(folderPath);
  const scanPrefix = normalizedScan + '/';
  await metadataHandler.transaction(async (current) => {
    const merged: Record<string, unknown> = {};
    let dropped = 0;
    for (const [seriesId, seriesData] of Object.entries(current)) {
      const fp = (seriesData as { folderPath?: string })?.folderPath;
      const isUnderScan = fp && (normalize(fp) === normalizedScan || normalize(fp).startsWith(scanPrefix));
      if (!isUnderScan || scannedIds.has(seriesId)) {
        merged[seriesId] = seriesData;
      } else {
        dropped++;
      }
    }
    if (dropped === 0) return { updated: null };
    return { updated: merged };
  });

  logger.info('folder', `Scan complete! Found ${scannedMedia.length} items`);
  return { success: true, count: scannedMedia.length };
}

ipcMain.handle('scan-and-fetch-metadata', async (_event, folderPath: string) => {
  try {
    const activeRoots = await configHandler.getFolderSources();
    return await runScanAndFetch(folderPath, activeRoots);
  } catch (error) {
    logger.error('folder', 'Error in scan-and-fetch-metadata');
    throw error;
  }
});

ipcMain.handle('get-series-episodes', async (_event, seriesId: string) => {
  try {
    const metadata = await metadataHandler.loadMetadata();
    const series = metadata[seriesId] as { episodes?: unknown[] } | undefined;
    return series?.episodes || [];
  } catch (error) {
    logger.error('metadata', 'Error getting series episodes');
    return [];
  }
});

app.on('before-quit', () => {
  videoProbeHandler.stop();
  void fileWatcher.stop();
});

