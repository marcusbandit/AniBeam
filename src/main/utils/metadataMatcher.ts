// Best-match orchestrator for series metadata.
//
// Replaces the old "MAL first, AniList fallback, accept first result with
// enough episodes" flow that mismatched fuzzy-similar shows (e.g. matched
// "Otaku ni Yasashii Gal wa Inai" → "Wotaku ni Koi wa Muzukashii"). Now:
//   1. Search both providers in parallel.
//   2. Score every candidate's title variants against the folder name with
//      a tokenized Jaccard similarity (titleSimilarity.bestTitleScore).
//   3. Filter by release status and episode-count viability.
//   4. Pick the highest score. Refuse to match below MIN_TITLE_SCORE so
//      bad data stops landing in metadata.json.
//
// On equal scores we prefer AniList; MAL is the fallback. AniList's
// schema (idMal, banners, season/year) is what the rest of the app is
// built around — picking it on ties keeps malId populated for AniSkip
// and avoids round-trips when the renderer needs AniList-only fields.

import malHandler from '../handlers/malHandler';
import anilistHandler from '../handlers/anilistHandler';
import { logger } from '../services/logger';
import { bestTitleScore } from './titleSimilarity';

const MIN_TITLE_SCORE = 0.4;
const SEARCH_LIMIT = 10;

export interface BestMatchResult {
  metadata: Record<string, unknown>;
  source: 'mal' | 'anilist';
  score: number;
}

interface MalCandidate {
  source: 'mal';
  result: {
    mal_id: number;
    title: string;
    title_english: string | null;
    title_japanese: string | null;
    episodes: number | null;
    status: string;
  };
  score: number;
  episodes: number | null;
  released: boolean;
}

interface AnilistCandidate {
  source: 'anilist';
  result: {
    id: number;
    title: { romaji: string; english: string | null; native: string };
    episodes: number | null;
    status: string;
  };
  score: number;
  episodes: number | null;
  released: boolean;
}

type Candidate = MalCandidate | AnilistCandidate;

function malReleased(status: string): boolean {
  const s = (status || '').toLowerCase();
  return !(s.includes('not yet') || s.includes('not aired'));
}

function anilistReleased(status: string): boolean {
  const s = (status || '').toUpperCase();
  return !['NOT_YET_RELEASED', 'CANCELLED', 'HIATUS'].includes(s);
}

function candidateTitle(c: Candidate): string {
  if (c.source === 'mal') {
    return c.result.title || c.result.title_english || c.result.title_japanese || '?';
  }
  return c.result.title.english || c.result.title.romaji || c.result.title.native || '?';
}

async function searchBoth(query: string): Promise<{ mal: MalCandidate['result'][]; anilist: AnilistCandidate['result'][] }> {
  const [malResults, anilistResults] = await Promise.all([
    malHandler.searchAnime(query, SEARCH_LIMIT).catch((err) => {
      logger.warn('metadata', `MAL search failed for "${query}": ${(err as Error).message}`);
      return [] as MalCandidate['result'][];
    }),
    anilistHandler.searchAnimeMultiple(query, SEARCH_LIMIT).catch((err) => {
      logger.warn('metadata', `AniList search failed for "${query}": ${(err as Error).message}`);
      return [] as AnilistCandidate['result'][];
    }),
  ]);
  return { mal: malResults, anilist: anilistResults };
}

function buildCandidates(
  seriesName: string,
  malResults: MalCandidate['result'][],
  anilistResults: AnilistCandidate['result'][],
): Candidate[] {
  const out: Candidate[] = [];
  for (const r of malResults) {
    out.push({
      source: 'mal',
      result: r,
      score: bestTitleScore(seriesName, [r.title, r.title_english, r.title_japanese]),
      episodes: r.episodes,
      released: malReleased(r.status),
    });
  }
  for (const r of anilistResults) {
    out.push({
      source: 'anilist',
      result: r,
      score: bestTitleScore(seriesName, [r.title.romaji, r.title.english, r.title.native]),
      episodes: r.episodes,
      released: anilistReleased(r.status),
    });
  }
  return out;
}

