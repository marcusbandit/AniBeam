// The anibeam-export format: one JSON document, written by Electron and read
// by every core version from here on, so it doubles as the native app's
// backup format. Fixed in https://github.com/marcusbandit/AniBeam/issues/11.
//
// An unticked export carries sources and every series' match; ticked adds
// accounts, the TMDB key, history and preferences. Everything private is
// plain text, tokens included: the checkbox is the only guard, there's no
// encryption.

import { app } from 'electron';
import configHandler from './configHandler';
import metadataHandler from './metadataHandler';
import {
  getAccount,
  getAccessToken,
  getClientSecret,
  getMainProvider,
  getRefreshToken,
  type TrackerProvider,
} from '../services/trackerStore';
import { getViewHistory } from '../services/viewHistory';
import type { RendererExportState } from '../preload';

export const EXPORT_FORMAT = 'anibeam-export';
export const EXPORT_VERSION = 1;

export type ExportMatch =
  | { provider: 'anilist' | 'mal'; anilistId: number | null; malId: number | null }
  | { provider: 'tmdb'; tmdbId: number; tmdbKind: 'movie' | 'tv' }
  | null;

export interface ExportSeries {
  kind: 'series' | 'movie';
  path: string;
  id: string;
  title: string;
  hidden: boolean;
  match: ExportMatch;
}

export interface ExportAccount {
  userId: number | null;
  username: string | null;
  clientId: string;
  clientSecret: string | null;
  accessToken: string;
  refreshToken: string | null;
  expiresAt: number | null;
}

export interface ExportHistoryView {
  series: string;
  lastEpisode: number;
  at: string;
}

export interface ExportHistoryCompleted {
  series: string;
  episode: number;
  at: string;
}

export type ExportResumePoint =
  | { series: string; episode: number; position: number; duration: number; at: string }
  | { file: string; position: number; duration: number; at: string };

export interface ExportPreferences {
  titleLanguage: 'romaji' | 'english';
  libraryTab: string;
  librarySort: { key: string; direction: string };
  feedSort: string;
}

export interface AnibeamExportV1 {
  format: typeof EXPORT_FORMAT;
  version: typeof EXPORT_VERSION;
  exportedAt: string;
  exportedBy: { app: 'anibeam'; line: 'electron'; version: string };
  private: boolean;
  sources: { path: string }[];
  series: ExportSeries[];
  accounts?: { main: TrackerProvider; anilist: ExportAccount | null; mal: ExportAccount | null };
  keys?: { tmdb: string | null };
  history?: {
    views: ExportHistoryView[];
    completed: ExportHistoryCompleted[];
    resumePoints: ExportResumePoint[];
  };
  preferences?: ExportPreferences;
}

// The metadata.json record shape is untyped elsewhere in the codebase
// (metadataHandler treats it as Record<string, unknown> throughout), so this
// mirrors just the fields the export reads.
interface RawSeriesRecord {
  folderPath?: string;
  type?: 'series' | 'movie';
  title?: string;
  hidden?: boolean;
  source?: string;
  anilistId?: number | null;
  malId?: number | null;
  tmdbId?: number;
  tmdbKind?: 'movie' | 'tv';
  fileEpisodes?: Array<{ filePath: string }>;
}

