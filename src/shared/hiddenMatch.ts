// Pure incognito matcher (isomorphic — no Electron imports). Returns true when
// any series entry carrying the given external media id is flagged hidden, so
// the main-process tracker guard can suppress AniList/MAL pushes for hidden
// series. Provider ids never cross: an AniList id is only matched against
// anilistId, a MAL id only against malId.
export type HiddenProvider = 'anilist' | 'mal';

interface HiddenLookupEntry {
  anilistId?: number;
  malId?: number | null;
  hidden?: boolean;
}

export function isSeriesHidden(
  metadata: Record<string, HiddenLookupEntry>,
  provider: HiddenProvider,
  mediaId: number,
): boolean {
  if (!mediaId) return false;
  for (const entry of Object.values(metadata)) {
    if (!entry || entry.hidden !== true) continue;
    if (provider === 'anilist' && entry.anilistId === mediaId) return true;
    if (provider === 'mal' && entry.malId === mediaId) return true;
  }
  return false;
}
