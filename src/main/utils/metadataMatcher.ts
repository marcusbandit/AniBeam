// Best-match orchestrator for series metadata.
//
// AniList is the ONLY metadata source, by explicit user decision
// (2026-07-14). MAL (Jikan) was removed as a matching/fetching provider;
// do not re-add it. (Jikan survives solely as the per-episode title
// side-fetch in malHandler.getEpisodes, which is not a metadata source.)
//
// Contract (mirrors posterMatch.ts findShowMatch):
//   1. Search AniList only. Its relevance-ordered search is the same list
//      the manual match picker shows, so the auto-matcher trusts it.
//   2. Score every candidate's title variants (romaji, english, native,
//      synonyms) against the folder name with tokenized similarity
//      (titleSimilarity.bestTitleScore), filter by release status and
//      episode-count viability, pick the highest score.
//   3. Refuse to match below MIN_TITLE_SCORE so bad data stops landing
//      in metadata.json.

import anilistHandler from '../handlers/anilistHandler';
import { logger } from '../services/logger';
import { bestTitleScore } from './titleSimilarity';

const MIN_TITLE_SCORE = 0.4;
const SEARCH_LIMIT = 10;

export interface BestMatchResult {
  metadata: Record<string, unknown>;
  source: 'anilist';
  score: number;
}

interface Candidate {
  result: {
    id: number;
    title: { romaji: string; english: string | null; native: string };
    synonyms?: string[];
    episodes: number | null;
    status: string;
  };
  score: number;
  episodes: number | null;
  released: boolean;
}

function anilistReleased(status: string): boolean {
  const s = (status || '').toUpperCase();
  return !['NOT_YET_RELEASED', 'CANCELLED', 'HIATUS'].includes(s);
}

function candidateTitle(c: Candidate): string {
  return c.result.title.english || c.result.title.romaji || c.result.title.native || '?';
}

// The search degrades to "no candidates" on failure: a dead provider must
// never crash the matcher; the series just stays unmatched for manual
// recovery via the Metadata tab.
async function searchAnilist(query: string): Promise<Candidate['result'][]> {
  try {
    return await anilistHandler.searchAnimeMultiple(query, SEARCH_LIMIT);
  } catch (err) {
    logger.warn('metadata', `AniList search failed for "${query}": ${(err as Error).message}`);
    return [];
  }
}

function buildCandidates(seriesName: string, anilistResults: Candidate['result'][]): Candidate[] {
  return anilistResults.map((r) => ({
    result: r,
    score: bestTitleScore(seriesName, [r.title.romaji, r.title.english, r.title.native, ...(r.synonyms ?? [])]),
    episodes: r.episodes,
    released: anilistReleased(r.status),
  }));
}

function pickWinner(candidates: Candidate[], folderEpisodeCount: number): Candidate | null {
  // Score ties keep AniList's relevance order (Array.prototype.sort is
  // stable), matching what the manual picker would surface first.
  // Strict tier: released + episode count covers what's on disk.
  const strict = candidates
    .filter((c) => c.released && (folderEpisodeCount === 0 || (c.episodes !== null && c.episodes >= folderEpisodeCount)))
    .sort((a, b) => b.score - a.score);
  if (strict.length > 0) return strict[0];

  // Fallback tier: released, episode count unknown. Same scoring.
  const loose = candidates
    .filter((c) => c.released && c.episodes === null)
    .sort((a, b) => b.score - a.score);
  return loose[0] ?? null;
}

async function fetchFullMetadata(
  winner: Candidate,
  seasonNumber: number | null | undefined,
): Promise<BestMatchResult | null> {
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

function logWinner(seriesName: string, winner: Candidate): void {
  logger.info(
    'metadata',
    `Best match for "${seriesName}": AniList "${candidateTitle(winner)}" (${winner.score.toFixed(2)}, ${winner.episodes ?? '?'} ep)`,
  );
}

/**
 * AniList-only best match for `seriesName`. Returns null when nothing
 * clears MIN_TITLE_SCORE.
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
  // Folder name goes to AniList verbatim. Season / Part are NOT appended:
  // the folder string already carries them if relevant, and appending
  // would double-tag (e.g. "Frieren Season 2 Season 2"). The seasonNumber
  // / partNumber args are still used downstream for title suffixing and
  // id generation.
  const searchQuery = seriesName;
  const wantEpCount = typeof folderEpisodeCount === 'number' ? folderEpisodeCount : 0;
  void partNumber;

  const candidates = buildCandidates(seriesName, await searchAnilist(searchQuery));
  const winner = pickWinner(candidates, wantEpCount);
  if (winner && winner.score >= MIN_TITLE_SCORE) {
    logWinner(seriesName, winner);
    return fetchFullMetadata(winner, seasonNumber);
  }

  // Nothing acceptable. One signal-level warn stating why: no candidates
  // at all, none eligible, or best score under the bar.
  if (candidates.length === 0) {
    logger.warn('metadata', `No candidates for "${searchQuery}"`);
    return null;
  }

  if (!winner) {
    logger.warn('metadata', `No eligible candidates for "${searchQuery}" (released + ep >= ${wantEpCount})`);
    return null;
  }

  logger.warn(
    'metadata',
    `Best candidate "${candidateTitle(winner)}" scored ${winner.score.toFixed(2)} (< ${MIN_TITLE_SCORE}); refusing to match "${seriesName}"`,
  );
  return null;
}
