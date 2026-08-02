import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useMetadata } from '../hooks/useMetadata';
import { useHiddenShows } from '../contexts/HiddenShowsContext';
import { Folder, RefreshCw, Plus, Trash2, Film, Rss, ChevronRight, Square, ExternalLink } from 'lucide-react';
import TrackersSection from './TrackersSection';
import { Page, Section, Inline, Tooltip, SegmentedSwitch } from './primitives';

interface CacheStats {
  count: number;
  sizeBytes: number;
}

function formatBytes(bytes: number): { value: string; unit: string } {
  if (bytes === 0) return { value: '0', unit: 'B' };
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const value = (bytes / Math.pow(k, i)).toFixed(i === 0 ? 0 : 1);
  return { value, unit: sizes[i] };
}

interface ToggleProps {
  on: boolean;
  onChange: (v: boolean) => void;
  ariaLabel?: string;
}

function Toggle({ on, onChange, ariaLabel }: ToggleProps) {
  return (
    <button
      type="button"
      className={`toggle${on ? ' on' : ''}`}
      onClick={() => onChange(!on)}
      aria-pressed={on}
      aria-label={ariaLabel}
    >
      <span className="toggle-thumb" />
    </button>
  );
}

type SubtitlePref = 'off' | 'auto' | 'always';

