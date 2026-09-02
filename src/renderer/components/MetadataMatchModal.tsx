import { useEffect, useRef, useState } from 'react';
import { Search, X, Loader2 } from 'lucide-react';
import type { AnilistSearchResult, TmdbSearchResult } from '../../types/electron';
import { parseMetadataLink, describeMetadataLink, type MetadataLink } from '../../shared/metadataLink';
import { Tooltip, SegmentedSwitch } from './primitives';

interface Props {
  open: boolean;
  seriesId: string;
  currentTitle: string;
  seasonNumber: number | null;
  onClose: () => void;
  onApplied: () => void | Promise<void>;
}

const SEARCH_DEBOUNCE_MS = 250;

/**
 * Which catalogue to match against. AniList is the anime authority and stays
 * the default; TMDB covers the part of a library it has no entry for at all -
 * live-action films and non-anime shows.
 */
type Source = 'anilist' | 'tmdb';

/** A pasted URL we can turn into a catalogue entry. */
type ReadableLink = Exclude<MetadataLink, { provider: 'unknown' }>;

/** A candidate row, normalised across the two catalogues so the list renders
 *  once rather than branching per source. */
interface Candidate {
  key: string;
  cover: string | null;
  primary: string;
  secondary: string | null;
  meta: string;
  apply: () => Promise<{ ok: boolean; reason?: string }>;
}

