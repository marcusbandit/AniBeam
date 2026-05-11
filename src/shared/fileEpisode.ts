// Shape of one row inside a series's `fileEpisodes[]` array, as it lives
// inside metadata.json. Other fields exist on disk (titles, thumbnails,
// etc.); the ones declared here are the bits that handlers in the main
// process consistently read back. Keep this in sync with what the scanner
// writes in folderHandler / main.ts ingest.
import type { FileStatus } from './fileStatus';

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