function SettingsTab() {
  const navigate = useNavigate();
  const { metadata, loadMetadata } = useMetadata();
  const { showHidden, setShowHidden } = useHiddenShows();
  const [folderSources, setFolderSources] = useState<string[]>([]);
  const [folderTitleCounts, setFolderTitleCounts] = useState<Record<string, number>>({});
  const [movieFoldersByRoot, setMovieFoldersByRoot] = useState<Record<string, string[]>>({});
  const [scanning, setScanning] = useState(false);
  const [scanningPath, setScanningPath] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [cacheStats, setCacheStats] = useState<CacheStats>({ count: 0, sizeBytes: 0 });

  // Design-only state (no persistence wired up yet; flagged in copy)
  const [sources, setSources] = useState({ anilist: true, mal: true });
  const [autoScan, setAutoScan] = useState(true);
  const [subtitles, setSubtitles] = useState<SubtitlePref>('auto');

  // Re-encoding controls. `transcodeAuto` mirrors main's master switch for the
  // background sweeps; `optedOutCount` is how many individual files the user
  // has stopped, which the copy surfaces so "off" doesn't look like the only
  // reason nothing is encoding.
  const [transcodeAuto, setTranscodeAuto] = useState(true);
  const [optedOutCount, setOptedOutCount] = useState(0);
  const [stopAllNote, setStopAllNote] = useState<string | null>(null);

  // TMDB key. The stored value is never read back into the field - the input
  // only ever holds something the user just typed, and the placeholder says
  // whether one is on file.
  const [tmdbKey, setTmdbKey] = useState('');
  const [tmdbKeySaved, setTmdbKeySaved] = useState(false);
  const [tmdbBusy, setTmdbBusy] = useState(false);
  const [tmdbNote, setTmdbNote] = useState<string | null>(null);


  useEffect(() => {
    loadFolderSources();
    loadCacheStats();
    void refreshTranscodeAuto();
    void window.electronAPI.tmdbHasApiKey?.().then(setTmdbKeySaved).catch(() => { /* best-effort */ });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleSaveTmdbKey(): Promise<void> {
    setTmdbBusy(true);
    setTmdbNote(null);
    try {
      // Main validates against TMDB before storing, so a typo'd key is caught
      // here rather than surfacing as a mystery empty search later.
      const res = await window.electronAPI.tmdbSetApiKey?.(tmdbKey);
      if (res?.ok) {
        setTmdbKeySaved(true);
        setTmdbKey('');
        setTmdbNote('Key verified and saved.');
      } else {
        setTmdbNote(res?.message ?? 'Could not save that key.');
      }
    } catch (err) {
      setTmdbNote(err instanceof Error ? err.message : 'Could not save that key.');
    } finally {
      setTmdbBusy(false);
    }
  }

  async function refreshTranscodeAuto(): Promise<void> {
    try {
      const state = await window.electronAPI.getTranscodeAuto?.();
      if (!state) return;
      setTranscodeAuto(state.auto);
      setOptedOutCount(state.optedOutCount);
    } catch (err) {
      console.error('[settings] could not read re-encode state:', err);
    }
  }

  async function handleTranscodeAuto(enabled: boolean): Promise<void> {
    setTranscodeAuto(enabled);   // optimistic - the toggle should feel instant
    try {
      const res = await window.electronAPI.setTranscodeAuto?.(enabled);
      if (!res) return;
      setTranscodeAuto(res.auto);
      // Turning it off stops what was running; turning it back on forgets the
      // per-file stops so the sweep genuinely resumes. Report whichever
      // happened rather than leaving the user guessing.
      if (!res.auto && res.stopped > 0) {
        setStopAllNote(`Stopped ${res.stopped} re-encode${res.stopped === 1 ? '' : 's'}.`);
      } else if (res.auto && res.resumed > 0) {
        setStopAllNote(`Resumed ${res.resumed} previously stopped file${res.resumed === 1 ? '' : 's'}.`);
      } else {
        setStopAllNote(null);
      }
      await refreshTranscodeAuto();
    } catch (err) {
      console.error('[settings] could not change re-encode state:', err);
      await refreshTranscodeAuto();
    }
  }

  async function handleStopAllTranscodes(): Promise<void> {
    try {
      const res = await window.electronAPI.cancelAllTranscodes?.();
      const n = res?.stopped ?? 0;
      setStopAllNote(n > 0
        ? `Stopped ${n} re-encode${n === 1 ? '' : 's'}.`
        : 'Nothing was encoding.');
      await refreshTranscodeAuto();
    } catch (err) {
      console.error('[settings] stop-all failed:', err);
      setStopAllNote('Could not stop re-encoding.');
    }
  }

  // Whenever metadata changes, recount titles per folder root (best-effort)
  useEffect(() => {
    const counts: Record<string, number> = {};
    folderSources.forEach((root) => {
      counts[root] = 0;
    });
    Object.values(metadata).forEach((data) => {
      const path = data.folderPath;
      if (!path) return;
      const root = folderSources.find((r) => path === r || path.startsWith(r + '/'));
      if (root) counts[root] = (counts[root] || 0) + 1;
    });
    setFolderTitleCounts(counts);
  }, [metadata, folderSources]);

  const loadCacheStats = async () => {
    try {
      const stats = await window.electronAPI.getImageCacheStats();
      setCacheStats(stats);
    } catch (err) {
      console.error('Error loading cache stats:', err);
    }
  };

  const loadFolderSources = async () => {
    try {
      setLoading(true);
      const list = await window.electronAPI.getFolderSources();
      setFolderSources(list);
      const detected = await Promise.all(
        list.map(async (root) => [root, await window.electronAPI.findMovieFolders(root)] as const),
      );
      setMovieFoldersByRoot(Object.fromEntries(detected));
    } catch (err) {
      console.error('Error loading folder sources:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleAddFolder = async () => {
    try {
      const selectedPath = await window.electronAPI.selectFolder();
      if (selectedPath) {
        await window.electronAPI.addFolderSource(selectedPath);
        await loadFolderSources();
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      alert('Error adding folder: ' + errorMessage);
    }
  };

  const handleRemoveFolder = async (folderPath: string) => {
    if (!confirm(`Remove "${folderPath}" from sources?`)) return;
    try {
      await window.electronAPI.removeFolderSource(folderPath);
      await loadFolderSources();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      alert('Error removing folder: ' + errorMessage);
    }
  };

  const handleScanFolder = async (folderPath: string) => {
    setScanning(true);
    setScanningPath(folderPath);
    try {
      await window.electronAPI.scanAndFetchMetadata(folderPath);
      await loadMetadata();
      await loadCacheStats();
    } finally {
      setScanning(false);
      setScanningPath(null);
    }
  };

  const handleClearMetadata = async () => {
    if (!confirm('Clear all metadata? You will need to re-scan to fetch it again.')) return;
    try {
      await window.electronAPI.clearMetadata();
      await loadMetadata();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      alert('Error clearing metadata: ' + errorMessage);
    }
  };

  const handleClearImageCache = async () => {
    if (!confirm('Clear all cached images? They will be re-downloaded on the next scan.')) return;
    try {
      await window.electronAPI.clearImageCache();
      await loadCacheStats();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      alert('Error clearing image cache: ' + errorMessage);
    }
  };

  const handleClearAll = async () => {
    if (!confirm('Clear ALL cached data (metadata + images)? You will need to re-scan your folders.')) return;
    try {
      await window.electronAPI.clearMetadata();
      await window.electronAPI.clearImageCache();
      await loadMetadata();
      await loadCacheStats();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      alert('Error clearing cache: ' + errorMessage);
    }
  };

  const handleScanAll = async () => {
    if (folderSources.length === 0) {
      alert('No folders added.');
      return;
    }
    setScanning(true);
    for (const folder of folderSources) {
      setScanningPath(folder);
      try {
        await window.electronAPI.scanAndFetchMetadata(folder);
      } catch (err) {
        console.error(`Error scanning ${folder}:`, err);
      }
    }
    await loadMetadata();
    await loadCacheStats();
    setScanning(false);
    setScanningPath(null);
  };

  if (loading) {
    return (
      <Page>
        <div className="loading">Loading settings…</div>
      </Page>
    );
  }

  const cacheSize = formatBytes(cacheStats.sizeBytes);
  const metadataRecordCount = Object.keys(metadata).length;

  return (
    <Page
      head={
        <div>
          <h1 className="page-title">Settings</h1>
          <p className="page-sub">Library folders, metadata sources, and playback preferences.</p>
        </div>
      }
    >

      <Section
        first
        title="Library folders"
        action={
          <Inline gap="s2">
            {folderSources.length > 0 && (
              <button className="btn btn-secondary" onClick={handleScanAll} disabled={scanning}>
                <RefreshCw size={14} className={scanning ? 'spin' : ''} />
                <span>{scanning ? 'Scanning…' : 'Scan all'}</span>
              </button>
            )}
            <button className="btn btn-secondary" onClick={handleAddFolder} disabled={scanning}>
              <Plus size={14} />
              <span>Add folder</span>
            </button>
          </Inline>
        }
      >
        <p className="section-sub">AniBeam scans these folders for video files. Subfolders are matched against series titles.</p>

        {folderSources.length === 0 ? (
          <div className="empty">
            <div className="empty-title">No folders yet</div>
            <div className="empty-text">Click <strong>Add folder</strong> to point AniBeam at your collection.</div>
          </div>
        ) : (
          <div className="folder-list">
            {folderSources.map((folderPath) => {
              const isScanningThis = scanningPath === folderPath;
              const count = folderTitleCounts[folderPath] ?? 0;
              const movieFolders = movieFoldersByRoot[folderPath] ?? [];
              return (
                <div key={folderPath} className="folder-group">
                  <div className="folder-row">
                    <div className="folder-icon"><Folder size={16} /></div>
                    <div className="folder-info">
                      <div className="folder-path">{folderPath}</div>
                      <div className="folder-meta">
                        {isScanningThis
                          ? <span className="scanning">Scanning…</span>
                          : <span>{count} {count === 1 ? 'title' : 'titles'}</span>}
                      </div>
                    </div>
                    <Tooltip label="Rescan">
                      <button
                        className="icon-btn"
                        aria-label="Rescan"
                        onClick={() => handleScanFolder(folderPath)}
                        disabled={scanning}
                      >
                        <RefreshCw size={15} className={isScanningThis ? 'spin' : ''} />
                      </button>
                    </Tooltip>
                    <Tooltip label="Remove">
                      <button
                        className="icon-btn icon-btn-danger"
                        aria-label="Remove"
                        onClick={() => handleRemoveFolder(folderPath)}
                        disabled={scanning}
                      >
                        <Trash2 size={15} />
                      </button>
                    </Tooltip>
                  </div>
                  {movieFolders.map((moviePath) => {
                    const relative = moviePath.startsWith(folderPath)
                      ? moviePath.slice(folderPath.length).replace(/^\/+/, '')
                      : moviePath;
                    return (
                      <Tooltip key={moviePath} label={moviePath}>
                        <div className="folder-row folder-row-nested">
                          <div className="folder-icon folder-icon-detected"><Film size={16} /></div>
                          <div className="folder-info">
                            <div className="folder-path">{relative}</div>
                            <div className="folder-meta">Detected · movies</div>
                          </div>
                        </div>
                      </Tooltip>
                    );
                  })}
                </div>
              );
            })}
          </div>
        )}

      </Section>

      <Section title="Metadata sources">
        <p className="section-sub">AniBeam queries enabled sources in priority order. The first match wins.</p>
        <div className="source-list">
          {([
            { id: 'anilist' as const, label: 'AniList', desc: 'GraphQL · Public, no key required', priority: 1 },
            { id: 'mal' as const, label: 'MyAnimeList', desc: 'Jikan API · Public, no key required', priority: 2 },
          ]).map((s) => (
            <div key={s.id} className="source-row">
              <div className="source-priority">{String(s.priority).padStart(2, '0')}</div>
              <div className="source-info">
                <div className="source-name">{s.label}</div>
                <div className="source-desc">{s.desc} · <span className="pref-note">not wired up yet</span></div>
              </div>
              <Toggle
                on={sources[s.id]}
                onChange={(v) => setSources({ ...sources, [s.id]: v })}
                ariaLabel={`Toggle ${s.label}`}
              />
            </div>
          ))}
        </div>

        <div className="pref-list">
          <div className="pref-row">
            <div>
              <div className="pref-label">TMDB API key</div>
              <div className="pref-help">
                Needed to match films and non-anime TV, which AniList has no entry for.
                Free from{' '}
                <button
                  type="button"
                  className="tracker-link"
                  onClick={() => void window.electronAPI.openExternal('https://www.themoviedb.org/settings/api')}
                >
                  themoviedb.org <ExternalLink size={11} />
                </button>
                {' '}(the v3 key).
                {tmdbNote && <> <span className="pref-note">{tmdbNote}</span></>}
              </div>
            </div>
            <Inline gap="s2">
              <input
                className="tracker-input"
                type="password"
                value={tmdbKey}
                onChange={(e) => setTmdbKey(e.target.value)}
                placeholder={tmdbKeySaved ? '••••••••  (saved)' : 'v3 API key'}
                spellCheck={false}
                autoComplete="off"
                aria-label="TMDB API key"
              />
              <button
                className="btn btn-secondary"
                onClick={() => void handleSaveTmdbKey()}
                disabled={tmdbBusy || tmdbKey.trim().length === 0}
              >
                {tmdbBusy ? 'Checking…' : 'Save'}
              </button>
            </Inline>
          </div>
        </div>
      </Section>

      <TrackersSection />

      <Section
        title="Subscriptions"
        action={
          <button className="btn btn-secondary" onClick={() => navigate('/subscriptions')}>
            <Rss size={14} />
            <span>Open subscriptions</span>
            <ChevronRight size={14} />
          </button>
        }
      >
        <p className="section-sub">RSS feeds anirss is watching for you. Moved out of the main nav; open the full list here.</p>
      </Section>

      <Section title="Library">
        <div className="pref-list">
          <div className="pref-row">
            <div>
              <div className="pref-label">Show hidden shows</div>
              <div className="pref-help">Reveal incognito series across all pages. Resets off when AniBeam restarts.</div>
            </div>
            <Toggle on={showHidden} onChange={setShowHidden} ariaLabel="Toggle hidden shows" />
          </div>
        </div>
      </Section>

      <Section title="Playback">
        <div className="pref-list">
          <div className="pref-row">
            <div>
              <div className="pref-label">Subtitles</div>
              <div className="pref-help">Default subtitle track when starting playback. <span className="pref-note">not wired up yet</span></div>
            </div>
            <SegmentedSwitch<SubtitlePref>
              value={subtitles}
              onChange={setSubtitles}
              ariaLabel="Default subtitles"
              options={[
                { value: 'off', label: 'Off' },
                { value: 'auto', label: 'Auto' },
                { value: 'always', label: 'Always English' },
              ]}
            />
          </div>
          <div className="pref-row">
            <div>
              <div className="pref-label">Auto-scan on launch</div>
              <div className="pref-help">Re-scan folders for new files when AniBeam starts. <span className="pref-note">not wired up yet</span></div>
            </div>
            <Toggle on={autoScan} onChange={setAutoScan} ariaLabel="Toggle auto-scan" />
          </div>
        </div>
      </Section>

      <Section title="Re-encoding">
        <p className="section-sub">
          Files whose codec the in-app player can&apos;t decode (HEVC, mostly) are converted to a
          cached h.264 copy in the background. Stopping one leaves the original untouched - it
          still plays in mpv.
        </p>
        <div className="pref-list">
          <div className="pref-row">
            <div>
              <div className="pref-label">Re-encode automatically</div>
              <div className="pref-help">
                {transcodeAuto
                  ? 'Incompatible files are converted as they are found.'
                  : 'Nothing is converted in the background. Opening an episode in the app still re-encodes that one.'}
                {optedOutCount > 0 && (
                  <> <span className="pref-note">{optedOutCount} file{optedOutCount === 1 ? '' : 's'} individually stopped</span></>
                )}
              </div>
            </div>
            <Toggle
              on={transcodeAuto}
              onChange={(v) => void handleTranscodeAuto(v)}
              ariaLabel="Toggle automatic re-encoding"
            />
          </div>
          <div className="pref-row">
            <div>
              <div className="pref-label">Stop everything now</div>
              <div className="pref-help">
                Kills the running encode and clears the queue.
                {stopAllNote && <> <span className="pref-note">{stopAllNote}</span></>}
              </div>
            </div>
            <button className="btn btn-secondary" onClick={() => void handleStopAllTranscodes()}>
              <Square size={14} />
              <span>Stop all</span>
            </button>
          </div>
        </div>
      </Section>

      <Section title="Cache">
        <div className="cache-stats">
          <div>
            <div className="cache-stat-num">{metadataRecordCount}</div>
            <div className="cache-stat-label">Metadata records</div>
          </div>
          <div>
            <div className="cache-stat-num">
              {cacheSize.value}<span className="unit">{cacheSize.unit}</span>
            </div>
            <div className="cache-stat-label">Disk cache</div>
          </div>
          <div>
            <div className="cache-stat-num">{cacheStats.count}</div>
            <div className="cache-stat-label">Cached images</div>
          </div>
          <div className="cache-actions">
            <button className="btn btn-secondary" onClick={handleClearImageCache} disabled={scanning}>
              <RefreshCw size={14} />
              <span>Clear images</span>
            </button>
            <button className="btn btn-secondary" onClick={handleClearMetadata} disabled={scanning}>
              <RefreshCw size={14} />
              <span>Clear metadata</span>
            </button>
            <button className="btn btn-danger" onClick={handleClearAll} disabled={scanning}>
              <Trash2 size={14} />
              <span>Clear all</span>
            </button>
          </div>
        </div>
      </Section>
    </Page>
  );
}

export default SettingsTab;