function pickWinner(candidates: Candidate[], folderEpisodeCount: number): Candidate | null {
  // Strict tier: released + episode count covers what's on disk.
  const strict = candidates
    .filter((c) => c.released && (folderEpisodeCount === 0 || (c.episodes !== null && c.episodes >= folderEpisodeCount)))
    .sort((a, b) => {
      if (b.score !== a.score) return b.score - a.score;
      return a.source === 'anilist' ? -1 : 1;
    });
  if (strict.length > 0) return strict[0];

  // Fallback tier: released, episode count unknown. Same scoring + tie-break.
  const loose = candidates
    .filter((c) => c.released && c.episodes === null)
    .sort((a, b) => {
      if (b.score !== a.score) return b.score - a.score;
      return a.source === 'anilist' ? -1 : 1;
    });
  return loose[0] ?? null;
}

async function fetchFullMetadata(
  winner: Candidate,
  seasonNumber: number | null | undefined,
): Promise<BestMatchResult | null> {
  if (winner.source === 'mal') {
    const a = winner.result;
    try {
      const episodes = await malHandler.getEpisodes(a.mal_id, a.episodes, seasonNumber);
      // Cast through unknown — the handler-local JikanAnime is not exported,
      // and the orchestrator already extracted just what it needed for
      // scoring. formatMetadata only reads the fields a JikanAnime has.
      const formatted = malHandler.formatMetadata(a as unknown as Parameters<typeof malHandler.formatMetadata>[0], episodes, seasonNumber);
      return {
        source: 'mal',
        score: winner.score,
        metadata: { ...formatted, source: 'mal' } as Record<string, unknown>,
      };
    } catch (err) {
      logger.error('metadata', `MAL fetch failed for id ${a.mal_id}: ${(err as Error).message}`);
      return null;
    }
  }
  const m = winner.result;
  try {
    const episodes = await anilistHandler.getEpisodes(m.id, m.episodes, seasonNumber);
    const formatted = anilistHandler.formatMetadata(
      m as unknown as Parameters<typeof anilistHandler.formatMetadata>[0],
      episodes,
      seasonNumber,
    );
    return {
      source: 'anilist',
      score: winner.score,
      metadata: { ...formatted, source: 'anilist' } as Record<string, unknown>,
    };
  } catch (err) {
    logger.error('metadata', `AniList fetch failed for id ${m.id}: ${(err as Error).message}`);
    return null;
  }
}

/**
 * Search MAL + AniList in parallel and return the best-scoring metadata
 * for `seriesName`. Returns null when nothing clears MIN_TITLE_SCORE.
 *
 * @param seriesName        Folder-derived name (used for scoring; never has
 *                          `Season N` / `Part N` appended even if those are
 *                          passed separately).
 * @param seasonNumber      Folder season number (used to refine the search
 *                          query when > 1, and forwarded to formatMetadata).
 * @param partNumber        Folder part number (overrides season for the
 *                          search query when > 1).
 * @param folderEpisodeCount Number of canonical episodes on disk; results
 *                           with fewer episodes than this are excluded from
 *                           the strict tier.
 */
export async function findBestMatch(
  seriesName: string,
  seasonNumber: number | null | undefined,
  partNumber: number | null | undefined,
  folderEpisodeCount: number | undefined,
): Promise<BestMatchResult | null> {
  // Folder name goes to both providers verbatim. Season / Part are NOT
  // appended — the folder string already carries them if relevant, and
  // appending would double-tag (e.g. "Frieren Season 2 Season 2"). The
  // seasonNumber / partNumber args are still used downstream for title
  // suffixing and id generation.
  const searchQuery = seriesName;
  const wantEpCount = typeof folderEpisodeCount === 'number' ? folderEpisodeCount : 0;
  void partNumber;

  const { mal, anilist } = await searchBoth(searchQuery);
  const candidates = buildCandidates(seriesName, mal, anilist);

  if (candidates.length === 0) {
    logger.warn('metadata', `No candidates for "${searchQuery}"`);
    return null;
  }

  const winner = pickWinner(candidates, wantEpCount);
  if (!winner) {
    logger.warn('metadata', `No eligible candidates for "${searchQuery}" (released + ep ≥ ${wantEpCount})`);
    return null;
  }

  if (winner.score < MIN_TITLE_SCORE) {
    logger.warn(
      'metadata',
      `Best candidate "${candidateTitle(winner)}" scored ${winner.score.toFixed(2)} (< ${MIN_TITLE_SCORE}); refusing to match "${seriesName}"`,
    );
    return null;
  }

  logger.info(
    'metadata',
    `Best match for "${seriesName}": ${winner.source.toUpperCase()} "${candidateTitle(winner)}" (${winner.score.toFixed(2)}, ${winner.episodes ?? '?'} ep)`,
  );

  return fetchFullMetadata(winner, seasonNumber);
}
