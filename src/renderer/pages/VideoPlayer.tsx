import { useParams, useNavigate } from 'react-router-dom';
import { useState, useEffect, useRef, useCallback } from 'react';
import { useMetadata, type FileEpisode } from '../hooks/useMetadata.js';
import { ArrowLeft, Play, Pause, Volume2, VolumeX, Maximize, Minimize, Subtitles } from 'lucide-react';

interface SubtitleTrack {
  src: string;        // possibly converted to a blob: URL for SRT
  origPath: string;   // original file path, used for label/extension lookup
  kind: string;
  label: string;
  default?: boolean;
}

/**
 * <track> only natively supports WebVTT. SRT files (very common for anime)
 * won't display unless converted. Fetch the file, prepend the WEBVTT header,
 * and swap "," for "." in timestamps. Returns a blob: URL.
 * Returns the original src if the conversion fails so we at least try.
 */
async function srtToVttUrl(srcUrl: string): Promise<string> {
  try {
    const text = await (await fetch(srcUrl)).text();
    const vtt = 'WEBVTT\n\n' + text.replace(/(\d\d:\d\d:\d\d),(\d\d\d)/g, '$1.$2');
    return URL.createObjectURL(new Blob([vtt], { type: 'text/vtt' }));
  } catch {
    return srcUrl;
  }
}

interface SubtitleStyle {
  fontSize: number;        // in vh — % of viewport height, scales with fullscreen
  positionBottom: number;  // distance above the bottom edge in vh
  color: string;
  bgColor: string;
  bgOpacity: number;
  fontFamily: 'sans-serif' | 'serif' | 'ui-monospace';
  outline: 'none' | 'light' | 'medium' | 'heavy';
}

const DEFAULT_SUB_STYLE: SubtitleStyle = {
  fontSize: 3.5,
  positionBottom: 8,
  color: '#ffffff',
  bgColor: '#000000',
  bgOpacity: 0.5,
  fontFamily: 'sans-serif',
  outline: 'medium',
};

const OUTLINE_PRESETS: Record<SubtitleStyle['outline'], string> = {
  none: 'none',
  light: '0 1px 2px rgba(0,0,0,0.7)',
  medium: '-1px -1px 0 #000, 1px -1px 0 #000, -1px 1px 0 #000, 1px 1px 0 #000',
  heavy: '-2px -2px 0 #000, 2px -2px 0 #000, -2px 2px 0 #000, 2px 2px 0 #000, 0 0 4px #000',
};

function hexToRgb(hex: string): string {
  const m = hex.match(/^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i);
  if (!m) return '0,0,0';
  return `${parseInt(m[1], 16)}, ${parseInt(m[2], 16)}, ${parseInt(m[3], 16)}`;
}

function pad(n: number): string {
  return n < 10 ? `0${n}` : `${n}`;
}

function formatTime(s: number): string {
  if (!Number.isFinite(s) || s < 0) return '0:00';
  const total = Math.floor(s);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const sec = total % 60;
  if (h > 0) return `${h}:${pad(m)}:${pad(sec)}`;
  return `${m}:${pad(sec)}`;
}

