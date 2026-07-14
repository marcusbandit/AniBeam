// One-time migration helpers for purging MAL as a registration source after
// its removal as a metadata provider (2026-07-14). Entries still registered
// to MAL get their registration fields nulled; the cleared attempt flags make
// the normal AniList auto-matcher pick these entries up again on the next
// library pass. Pure module: no Electron imports, safe for verify scripts.

// True when the entry is registered to MAL via either the display source
// column (`source`) or the matcher provenance field (`matchSource`).
export function isMalRegistered(entry: Record<string, unknown>): boolean {
  return entry.source === 'mal' || entry.matchSource === 'mal';
}

// Returns a NEW entry with the MAL registration cleared: source/matchSource/
// matchedTitle/matchScore nulled, posterMatched/posterMatchAttempted reset so
// the auto-matcher re-attempts via AniList. Everything else is preserved
// verbatim, EXPLICITLY including `malId` (AniSkip and the Jikan episode-title
// side-fetch key off it; it is a valid cross-reference regardless of
// provider) and `anilistId` (already cross-resolved for most entries, which
// gives the re-matcher a head start). Never mutates the input.
export function stripMalRegistration(entry: Record<string, unknown>): Record<string, unknown> {
  return {
    ...entry,
    source: null,
    matchSource: null,
    matchedTitle: null,
    matchScore: null,
    posterMatched: false,
    posterMatchAttempted: false,
  };
}
