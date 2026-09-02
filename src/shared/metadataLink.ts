// Turns a pasted AniList, MyAnimeList or TMDB URL into a provider id. The
// match modal accepts links in the same box as a search query, so the first
// question is whether the text is a URL at all: null means "search for it".
// Pure, no Electron or Node imports, so main, renderer and the verify script
// all share the one parser.

export type MetadataLink =
  | { provider: 'anilist'; id: number }
  | { provider: 'mal'; id: number }
  | { provider: 'tmdb'; kind: 'movie' | 'tv'; id: number }
  | { provider: 'unknown' };

const UNKNOWN: MetadataLink = { provider: 'unknown' };

const HAS_SCHEME = /^https?:\/\//i;

// People paste from the address bar, which drops the scheme. Only the hosts
// we can look up get that leniency; "example.com/anime/21" stays a search.
const KNOWN_HOST_PREFIX = /^(www\.)?(anilist\.co|myanimelist\.net|themoviedb\.org)\//i;

/** null when the text is not a URL at all (treat it as a search query). */
export function parseMetadataLink(text: string): MetadataLink | null {
  const trimmed = text.trim();
  if (!trimmed) return null;

  const hasScheme = HAS_SCHEME.test(trimmed);
  if (!hasScheme && !KNOWN_HOST_PREFIX.test(trimmed)) return null;

  let url: URL;
  try {
    url = new URL(hasScheme ? trimmed : `https://${trimmed}`);
  } catch {
    return null;
  }

  const host = url.hostname.toLowerCase().replace(/^www\./, '');
  const segments = url.pathname.split('/').filter(Boolean);

  switch (host) {
    case 'anilist.co': {
      const id = segments[0] === 'anime' ? positiveInt(segments[1]) : null;
      return id === null ? UNKNOWN : { provider: 'anilist', id };
    }
    case 'myanimelist.net': {
      // Two shapes: /anime/{id}/Slug and the legacy /anime.php?id={id}.
      let id: number | null = null;
      if (segments[0] === 'anime') id = positiveInt(segments[1]);
      else if (segments[0] === 'anime.php') id = positiveInt(url.searchParams.get('id'));
      return id === null ? UNKNOWN : { provider: 'mal', id };
    }
    case 'themoviedb.org': {
      const kind = segments[0];
      if (kind !== 'movie' && kind !== 'tv') return UNKNOWN;
      // The id segment is "550" or "550-fight-club": the digits before the slug.
      const id = positiveInt(segments[1]?.match(/^(\d+)(?:-|$)/)?.[1]);
      return id === null ? UNKNOWN : { provider: 'tmdb', kind, id };
    }
    default:
      return UNKNOWN;
  }
}

/** Short human label for a parsed link, e.g. "AniList #21". "Link" for unknown. */
export function describeMetadataLink(link: MetadataLink): string {
  switch (link.provider) {
    case 'anilist':
      return `AniList #${link.id}`;
    case 'mal':
      return `MyAnimeList #${link.id}`;
    case 'tmdb':
      return `TMDB ${link.kind === 'movie' ? 'film' : 'show'} #${link.id}`;
    default:
      return 'Link';
  }
}

// Digits only and above zero. "21abc", "-21" and "0" are all links with
// nothing behind them, not search queries, so the caller reports unknown.
function positiveInt(value: string | null | undefined): number | null {
  if (!value || !/^\d+$/.test(value)) return null;
  const n = Number(value);
  return Number.isSafeInteger(n) && n > 0 ? n : null;
}
