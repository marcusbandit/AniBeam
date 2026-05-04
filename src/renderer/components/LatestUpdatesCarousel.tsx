import { useRef } from "react";
import { Link } from "react-router-dom";
import { ChevronLeft, ChevronRight } from "lucide-react";
import ShowCard from "./ShowCard";
import { getAiringShows } from "../utils/airingUtils";
import type { SeriesMetadata } from "../hooks/useMetadata";

interface LatestUpdatesCarouselProps {
  metadata: Record<string, SeriesMetadata>;
  limit?: number;
}

function LatestUpdatesCarousel({ metadata, limit = 10 }: LatestUpdatesCarouselProps) {
  const scrollerRef = useRef<HTMLDivElement>(null);
  const airing = getAiringShows(metadata).slice(0, limit);

  if (airing.length === 0) return null;

  const scrollBy = (dir: 1 | -1) => {
    const el = scrollerRef.current;
    if (!el) return;
    const amount = Math.max(200, Math.round(el.clientWidth * 0.8));
    el.scrollBy({ left: dir * amount, behavior: "smooth" });
  };

  return (
    <section className="feed-section">
      <div className="section-head">
        <h2 className="section-h2">Latest updates</h2>
        <div className="section-head-actions">
          <Link to="/feed" className="section-link">View all →</Link>
          <button
            type="button"
            className="icon-btn"
            onClick={() => scrollBy(-1)}
            aria-label="Scroll left"
          >
            <ChevronLeft size={16} />
          </button>
          <button
            type="button"
            className="icon-btn"
            onClick={() => scrollBy(1)}
            aria-label="Scroll right"
          >
            <ChevronRight size={16} />
          </button>
        </div>
      </div>
      <div className="feed-carousel" ref={scrollerRef}>
        {airing.map(({ seriesId, data }) => (
          <div className="feed-carousel-item" key={seriesId}>
            <ShowCard seriesId={seriesId} seriesData={data} variant="feed" />
          </div>
        ))}
      </div>
    </section>
  );
}

export default LatestUpdatesCarousel;