function parseJson<T>(raw: string | null | undefined): T | null {
  if (!raw) return null;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

async function buildAccount(provider: TrackerProvider): Promise<ExportAccount | null> {
  const account = await getAccount(provider);
  if (!account) return null;
  const clientSecret = await getClientSecret(provider);
  const accessToken = await getAccessToken(provider);
  const refreshToken = await getRefreshToken(provider);
  return {
    userId: account.userId,
    username: account.username,
    clientId: account.clientId,
    clientSecret: clientSecret || null,
    accessToken: accessToken ?? '',
    refreshToken,
    expiresAt: account.expiresAt,
  };
}

export function defaultExportFileName(includePrivate: boolean): string {
  const date = new Date().toISOString().slice(0, 10);
  return includePrivate ? `anibeam-export-full-${date}.json` : `anibeam-export-${date}.json`;
}

export async function buildExport(
  includePrivate: boolean,
  rendererState: RendererExportState,
): Promise<AnibeamExportV1> {
  const config = await configHandler.loadConfig();
  const metadata = await metadataHandler.loadMetadata();

  // Identity is kind plus path: a show is its folder, a film is its file,
  // because several films share one "Movies" folder. Keep the resolved path
  // per seriesId so the history sections below can look series up by id.
  const seriesPathById = new Map<string, string>();
  const series: ExportSeries[] = [];

  for (const [seriesId, raw] of Object.entries(metadata)) {
    const record = raw as RawSeriesRecord;
    const kind: 'series' | 'movie' = record.type === 'movie' ? 'movie' : 'series';
    const path = kind === 'movie'
      ? record.fileEpisodes?.[0]?.filePath ?? record.folderPath ?? ''
      : record.folderPath ?? '';
    if (!path) continue;

    seriesPathById.set(seriesId, path);

    // Only the provider named by `source` is written: applying an AniList
    // match leaves a stale tmdbId behind, and vice versa.
    let match: ExportMatch = null;
    if (record.source === 'tmdb' && typeof record.tmdbId === 'number' && record.tmdbKind) {
      match = { provider: 'tmdb', tmdbId: record.tmdbId, tmdbKind: record.tmdbKind };
    } else if (record.source === 'anilist' || record.source === 'mal') {
      match = {
        provider: record.source,
        anilistId: record.anilistId ?? null,
        malId: record.malId ?? null,
      };
    }

    series.push({
      kind,
      path,
      id: seriesId,
      title: record.title ?? seriesId,
      hidden: record.hidden === true,
      match,
    });
  }

  const exported: AnibeamExportV1 = {
    format: EXPORT_FORMAT,
    version: EXPORT_VERSION,
    exportedAt: new Date().toISOString(),
    exportedBy: { app: 'anibeam', line: 'electron', version: app.getVersion() },
    private: includePrivate,
    sources: config.folderSources.map((path) => ({ path })),
    series,
  };

  if (!includePrivate) return exported;

  const [mainProvider, anilistAccount, malAccount] = await Promise.all([
    getMainProvider(),
    buildAccount('anilist'),
    buildAccount('mal'),
  ]);
  exported.accounts = { main: mainProvider, anilist: anilistAccount, mal: malAccount };

  const tmdbKey = (config.tmdbApiKey ?? '').trim();
  exported.keys = { tmdb: tmdbKey.length > 0 ? tmdbKey : null };

  const viewHistory = await getViewHistory();
  const views: ExportHistoryView[] = [];
  for (const [seriesId, entry] of Object.entries(viewHistory)) {
    const path = seriesPathById.get(seriesId);
    if (!path) continue;
    views.push({
      series: path,
      lastEpisode: entry.lastEpisode,
      at: new Date(entry.lastViewedAt).toISOString(),
    });
  }

  const lastEpMap = parseJson<Record<string, { ep: number; updated: number }>>(rendererState.videoLastEpisode) ?? {};
  const completed: ExportHistoryCompleted[] = [];
  for (const [seriesId, entry] of Object.entries(lastEpMap)) {
    const path = seriesPathById.get(seriesId);
    if (!path) continue;
    completed.push({ series: path, episode: entry.ep, at: new Date(entry.updated).toISOString() });
  }

  // Progress keys are `${seriesId}::${episodeNumber}` for a real episode, or
  // `${seriesId}::x:${filePath}` for an extra (OP/ED/PV/SP or a film), which
  // shares an episode number with a real episode and so is keyed by its file
  // instead. See src/renderer/utils/playbackProgress.ts.
  const progressMap = parseJson<Record<string, { t: number; d: number; updated: number }>>(rendererState.videoProgress) ?? {};
  const resumePoints: ExportResumePoint[] = [];
  for (const [key, entry] of Object.entries(progressMap)) {
    const sep = key.indexOf('::');
    if (sep === -1) continue;
    const seriesId = key.slice(0, sep);
    const rest = key.slice(sep + 2);
    const at = new Date(entry.updated).toISOString();
    if (rest.startsWith('x:')) {
      resumePoints.push({ file: rest.slice(2), position: entry.t, duration: entry.d, at });
      continue;
    }
    const path = seriesPathById.get(seriesId);
    if (!path) continue;
    const episode = Number(rest);
    if (!Number.isFinite(episode)) continue;
    resumePoints.push({ series: path, episode, position: entry.t, duration: entry.d, at });
  }

  exported.history = { views, completed, resumePoints };
  exported.preferences = {
    titleLanguage: rendererState.titleLanguage === 'EN' ? 'english' : 'romaji',
    libraryTab: rendererState.libraryTab ?? 'all',
    librarySort: {
      key: rendererState.librarySortKey ?? 'alpha',
      direction: rendererState.librarySortDir ?? 'asc',
    },
    feedSort: rendererState.feedSort ?? 'recent',
  };

  return exported;
}
