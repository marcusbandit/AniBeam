import { useState, useMemo, useEffect } from 'react';
import { useMetadata, type SeriesMetadata } from '../hooks/useMetadata';
import { BookOpen, Tv, Film, Search, RefreshCw, Trash2, AlertTriangle, Link2 } from 'lucide-react';
import MetadataMatchModal from '../components/MetadataMatchModal';
import { useDebouncedCallback } from '../hooks/useDebouncedCallback';
import { useHiddenShows } from '../contexts/HiddenShowsContext';
import { Page, Inline, Tooltip, Pill } from '../components/primitives';

type FilterOption = 'all' | 'series' | 'movies' | 'missing';

function FranchiseCrawlProgress() {
  const [stats, setStats] = useState<{ total: number; crawled: number } | null>(null);

  useEffect(() => {
    let cancelled = false;
    const fetch = () => {
      void window.electronAPI.getFranchiseCrawlProgress?.().then((s) => {
        if (!cancelled) setStats(s ?? null);
      });
    };
    fetch();
    const off = window.electronAPI.onFranchiseStoreUpdated?.(() => fetch());
    return () => { cancelled = true; off?.(); };
  }, []);

  if (!stats) return null;
  const pct = stats.total === 0 ? 0 : (stats.crawled / stats.total) * 100;
  return (
    <div className="franchise-crawl-progress">
      <div className="franchise-crawl-progress__header">
        <span>Franchise crawl</span>
        <span className="franchise-crawl-progress__count">{stats.crawled} / {stats.total} ({pct.toFixed(1)}%)</span>
      </div>
      <div className="franchise-crawl-progress__bar">
        <div className="franchise-crawl-progress__fill" style={{ width: `${pct}%` }} />
      </div>
    </div>
  );
}

function isMovie(data: SeriesMetadata): boolean {
  return data.type === 'movie' || data.format === 'MOVIE';
}

function getImageUrl(localPath?: string | null, remotePath?: string | null): string | null {
  if (localPath) return `media://${encodeURIComponent(localPath)}`;
  return remotePath || null;
}

