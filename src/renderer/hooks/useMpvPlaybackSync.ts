// Applies an mpv session's final playhead to the renderer-owned resume map.
//
// Main does the parts it owns when an mpv window closes (view history, the
// AniList/MAL bump) but the resume position lives in localStorage, which only
// the renderer can touch. This hook is the other half: one app-wide
// subscription that writes the same entry the in-window player would have
// written, so an episode watched in mpv resumes in the app and shows the same
// partial-progress bar on its row.

import { useEffect } from 'react';
import {
  progressId,
  extraProgressToken,
  readProgress,
  writeProgress,
  recordEpisodeCompleted,
  RESUME_HEAD_SKIP,
  RESUME_TAIL_SKIP,
} from '../utils/playbackProgress';

/**
 * Subscribe once, for the lifetime of the app. Mount from App, not from a
 * page: an mpv window commonly outlives the page that launched it (the user
 * navigates away, or back to the library, while the episode plays).
 */
export function useMpvPlaybackSync(): void {
  useEffect(() => {
    const off = window.electronAPI.onMpvPlaybackEnded?.((report) => {
      const { seriesId, episodeNumber, isExtra, position, duration, filePath } = report;
      if (!seriesId || episodeNumber == null) return;
      if (!Number.isFinite(position) || !Number.isFinite(duration) || duration <= 0) return;

      // Extras share an episodeNumber with a real episode, so they're keyed by
      // path — exactly as the in-window player keys them.
      const key = progressId(seriesId, isExtra ? extraProgressToken(filePath) : episodeNumber);
      const map = readProgress();

      if (position >= duration - RESUME_TAIL_SKIP) {
        // Watched to the end: drop any resume entry so the next play starts
        // fresh instead of jumping straight to the credits.
        if (map[key]) { delete map[key]; writeProgress(map); }
        if (!isExtra) recordEpisodeCompleted(seriesId, episodeNumber);
      } else if (position > RESUME_HEAD_SKIP) {
        map[key] = { t: position, d: duration, updated: Date.now() };
        writeProgress(map);
      }
      // Below the head window: barely started, so there is nothing worth
      // resuming and any stale entry is still the better resume point.
    });
    return () => { off?.(); };
  }, []);
}
