// Shape of one row inside a series's `fileEpisodes[]` array, as it lives
// inside metadata.json. Other fields exist on disk (titles, thumbnails,
// etc.); the ones declared here are the bits that handlers in the main
// process consistently read back. Keep this in sync with what the scanner
// writes in folderHandler / main.ts ingest.
import type { FileStatus } from './fileStatus';
import type { EpisodeKind } from './episodeClassifier';
import type { SubtitleState } from './subtitleSupport';

export interface FileEpisodeEntry {
  filePath: string;
  episodeNumber?: number;
  seasonNumber?: number | null;
  status?: FileStatus;
  lastProbedAt?: number;
  transcodedPath?: string | null;
  filename?: string;
  title?: string;
  subtitlePath?: string | null;
  subtitlePaths?: string[];
  // Classifier output. Optional because pre-classifier entries in metadata.json
  // won't have it; consumers MUST treat a missing `kind` as 'episode' for
  // backward compatibility with existing user data on disk.
  kind?: EpisodeKind;
  extraIndex?: number | null;
  extraVariant?: string | null;
  rawLabel?: string | null;
  // Subtitle availability, surfaced as an episode-row marker. Set by the
  // series-view probe sweep (cheap, bitmap/unreadable detection) and refined by
  // the authoritative play-time outcome. `subtitleCheckedAt` is the file mtimeMs
  // the state was computed against, so the sweep can skip unchanged files and
  // re-check ones that were replaced on disk. See shared/subtitleSupport.ts.
  subtitleState?: SubtitleState | null;
  subtitleCheckedAt?: number;
  // Display aspect ratio (width/height in display pixels, anamorphic-aware).
  // Backfilled by the transcode:ensure-series sweep on series open, so the
  // player can size its chrome to the real picture BEFORE video metadata
  // loads. null = probed but underdetermined; missing = never probed.
  displayAspect?: number | null;
}

// Scan every series in metadata for a fileEpisodes entry whose `filePath`
// matches. Used by the per-file update paths (status changes, transcode
// completion, unlink cleanup). Returns null if the file isn't tracked yet.
export function findFileEpisode(
  metadata: Record<string, unknown>,
  filePath: string,
): FileEpisodeEntry | null {
  for (const series of Object.values(metadata)) {
    const s = series as { fileEpisodes?: FileEpisodeEntry[] };
    if (!Array.isArray(s.fileEpisodes)) continue;
    const hit = s.fileEpisodes.find((f) => f.filePath === filePath);
    if (hit) return hit;
  }
  return null;
}
