// Folder-name -> AniList background auto-match.
//
// AniList is the ONLY matching provider, by explicit user decision
// (2026-07-14). MAL (Jikan) was removed as a metadata source; do not
// re-add it. Jikan survives in this file solely as the per-episode
// title/air-date side-fetch in fetchEpisodeAirDates below.
//
// This mirrors the manual match picker: it queries AniList's
// relevance-ordered search (the same list the user sees when matching by
// hand), the top of which is usually the correct show.
//
// Folder name goes in VERBATIM. No stripping. The user keeps folder
// names clean, that's the contract.
//
// Each candidate is scored against romaji + english + native + synonyms
// via tokenized Dice similarity (bestTitleScore). We take the
// BEST-scoring candidate, breaking ties toward AniList's relevance order
// (an earlier result wins). Accept iff the best score >= THRESHOLD (0.5):
// a "pretty close" bar. Anything further off returns null and is left for
// manual matching.

import malHandler from "../handlers/malHandler";
import anilistHandler from "../handlers/anilistHandler";
import { bestTitleScore } from "./titleSimilarity";
import { logger } from "../services/logger";

const THRESHOLD = 0.5;

export interface ShowMatch {
  source: "anilist"; // AniList is the only matching provider
  anilistId: number | null; // the matched AniList media id
  malId: number | null; // AniList's idMal cross-reference; kept for AniSkip + Jikan episode titles
  matchedTitle: string; // primary romaji-ish title (back-compat)
  titleRomaji: string | null; // explicit romaji form
  titleEnglish: string | null; // English localization, when available
  posterUrl: string;
  score: number;
  status: string | null; // raw from source — caller normalizes if needed
  startDate: string | null; // YYYY-MM-DD
  totalEpisodes: number | null;
}

export interface EpisodeAirDate {
  episodeNumber: number;
  airDate: string | null;
  /** Per-episode title from MAL/Jikan when available — AniList's
   *  airingSchedule doesn't carry titles, so we side-fetch from Jikan
   *  whenever we know a malId. Null when no title source returned one. */
  title: string | null;
}

function aniListDate(
  d:
    | { year: number | null; month: number | null; day: number | null }
    | null
    | undefined,
): string | null {
  if (!d?.year) return null;
  const m = String(d.month ?? 1).padStart(2, "0");
  const day = String(d.day ?? 1).padStart(2, "0");
  return `${d.year}-${m}-${day}`;
}

export async function findShowMatch(
  folderName: string,
): Promise<ShowMatch | null> {
  // 1. AniList relevance-ordered search (primary, the manual picker's list).
  //    Score every candidate and take the best; ties break toward the
  //    earlier (higher-relevance) result via strict greater-than.
  try {
    const aniResults = await anilistHandler.searchAnimeMultiple(folderName, 10);
    let bestScore = -1;
    let best: (typeof aniResults)[number] | null = null;
    for (const r of aniResults) {
      const score = bestTitleScore(folderName, [
        r.title?.romaji,
        r.title?.english,
        r.title?.native,
        ...(r.synonyms ?? []),
      ]);
      if (score > bestScore) {
        bestScore = score;
        best = r;
      }
    }
    if (best && bestScore >= THRESHOLD) {
      const poster =
        best.coverImage?.extraLarge ?? best.coverImage?.large ?? null;
      if (poster) {
        const matchedTitle = best.title?.romaji ?? best.title?.english ?? "?";
        logger.info(
          "metadata",
          `Match (AniList ${bestScore.toFixed(2)}): ${folderName} → ${matchedTitle}`,
          { series: folderName },
        );
        return {
          source: "anilist",
          anilistId: best.id,
          // SEARCH_MULTIPLE_QUERY includes idMal — null when AniList has no
          // MAL cross-reference for this entry.
          malId: best.idMal ?? null,
          matchedTitle,
          titleRomaji: best.title?.romaji ?? null,
          titleEnglish: best.title?.english ?? null,
          posterUrl: poster,
          score: bestScore,
          status: best.status ?? null,
          startDate: aniListDate(best.startDate),
          totalEpisodes: best.episodes ?? null,
        };
      }
    }
  } catch (err) {
    logger.warn(
      "metadata",
      `AniList search failed for ${folderName}: ${(err as Error).message}`,
      { series: folderName },
    );
  }

  logger.info(
    "metadata",
    `No match for ${folderName} (threshold ${THRESHOLD})`,
    { series: folderName },
  );
  return null;
}

