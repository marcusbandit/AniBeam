// Folder-name → MAL/AniList poster match. Used during the
// metadata-paused phase: only the poster URL is fetched, nothing else.
//
// Folder name goes in VERBATIM. No stripping. The user keeps folder
// names clean — that's the contract. We compare the folder name's
// tokens against each result's romaji + english title via the existing
// Dice similarity helper, take the max, and accept iff >= THRESHOLD.
//
// Source preference: MAL first (per Liam's spec). AniList only runs if
// MAL doesn't clear the threshold. We do NOT merge results — first
// confident match wins.

import malHandler from '../handlers/malHandler';
import anilistHandler from '../handlers/anilistHandler';
import { bestTitleScore } from './titleSimilarity';
import { logger } from '../services/logger';

const THRESHOLD = 0.95;

export interface ShowMatch {
  source: 'mal' | 'anilist';       // which provider scored ≥ THRESHOLD first
  anilistId: number | null;        // populated for AniList primary matches; cross-resolved for MAL
  malId: number | null;            // populated for MAL primary matches; cross-referenced via AniList's idMal
  matchedTitle: string;            // primary romaji-ish title (back-compat)
  titleRomaji: string | null;      // explicit romaji form
  titleEnglish: string | null;     // English localization, when available
  posterUrl: string;
  score: number;
  status: string | null;        // raw from source — caller normalizes if needed
  startDate: string | null;     // YYYY-MM-DD
  totalEpisodes: number | null;
}

export interface EpisodeAirDate {
  episodeNumber: number;
  airDate: string | null;
}

function aniListDate(d: { year: number | null; month: number | null; day: number | null } | null | undefined): string | null {
  if (!d?.year) return null;
  const m = String(d.month ?? 1).padStart(2, '0');
  const day = String(d.day ?? 1).padStart(2, '0');
  return `${d.year}-${m}-${day}`;
}

export async function findShowMatch(folderName: string): Promise<ShowMatch | null> {
  // 1. MAL via Jikan.
  try {
    const malResults = await malHandler.searchAnime(folderName, 10);
    for (const r of malResults) {
      const score = bestTitleScore(folderName, [r.title, r.title_english]);
      if (score >= THRESHOLD) {
        const poster = r.images?.jpg?.large_image_url ?? r.images?.jpg?.image_url ?? null;
        if (poster) {
          logger.info('metadata', `Match (MAL ${score.toFixed(2)}): ${folderName} → ${r.title}`, { series: folderName });
          // AniList accepts an idMal filter, so we can grab the AniList id
          // for free with a single extra query. Best-effort — null on
          // failure means the MAL tracker still works, just not AniList.
          const anilistId = await anilistHandler.resolveAnilistIdByMal(r.mal_id);
          return {
            source: 'mal',
            anilistId,
            malId: r.mal_id,
            matchedTitle: r.title,
            titleRomaji: r.title,
            titleEnglish: r.title_english ?? null,
            posterUrl: poster,
            score,
            status: r.status ?? null,
            startDate: r.aired?.from ? new Date(r.aired.from).toISOString().split('T')[0] : null,
            totalEpisodes: r.episodes ?? null,
          };
        }
      }
    }
  } catch (err) {
    logger.warn('metadata', `MAL search failed for ${folderName}: ${(err as Error).message}`, { series: folderName });
  }

  // 2. AniList fallback.
  try {
    const aniResults = await anilistHandler.searchAnimeMultiple(folderName, 10);
    for (const r of aniResults) {
      const score = bestTitleScore(folderName, [r.title?.romaji, r.title?.english]);
      if (score >= THRESHOLD) {
        const poster = r.coverImage?.extraLarge ?? r.coverImage?.large ?? null;
        if (poster) {
          const matchedTitle = r.title?.romaji ?? r.title?.english ?? '?';
          logger.info('metadata', `Match (AniList ${score.toFixed(2)}): ${folderName} → ${matchedTitle}`, { series: folderName });
          return {
            source: 'anilist',
            anilistId: r.id,
            // SEARCH_MULTIPLE_QUERY now includes idMal — null when AniList
            // has no MAL cross-reference for this entry.
            malId: r.idMal ?? null,
            matchedTitle,
            titleRomaji: r.title?.romaji ?? null,
            titleEnglish: r.title?.english ?? null,
            posterUrl: poster,
            score,
            status: r.status ?? null,
            startDate: aniListDate(r.startDate),
            totalEpisodes: r.episodes ?? null,
          };
        }
      }
    }
  } catch (err) {
    logger.warn('metadata', `AniList search failed for ${folderName}: ${(err as Error).message}`, { series: folderName });
  }

  logger.info('metadata', `No match for ${folderName} (threshold ${THRESHOLD})`, { series: folderName });
  return null;
}

/**
 * Pull per-episode air dates for a matched show. AniList airingSchedule
 * is the primary source — it's prompt for current shows and can be
 * queried by AniList id OR by MAL id (so MAL-matched shows still get
 * accurate dates without a second title search).
 *
 * MAL `/episodes` is a fallback for older / completed shows where
 * AniList's schedule may be empty.
 */
export async function fetchEpisodeAirDates(
  source: 'mal' | 'anilist',
  externalId: number,
  totalEpisodes: number | null,
): Promise<EpisodeAirDate[]> {
  // 1. AniList airingSchedule (preferred).
  try {
    const nodes = await anilistHandler.getAiringSchedule(
      source === 'anilist' ? { anilistId: externalId } : { malId: externalId },
    );
    if (nodes.length > 0) {
      // airingAt is Unix seconds (UTC). Convert to ISO so the rest of the
      // app, which already speaks ISO date strings, doesn't have to care.
      return nodes
        .filter((n) => Number.isFinite(n.airingAt) && n.airingAt > 0)
        .map((n) => ({
          episodeNumber: n.episode,
          airDate: new Date(n.airingAt * 1000).toISOString(),
        }));
    }
  } catch (err) {
    logger.warn('metadata', `AniList airingSchedule failed: ${(err as Error).message}`);
  }

  // 2. MAL /episodes fallback.
  if (source === 'mal') {
    try {
      const eps = await malHandler.getEpisodes(externalId, totalEpisodes);
      return eps.map((e) => ({ episodeNumber: e.episodeNumber, airDate: e.airDate }));
    } catch (err) {
      logger.warn('metadata', `MAL episodes fetch failed for ${externalId}: ${(err as Error).message}`);
    }
  }

  return [];
}