function MetadataTab() {
  const { metadata, loading, updateSeriesMetadata, loadMetadata } = useMetadata();
  const [refreshing, setRefreshing] = useState<Record<string, boolean>>({});
  const [searchQuery, setSearchQuery] = useState('');
  const [filter, setFilter] = useState<FilterOption>('all');
  const [bulkRefreshing, setBulkRefreshing] = useState(false);
  const [matchTarget, setMatchTarget] = useState<{ seriesId: string; data: SeriesMetadata } | null>(null);
  const { showHidden } = useHiddenShows();

  // Debounced: bursts of metadata pings on new ingests would otherwise
  // hammer loadMetadata + re-render the entire grid per file.
  const debouncedLoad = useDebouncedCallback(() => { void loadMetadata(); }, 250);
  useEffect(() => {
    const unsubscribe = window.electronAPI.onMetadataFileStatusChanged?.(() => {
      debouncedLoad();
    });
    return () => unsubscribe?.();
  }, [debouncedLoad]);

  const seriesList = useMemo(() => Object.entries(metadata), [metadata]);

  // Hidden series drop out of the table (and its counts) unless reveal is on.
  const visibleSeries = useMemo(
    () => (showHidden ? seriesList : seriesList.filter(([, d]) => !d.hidden)),
    [seriesList, showHidden],
  );

  const filterCounts = useMemo(() => ({
    all: visibleSeries.length,
    series: visibleSeries.filter(([, d]) => !isMovie(d)).length,
    movies: visibleSeries.filter(([, d]) => isMovie(d)).length,
    missing: visibleSeries.filter(([, d]) => !(d.fileEpisodes?.length)).length,
  }), [visibleSeries]);

  const filteredSeries = useMemo(() => {
    return visibleSeries.filter(([id, data]) => {
      if (filter === 'series' && isMovie(data)) return false;
      if (filter === 'movies' && !isMovie(data)) return false;
      if (filter === 'missing' && (data.fileEpisodes?.length ?? 0) > 0) return false;

      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        const hay = `${data.title || id} ${data.titleRomaji || ''} ${data.titleEnglish || ''}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      return true;
    });
  }, [visibleSeries, searchQuery, filter]);

  const handleRefresh = async (seriesId: string, seriesName: string) => {
    setRefreshing(prev => ({ ...prev, [seriesId]: true }));
    try {
      const fetchedMetadata = await window.electronAPI.fetchMetadata(seriesName) as SeriesMetadata | null;
      if (fetchedMetadata) {
        await updateSeriesMetadata(seriesId, fetchedMetadata);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      alert('Error refreshing metadata: ' + errorMessage);
    } finally {
      setRefreshing(prev => ({ ...prev, [seriesId]: false }));
    }
  };

  const handleBulkRefresh = async () => {
    if (!confirm(`Refresh metadata for all ${filteredSeries.length} items? This may take a while.`)) return;

    setBulkRefreshing(true);
    let successCount = 0;
    let errorCount = 0;

    for (const [seriesId, data] of filteredSeries) {
      try {
        setRefreshing(prev => ({ ...prev, [seriesId]: true }));
        const fetchedMetadata = await window.electronAPI.fetchMetadata(data.title || seriesId) as SeriesMetadata | null;
        if (fetchedMetadata) {
          await updateSeriesMetadata(seriesId, fetchedMetadata);
          successCount++;
        }
      } catch (err) {
        console.error('Error refreshing:', seriesId, err);
        errorCount++;
      } finally {
        setRefreshing(prev => ({ ...prev, [seriesId]: false }));
      }
      await new Promise(resolve => setTimeout(resolve, 500));
    }

    setBulkRefreshing(false);
    alert(`Refresh complete\nSuccessful: ${successCount}\nFailed: ${errorCount}`);
    await loadMetadata();
  };

  const handleDelete = async (seriesId: string, seriesName: string) => {
    if (!confirm(`Delete "${seriesName}"?\n\nThis will remove all cached images and metadata for this series.`)) return;
    try {
      await window.electronAPI.deleteSeries(seriesId);
      await loadMetadata();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      alert('Error deleting series: ' + errorMessage);
    }
  };

  if (loading) {
    return (
      <Page>
        <div className="loading">Loading metadata…</div>
      </Page>
    );
  }

  if (seriesList.length === 0) {
    return (
      <Page>
        <div className="empty">
          <div className="empty-icon"><BookOpen size={48} /></div>
          <div className="empty-title">No metadata yet</div>
          <div className="empty-text">
            Your library is empty. Add a folder in <strong>Settings</strong> and scan it to get started.
          </div>
        </div>
      </Page>
    );
  }

  return (
    <Page
      head={
        <Inline gap="s4" justify="space-between" align="flex-start">
          <div>
            <h1 className="page-title">Metadata</h1>
            <p className="page-sub">All matched titles in your library, the source they came from, and the files on disk.</p>
          </div>
          <button
            className="btn btn-secondary"
            onClick={handleBulkRefresh}
            disabled={bulkRefreshing || filteredSeries.length === 0}
          >
            <RefreshCw size={14} className={bulkRefreshing ? 'spin' : ''} />
            <span>{bulkRefreshing ? 'Refreshing…' : 'Refresh all'}</span>
          </button>
        </Inline>
      }
    >

      <FranchiseCrawlProgress />

      <div className="meta-toolbar">
        <div className="filter-pills">
          {([
            { id: 'all', label: 'All' },
            { id: 'series', label: 'Series' },
            { id: 'movies', label: 'Movies' },
            { id: 'missing', label: 'Missing files' },
          ] as Array<{ id: FilterOption; label: string }>).map((f) => (
            <Pill
              key={f.id}
              toggle
              on={filter === f.id}
              onClick={() => setFilter(f.id)}
            >
              {f.label} {filterCounts[f.id]}
            </Pill>
          ))}
        </div>
        <div className="meta-search">
          <Search size={14} />
          <input
            placeholder="Filter titles…"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            spellCheck={false}
            autoComplete="off"
          />
        </div>
      </div>

      {filteredSeries.length === 0 ? (
        <div className="empty">
          <div className="empty-icon"><Search size={48} /></div>
          <div className="empty-title">No matches</div>
          <div className="empty-text">No series match your filters.</div>
        </div>
      ) : (
        <div className="meta-table">
          <div className="meta-row meta-row-head">
            <div className="col-thumb"></div>
            <div className="col-title">Title</div>
            <div className="col-type">Type</div>
            <div className="col-source">Source</div>
            <div className="col-files">Files</div>
            <div className="col-actions"></div>
          </div>
          {filteredSeries.map(([seriesId, data]) => {
            const movie = isMovie(data);
            // Count only REAL episodes. OP/ED/PV/SP/extras are legitimate bonus
            // files (Bakemonogatari has ~19 of them); they must not inflate the
            // count or trip the "more files than episodes" flag below. Entries
            // persisted before the classifier have no `kind`, so they still
            // count as episodes (backward compatible). Mirrors the realEpisodes
            // split on the series page.
            const have = (data.fileEpisodes ?? []).filter((f) => (f.kind ?? 'episode') === 'episode').length;
            const total = data.totalEpisodes || data.episodes?.length || (movie ? 1 : 0);
            const pct = total ? Math.round((have / total) * 100) : 0;
            // More EPISODES on disk than the matched title has: flag it so
            // misnamed/duplicate/stray files get the user's attention. Bonus
            // content is excluded above, so a show full of extras no longer
            // false-flags.
            const extra = total > 0 ? Math.max(0, have - total) : 0;
            const sourceClass = data.source ? data.source.toLowerCase() : 'none';
            const posterUrl = getImageUrl(data.posterLocal, data.poster);
            const isRefreshing = refreshing[seriesId];

            return (
              <div key={seriesId} className="meta-row">
                <div className="col-thumb">
                  <div className="meta-thumb">
                    {posterUrl ? (
                      <img
                        src={posterUrl}
                        alt={data.title || seriesId}
                        loading="lazy"
                        decoding="async"
                        onError={(e) => {
                          const t = e.target as HTMLImageElement;
                          t.style.display = 'none';
                        }}
                      />
                    ) : (
                      movie ? <Film size={16} /> : <Tv size={16} />
                    )}
                  </div>
                </div>
                <Tooltip label="Match to a different show">
                  <button
                    type="button"
                    className="col-title col-title-clickable"
                    onClick={() => setMatchTarget({ seriesId, data })}
                    disabled={isRefreshing || bulkRefreshing}
                  >
                    <div className="meta-title-main">{data.title || seriesId}</div>
                    {data.titleRomaji && data.titleRomaji !== data.title && (
                      <div className="meta-title-alt">{data.titleRomaji}</div>
                    )}
                  </button>
                </Tooltip>
                <div className="col-type">
                  <Pill size="sm" format={movie ? 'MOVIE' : 'TV'}>{movie ? 'Movie' : 'Series'}</Pill>
                </div>
                <div className="col-source">
                  <span className={`chip chip--sm source-chip source-${sourceClass}`}>
                    {data.source || 'none'}
                  </span>
                </div>
                <div className="col-files">
                  <div className="files-bar">
                    <div className="files-bar-track">
                      <div
                        className={`files-bar-fill${extra > 0 ? ' over' : ''}`}
                        style={{ width: `${Math.min(pct, 100)}%` }}
                      />
                    </div>
                    <span className="files-bar-text">
                      {have}<span className="muted">/{total}</span>
                    </span>
                    {extra > 0 && (
                      <Tooltip label={`${extra} file${extra === 1 ? '' : 's'} beyond the expected ${total}, needs attention`}>
                        <span className="chip chip--sm chip--amber" aria-label={`${extra} extra file${extra === 1 ? '' : 's'} detected`}>
                          <AlertTriangle size={13} aria-hidden="true" />
                          +{extra}
                        </span>
                      </Tooltip>
                    )}
                  </div>
                </div>
                <div className="col-actions">
                  <Tooltip label="Match to a different show">
                    <button
                      className="icon-btn"
                      aria-label="Match to a different show"
                      onClick={() => setMatchTarget({ seriesId, data })}
                      disabled={isRefreshing || bulkRefreshing}
                    >
                      <Link2 size={14} />
                    </button>
                  </Tooltip>
                  <Tooltip label="Refresh">
                    <button
                      className="icon-btn"
                      aria-label="Refresh"
                      onClick={() => handleRefresh(seriesId, data.title || seriesId)}
                      disabled={isRefreshing || bulkRefreshing}
                    >
                      <RefreshCw size={14} className={isRefreshing ? 'spin' : ''} />
                    </button>
                  </Tooltip>
                  <Tooltip label="Delete">
                    <button
                      className="icon-btn icon-btn-danger"
                      aria-label="Delete"
                      onClick={() => handleDelete(seriesId, data.title || seriesId)}
                      disabled={isRefreshing || bulkRefreshing}
                    >
                      <Trash2 size={14} />
                    </button>
                  </Tooltip>
                </div>
              </div>
            );
          })}
        </div>
      )}

      <MetadataMatchModal
        open={matchTarget !== null}
        seriesId={matchTarget?.seriesId ?? ''}
        currentTitle={matchTarget ? (matchTarget.data.titleRomaji || matchTarget.data.title || matchTarget.seriesId) : ''}
        // The user is picking an exact AniList media; no need to re-derive
        // a season suffix server-side.
        seasonNumber={null}
        onClose={() => setMatchTarget(null)}
        onApplied={loadMetadata}
      />
    </Page>
  );
}

export default MetadataTab;