function VideoPlayer() {
  const { seriesId, episodeNumber } = useParams<{ seriesId?: string; episodeNumber?: string }>();
  const navigate = useNavigate();
  const { metadata } = useMetadata();
  const [videoSrc, setVideoSrc] = useState<string>('');
  const [subtitleSrcs, setSubtitleSrcs] = useState<SubtitleTrack[]>([]);
  const [episodeData, setEpisodeData] = useState<FileEpisode | null>(null);
  const [chrome, setChrome] = useState(true);
  const [isPlaying, setIsPlaying] = useState(false);
  const [duration, setDuration] = useState(0);
  const [currentTime, setCurrentTime] = useState(0);
  const [volume, setVolume] = useState(0.7);
  const [muted, setMuted] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [activeSubIdx, setActiveSubIdx] = useState<number>(-1); // -1 = off
  const [subMenuOpen, setSubMenuOpen] = useState(false);
  const [subMenuTab, setSubMenuTab] = useState<'tracks' | 'style'>('tracks');
  const [subStyle, setSubStyle] = useState<SubtitleStyle>(() => {
    // Storage key bumped to v2 because units changed from px → vh; old saved
    // values would otherwise interpret as enormous (22vh ≈ 240px on 1080p).
    try {
      const saved = localStorage.getItem('subtitle-style-v2');
      if (saved) return { ...DEFAULT_SUB_STYLE, ...JSON.parse(saved) as Partial<SubtitleStyle> };
    } catch { /* ignore */ }
    return DEFAULT_SUB_STYLE;
  });

  const videoRef = useRef<HTMLVideoElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const subMenuRef = useRef<HTMLDivElement>(null);
  const ccBtnRef = useRef<HTMLButtonElement>(null);

  // Auto-hide chrome after inactivity
  useEffect(() => {
    const reset = () => {
      if (hideTimer.current) clearTimeout(hideTimer.current);
      hideTimer.current = setTimeout(() => setChrome(false), 2500);
    };
    reset();
    return () => {
      if (hideTimer.current) clearTimeout(hideTimer.current);
    };
  }, []);

  // Web Audio gain stage — exponential volume curve with boost past unity.
  // Curve: total output = slider² × MAX_BOOST. Slider 100% = 250%.
  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const AC = window.AudioContext || (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!AC) return;

    let audioCtx: AudioContext;
    let source: MediaElementAudioSourceNode;
    let gain: GainNode;
    try {
      audioCtx = new AC();
      source = audioCtx.createMediaElementSource(video);
      gain = audioCtx.createGain();
      source.connect(gain).connect(audioCtx.destination);
    } catch {
      return;
    }

    const MAX_BOOST = 2.5;
    const updateGain = () => {
      gain.gain.value = video.volume * MAX_BOOST;
    };
    const resume = () => { if (audioCtx.state === 'suspended') void audioCtx.resume(); };
    updateGain();

    video.addEventListener('volumechange', updateGain);
    video.addEventListener('play', resume);

    return () => {
      video.removeEventListener('volumechange', updateGain);
      video.removeEventListener('play', resume);
      try { source.disconnect(); } catch { /* ignore */ }
      try { gain.disconnect(); } catch { /* ignore */ }
      void audioCtx.close();
    };
  }, []);

  const showChrome = useCallback(() => {
    setChrome(true);
    if (hideTimer.current) clearTimeout(hideTimer.current);
    hideTimer.current = setTimeout(() => setChrome(false), 2500);
  }, []);

  // Load episode
  useEffect(() => {
    if (!seriesId || !episodeNumber || !metadata[seriesId]) return;

    const seriesData = metadata[seriesId];
    const episodeNum = parseInt(episodeNumber, 10);
    if (isNaN(episodeNum)) return;

    const fileEpisodes = seriesData.fileEpisodes || [];
    const episode = fileEpisodes.find(
      (ep: FileEpisode) => ep.episodeNumber === episodeNum
    );

    if (episode && episode.filePath) {
      setEpisodeData(episode);
      setVideoSrc(`file://${episode.filePath}`);

      // Build the subtitle list:
      //  1. External .srt/.vtt sidecar files (convert .srt → VTT blob)
      //  2. Embedded text-based subtitle streams in the MKV (extract via
      //     ffmpeg in main, served from the on-disk cache via media://)
      const buildSubs = async () => {
        const out: SubtitleTrack[] = [];

        // External sidecars
        const sidecars: { path: string; label: string; default: boolean }[] = [];
        if (episode.subtitlePath) {
          sidecars.push({ path: episode.subtitlePath, label: 'External', default: true });
        }
        if (episode.subtitlePaths && episode.subtitlePaths.length > 0) {
          episode.subtitlePaths.forEach((p: string, i: number) => {
            if (p !== episode.subtitlePath) {
              sidecars.push({ path: p, label: `External ${i + 2}`, default: false });
            }
          });
        }
        for (const r of sidecars) {
          const fileUrl = `file://${r.path}`;
          const ext = r.path.toLowerCase().split('.').pop();
          const src = ext === 'srt' ? await srtToVttUrl(fileUrl) : fileUrl;
          out.push({ src, origPath: r.path, kind: 'subtitles', label: r.label, default: r.default });
        }

        // Embedded streams (MKV / MP4 with internal subs)
        try {
          const embedded = await window.electronAPI.listEmbeddedSubtitles(episode.filePath);
          for (const e of embedded) {
            const cachePath = await window.electronAPI.extractEmbeddedSubtitle(episode.filePath, e.streamIndex);
            if (!cachePath) continue;
            const lang = e.language ? e.language.toUpperCase() : null;
            const label = e.title && lang
              ? `${lang} — ${e.title}`
              : e.title ?? (lang ?? `Track #${e.streamIndex}`);
            // Cache files live under userData; media:// protocol allows that
            // path. Make this the default if there's no external sidecar
            // already marked default.
            const isDefault = !out.some((s) => s.default) && (lang === 'ENG' || lang === 'EN' || out.length === 0);
            out.push({
              src: `media://${cachePath}`,
              origPath: episode.filePath,
              kind: 'subtitles',
              label,
              default: isDefault,
            });
          }
        } catch (err) {
          console.warn('Embedded subtitle extraction failed:', err);
        }

        setSubtitleSrcs((prev) => {
          // Revoke previous blob URLs so they don't leak.
          prev.forEach((p) => { if (p.src.startsWith('blob:')) URL.revokeObjectURL(p.src); });
          return out;
        });
      };
      void buildSubs();
    }
  }, [seriesId, episodeNumber, metadata]);

  // Activate the default subtitle track once the video metadata loads.
  // Re-runs whenever the subtitle list changes (i.e. per episode).
  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const apply = () => {
      const tracks = video.textTracks;
      let defaultIdx = -1;
      for (let i = 0; i < tracks.length; i++) {
        if (subtitleSrcs[i]?.default) { defaultIdx = i; break; }
      }
      for (let i = 0; i < tracks.length; i++) {
        tracks[i].mode = i === defaultIdx ? 'showing' : 'disabled';
      }
      setActiveSubIdx(defaultIdx);
    };
    video.addEventListener('loadedmetadata', apply);
    // Also try right now in case metadata is already loaded.
    if (video.readyState >= 1) apply();
    return () => video.removeEventListener('loadedmetadata', apply);
  }, [subtitleSrcs]);

  const selectSubtitle = (idx: number) => {
    const video = videoRef.current;
    if (!video) return;
    const tracks = video.textTracks;
    for (let i = 0; i < tracks.length; i++) {
      tracks[i].mode = i === idx ? 'showing' : 'disabled';
    }
    setActiveSubIdx(idx);
  };

  // Persist subtitle style to localStorage on change.
  useEffect(() => {
    try { localStorage.setItem('subtitle-style-v2', JSON.stringify(subStyle)); } catch { /* ignore */ }
  }, [subStyle]);

  // Apply the bottom-offset to every cue on the active subtitle track.
  // Chromium ignores CSS attempts to move ::-webkit-media-text-track-container,
  // so we have to use the WebVTT spec's own positioning: set cue.snapToLines
  // = false and cue.line = "% from top". subStyle.positionBottom is in
  // percent-of-video-height (0 = sitting on the bottom edge, 50 = mid screen).
  useEffect(() => {
    const video = videoRef.current;
    if (!video || activeSubIdx < 0) return;
    const track = video.textTracks[activeSubIdx];
    if (!track) return;

    const apply = () => {
      if (!track.cues) return;
      const linePct = Math.max(0, Math.min(100, 100 - subStyle.positionBottom));
      for (let i = 0; i < track.cues.length; i++) {
        const cue = track.cues[i] as VTTCue;
        if (typeof cue.line !== 'undefined') {
          cue.snapToLines = false;
          cue.line = linePct;
        }
      }
    };
    apply();
    // Cues for an extracted track may finish loading shortly after the track
    // becomes active; re-apply on cuechange so newly-arriving cues get
    // positioned correctly too.
    track.addEventListener('cuechange', apply);
    return () => track.removeEventListener('cuechange', apply);
  }, [activeSubIdx, subStyle.positionBottom, subtitleSrcs]);

  // Close subtitle menu on outside click.
  useEffect(() => {
    if (!subMenuOpen) return;
    const onClick = (e: MouseEvent) => {
      const target = e.target as Node | null;
      if (!target) return;
      if (subMenuRef.current?.contains(target)) return;
      if (ccBtnRef.current?.contains(target)) return;
      setSubMenuOpen(false);
    };
    document.addEventListener('mousedown', onClick);
    return () => document.removeEventListener('mousedown', onClick);
  }, [subMenuOpen]);

  // Keep chrome visible while the subtitle menu is open.
  useEffect(() => {
    if (subMenuOpen) {
      setChrome(true);
      if (hideTimer.current) clearTimeout(hideTimer.current);
    } else {
      showChrome();
    }
  }, [subMenuOpen, showChrome]);

  // Wire <video> events to local state.
  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const onPlay = () => setIsPlaying(true);
    const onPause = () => setIsPlaying(false);
    const onTime = () => setCurrentTime(video.currentTime);
    const onMeta = () => setDuration(video.duration);
    const onVol = () => { setVolume(video.volume); setMuted(video.muted); };

    video.addEventListener('play', onPlay);
    video.addEventListener('pause', onPause);
    video.addEventListener('timeupdate', onTime);
    video.addEventListener('loadedmetadata', onMeta);
    video.addEventListener('durationchange', onMeta);
    video.addEventListener('volumechange', onVol);

    // Pick up state right now in case the relevant events already fired
    // before this effect attached (race that froze the scrub bar).
    if (video.readyState >= 1 && Number.isFinite(video.duration)) setDuration(video.duration);
    setCurrentTime(video.currentTime);
    setIsPlaying(!video.paused);
    setVolume(video.volume);
    setMuted(video.muted);

    return () => {
      video.removeEventListener('play', onPlay);
      video.removeEventListener('pause', onPause);
      video.removeEventListener('timeupdate', onTime);
      video.removeEventListener('loadedmetadata', onMeta);
      video.removeEventListener('durationchange', onMeta);
      video.removeEventListener('volumechange', onVol);
    };
  }, [videoSrc]);

  // Set initial volume on mount
  useEffect(() => {
    const video = videoRef.current;
    if (video) video.volume = 0.7;
  }, []);

  // Fullscreen state sync
  useEffect(() => {
    const onFs = () => setIsFullscreen(!!document.fullscreenElement);
    document.addEventListener('fullscreenchange', onFs);
    return () => document.removeEventListener('fullscreenchange', onFs);
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const video = videoRef.current;
      if (!video) return;
      const tag = (e.target as HTMLElement | null)?.tagName?.toLowerCase();
      if (tag === 'input' || tag === 'textarea') return;
      if (e.key === ' ' || e.key === 'k') {
        e.preventDefault();
        if (video.paused) void video.play(); else video.pause();
        showChrome();
      } else if (e.key === 'ArrowRight') {
        e.preventDefault();
        video.currentTime = Math.min(video.duration || Infinity, video.currentTime + 5);
        showChrome();
      } else if (e.key === 'ArrowLeft') {
        e.preventDefault();
        video.currentTime = Math.max(0, video.currentTime - 5);
        showChrome();
      } else if (e.key === 'm') {
        e.preventDefault();
        video.muted = !video.muted;
        showChrome();
      } else if (e.key === 'f') {
        e.preventDefault();
        toggleFullscreen();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [showChrome]);

  const togglePlay = () => {
    const video = videoRef.current;
    if (!video) return;
    if (video.paused) void video.play(); else video.pause();
  };

  const toggleMute = () => {
    const video = videoRef.current;
    if (!video) return;
    video.muted = !video.muted;
  };

  const toggleFullscreen = () => {
    const wrap = wrapRef.current;
    if (!wrap) return;
    if (!document.fullscreenElement) {
      void wrap.requestFullscreen();
    } else {
      void document.exitFullscreen();
    }
  };

  const onSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const video = videoRef.current;
    if (!video) return;
    video.currentTime = Number(e.target.value);
  };

  const onVolume = (e: React.ChangeEvent<HTMLInputElement>) => {
    const video = videoRef.current;
    if (!video) return;
    const v = Number(e.target.value);
    video.volume = v;
    if (v > 0) video.muted = false;
  };

  if (!episodeData || !seriesId || !episodeNumber) {
    return (
      <div className="player-wrap">
        <div className="player-header" style={{ opacity: 1 }}>
          <button className="player-back" onClick={() => navigate(seriesId ? `/series/${seriesId}` : '/')}>
            <ArrowLeft size={15} />
            <span>Back</span>
          </button>
        </div>
        <div className="player-canvas">
          <div style={{ color: 'rgba(255,255,255,0.6)', fontFamily: 'var(--font-mono)' }}>
            Loading episode…
          </div>
        </div>
      </div>
    );
  }

  const seriesTitle = metadata[seriesId]?.title || '';
  const seasonNumber = episodeData.seasonNumber ?? null;
  const episodeNum = parseInt(episodeNumber, 10);
  const code = seasonNumber !== null
    ? `S${pad(seasonNumber)}E${pad(episodeNum)}`
    : `EP ${episodeNum}`;

  const cueCss = `
    .player-canvas video::cue {
      background-color: rgba(${hexToRgb(subStyle.bgColor)}, ${subStyle.bgOpacity});
      color: ${subStyle.color};
      font-size: ${subStyle.fontSize}vh;
      font-family: ${subStyle.fontFamily};
      text-shadow: ${OUTLINE_PRESETS[subStyle.outline]};
    }
  `;

  return (
    <div className="player-wrap" ref={wrapRef} onMouseMove={showChrome}>
      <style>{cueCss}</style>
      <div className="player-header" style={{ opacity: chrome ? 1 : 0 }}>
        <button
          className="player-back"
          onClick={() => navigate(`/series/${seriesId}`)}
          aria-label="Back to series"
        >
          <ArrowLeft size={15} />
          <span>Back</span>
        </button>
        <div className="player-titles">
          {seriesTitle && <div className="player-eyebrow">{seriesTitle}</div>}
          <h2 className="player-title">
            {episodeData.title || `Episode ${episodeNum}`}
          </h2>
          <div className="player-meta">{code}</div>
        </div>
      </div>
      <div className="player-canvas" onClick={togglePlay}>
        <video
          ref={videoRef}
          src={videoSrc}
          autoPlay
        >
          {subtitleSrcs.map((subtitle, index) => (
            <track
              key={index}
              src={subtitle.src}
              kind={subtitle.kind}
              label={subtitle.label}
              default={subtitle.default}
            />
          ))}
          Your browser does not support the video tag.
        </video>
      </div>
      <div
        className="player-controls"
        style={{ opacity: chrome ? 1 : 0, pointerEvents: chrome ? 'auto' : 'none' }}
        onClick={(e) => e.stopPropagation()}
      >
        <input
          className="player-scrub"
          type="range"
          min={0}
          max={duration || 0}
          step={0.1}
          value={Math.min(currentTime, duration || 0)}
          onChange={onSeek}
          style={{ '--progress': `${duration > 0 ? (currentTime / duration) * 100 : 0}%` } as React.CSSProperties}
        />
        <div className="player-controls-row">
          <button className="player-ctl-btn" onClick={togglePlay} aria-label={isPlaying ? 'Pause' : 'Play'}>
            {isPlaying ? <Pause size={18} /> : <Play size={18} />}
          </button>
          <span className="player-time">{formatTime(currentTime)} / {formatTime(duration)}</span>
          <div className="player-volume">
            <button className="player-ctl-btn" onClick={toggleMute} aria-label={muted ? 'Unmute' : 'Mute'}>
              {muted || volume === 0 ? <VolumeX size={18} /> : <Volume2 size={18} />}
            </button>
            <input
              className="player-vol-slider"
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={muted ? 0 : volume}
              onChange={onVolume}
              style={{ '--progress': `${(muted ? 0 : volume) * 100}%` } as React.CSSProperties}
            />
          </div>
          <div className="player-right-group">
            {subtitleSrcs.length > 0 && (
              <div className="player-sub-anchor">
                <button
                  ref={ccBtnRef}
                  className={`player-ctl-btn${activeSubIdx >= 0 ? ' active' : ''}`}
                  onClick={() => setSubMenuOpen((v) => !v)}
                  aria-label="Subtitles menu"
                  title={activeSubIdx >= 0 ? subtitleSrcs[activeSubIdx]?.label ?? 'Subtitles' : 'Subtitles off'}
                >
                  <Subtitles size={18} />
                </button>
                {subMenuOpen && (
                  <div className="sub-menu" ref={subMenuRef}>
                    <div className="sub-menu-tabs">
                      <button
                        className={`sub-menu-tab${subMenuTab === 'tracks' ? ' active' : ''}`}
                        onClick={() => setSubMenuTab('tracks')}
                      >Tracks</button>
                      <button
                        className={`sub-menu-tab${subMenuTab === 'style' ? ' active' : ''}`}
                        onClick={() => setSubMenuTab('style')}
                      >Style</button>
                    </div>
                    {subMenuTab === 'tracks' ? (
                      <div className="sub-menu-body">
                        <button
                          className={`sub-menu-row${activeSubIdx === -1 ? ' active' : ''}`}
                          onClick={() => { selectSubtitle(-1); setSubMenuOpen(false); }}
                        >Off</button>
                        {subtitleSrcs.map((s, i) => (
                          <button
                            key={i}
                            className={`sub-menu-row${activeSubIdx === i ? ' active' : ''}`}
                            onClick={() => { selectSubtitle(i); setSubMenuOpen(false); }}
                          >{s.label}</button>
                        ))}
                      </div>
                    ) : (
                      <div className="sub-menu-body sub-menu-style">
                        <label className="sub-style-row">
                          <span>Size</span>
                          <input
                            type="range" min={1} max={15} step={0.25}
                            value={subStyle.fontSize}
                            onChange={(e) => setSubStyle((s) => ({ ...s, fontSize: Number(e.target.value) }))}
                          />
                          <span className="sub-style-val">{subStyle.fontSize.toFixed(2)}vh</span>
                        </label>
                        <label className="sub-style-row">
                          <span>Bottom %</span>
                          <input
                            type="range" min={0} max={50} step={1}
                            value={subStyle.positionBottom}
                            onChange={(e) => setSubStyle((s) => ({ ...s, positionBottom: Number(e.target.value) }))}
                          />
                          <span className="sub-style-val">{Math.round(subStyle.positionBottom)}%</span>
                        </label>
                        <label className="sub-style-row">
                          <span>Text color</span>
                          <input
                            type="color"
                            value={subStyle.color}
                            onChange={(e) => setSubStyle((s) => ({ ...s, color: e.target.value }))}
                          />
                        </label>
                        <label className="sub-style-row">
                          <span>Background</span>
                          <input
                            type="color"
                            value={subStyle.bgColor}
                            onChange={(e) => setSubStyle((s) => ({ ...s, bgColor: e.target.value }))}
                          />
                        </label>
                        <label className="sub-style-row">
                          <span>Bg opacity</span>
                          <input
                            type="range" min={0} max={1} step={0.05}
                            value={subStyle.bgOpacity}
                            onChange={(e) => setSubStyle((s) => ({ ...s, bgOpacity: Number(e.target.value) }))}
                          />
                          <span className="sub-style-val">{Math.round(subStyle.bgOpacity * 100)}%</span>
                        </label>
                        <label className="sub-style-row">
                          <span>Font</span>
                          <select
                            value={subStyle.fontFamily}
                            onChange={(e) => setSubStyle((s) => ({ ...s, fontFamily: e.target.value as SubtitleStyle['fontFamily'] }))}
                          >
                            <option value="sans-serif">Sans</option>
                            <option value="serif">Serif</option>
                            <option value="ui-monospace">Mono</option>
                          </select>
                        </label>
                        <label className="sub-style-row">
                          <span>Outline</span>
                          <select
                            value={subStyle.outline}
                            onChange={(e) => setSubStyle((s) => ({ ...s, outline: e.target.value as SubtitleStyle['outline'] }))}
                          >
                            <option value="none">None</option>
                            <option value="light">Light</option>
                            <option value="medium">Medium</option>
                            <option value="heavy">Heavy</option>
                          </select>
                        </label>
                        <button
                          className="sub-style-reset"
                          onClick={() => setSubStyle(DEFAULT_SUB_STYLE)}
                        >Reset to defaults</button>
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}
            <button className="player-ctl-btn" onClick={toggleFullscreen} aria-label={isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}>
              {isFullscreen ? <Minimize size={18} /> : <Maximize size={18} />}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default VideoPlayer;
