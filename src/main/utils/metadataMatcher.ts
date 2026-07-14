// Best-match orchestrator for series metadata.
//
// Contract (mirrors posterMatch.ts findShowMatch): AniList is primary, MAL
// is a fallback only. AniList's relevance-ordered search is the same list
// the manual match picker shows, so the auto-matcher trusts it first:
//   1. Search AniList only. Score every candidate's title variants
//      (romaji, english, native, synonyms) against the folder name with
//      tokenized similarity (titleSimilarity.bestTitleScore), filter by
//      release status and episode-count viability, pick the highest score.
//   2. If that winner clears MIN_TITLE_SCORE, AniList wins outright and
//      MAL is never consulted.
//   3. Only when AniList yields nothing acceptable, run the same search
//      and scoring against MAL (Jikan).
//   4. Refuse to match below MIN_TITLE_SCORE so bad data stops landing
//      in metadata.json.
//
// MAL-sourced metadata is brought to renderer parity: the AniList id is
// cross-resolved best-effort (resolveAnilistIdByMal) and included only
// when it resolves, so merges never clobber an existing good id.

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
    synonyms?: string[];
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

// Provider searches degrade to "no candidates" on failure: a dead provider
// must never sink the whole match (the other stage still gets its shot).
async function searchAnilist(query: string): Promise<AnilistCandidate['result'][]> {
  try {
    return await anilistHandler.searchAnimeMultiple(query, SEARCH_LIMIT);
  } catch (err) {
    logger.warn('metadata', `AniList search failed for "${query}": ${(err as Error).message}`);
    return [];
  }
}

async function searchMal(query: string): Promise<MalCandidate['result'][]> {
  try {
    return await malHandler.searchAnime(query, SEARCH_LIMIT);
  } catch (err) {
    logger.warn('metadata', `MAL search failed for "${query}": ${(err as Error).message}`);
    return [];
  }
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
      score: bestTitleScore(seriesName, [r.title.romaji, r.title.english, r.title.native, ...(r.synonyms ?? [])]),
      episodes: r.episodes,
      released: anilistReleased(r.status),
    });
  }
  return out;
}

function pickWinner(candidates: Candidate[], folderEpisodeCount: number): Candidate | null {
  // Candidate lists are single-provider now (AniList stage or MAL stage),
  // so the AniList tie-break below is inert; kept so the function stays
  // shared and order-stable.
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
      // Cast through unknown: the handler-local JikanAnime is not exported,
      // and the orchestrator already extracted just what it needed for
      // scoring. formatMetadata only reads the fields a JikanAnime has.
      const formatted = malHandler.formatMetadata(a as unknown as Parameters<typeof malHandler.formatMetadata>[0], episodes, seasonNumber);
      // Cross-resolve the AniList id best-effort (same as posterMatch.ts).
      // Include the key ONLY when it resolves: a null/undefined anilistId in
      // the payload would let a downstream merge clobber an existing good id.
      const anilistId = await anilistHandler.resolveAnilistIdByMal(a.mal_id).catch(() => null);
      return {
        source: 'mal',
        score: winner.score,
        metadata: { ...formatted, ...(anilistId != null ? { anilistId } : {}), source: 'mal' } as Record<string, unknown>,
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

function logWinner(seriesName: string, winner: Candidate): void {
  logger.info(
    'metadata',
    `Best match for "${seriesName}": ${winner.source.toUpperCase()} "${candidateTitle(winner)}" (${winner.score.toFixed(2)}, ${winner.episodes ?? '?'} ep)`,
  );
}

/**
 * AniList-first best match for `seriesName`, with MAL as the fallback when
 * AniList yields nothing acceptable (mirrors posterMatch.ts). Returns null
 * when nothing clears MIN_TITLE_SCORE.
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
  // Folder name goes to the providers verbatim. Season / Part are NOT
  // appended: the folder string already carries them if relevant, and
  // appending would double-tag (e.g. "Frieren Season 2 Season 2"). The
  // seasonNumber / partNumber args are still used downstream for title
  // suffixing and id generation.
  const searchQuery = seriesName;
  const wantEpCount = typeof folderEpisodeCount === 'number' ? folderEpisodeCount : 0;
  void partNumber;

  // Stage 1: AniList only (primary). Its relevance list is what the manual
  // picker shows; an acceptable winner here ends the hunt without touching
  // MAL at all.
  const anilistCandidates = buildCandidates(seriesName, [], await searchAnilist(searchQuery));
  const anilistWinner = pickWinner(anilistCandidates, wantEpCount);
  if (anilistWinner && anilistWinner.score >= MIN_TITLE_SCORE) {
    logWinner(seriesName, anilistWinner);
    return fetchFullMetadata(anilistWinner, seasonNumber);
  }

  // Stage 2: MAL fallback, reached only when AniList produced no winner at
  // or over MIN_TITLE_SCORE.
  const malCandidates = buildCandidates(seriesName, await searchMal(searchQuery), []);
  const malWinner = pickWinner(malCandidates, wantEpCount);
  if (malWinner && malWinner.score >= MIN_TITLE_SCORE) {
    logWinner(seriesName, malWinner);
    return fetchFullMetadata(malWinner, seasonNumber);
  }

  // Nothing acceptable on either provider. One signal-level warn stating
  // why: no candidates at all, none eligible, or best score under the bar.
  if (anilistCandidates.length === 0 && malCandidates.length === 0) {
    logger.warn('metadata', `No candidates for "${searchQuery}"`);
    return null;
  }

  const closest = [anilistWinner, malWinner]
    .filter((c): c is Candidate => c !== null)
    .sort((a, b) => b.score - a.score)[0];
  if (!closest) {
    logger.warn('metadata', `No eligible candidates for "${searchQuery}" (released + ep >= ${wantEpCount})`);
    return null;
  }

  logger.warn(
    'metadata',
    `Best candidate "${candidateTitle(closest)}" scored ${closest.score.toFixed(2)} (< ${MIN_TITLE_SCORE}); refusing to match "${seriesName}"`,
  );
  return null;
}
