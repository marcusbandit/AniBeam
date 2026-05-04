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

  const videoRef = useRef<HTMLVideoElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

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

      // Build the subtitle list. For .srt files we convert to VTT on the fly
      // (Chromium's <track> only supports WebVTT natively).
      const buildSubs = async () => {
        const raw: { path: string; label: string; default: boolean }[] = [];
        if (episode.subtitlePath) {
          raw.push({ path: episode.subtitlePath, label: 'Subtitle', default: true });
        }
        if (episode.subtitlePaths && episode.subtitlePaths.length > 0) {
          episode.subtitlePaths.forEach((p: string, i: number) => {
            if (p !== episode.subtitlePath) {
              raw.push({ path: p, label: `Subtitle ${i + 2}`, default: false });
            }
          });
        }
        const out: SubtitleTrack[] = await Promise.all(raw.map(async (r) => {
          const fileUrl = `file://${r.path}`;
          const ext = r.path.toLowerCase().split('.').pop();
          const src = ext === 'srt' ? await srtToVttUrl(fileUrl) : fileUrl;
          return { src, origPath: r.path, kind: 'subtitles', label: r.label, default: r.default };
        }));
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

  const cycleSubtitle = () => {
    const video = videoRef.current;
    if (!video) return;
    const tracks = video.textTracks;
    const total = tracks.length;
    if (total === 0) return;
    // -1 (off) → 0 → 1 → ... → total-1 → -1
    const next = activeSubIdx + 1 >= total ? -1 : activeSubIdx + 1;
    for (let i = 0; i < total; i++) {
      tracks[i].mode = i === next ? 'showing' : 'disabled';
    }
    setActiveSubIdx(next);
    showChrome();
  };

  // Wire <video> events to local state
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
    video.addEventListener('volumechange', onVol);
    return () => {
      video.removeEventListener('play', onPlay);
      video.removeEventListener('pause', onPause);
      video.removeEventListener('timeupdate', onTime);
      video.removeEventListener('loadedmetadata', onMeta);
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

  return (
    <div className="player-wrap" ref={wrapRef} onMouseMove={showChrome}>
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
              <button
                className={`player-ctl-btn${activeSubIdx >= 0 ? ' active' : ''}`}
                onClick={cycleSubtitle}
                aria-label="Toggle subtitles"
                title={activeSubIdx >= 0 ? subtitleSrcs[activeSubIdx]?.label ?? 'Subtitles' : 'Subtitles off'}
              >
                <Subtitles size={18} />
              </button>
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
