import { useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { Tv } from "lucide-react";
import type { LibraryItem } from "../../types/electron";
import { useTitleLanguage } from "../contexts/TitleLanguageContext";
import { useTrackerProgress } from "../contexts/TrackerProgressContext";
import { smoothScalar, type SmoothHandle } from "../utils/motion";
import {
  classifyWatchProgress,
  findNextUpcomingEpisode,
  formatCountdownMinutes,
  formatWatchedLabel,
  getLatestAiredEpisodeNumber,
} from "../utils/airingUtils";
import { getDisplayRating } from "../utils/ratingUtils";
import { Tooltip } from "./primitives";

const LIFT_SPEED = 12;
const LIFT_AMOUNT_PX = 3;

interface ShowCardProps {
  item: LibraryItem;
  /** Render an "EP NN" badge top-left on the poster. */
  episodeBadgeNumber?: number | null;
  /** Left text in the meta row (e.g. "2d ago", "5 files"). When unset
   *  the row falls back to the file count so empty cards never look
   *  blank. */
  metaLeftText?: string;
  /** Native tooltip for the meta-left text. */
  metaLeftTitle?: string;
  /** Current ms used for the live next-episode countdown. The parent
   *  owns the ticker (a single setInterval) so multiple cards stay in
   *  lockstep without N timers. Omit to disable the countdown entirely
   *  even when the show has a known upcoming air date. */
  nowMs?: number;
  /** Draw the poster border. Defaults to true. The Watching tab passes
   *  false for shows that are on the tracker list but not in the local
   *  library, so they read as "not owned" (borderless). */
  outlined?: boolean;
  /** Override the default click action (navigate to the in-app series
   *  page). The Watching tab passes a handler that opens the show's
   *  AniList page in the browser for non-library cards. */
  onActivate?: () => void;
}

/**
 * Single shared show-card used by every grid in the app (Home → Airing /
 * Series / Movies, Feed). Owns the poster, the four corner badges, and
 * the meta row so a visual tweak made here lands in every list at once.
 */
function ShowCard({
  item,
  episodeBadgeNumber,
  metaLeftText,
  metaLeftTitle,
  nowMs,
  outlined = true,
  onActivate,
}: ShowCardProps) {
  const navigate = useNavigate();
  const { pickTitle } = useTitleLanguage();
  const { getWatched, getUserScore } = useTrackerProgress();

  // Smoothed hover-lift is applied to the poster-wrap only — the info row
  // below stays anchored so titles don't slide when the cursor enters/leaves.
  // The button itself is a transparent shell (no background/border); the
  // visible "card" is the poster's own border + radius.
  const posterWrapRef = useRef<HTMLDivElement | null>(null);
  const liftRef = useRef<SmoothHandle | null>(null);
  useEffect(() => {
    const el = posterWrapRef.current;
    if (!el) return;
    const handle = smoothScalar(0, LIFT_SPEED, (v) => {
      el.style.transform = Math.abs(v) > 0.05 ? `translateY(${v.toFixed(2)}px)` : "";
    });
    liftRef.current = handle;
    return () => { handle.release(); el.style.transform = ""; };
  }, []);

  const posterUrl = item.posterLocal
    ? `media://${encodeURIComponent(item.posterLocal)}`
    : item.poster;
  const displayTitle = pickTitle({
    titleRomaji: item.titleRomaji ?? item.matchedTitle,
    titleEnglish: item.titleEnglish,
    folderName: item.folderName,
  });
  const score = item.averageScore != null
    ? getDisplayRating(item.averageScore, item.source)
    : null;

  // User's personal score from their tracker (AniList POINT_10_DECIMAL or
  // MAL native — both already 0–10 thanks to TrackerProgressContext).
  // Rendered as a sibling badge to the community score, same shape with a
  // teal star instead of amber so the eye can tell the two apart.
  const myScoreRaw = getUserScore({
    anilistId: item.anilistId ?? undefined,
    malId: item.malId ?? undefined,
  });
  const myScore = myScoreRaw != null ? myScoreRaw.toFixed(1) : null;

  const watched = getWatched({
    anilistId: item.anilistId ?? undefined,
    malId: item.malId ?? undefined,
  });
  const latestAiredNum = getLatestAiredEpisodeNumber(item.episodes);
  // Highest episode number actually sitting on disk. A downloaded-but-
  // unwatched episode counts toward "behind" even when its airDate is
  // missing (common — see classifyWatchProgress).
  const latestDownloadedNum = item.files.length > 0
    ? item.files.reduce((max, f) => (f.episodeNumber > max ? f.episodeNumber : max), 0)
    : null;
  const watchedState = watched != null
    ? classifyWatchProgress({
        watched,
        totalEpisodes: item.totalEpisodes,
        latestAiredEpisode: latestAiredNum,
        latestDownloadedEpisode: latestDownloadedNum,
      })
    : null;
  const watchedLabel = formatWatchedLabel({
    watched,
    totalEpisodes: item.totalEpisodes,
    latestAiredEpisode: latestAiredNum,
    state: watchedState,
  });

  const epBadge = episodeBadgeNumber != null
    ? String(episodeBadgeNumber).padStart(2, "0")
    : null;

  // Countdown is opt-in (caller passes nowMs) so non-ticking grids don't
  // accidentally subscribe to a re-render every tick they don't need.
  const nextUpcoming = nowMs != null
    ? findNextUpcomingEpisode(item.episodes, nowMs)
    : null;

  const leftText = metaLeftText
    ?? `${item.files.length} file${item.files.length === 1 ? "" : "s"}`;

  return (
    <button
      type="button"
      className="show-card"
      data-halo-bias
      data-flip-id={item.id}
      onClick={() => (onActivate ? onActivate() : navigate(`/series/${encodeURIComponent(item.id)}`))}
      onMouseEnter={() => liftRef.current?.setTarget(-LIFT_AMOUNT_PX)}
      onMouseLeave={() => liftRef.current?.setTarget(0)}
    >
      <div ref={posterWrapRef} className={`show-card-poster-wrap${outlined ? "" : " show-card-poster-wrap--bare"}`}>
        {watchedLabel && (
          <span
            className={`show-card-watched-badge${watchedState ? ` ${watchedState}` : ""}`}
            aria-label={`Watched ${watchedLabel}`}
          >
            {watchedLabel}
          </span>
        )}
        {epBadge && (
          <span className="show-card-ep-badge" aria-label={`Episode ${epBadge}`}>
            EP {epBadge}
          </span>
        )}
        {(score || myScore) && (
          <div className="show-card-ratings">
            {score && (
              <span className="show-card-rating-badge" aria-label={`Rating ${score}`}>
                {score}
              </span>
            )}
            {myScore && (
              <Tooltip label="Your score">
                <span
                  className="show-card-rating-badge show-card-rating-badge--mine"
                  aria-label={`Your rating ${myScore}`}
                >
                  {myScore}
                </span>
              </Tooltip>
            )}
          </div>
        )}
        {posterUrl ? (
          <img
            className="show-card-poster"
            src={posterUrl}
            alt={displayTitle}
            loading="lazy"
            decoding="async"
          />
        ) : (
          <div className="show-card-no-image"><Tv size={32} /></div>
        )}
      </div>
      <div className="show-card-info">
        <Tooltip label={item.folderName}>
          <div className="show-card-title">
            {displayTitle}
          </div>
        </Tooltip>
        <div className="show-card-meta">
          {metaLeftTitle ? (
            <Tooltip label={metaLeftTitle}>
              <span className="show-card-meta-ago">
                {leftText}
              </span>
            </Tooltip>
          ) : (
            <span className="show-card-meta-ago">
              {leftText}
            </span>
          )}
          {nextUpcoming && nowMs != null && (
            <Tooltip label={`Episode ${nextUpcoming.episodeNumber} airs ${new Date(nextUpcoming.airDateMs).toLocaleString()}`}>
              <span className="show-card-meta-countdown">
                {formatCountdownMinutes(nextUpcoming.airDateMs - nowMs)}
              </span>
            </Tooltip>
          )}
        </div>
      </div>
    </button>
  );
}

export default ShowCard;
