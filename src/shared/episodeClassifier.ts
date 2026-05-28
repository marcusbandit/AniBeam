// Single source of truth for "what kind of file is this": a real episode,
// an opening/ending/promo/special, or something we can't classify. The
// upstream parser used to look at filename digits only — anything ending in
// a digit (ED1, OP3, PV07) collapsed onto "episode 1/3/7" and polluted the
// episode list. This module token-scans for the well-known release labels
// FIRST, then falls through to the legacy episode regex cascade.
//
// Pure string in → enum + indices out. No fs, no electron. Safe to call
// from main or renderer.

import { basename, extname } from 'path';

export type EpisodeKind = 'episode' | 'op' | 'ed' | 'pv' | 'sp' | 'other';

export interface ClassifiedFile {
  kind: EpisodeKind;
  // Episodes: real episode number (1, 2, 6.5, …). Decimals preserved.
  // OP/ED/PV/SP: the extras index lifted to this field too, so existing
  // code that sorts by episodeNumber still produces a sensible ordering
  // within a kind. Discriminate with `kind` — never with episodeNumber alone.
  // 'other'/unresolved: 0.
  episodeNumber: number;
  seasonNumber: number | null;
  // Numeric index extracted from the extras token. Null for kind='episode'.
  // ED1 → 1; OP4a → 4; PV12 → 12; "Special" with no digit → null.
  extraIndex: number | null;
  // Letter suffix on the extras token. Null when absent.
  // OP4a → 'a'; OP4b → 'b'; OP3 → null.
  extraVariant: string | null;
  // The matched token verbatim ("OP4a", "ED1", "PV12", "Special"). Useful
  // for display in an Extras list without having to reconstruct the label.
  // Null for kind='episode'.
  rawLabel: string | null;
}

// Strip extension + release-group brackets, keep everything else verbatim.
// This is what extractEpisode runs on so the existing patterns (`Episode 6.5`,
// `Show.Name.S02E07`) still see their separators intact.
function stripBracketsAndExt(filename: string): string {
  return basename(filename, extname(filename))
    .replace(/\s*\[[^\]]*\]\s*/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

// Same starting point, but additionally collapse `_` and `.` into spaces so
// the token scan downstream can isolate OP/ED/PV markers in releases that
// use those characters as word separators ("Bakemonogatari_ED1_…",
// "Show.Name.OP1.mkv"). Decimal episodes are detected upstream of this in
// extractEpisode, so flattening dots here is safe for token scanning.
function normaliseForTokens(filename: string): string {
  return stripBracketsAndExt(filename)
    .replace(/[_.]+/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

// Token patterns for extras. Each matcher must be anchored (^…$) so we
// only fire on standalone tokens — never on a substring of an episode title
// like "Operations" or "Edge".
const EXTRA_PATTERNS: ReadonlyArray<{
  kind: EpisodeKind;
  re: RegExp;
}> = [
  { kind: 'op',  re: /^(?:NCOP|OP)(\d+)([a-z])?$/i },
  { kind: 'ed',  re: /^(?:NCED|ED)(\d+)([a-z])?$/i },
  { kind: 'pv',  re: /^(?:PV|Trailer|Teaser)(\d+)?([a-z])?$/i },
  { kind: 'sp',  re: /^(?:SP|Special|Specials)(\d+)?([a-z])?$/i },
];

// Standalone marker tokens without an index — bucket as 'other'.
const OTHER_TOKENS = new Set(['menu', 'cm', 'bonus', 'extra', 'extras']);

interface ExtraHit {
  kind: EpisodeKind;
  index: number | null;
  variant: string | null;
  rawLabel: string;
}

function findExtraToken(filename: string): ExtraHit | null {
  const tokens = normaliseForTokens(filename).split(/\s+/);
  for (const token of tokens) {
    for (const { kind, re } of EXTRA_PATTERNS) {
      const m = token.match(re);
      if (m) {
        return {
          kind,
          index: m[1] ? parseInt(m[1], 10) : null,
          variant: m[2] ? m[2].toLowerCase() : null,
          rawLabel: token,
        };
      }
    }
    if (OTHER_TOKENS.has(token.toLowerCase())) {
      return { kind: 'other', index: null, variant: null, rawLabel: token };
    }
  }
  return null;
}

// Episode-number extraction. Pulled across from the old
// extractSeasonAndEpisode in folderHandler so the legacy patterns keep
// behaving identically for real episodes. The classifier only delegates
// here when no extras token was found.
function extractEpisode(filename: string): { season: number | null; episode: number } {
  // Keep underscores/dots intact at this stage — `Episode 6.5` and
  // `Show.Name.S02E07.1080p` both depend on those characters being preserved
  // for the patterns below to anchor correctly.
  const baseName = stripBracketsAndExt(filename);

  const finalize = (season: number | null, episode: number): { season: number | null; episode: number } => {
    if (episode === 0) return { season: 0, episode: 0 };
    return { season, episode };
  };

  const seasonEpisode = baseName.match(/\bS(\d+)E(\d+)\b/i);
  if (seasonEpisode) {
    return finalize(parseInt(seasonEpisode[1], 10), parseInt(seasonEpisode[2], 10));
  }

  const decimalEpisode = baseName.match(/Episode\s*(\d+)\.(\d+)/i);
  if (decimalEpisode) {
    const whole = parseInt(decimalEpisode[1], 10);
    const decimal = parseInt(decimalEpisode[2], 10);
    return finalize(null, whole + decimal / 10);
  }

  const patterns = [
    /Episode\s*(\d+)/i,
    /Ep\.?\s*(\d+)/i,
    /\bE(\d{2,})\b/i,
    /\s-\s*(\d+)(?:\s|$)/,
    /\s(\d{1,3})(?:\s|$)/,
  ];
  for (const pattern of patterns) {
    const match = baseName.match(pattern);
    if (match) return finalize(null, parseInt(match[1], 10));
  }

  // Fallback: pick a non-year, sub-1000 number from anywhere in the name.
  const numbers = baseName.match(/\d+/g);
  if (numbers && numbers.length > 0) {
    const filtered = numbers.filter((n) => {
      const num = parseInt(n, 10);
      if (num >= 1900 && num <= 2099) return false;
      if (num >= 1000) return false;
      return true;
    });
    if (filtered.length > 0) {
      return finalize(null, parseInt(filtered[filtered.length - 1], 10));
    }
  }

  return { season: null, episode: 1 };
}

export function classifyFile(filename: string): ClassifiedFile {
  const hit = findExtraToken(filename);
  if (hit) {
    // For extras we hoist the index into episodeNumber so downstream sorts
    // still work; the `kind` field is the real discriminator. SP without
    // an index falls back to the codebase's existing 0 = special convention.
    const epForSort = hit.index ?? (hit.kind === 'sp' ? 0 : 0);
    return {
      kind: hit.kind,
      episodeNumber: epForSort,
      seasonNumber: hit.kind === 'sp' ? 0 : null,
      extraIndex: hit.index,
      extraVariant: hit.variant,
      rawLabel: hit.rawLabel,
    };
  }

  const { season, episode } = extractEpisode(filename);
  return {
    kind: 'episode',
    episodeNumber: episode,
    seasonNumber: season,
    extraIndex: null,
    extraVariant: null,
    rawLabel: null,
  };
}
