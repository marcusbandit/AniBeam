import { useEffect, useRef, useState } from 'react';
import { Search, X, Loader2 } from 'lucide-react';
import type { AnilistSearchResult } from '../../types/electron';
import { Tooltip } from './primitives';

interface Props {
  open: boolean;
  seriesId: string;
  currentTitle: string;
  seasonNumber: number | null;
  onClose: () => void;
  onApplied: () => void | Promise<void>;
}

const SEARCH_DEBOUNCE_MS = 250;

function MetadataMatchModal({ open, seriesId, currentTitle, seasonNumber, onClose, onApplied }: Props) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<AnilistSearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const [applyingId, setApplyingId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const inputRef = useRef<HTMLInputElement>(null);
  const requestSeq = useRef(0);

  // Reset + focus + seed query on open. Keying behavior off `open` rather
  // than mounting on demand keeps the modal animation predictable.
  useEffect(() => {
    if (!open) return;
    setQuery(currentTitle);
    setResults([]);
    setError(null);
    setApplyingId(null);
    // Defer focus so the input is in the layout when we focus it.
    const t = setTimeout(() => inputRef.current?.focus(), 0);
    return () => clearTimeout(t);
  }, [open, currentTitle]);

  // Debounced search. Bumping a sequence number guards against out-of-order
  // responses (slow first request resolving after a fast second one and
  // overwriting the newer results).
  useEffect(() => {
    if (!open) return;
    const trimmed = query.trim();
    if (trimmed.length < 2) {
      setResults([]);
      setSearching(false);
      return;
    }
    const seq = ++requestSeq.current;
    setSearching(true);
    const timer = setTimeout(async () => {
      try {
        const res = await window.electronAPI.searchAnilist(trimmed, 12);
        if (seq !== requestSeq.current) return;
        setResults(res || []);
        setError(null);
      } catch (err) {
        if (seq !== requestSeq.current) return;
        setError(err instanceof Error ? err.message : 'Search failed');
      } finally {
        if (seq === requestSeq.current) setSearching(false);
      }
    }, SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [query, open]);

  // Esc closes (when not mid-apply).
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && applyingId === null) onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, applyingId, onClose]);

  if (!open) return null;

  const handlePick = async (result: AnilistSearchResult) => {
    setApplyingId(result.id);
    setError(null);
    try {
      const res = await window.electronAPI.applyAnilistMatch(seriesId, result.id, seasonNumber);
      if (!res?.ok) {
        setError(`Could not apply match${res?.reason ? `: ${res.reason}` : ''}`);
        setApplyingId(null);
        return;
      }
      await onApplied();
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Apply failed');
      setApplyingId(null);
    }
  };

  return (
    <div className="match-modal-backdrop" onMouseDown={(e) => { if (e.target === e.currentTarget && applyingId === null) onClose(); }}>
      <div className="match-modal" role="dialog" aria-modal="true" aria-labelledby={`match-modal-title-${seriesId}`}>
        <div className="match-modal-head">
          <div>
            <div id={`match-modal-title-${seriesId}`} className="match-modal-title">Match metadata</div>
            <div className="match-modal-sub">Pick the right show — its data will replace the current entry.</div>
          </div>
          <button className="match-modal-close" onClick={onClose} disabled={applyingId !== null} aria-label="Close">
            <X size={16} />
          </button>
        </div>

        <div className="match-modal-search">
          <Search size={14} />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search AniList…"
            spellCheck={false}
            autoComplete="off"
            disabled={applyingId !== null}
          />
          {searching && <Loader2 size={14} className="spin" />}
        </div>

        {error && <div className="match-modal-error">{error}</div>}

        <div className="match-modal-results">
          {results.length === 0 && !searching && query.trim().length >= 2 && !error && (
            <div className="match-modal-empty">No matches.</div>
          )}
          {results.length === 0 && query.trim().length < 2 && (
            <div className="match-modal-empty muted">Type at least 2 characters.</div>
          )}
          {results.map((r) => {
            const cover = r.coverImage?.extraLarge || r.coverImage?.large || null;
            const primary = r.title.english || r.title.romaji || r.title.native;
            const secondary = r.title.romaji && r.title.romaji !== primary ? r.title.romaji : null;
            const yearBit = r.seasonYear ? `${r.seasonYear}` : '';
            const epBit = r.episodes !== null ? `${r.episodes} ep` : '';
            const meta = [r.format, yearBit, epBit].filter(Boolean).join(' · ');
            const isApplying = applyingId === r.id;
            const otherApplying = applyingId !== null && !isApplying;
            return (
              <Tooltip key={r.id} label={primary}>
                <button
                  className={`match-result${isApplying ? ' applying' : ''}`}
                  onClick={() => handlePick(r)}
                  disabled={otherApplying}
                >
                  <div className="match-result-thumb">
                    {cover ? <img src={cover} alt="" loading="lazy" decoding="async" /> : <span className="match-result-placeholder">?</span>}
                    {isApplying && <span className="match-result-applying"><Loader2 size={18} className="spin" /></span>}
                  </div>
                  <div className="match-result-text">
                    <div className="match-result-title">{primary}</div>
                    {secondary && <div className="match-result-alt">{secondary}</div>}
                    {meta && <div className="match-result-meta">{meta}</div>}
                  </div>
                </button>
              </Tooltip>
            );
          })}
        </div>
      </div>
    </div>
  );
}

export default MetadataMatchModal;
