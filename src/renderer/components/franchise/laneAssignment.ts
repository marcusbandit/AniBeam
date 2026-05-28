// Pure display logic: map a relation onto a lane, label it, and order spine
// nodes chronologically. Used by FranchiseMap and exercised by a verify script.

export type FranchiseLane = 'spine' | 'top' | 'bottom' | 'sidebranch' | 'excluded';

/** Formats treated as upstream source material (not screen adaptations). */
const PRINT_FORMATS = new Set(['MANGA', 'NOVEL', 'LIGHT_NOVEL', 'ONE_SHOT', 'VISUAL_NOVEL']); // AniList-specific; extend (with matching tests) if new print formats like MANHWA/MANHUA are added

/**
 * Lane for a related node, given the edge's relationType and the *target's*
 * media type/format. SOURCE/PARENT sit above; PREQUEL/SEQUEL form the spine;
 * ALTERNATIVE branches; CHARACTER is excluded; everything else hangs below.
 * ADAPTATION direction is resolved by media type: an adaptation that is print
 * material is upstream (top); a screen adaptation is downstream (bottom).
 */
export function relationLane(
  relationType: string,
  targetType: 'ANIME' | 'MANGA' | null,
  targetFormat: string | null,
): FranchiseLane {
  switch (relationType) {
    case 'PREQUEL':
    case 'SEQUEL':
      return 'spine';
    case 'ALTERNATIVE':
      return 'sidebranch';
    case 'CHARACTER':
      return 'excluded';
    case 'SOURCE':
    case 'PARENT':
      return 'top';
    case 'ADAPTATION': {
      const isPrint = targetFormat ? PRINT_FORMATS.has(targetFormat) : targetType === 'MANGA';
      return isPrint ? 'top' : 'bottom';
    }
    default:
      // SIDE_STORY, SPIN_OFF, SUMMARY, COMPILATION, CONTAINS, OTHER, unknown
      return 'bottom';
  }
}

export const RELATION_LABEL: Record<string, string> = {
  SEQUEL: 'Sequel',
  PREQUEL: 'Prequel',
  PARENT: 'Parent story',
  SIDE_STORY: 'Side story',
  SUMMARY: 'Summary',
  ALTERNATIVE: 'Alternative',
  SPIN_OFF: 'Spin-off',
  COMPILATION: 'Compilation',
  SOURCE: 'Source',
  ADAPTATION: 'Adaptation',
  CHARACTER: 'Shared characters',
  CONTAINS: 'Contains',
  OTHER: 'Other',
};

export function relationLabel(relationType: string): string {
  return RELATION_LABEL[relationType] ?? relationType.replace(/_/g, ' ').toLowerCase();
}

/** Chronological compare by seasonYear; nulls sort last. */
export function compareByYear(
  a: { seasonYear: number | null },
  b: { seasonYear: number | null },
): number {
  const ay = a.seasonYear ?? Number.POSITIVE_INFINITY;
  const by = b.seasonYear ?? Number.POSITIVE_INFINITY;
  return ay - by;
}
