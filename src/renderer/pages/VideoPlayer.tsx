import { useParams, useNavigate } from 'react-router-dom';
import { useState, useEffect, useRef, useCallback } from 'react';
import { useMetadata, type FileEpisode } from '../hooks/useMetadata.js';
import { ArrowLeft, Play, Pause, Volume2, VolumeX, Maximize, Minimize, Subtitles } from 'lucide-react';
import JASSUB from 'jassub';
import jassubWorkerUrl from 'jassub/dist/worker/worker.js?url';
import jassubWasmUrl from 'jassub/dist/wasm/jassub-worker.wasm?url';
import jassubWasmModernUrl from 'jassub/dist/wasm/jassub-worker-modern.wasm?url';
import jassubDefaultFontUrl from 'jassub/dist/default.woff2?url';

// Vite's dev server serves .wasm with application/octet-stream, which makes
// WebAssembly.instantiateStreaming() in the JASSUB worker reject the response.
// Fetch the bytes once in the renderer, wrap as a Blob with the correct MIME,
// and hand JASSUB the resulting blob: URLs. Cached so we don't refetch.
//
// JASSUB has TWO wasm bundles — a legacy one and a "modern" one for browsers
// that support newer WebAssembly features. Chromium picks the modern one, so
// we MUST override both or the worker silently falls back to its default URL
// (which still has the wrong MIME).
async function fetchAsWasmBlobUrl(url: string): Promise<string> {
  const buf = await (await fetch(url)).arrayBuffer();
  return URL.createObjectURL(new Blob([buf], { type: 'application/wasm' }));
}
let wasmUrlsPromise: Promise<{ wasmUrl: string; modernWasmUrl: string }> | null = null;
function getJassubWasmUrls() {
  if (!wasmUrlsPromise) {
    wasmUrlsPromise = Promise.all([
      fetchAsWasmBlobUrl(jassubWasmUrl),
      fetchAsWasmBlobUrl(jassubWasmModernUrl),
    ]).then(([wasmUrl, modernWasmUrl]) => ({ wasmUrl, modernWasmUrl }));
  }
  return wasmUrlsPromise;
}

interface SubtitleTrack {
  src: string;        // file:// or media:// or blob: URL — used for native VTT or JASSUB
  origPath: string;   // original file path
  kind: string;       // 'subtitles'
  label: string;
  default?: boolean;
  format: 'vtt' | 'ass';
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

type SubtitleFont = 'Arial, sans-serif' | 'sans-serif' | 'serif' | 'ui-monospace';

interface SubtitleStyle {
  fontSize: number;        // in vh — % of viewport height, scales with fullscreen
  positionBottom: number;  // distance above the bottom edge in vh
  color: string;
  bgColor: string;
  bgOpacity: number;
  fontFamily: SubtitleFont;
  outline: 'none' | 'light' | 'medium' | 'heavy';
}

const DEFAULT_SUB_STYLE: SubtitleStyle = {
  fontSize: 5,
  positionBottom: 8,
  color: '#ffffff',
  bgColor: '#000000',
  bgOpacity: 0,
  fontFamily: 'Arial, sans-serif',
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
  const [skipTimes, setSkipTimes] = useState<{ op?: { start: number; end: number }; ed?: { start: number; end: number } }>({});
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
  const jassubRef = useRef<JASSUB | null>(null);

  const tearDownJassub = () => {
    if (jassubRef.current) {
      try { jassubRef.current.destroy(); } catch { /* ignore */ }
      jassubRef.current = null;
    }
  };

  // Always tear down the libass renderer on unmount to free its worker.
  useEffect(() => () => tearDownJassub(), []);

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
          const ext = r.path.toLowerCase().split('.').pop();
          const fileUrl = `file://${r.path}`;
          if (ext === 'ass' || ext === 'ssa') {
            // JASSUB reads the file directly via its URL.
            out.push({ src: fileUrl, origPath: r.path, kind: 'subtitles', label: r.label, default: r.default, format: 'ass' });
          } else {
            const src = ext === 'srt' ? await srtToVttUrl(fileUrl) : fileUrl;
            out.push({ src, origPath: r.path, kind: 'subtitles', label: r.label, default: r.default, format: 'vtt' });
          }
        }

