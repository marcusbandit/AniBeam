import { useMetadata } from "../hooks/useMetadata";
import ShowCard from "../components/ShowCard";
import { getAiringShows } from "../utils/airingUtils";
import { Activity } from "lucide-react";

function FeedPage() {
  const { metadata, loading, error } = useMetadata();

  if (loading) {
    return (
      <div className="page">
        <div className="loading">Loading feed…</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="page">
        <div className="error">Error: {error}</div>
      </div>
    );
  }

  const airing = getAiringShows(metadata);

  return (
    <div className="page feed-page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Feed</h1>
          <p className="page-sub">Currently airing series in your library, sorted by latest release.</p>
        </div>
      </div>

      {airing.length === 0 ? (
        <div className="empty">
          <div className="empty-icon"><Activity size={48} /></div>
          <div className="empty-title">Nothing airing</div>
          <div className="empty-text">
            None of your on-disk series are currently releasing. Add an airing show to see it here.
          </div>
        </div>
      ) : (
        <>
          <div className="section-head">
            <h2 className="section-h2">Latest updates</h2>
            <span className="section-count">
              {airing.length} {airing.length === 1 ? "title" : "titles"}
            </span>
          </div>
          <div className="show-grid">
            {airing.map(({ seriesId, data }) => (
              <ShowCard
                key={seriesId}
                seriesId={seriesId}
                seriesData={data}
                variant="feed"
              />
            ))}
          </div>
        </>
      )}
    </div>
  );
}

export default FeedPage;
