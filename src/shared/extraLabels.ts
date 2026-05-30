// Display labels for bonus content (openings/endings/PVs/specials/extras).
// Shared by the series page (bonus rows) and the player header so the two never
// drift. Pure string in → string out; no fs, no electron.
import type { EpisodeKind } from './episodeClassifier';

// Short category tag shown in the row's code-chip slot — the kind, not the raw
// filename token. Using the kind (not rawLabel) avoids "BONUS  Bonus" style
// duplication against the friendly title, and keeps a "TRAILER" token from
// contradicting its "Preview" title (both are just kind 'pv').
const KIND_CODE: Record<string, string> = {
  op: 'OP',
  ed: 'ED',
  pv: 'PV',
  sp: 'SP',
  other: 'EXTRA',
};

export function extraCode(kind: EpisodeKind | string | null | undefined): string {
  return (kind && KIND_CODE[kind]) || 'EXTRA';
}

// A clean human title derived from the classifier output — "Opening 4a",
// "Ending 1", "Preview 12", "Special" — instead of the release-tagged filename.
// Falls back to a title-cased raw token (Bonus, Menu, CM) for kind 'other'.
export function friendlyExtraTitle(
  kind: EpisodeKind | string | null | undefined,
  extraIndex: number | null | undefined,
  extraVariant: string | null | undefined,
  rawLabel: string | null | undefined,
): string {
  const idxPart =
    extraIndex != null ? ` ${extraIndex}${extraVariant ?? ''}` : extraVariant ? ` ${extraVariant}` : '';
  switch (kind) {
    case 'op': return `Opening${idxPart}`;
    case 'ed': return `Ending${idxPart}`;
    case 'pv': return `Preview${idxPart}`;
    case 'sp': return `Special${idxPart}`;
    default:
      if (rawLabel) return rawLabel.charAt(0).toUpperCase() + rawLabel.slice(1).toLowerCase();
      return 'Bonus';
  }
}
