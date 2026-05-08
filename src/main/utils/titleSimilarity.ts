// Tokenized Dice similarity for fuzzy-matching folder names against
// search results. Pure functions, no I/O.
//
// Tokenization is intentionally minimal — lowercase + split on
// non-alphanumerics. We do NOT strip particles, stopwords, brackets, or
// quality tags from the input. Folder names go in literally; the user
// is expected to keep folders clean (no torrent decorations). Cleaning
// happens at the file level in folderHandler, not here.
//
// Why Dice on tokens (not Jaccard / Levenshtein / substring):
//  - "Otaku ni Yasashii Gal wa Inai" vs "Wotaku ni Koi wa Muzukashii"
//    share characters and a substring ("otaku") that fool char-level
//    metrics. Token sets only overlap on particles, so Dice ≈ 0.36 →
//    rejected at the 0.4 threshold.
//  - Anime titles are short (3–8 tokens) — set-based scoring is cheap
//    and stable across word-order shuffles.
//  - Dice (2·|A∩B| / (|A|+|B|)) over Jaccard because Jaccard over-penalizes
//    size mismatch ("Frieren" vs "Sousou no Frieren" → Dice 0.50 vs
//    Jaccard 0.33).

function tokenize(s: string): string[] {
  return s
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s]/gu, ' ')
    .split(/\s+/)
    .filter((t) => t.length > 0);
}

function dice(a: string[], b: string[]): number {
  if (a.length === 0 || b.length === 0) return 0;
  const A = new Set(a);
  const B = new Set(b);
  let inter = 0;
  for (const t of A) if (B.has(t)) inter++;
  const denom = A.size + B.size;
  return denom === 0 ? 0 : (2 * inter) / denom;
}

/**
 * Score `query` against the best of several title variants (romaji,
 * english, native, etc.) and return the max. 0 means "no overlap";
 * 1 means "identical token sets after normalization."
 */
export function bestTitleScore(query: string, candidateTitles: (string | null | undefined)[]): number {
  const q = tokenize(query);
  if (q.length === 0) return 0;
  let best = 0;
  for (const t of candidateTitles) {
    if (!t) continue;
    const score = dice(q, tokenize(t));
    if (score > best) best = score;
  }
  return best;
}
