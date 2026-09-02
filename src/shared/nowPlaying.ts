// Now-playing lines for the in-window player. The renderer hands these to
// navigator.mediaSession and Chromium's MPRIS bridge republishes them on
// D-Bus, so this is what Linux media widgets (playerctl, waybar, the KDE and
// GNOME media applets) show: `title` lands in xesam:title, `artist` in
// xesam:artist. The show name is romaji first because that is how the library
// and the trackers name a series; English is only the fallback.
// Pure string logic, no Electron or React, so the verify script can import it.

export interface NowPlayingInput {
  showTitle: string;
  episodeNumber: number | null;
  episodeTitle: string | null | undefined;
  extraLabel?: string | null;
}

export interface NowPlayingLines {
  title: string;
  artist: string;
}

interface ShowTitleSource {
  titleRomaji?: string | null;
  titleEnglish?: string | null;
  title?: string | null;
  folderPath?: string | null;
}

// Same separator the player header and tracker toasts use.
const SEP = ' · ';

// A bare episode-number token: "Episode 5", "Ep 5", "Ep. 5", "E05", "5", "#5".
const EPISODE_TOKEN = /^(?:episode|ep\.?|e|#)?\s*(\d+)$/;

// Separators a release may put between the show name and the episode token:
// spaces, hyphen, the two long dashes (U+2013, U+2014), colon, underscore.
const LEADING_SEPARATORS = /^[\s\-\u2013\u2014:_]+/;

function nonEmpty(value: string | null | undefined): string | null {
  const trimmed = value?.trim() ?? '';
  return trimmed.length > 0 ? trimmed : null;
}

function fold(value: string | null | undefined): string {
  return (value ?? '').trim().toLowerCase().replace(/\s+/g, ' ');
}

function basename(path: string | null | undefined): string {
  if (!path) return '';
  const parts = path.split(/[\\/]+/).filter((p) => p.length > 0);
  return parts.length > 0 ? parts[parts.length - 1] : '';
}

function isEpisodeToken(text: string, episodeNumber: number | null): boolean {
  const match = EPISODE_TOKEN.exec(text);
  if (!match) return false;
  return episodeNumber === null || parseInt(match[1], 10) === episodeNumber;
}

export function broadcastShowTitle(series: ShowTitleSource | null | undefined): string {
  if (!series) return '';
  return nonEmpty(series.titleRomaji)
    ?? nonEmpty(series.titleEnglish)
    ?? nonEmpty(series.title)
    ?? nonEmpty(basename(series.folderPath))
    ?? '';
}

// A title is real when it names the episode. Empty, the show name, a bare
// episode-number token, or the show name plus separators plus such a token are
// all placeholders and not worth a line of their own.
export function isRealEpisodeTitle(
  episodeTitle: string | null | undefined,
  showTitle: string,
  episodeNumber: number | null,
): boolean {
  const title = fold(episodeTitle);
  if (title.length === 0) return false;
  const show = fold(showTitle);
  if (show.length > 0 && title === show) return false;
  const rest = show.length > 0 && title.startsWith(show)
    ? title.slice(show.length).replace(LEADING_SEPARATORS, '')
    : title;
  return !isEpisodeToken(rest, episodeNumber);
}

export function nowPlayingLines(input: NowPlayingInput): NowPlayingLines {
  const show = input.showTitle.trim();
  const extra = nonEmpty(input.extraLabel);
  if (extra) return { title: extra, artist: show };
  if (input.episodeNumber === null) return { title: show, artist: '' };
  const episode = `Episode ${input.episodeNumber}`;
  const episodeTitle = nonEmpty(input.episodeTitle);
  if (episodeTitle && isRealEpisodeTitle(episodeTitle, show, input.episodeNumber)) {
    return { title: episodeTitle, artist: [show, episode].filter((s) => s.length > 0).join(SEP) };
  }
  return { title: show, artist: episode };
}