/**
 * Pull per-episode metadata for a matched show. AniList airingSchedule is
 * the primary source for air dates: prompt for current shows and queryable
 * by AniList id OR by MAL id. Episode titles only come from Jikan
 * (AniList's airing schedule doesn't carry them), so when a malId is known
 * we side-fetch and merge titles in by episodeNumber. This Jikan call is
 * an episode-title side-fetch, NOT a metadata source; it stays by design.
 *
 * Jikan `/episodes` is also the fallback for shows where AniList's schedule
 * is empty (older / completed runs).
 *
 * `source` keeps its union for the malId-primary query arm, but every
 * current caller passes "anilist": findShowMatch never produces "mal".
 */
export async function fetchEpisodeAirDates(
  source: "mal" | "anilist",
  externalId: number,
  totalEpisodes: number | null,
  malIdForTitles?: number | null,
): Promise<EpisodeAirDate[]> {
  // 1. AniList airingSchedule (preferred for dates).
  let fromAnilist: Array<{
    episodeNumber: number;
    airDate: string | null;
  }> | null = null;
  // The next broadcast, kept separate so it survives even when the schedule
  // page itself is empty or stale (see below).
  let nextAiring: { episodeNumber: number; airDate: string } | null = null;
  try {
    const schedule = await anilistHandler.getAiringSchedule(
      source === "anilist" ? { anilistId: externalId } : { malId: externalId },
    );
    if (schedule.nodes.length > 0) {
      fromAnilist = schedule.nodes
        .filter((n) => Number.isFinite(n.airingAt) && n.airingAt > 0)
        .map((n) => ({
          episodeNumber: n.episode,
          airDate: new Date(n.airingAt * 1000).toISOString(),
        }));
    }
    const next = schedule.nextAiringEpisode;
    if (next && Number.isFinite(next.airingAt) && next.airingAt > 0) {
      nextAiring = {
        episodeNumber: next.episode,
        airDate: new Date(next.airingAt * 1000).toISOString(),
      };
    }
  } catch (err) {
    logger.warn(
      "metadata",
      `AniList airingSchedule failed: ${(err as Error).message}`,
    );
  }

  // 2. MAL /episodes — used for titles whenever malId is known, and as the
  // air-date source when AniList didn't return any.
  const malId = source === "mal" ? externalId : (malIdForTitles ?? null);
  let fromMal: Array<{
    episodeNumber: number;
    title: string | null;
    airDate: string | null;
  }> | null = null;
  if (malId != null) {
    try {
      const eps = await malHandler.getEpisodes(malId, totalEpisodes);
      fromMal = eps.map((e) => ({
        episodeNumber: e.episodeNumber,
        // Jikan returns placeholder "Episode N" titles when the real title
        // isn't known — strip those so the renderer falls back to file-derived
        // titles (which often have the actual name parsed from filenames).
        title: e.title && !/^Episode\s+\d+$/i.test(e.title) ? e.title : null,
        airDate: e.airDate,
      }));
    } catch (err) {
      logger.warn(
        "metadata",
        `MAL episodes fetch failed for ${malId}: ${(err as Error).message}`,
      );
    }
  }

  // Merge — prefer AniList dates, MAL titles. Build the union of episode
  // numbers from whichever sources returned data, in case AniList covers
  // only the currently airing batch while MAL covers the full run.
  const byEp = new Map<number, EpisodeAirDate>();
  for (const a of fromAnilist ?? []) {
    byEp.set(a.episodeNumber, {
      episodeNumber: a.episodeNumber,
      airDate: a.airDate,
      title: null,
    });
  }
  for (const m of fromMal ?? []) {
    const existing = byEp.get(m.episodeNumber);
    if (existing) {
      // Keep AniList airDate (more accurate for currently airing), backfill
      // title from MAL.
      existing.title = m.title;
      if (!existing.airDate) existing.airDate = m.airDate;
    } else {
      byEp.set(m.episodeNumber, {
        episodeNumber: m.episodeNumber,
        airDate: m.airDate,
        title: m.title,
      });
    }
  }

  // Fold in the next broadcast last, and let it win on its own episode.
  // airingSchedule is paginated at 25 nodes and we hold one page, so for a
  // long-running series the upcoming episode is usually missing from every
  // list above (One Piece: page 1 is episodes 1123-1147 while episode 1172
  // is the one actually airing next). Without this the renderer's
  // findNextUpcomingEpisode has nothing in the future to find and the
  // countdown silently never appears.
  if (nextAiring) {
    const existing = byEp.get(nextAiring.episodeNumber);
    if (existing) {
      existing.airDate = nextAiring.airDate;
    } else {
      byEp.set(nextAiring.episodeNumber, {
        episodeNumber: nextAiring.episodeNumber,
        airDate: nextAiring.airDate,
        title: null,
      });
    }
  }

  return Array.from(byEp.values()).sort(
    (a, b) => a.episodeNumber - b.episodeNumber,
  );
}
