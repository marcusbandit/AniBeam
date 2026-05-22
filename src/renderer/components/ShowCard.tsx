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
}: ShowCardProps) {
  const navigate = useNavigate();
  const { pickTitle } = useTitleLanguage();
  const { getWatched } = useTrackerProgress();

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

  const watched = getWatched({
    anilistId: item.anilistId ?? undefined,
    malId: item.malId ?? undefined,
  });
  const latestAiredNum = getLatestAiredEpisodeNumber(item.episodes);
  const watchedState = watched != null
    ? classifyWatchProgress({
        watched,
        totalEpisodes: item.totalEpisodes,
        latestAiredEpisode: latestAiredNum,
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
      onClick={() => navigate(`/series/${encodeURIComponent(item.id)}`)}
      onMouseEnter={() => liftRef.current?.setTarget(-LIFT_AMOUNT_PX)}
      onMouseLeave={() => liftRef.current?.setTarget(0)}
    >
      <div ref={posterWrapRef} className="show-card-poster-wrap">
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
        {score && (
          <span className="show-card-rating-badge" aria-label={`Rating ${score}`}>
            {score}
          </span>
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
        <div className="show-card-title" title={item.folderName}>
          {displayTitle}
        </div>
        <div className="show-card-meta">
          <span className="show-card-meta-ago" title={metaLeftTitle}>
            {leftText}
          </span>
          {nextUpcoming && nowMs != null && (
            <span
              className="show-card-meta-countdown"
              title={`Episode ${nextUpcoming.episodeNumber} airs ${new Date(nextUpcoming.airDateMs).toLocaleString()}`}
            >
              {formatCountdownMinutes(nextUpcoming.airDateMs - nowMs)}
            </span>
          )}
        </div>
      </div>
    </button>
  );
}

export default ShowCard;