        // Embedded streams (MKV / MP4 with internal subs)
        try {
          const embedded = await window.electronAPI.listEmbeddedSubtitles(episode.filePath);
          for (const e of embedded) {
            const result = await window.electronAPI.extractEmbeddedSubtitle(episode.filePath, e.streamIndex, e.codec);
            if (!result) continue;
            const lang = e.language ? e.language.toUpperCase() : null;
            const label = e.title && lang
              ? `${lang} — ${e.title}`
              : e.title ?? (lang ?? `Track #${e.streamIndex}`);
            const isDefault = !out.some((s) => s.default) && (lang === 'ENG' || lang === 'EN' || out.length === 0);
            out.push({
              src: `media://${result.path}`,
              origPath: episode.filePath,
              kind: 'subtitles',
              label,
              default: isDefault,
              format: result.format,
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

  // Find the native textTrack that matches a given subtitleSrcs entry. Native
  // <track> elements are only rendered for VTT-format subs; ASS subs go to
  // JASSUB instead, so subtitleSrcs index ≠ textTracks index. We match by
  // label which we make unique per track.
  const findVttTrack = (sub: SubtitleTrack): TextTrack | null => {
    const video = videoRef.current;
    if (!video) return null;
    for (let i = 0; i < video.textTracks.length; i++) {
      if (video.textTracks[i].label === sub.label) return video.textTracks[i];
    }
    return null;
  };

  const selectSubtitle = (idx: number) => {
    const video = videoRef.current;
    if (!video) return;

    tearDownJassub();
    for (let i = 0; i < video.textTracks.length; i++) {
      video.textTracks[i].mode = 'disabled';
    }
    setActiveSubIdx(idx);
    if (idx < 0 || idx >= subtitleSrcs.length) return;

    const sub = subtitleSrcs[idx];

    if (sub.format === 'ass') {
      // Fetch the ASS file in the renderer (where the media:// protocol IS
      // registered) and hand the content to JASSUB. The JASSUB worker can't
      // fetch arbitrary URLs reliably from its own context.
      void (async () => {
        try {
          const [resp, wasmUrls] = await Promise.all([fetch(sub.src), getJassubWasmUrls()]);
          if (!resp.ok) throw new Error(`fetch ${sub.src} → ${resp.status}`);
          const subContent = await resp.text();
          // The user may have switched again before this resolves; bail.
          if (jassubRef.current) return;
          const inst = new JASSUB({
            video,
            subContent,
            workerUrl: jassubWorkerUrl,
            wasmUrl: wasmUrls.wasmUrl,
            modernWasmUrl: wasmUrls.modernWasmUrl,
            // Default font MUST be reachable. JASSUB's internal URL.import.meta
            // resolution doesn't survive Vite bundling, so we provide the URL
            // explicitly. Without this, libass has no glyphs and renders nothing.
            availableFonts: { 'liberation sans': jassubDefaultFontUrl },
            defaultFont: 'liberation sans',
            queryFonts: false,
            // Render at 4× display × DPR for high SSAA quality. Browser
            // downsamples on blit, killing aliasing in font edges. 4× is
            // heavier than 2× but font rendering is the visible artifact —
            // raise/lower this if perf becomes an issue.
            prescaleHeightLimit: 0,
            prescaleFactor: 4.0,
          } as ConstructorParameters<typeof JASSUB>[0]);
          jassubRef.current = inst;
          await (inst as unknown as { ready?: Promise<unknown> }).ready;
          console.log('[subs] JASSUB ready for', sub.label);

          // Sync sub frames to actual video frames via requestVideoFrameCallback.
          // This is the right API for sub timing — each callback delivers the
          // exact mediaTime + expected display time of a presented frame.
          // Falls back to rAF if rVFC doesn't tick for 500ms (defensive: in
          // earlier debugging we saw rVFC sometimes not fire in this Electron
          // context, but with the corrected font init it should work now).
          const renderRef = (inst as unknown as { manualRender: (m: { expectedDisplayTime: number; width: number; height: number; mediaTime: number }) => Promise<unknown> });
          let lastRvfcAt = 0;
          let usingRaf = false;

          const onFrame = (_now: number, meta: VideoFrameCallbackMetadata) => {
            if (jassubRef.current !== inst) return; // disposed
            lastRvfcAt = performance.now();
            try {
              void renderRef.manualRender({
                expectedDisplayTime: meta.expectedDisplayTime,
                width: meta.width,
                height: meta.height,
                mediaTime: meta.mediaTime,
              });
            } catch { /* ignore */ }
            videoRef.current?.requestVideoFrameCallback(onFrame);
          };
          if ('requestVideoFrameCallback' in video) {
            video.requestVideoFrameCallback(onFrame);
          }

          const rafPump = () => {
            if (jassubRef.current !== inst) return;
            // Only run rAF fallback if rVFC has gone quiet for >500ms while
            // the video is playing.
            const v = videoRef.current;
            const rvfcStale = performance.now() - lastRvfcAt > 500;
            if (v && !v.paused && v.readyState >= 2 && rvfcStale) {
              if (!usingRaf) { console.log('[subs] rVFC stalled, falling back to rAF pump'); usingRaf = true; }
              try {
                void renderRef.manualRender({
                  expectedDisplayTime: performance.now(),
                  width: v.videoWidth,
                  height: v.videoHeight,
                  mediaTime: v.currentTime,
                });
              } catch { /* ignore */ }
            } else if (usingRaf && !rvfcStale) {
              console.log('[subs] rVFC resumed');
              usingRaf = false;
            }
            requestAnimationFrame(rafPump);
          };
          requestAnimationFrame(rafPump);
        } catch (err) {
          console.error('JASSUB init failed:', err);
        }
      })();
      return;
    }

    // VTT path. The native <track>'s TextTrack may not have appeared in
    // video.textTracks yet right after subtitleSrcs changes — give it a
    // microtask to materialize, then retry.
    const tryEnable = (attempt = 0) => {
      const t = findVttTrack(sub);
      if (t) { t.mode = 'showing'; return; }
      if (attempt < 10) setTimeout(() => tryEnable(attempt + 1), 50);
      else console.warn('No matching textTrack for', sub.label);
    };
    tryEnable();
  };

  // Activate the default subtitle (VTT or ASS) when the list changes.
  // Re-runs per episode.
  useEffect(() => {
    if (subtitleSrcs.length === 0) {
      tearDownJassub();
      setActiveSubIdx(-1);
      return;
    }
    const defaultIdx = subtitleSrcs.findIndex((s) => s.default);
    const target = defaultIdx >= 0 ? defaultIdx : -1;
    const apply = () => selectSubtitle(target);
    const video = videoRef.current;
    if (video?.readyState && video.readyState >= 1) {
      apply();
    } else if (video) {
      video.addEventListener('loadedmetadata', apply, { once: true });
      return () => video.removeEventListener('loadedmetadata', apply);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [subtitleSrcs]);

  // Persist subtitle style to localStorage on change.
  useEffect(() => {
    try { localStorage.setItem('subtitle-style-v2', JSON.stringify(subStyle)); } catch { /* ignore */ }
  }, [subStyle]);

  // Bottom-offset for native VTT cues. Doesn't apply to ASS — JASSUB renders
  // those with the file's own positioning, untouched. We only adjust cues
  // whose original line === 'auto' so author-positioned signs stay put.
  useEffect(() => {
    if (activeSubIdx < 0) return;
    const sub = subtitleSrcs[activeSubIdx];
    if (!sub || sub.format !== 'vtt') return;
    const track = findVttTrack(sub);
    if (!track) return;

    const autoCues = new WeakSet<VTTCue>();
    const apply = () => {
      if (!track.cues) return;
      const linePct = Math.max(0, Math.min(100, 100 - subStyle.positionBottom));
      for (let i = 0; i < track.cues.length; i++) {
        const cue = track.cues[i] as VTTCue;
        if (!autoCues.has(cue) && cue.line === 'auto') autoCues.add(cue);
        if (autoCues.has(cue)) {
          cue.snapToLines = false;
          cue.line = linePct;
        }
      }
    };
    apply();
    track.addEventListener('cuechange', apply);
    return () => track.removeEventListener('cuechange', apply);
    // eslint-disable-next-line react-hooks/exhaustive-deps
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

  // Load AniSkip times for this episode. Use cached values from metadata if
  // already fetched; otherwise fetch (only when MAL ID is available — AniSkip
  // is keyed on MAL). Wait until we know `duration` so we can pass it to the
  // API for the better cross-version match.
  useEffect(() => {
    if (!seriesId || !episodeNumber || duration <= 0) return;
    const seriesData = metadata[seriesId];
    if (!seriesData) return;
    const epNum = parseInt(episodeNumber, 10);
    if (isNaN(epNum)) return;

    const epMeta = seriesData.episodes?.find((e) => e.episodeNumber === epNum);
    if (epMeta?.skipFetched) {
      setSkipTimes({
        op: epMeta.opStart != null && epMeta.opEnd != null ? { start: epMeta.opStart, end: epMeta.opEnd } : undefined,
        ed: epMeta.edStart != null && epMeta.edEnd != null ? { start: epMeta.edStart, end: epMeta.edEnd } : undefined,
      });
      return;
    }

    const malId = (seriesData as { malId?: number }).malId;
    if (!malId) return;
    let cancelled = false;
    void window.electronAPI.fetchSkipTimes(seriesId, malId, epNum, duration).then((res) => {
      if (!cancelled) setSkipTimes(res);
    });
    return () => { cancelled = true; };
  }, [seriesId, episodeNumber, duration, metadata]);

  // Reset skip times when the episode changes so the previous episode's
  // window doesn't briefly show on the new one.
  useEffect(() => {
    setSkipTimes({});
  }, [seriesId, episodeNumber]);

  const inOpWindow = skipTimes.op && currentTime >= skipTimes.op.start && currentTime < skipTimes.op.end;
  const inEdWindow = skipTimes.ed && currentTime >= skipTimes.ed.start && currentTime < skipTimes.ed.end;

  const skipForward = (toTime: number) => {
    const video = videoRef.current;
    if (!video) return;
    video.currentTime = Math.min(video.duration || toTime, toTime + 1);
  };

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
            // ASS tracks are rendered by JASSUB on a canvas overlay; only VTT
            // entries become native <track> children of the <video>.
            subtitle.format === 'vtt' ? (
              <track
                key={index}
                src={subtitle.src}
                kind={subtitle.kind}
                label={subtitle.label}
                default={subtitle.default}
              />
            ) : null
          ))}
          Your browser does not support the video tag.
        </video>
      </div>
      {(inOpWindow || inEdWindow) && (
        <button
          className="player-skip"
          onClick={(e) => {
            e.stopPropagation();
            if (inOpWindow && skipTimes.op) skipForward(skipTimes.op.end);
            else if (inEdWindow && skipTimes.ed) skipForward(skipTimes.ed.end);
          }}
        >
          {inOpWindow ? 'Skip Intro' : 'Skip Outro'}
        </button>
      )}
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
                            <option value="Arial, sans-serif">Arial</option>
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
