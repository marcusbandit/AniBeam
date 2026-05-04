import { useState, useEffect } from 'react';
import { useMetadata } from '../hooks/useMetadata';
import { Folder, RefreshCw, Plus, Trash2 } from 'lucide-react';

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

interface SegmentProps<T extends string> {
  value: T;
  onChange: (v: T) => void;
  options: Array<{ id: T; label: string }>;
}

function Segment<T extends string>({ value, onChange, options }: SegmentProps<T>) {
  return (
    <div className="segment">
      {options.map((o) => (
        <button
          key={o.id}
          type="button"
          className={`segment-opt${value === o.id ? ' on' : ''}`}
          onClick={() => onChange(o.id)}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

type SubtitlePref = 'off' | 'auto' | 'always';

function SettingsTab() {
  const { metadata, loadMetadata } = useMetadata();
  const [folderSources, setFolderSources] = useState<string[]>([]);
  const [folderTitleCounts, setFolderTitleCounts] = useState<Record<string, number>>({});
  const [scanning, setScanning] = useState(false);
  const [scanningPath, setScanningPath] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [cacheStats, setCacheStats] = useState<CacheStats>({ count: 0, sizeBytes: 0 });

  // Design-only state (no persistence wired up yet — flagged in copy)
  const [sources, setSources] = useState({ anilist: true, mal: true, tvdb: false });
  const [tvdbKey, setTvdbKey] = useState('');
  const [autoScan, setAutoScan] = useState(true);
  const [subtitles, setSubtitles] = useState<SubtitlePref>('auto');

  useEffect(() => {
    loadFolderSources();
    loadCacheStats();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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
      <div className="page">
        <div className="loading">Loading settings…</div>
      </div>
    );
  }

  const cacheSize = formatBytes(cacheStats.sizeBytes);
  const metadataRecordCount = Object.keys(metadata).length;

  return (
    <div className="page settings-page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Settings</h1>
          <p className="page-sub">Library folders, metadata sources, and playback preferences.</p>
        </div>
      </div>

      <section className="settings-section">
        <div className="settings-section-head">
          <div>
            <h2 className="section-h2">Library folders</h2>
            <p className="section-sub">AniBeam scans these folders for video files. Subfolders are matched against series titles.</p>
          </div>
          <div style={{ display: 'flex', gap: '0.4rem' }}>
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
          </div>
        </div>

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
              return (
                <div key={folderPath} className="folder-row">
                  <div className="folder-icon"><Folder size={16} /></div>
                  <div className="folder-info">
                    <div className="folder-path">{folderPath}</div>
                    <div className="folder-meta">
                      {isScanningThis
                        ? <span className="scanning">Scanning…</span>
                        : <span>{count} {count === 1 ? 'title' : 'titles'}</span>}
                    </div>
                  </div>
                  <button
                    className="icon-btn"
                    title="Rescan"
                    onClick={() => handleScanFolder(folderPath)}
                    disabled={scanning}
                  >
                    <RefreshCw size={15} className={isScanningThis ? 'spin' : ''} />
                  </button>
                  <button
                    className="icon-btn icon-btn-danger"
                    title="Remove"
                    onClick={() => handleRemoveFolder(folderPath)}
                    disabled={scanning}
                  >
                    <Trash2 size={15} />
                  </button>
                </div>
              );
            })}
          </div>
        )}

      </section>

      <section className="settings-section">
        <div className="settings-section-head">
          <div>
            <h2 className="section-h2">Metadata sources</h2>
            <p className="section-sub">AniBeam queries enabled sources in priority order. The first match wins.</p>
          </div>
        </div>
        <div className="source-list">
          {([
            { id: 'anilist' as const, label: 'AniList', desc: 'GraphQL · Public, no key required', priority: 1, needsKey: false },
            { id: 'mal' as const, label: 'MyAnimeList', desc: 'Jikan API · Public, no key required', priority: 2, needsKey: false },
            { id: 'tvdb' as const, label: 'TVDB', desc: 'Requires API key', priority: 3, needsKey: true },
          ]).map((s) => (
            <div key={s.id} className="source-row">
              <div className="source-priority">{String(s.priority).padStart(2, '0')}</div>
              <div className="source-info">
                <div className="source-name">{s.label}</div>
                <div className="source-desc">{s.desc}</div>
              </div>
              {s.needsKey && (
                <input
                  className="source-key"
                  placeholder="API key"
                  value={tvdbKey}
                  onChange={(e) => setTvdbKey(e.target.value)}
                />
              )}
              <Toggle
                on={sources[s.id]}
                onChange={(v) => setSources({ ...sources, [s.id]: v })}
                ariaLabel={`Toggle ${s.label}`}
              />
            </div>
          ))}
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-head">
          <div>
            <h2 className="section-h2">Playback</h2>
          </div>
        </div>
        <div className="pref-list">
          <div className="pref-row">
            <div>
              <div className="pref-label">Subtitles</div>
              <div className="pref-help">Default subtitle track when starting playback.</div>
            </div>
            <Segment<SubtitlePref>
              value={subtitles}
              onChange={setSubtitles}
              options={[
                { id: 'off', label: 'Off' },
                { id: 'auto', label: 'Auto' },
                { id: 'always', label: 'Always English' },
              ]}
            />
          </div>
          <div className="pref-row">
            <div>
              <div className="pref-label">Auto-scan on launch</div>
              <div className="pref-help">Re-scan folders for new files when AniBeam starts.</div>
            </div>
            <Toggle on={autoScan} onChange={setAutoScan} ariaLabel="Toggle auto-scan" />
          </div>
        </div>
      </section>

      <section className="settings-section">
        <div className="settings-section-head">
          <div>
            <h2 className="section-h2">Cache</h2>
          </div>
        </div>
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
      </section>
    </div>
  );
}

export default SettingsTab;