/** Scheme and www. carry nothing the user needs to see in a result row. */
function bareUrl(text: string): string {
  return text.replace(/^[a-z][a-z0-9+.-]*:\/\//i, '').replace(/^www\./i, '');
}

function MetadataMatchModal({ open, seriesId, currentTitle, seasonNumber, onClose, onApplied }: Props) {
  const [source, setSource] = useState<Source>('anilist');
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<Candidate[]>([]);
  const [searching, setSearching] = useState(false);
  const [applyingKey, setApplyingKey] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [needsTmdbKey, setNeedsTmdbKey] = useState(false);

  const inputRef = useRef<HTMLInputElement>(null);
  const requestSeq = useRef(0);
  // The last pasted link we applied. The search effect re-runs when `source`
  // flips, and a link must apply once per paste, not once per re-run.
  const lastLinkRef = useRef<string | null>(null);

  const handlePick = async (candidate: Candidate) => {
    setApplyingKey(candidate.key);
    setError(null);
    try {
      const res = await candidate.apply();
      if (!res?.ok) {
        const reason = res?.reason;
        setError(reason === 'no-api-key'
          ? 'Add a TMDB API key in Settings first.'
          : reason === 'no-anilist-entry'
            ? 'AniList has no entry for that MyAnimeList id.'
            : `Could not apply match${reason ? `: ${reason}` : ''}`);
        setApplyingKey(null);
        return;
      }
      await onApplied();
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Apply failed');
      setApplyingKey(null);
    }
  };

  function toAnilistCandidates(res: AnilistSearchResult[]): Candidate[] {
    return (res || []).map((r) => {
      const primary = r.title.english || r.title.romaji || r.title.native;
      return {
        key: `anilist:${r.id}`,
        cover: r.coverImage?.extraLarge || r.coverImage?.large || null,
        primary,
        secondary: r.title.romaji && r.title.romaji !== primary ? r.title.romaji : null,
        meta: [r.format, r.seasonYear ? `${r.seasonYear}` : '', r.episodes !== null ? `${r.episodes} ep` : '']
          .filter(Boolean).join(' · '),
        apply: () => window.electronAPI.applyAnilistMatch(seriesId, r.id, seasonNumber),
      };
    });
  }

  function toTmdbCandidates(res: TmdbSearchResult[]): Candidate[] {
    return (res || []).map((r) => ({
      key: `tmdb:${r.kind}:${r.id}`,
      cover: r.posterUrl,
      primary: r.title,
      secondary: r.originalTitle,
      meta: [r.kind === 'movie' ? 'FILM' : 'TV', r.year ? `${r.year}` : ''].filter(Boolean).join(' · '),
      apply: () => window.electronAPI.applyTmdbMatch(seriesId, r.id, r.kind, seasonNumber),
    }));
  }

  /** One row for a pasted URL, in the same shape as a search hit so the list
   *  markup, the applying spinner and the click-to-retry all come for free. */
  function toLinkCandidate(link: ReadableLink, url: string): Candidate {
    const apply = async () => {
      if (link.provider === 'anilist') return window.electronAPI.applyAnilistMatch(seriesId, link.id, seasonNumber);
      if (link.provider === 'tmdb') return window.electronAPI.applyTmdbMatch(seriesId, link.id, link.kind, seasonNumber);
      // The store is keyed on AniList, so a MAL id is only usable once mapped.
      const anilistId = await window.electronAPI.resolveAnilistIdByMal(link.id);
      if (anilistId === null) return { ok: false, reason: 'no-anilist-entry' };
      return window.electronAPI.applyAnilistMatch(seriesId, anilistId, seasonNumber);
    };
    return {
      key: 'link',
      cover: null,
      primary: describeMetadataLink(link),
      secondary: bareUrl(url),
      meta: link.provider === 'mal' ? 'From link · resolved through AniList' : 'From link',
      apply,
    };
  }

  // Reset + focus + seed query on open. Keying behavior off `open` rather
  // than mounting on demand keeps the modal animation predictable.
  useEffect(() => {
    if (!open) return;
    setQuery(currentTitle);
    setResults([]);
    setError(null);
    setApplyingKey(null);
    setSource('anilist');
    setNeedsTmdbKey(false);
    lastLinkRef.current = null;
    // Defer focus so the input is in the layout when we focus it.
    const t = setTimeout(() => inputRef.current?.focus(), 0);
    return () => clearTimeout(t);
  }, [open, currentTitle]);

  // Debounced search. Bumping a sequence number guards against out-of-order
  // responses (slow first request resolving after a fast second one and
  // overwriting the newer results). Re-runs on `source` too, so flipping the
  // switch re-searches the same query against the other catalogue.
  // A pasted catalogue URL skips the search entirely and applies that exact
  // entry: some titles never surface in search, and a paste is one change
  // event, so there is nothing to debounce.
  useEffect(() => {
    if (!open) return;
    const trimmed = query.trim();
    const link = parseMetadataLink(trimmed);
    if (link === null) {
      lastLinkRef.current = null;
    } else if (link.provider === 'unknown') {
      ++requestSeq.current;
      setSearching(false);
      setResults([]);
      setError("Couldn't read that link. Paste an AniList, MyAnimeList or TMDB page URL.");
      return;
    } else {
      ++requestSeq.current;
      setSearching(false);
      if (lastLinkRef.current === trimmed) return;
      lastLinkRef.current = trimmed;
      // Flip the switch so it shows which catalogue the link is applied against.
      setSource(link.provider === 'tmdb' ? 'tmdb' : 'anilist');
      const candidate = toLinkCandidate(link, trimmed);
      setResults([candidate]);
      setError(null);
      setNeedsTmdbKey(false);
      void handlePick(candidate);
      return;
    }
    if (trimmed.length < 2) {
      setResults([]);
      setSearching(false);
      return;
    }
    const seq = ++requestSeq.current;
    setSearching(true);
    const timer = setTimeout(async () => {
      try {
        const candidates = source === 'anilist'
          ? toAnilistCandidates(await window.electronAPI.searchAnilist(trimmed, 12))
          : toTmdbCandidates(await window.electronAPI.searchTmdb(trimmed, 12));
        if (seq !== requestSeq.current) return;
        setResults(candidates);
        setError(null);
        setNeedsTmdbKey(false);
      } catch (err) {
        if (seq !== requestSeq.current) return;
        const message = err instanceof Error ? err.message : 'Search failed';
        // Main rejects TMDB searches with this marker when no key is stored;
        // that's a setup step, not a failure, so it gets its own copy.
        if (message.includes('no-api-key')) {
          setNeedsTmdbKey(true);
          setResults([]);
          setError(null);
        } else {
          setError(message);
        }
      } finally {
        if (seq === requestSeq.current) setSearching(false);
      }
    }, SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [query, open, source, seriesId, seasonNumber]);

  // Esc closes (when not mid-apply).
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && applyingKey === null) onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, applyingKey, onClose]);

  if (!open) return null;

  const placeholder = source === 'anilist'
    ? 'Search AniList, or paste a link…'
    : 'Search TMDB (films & TV), or paste a link…';

  return (
    <div className="match-modal-backdrop" onMouseDown={(e) => { if (e.target === e.currentTarget && applyingKey === null) onClose(); }}>
      <div className="match-modal" data-liquid-glass="" data-lg-bezel="16" role="dialog" aria-modal="true" aria-labelledby={`match-modal-title-${seriesId}`}>
        <div className="match-modal-head">
          <div>
            <div id={`match-modal-title-${seriesId}`} className="match-modal-title">Match metadata</div>
            <div className="match-modal-sub">Pick a title or paste a link. Its data replaces the current entry.</div>
          </div>
          <button className="icon-btn" onClick={onClose} disabled={applyingKey !== null} aria-label="Close">
            <X size={16} />
          </button>
        </div>

        <div className="match-modal-source">
          <SegmentedSwitch<Source>
            value={source}
            onChange={setSource}
            ariaLabel="Metadata source"
            options={[
              { value: 'anilist', label: 'Anime' },
              { value: 'tmdb', label: 'Film & TV' },
            ]}
          />
        </div>

        <div className="match-modal-search">
          <Search size={14} />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={placeholder}
            spellCheck={false}
            autoComplete="off"
            disabled={applyingKey !== null}
          />
          {searching && <Loader2 size={14} className="spin" />}
        </div>

        {error && <div className="match-modal-error">{error}</div>}
        {needsTmdbKey && (
          <div className="match-modal-empty">
            Matching films and non-anime TV needs a free TMDB API key.
            Add one under Settings → Metadata sources.
          </div>
        )}

        <div className="match-modal-results">
          {results.length === 0 && !searching && !needsTmdbKey && query.trim().length >= 2 && !error && (
            <div className="match-modal-empty">No matches.</div>
          )}
          {results.length === 0 && query.trim().length < 2 && (
            <div className="match-modal-empty muted">Type at least 2 characters.</div>
          )}
          {results.map((r) => {
            const isApplying = applyingKey === r.key;
            const otherApplying = applyingKey !== null && !isApplying;
            return (
              <Tooltip key={r.key} label={r.primary}>
                <button
                  className={`match-result${isApplying ? ' applying' : ''}`}
                  onClick={() => handlePick(r)}
                  disabled={otherApplying}
                >
                  <div className="match-result-thumb">
                    {r.cover ? <img src={r.cover} alt="" loading="lazy" decoding="async" /> : <span className="match-result-placeholder">?</span>}
                    {isApplying && <span className="match-result-applying"><Loader2 size={18} className="spin" /></span>}
                  </div>
                  <div className="match-result-text">
                    <div className="match-result-title">{r.primary}</div>
                    {r.secondary && <div className="match-result-alt">{r.secondary}</div>}
                    {r.meta && <div className="match-result-meta">{r.meta}</div>}
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
