import { useParams, useNavigate, useLocation } from 'react-router-dom';
import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { useMetadata, type FileEpisode } from '../hooks/useMetadata.js';
import { useLocalStorage, useLocalStorageRecord } from '../hooks/useLocalStorage';
import { progressId, readProgress, writeProgress, recordEpisodeCompleted, RESUME_HEAD_SKIP, RESUME_TAIL_SKIP } from '../utils/playbackProgress';
import { ScorePicker, Tooltip } from '../components/primitives';
import { ArrowLeft, Play, Pause, Volume2, VolumeX, Maximize, Minimize, Subtitles, SkipBack, SkipForward, CheckCheck, AlertTriangle, ExternalLink, Loader2, RotateCcw } from 'lucide-react';
import JASSUB from 'jassub';
// `?worker&url` (not `?url`) so Vite bundles the worker as a self-contained
// ES module with its bare-import dependencies (abslink, lfa-ponyfill, etc.)
// resolved. With plain `?url` Vite copies the worker file raw — fine in dev
// because the dev server resolves bare imports on the fly, but in a packaged
// build the worker fetches the static file and chokes on `import 'abslink'`.
import jassubWorkerUrl from 'jassub/dist/worker/worker.js?worker&url';
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
const ASS_OUTLINE_THICKNESS: Record<SubtitleStyle['outline'], number> = {
  none: 0, light: 1, medium: 2, heavy: 3,
};

function hexToRgb(hex: string): string {
  const m = hex.match(/^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i);
  if (!m) return '0,0,0';
  return `${parseInt(m[1], 16)}, ${parseInt(m[2], 16)}, ${parseInt(m[3], 16)}`;
}

/**
 * Heuristic: is this ASS style name most likely a dialogue/spoken-text style
 * (vs a typesetting/sign style)? Conservative — when unsure we say "yes" so
 * the user sees more options to pick from.
 */
function isDialogueStyleName(name: string): boolean {
  const n = name.toLowerCase().trim();
  // "Default" is an ASS placeholder name that's almost never used by actual
  // dialogue events — exclude it so it doesn't pollute the dropdown.
  if (n === 'default' || n === 'default style' || n.startsWith('default ')) return false;
  // Hard exclude — these are clearly typesetting / signs.
  if (/^sign[_\s-]|^_sign|sign_\d|_sign_/i.test(name)) return false;
  if (/(^|[_\s-])(sign|signs|box|caption|note|disclaimer|credit|next.?episode|preview|circuit|attack|button|menu|overlay|on.?screen|location|placard|title.?card|subtitle.?list|opening|ending|op[_\s-]|ed[_\s-])/i.test(n)) return false;
  // Hard include — common dialogue style names.
  if (['main', 'dialogue', 'dialog', 'italics', 'italic', 'narrator', 'narration', 'top', 'alt'].includes(n)) return true;
  if (n.startsWith('main') || n.endsWith('italics') || n.endsWith(' alt')) return true;
  // Otherwise: include (let the user filter visually).
  return true;
}

function parsePlayResY(assText: string): number {
  const m = assText.match(/^\s*PlayResY:\s*(\d+)/im);
  return m ? parseInt(m[1], 10) : 288;
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

// Note: VideoPlayer is intentionally NOT migrated to the design-system
// primitives (Page/Section/Stack/Card/etc.). The fullscreen player is a
// single-purpose route with bespoke chrome (.player-*) that already reads
// consistently and has no off-center / pixel-brother gestalt bugs. JASSUB
// and the mpv-fallback wiring are load-bearing workarounds — don't touch
// the internals here. See docs/superpowers/specs/2026-05-21-gestalt-overhaul-design.md.
function VideoPlayer() {
  const { seriesId, episodeNumber } = useParams<{ seriesId?: string; episodeNumber?: string }>();
  const navigate = useNavigate();
  const location = useLocation();
  const { metadata } = useMetadata();
  const [videoSrc, setVideoSrc] = useState<string>('');
  // Playback gate. `null` = direct playback; otherwise an overlay is showing.
  //  - mode 'transcoding': ffmpeg is producing the cached MP4. Overlay shows
  //    a progress bar; playback starts automatically when it finishes.
  //  - mode 'decode-failed': <video> errored out even though we thought the
  //    codec was OK. No transcode in flight — only mpv fallback.
  const [unsupported, setUnsupported] = useState<
    | { mode: 'transcoding'; vCodec?: string; aCodec?: string }
    | { mode: 'decode-failed' }
    | null
  >(null);
  const [transcodeProgress, setTranscodeProgress] = useState<{
    currentSec: number;
    totalSec: number;
    fraction: number;
    speed: number | null;
    etaSec: number | null;
  } | null>(null);
  const [mpvLaunching, setMpvLaunching] = useState(false);
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
  const [skipTimes, setSkipTimes] = useState<{ op?: { start: number; end: number }; ed?: { start: number; end: number }; source?: 'chapters' | 'aniskip' }>({});
  const [subMenuOpen, setSubMenuOpen] = useState(false);
  const [subMenuTab, setSubMenuTab] = useState<'tracks' | 'style'>('tracks');
  // VTT styling — applied via ::cue CSS. One global setting for all SRT/VTT.
  const [vttStyle, setVttStyle] = useLocalStorage<SubtitleStyle>('subtitle-style-v2', DEFAULT_SUB_STYLE);
  // ASS styling — keyed by style name so settings follow the style across
  // shows. Edit "Main" once → applies anywhere "Main" appears.
  const [assStyles, setAssStyles] = useLocalStorageRecord<SubtitleStyle>('subtitle-style-ass-v1', {});
  // Dialogue style names detected in the current ASS track (populated when
  // an ASS track is selected). Used to drive the dropdown in the Style tab.
  const [assDialogueStyleNames, setAssDialogueStyleNames] = useState<string[]>([]);
  const [selectedAssStyle, setSelectedAssStyle] = useState<string | null>(null);
  const assPlayResYRef = useRef<number>(288);
  // Captured once when JASSUB initializes — every override is applied on top
  // of these originals, and clearing an override restores the original.
  // Without this snapshot, setStyle mutates the wasm's style in place and
  // there's no way back to the file's defaults.
  const assOriginalsRef = useRef<Array<Record<string, unknown>>>([]);
  // Bumps each time a JASSUB instance finishes initializing, so dependent
  // effects can react after JASSUB is actually ready (selectSubtitle is async
  // — activeSubIdx changes BEFORE jassubRef.current is set).
  const [jassubReadyTick, setJassubReadyTick] = useState(0);
  // Auto-play-next overlay. `nextVisible` shows the idle "Next" pill the moment
  // the outro begins; `nextCounting` starts the 5s fill + auto-advance timer
  // (3s before the end, or right when the outro ends if the post-outro tail is
  // under 20s); `nextDismissed` latches the overlay off for the rest of the
  // episode once the user picks Stay. `videoEnded` drives the center replay icon.
  const [nextVisible, setNextVisible] = useState(false);
  const [nextCounting, setNextCounting] = useState(false);
  const [nextDismissed, setNextDismissed] = useState(false);
  const [videoEnded, setVideoEnded] = useState(false);

  const videoRef = useRef<HTMLVideoElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const hideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const subMenuRef = useRef<HTMLDivElement>(null);
  const ccBtnRef = useRef<HTMLButtonElement>(null);
  const jassubRef = useRef<JASSUB | null>(null);
  // Last subtitle track the user had on, so the "C" key toggles captions off
  // and back on to the SAME language rather than cycling through tracks.
  const lastSubIdxRef = useRef<number | null>(null);
  // Mirror of the captions-toggle closure so the keydown listener (bound on a
  // narrow dep set) always invokes the latest state without restaling.
  const toggleCaptionsRef = useRef<() => void>(() => {});
  // Gate for the auto-next overlay. Episode navigation reuses this same
  // component (route param change, no unmount), so the previous episode's
  // end-of-stream currentTime/duration linger for a frame after we navigate —
  // long enough to instantly re-trigger the countdown and skip again. We arm
  // only once the *new* video has loaded, and disarm synchronously on every
  // episode change, so a stale frame can never fire the auto-advance.
  const autoNextArmedRef = useRef(false);

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

  // Resume tracking. The id ref tracks which episode the video element is
  // currently playing; the persistence effect uses it from a long-lived
  // listener so we don't have to re-attach handlers on every navigation.
  // We update the id only after videoSrc has actually changed so timeupdate
  // events that fire BEFORE the new src is committed still attribute to the
  // outgoing episode (otherwise we'd briefly write the wrong id).
  const currentIdRef = useRef<string>('');
  useEffect(() => {
    currentIdRef.current = (seriesId && episodeNumber && videoSrc)
      ? progressId(seriesId, episodeNumber)
      : '';
  }, [seriesId, episodeNumber, videoSrc]);

  // Restore saved playback position when an episode loads. Runs once per
  // episode-change via the loadedmetadata event so we have a real duration
  // before deciding whether to resume.
  useEffect(() => {
    if (!videoSrc || !seriesId || !episodeNumber) return;
    const video = videoRef.current;
    if (!video) return;
    const id = progressId(seriesId, episodeNumber);
    // The Next/Prev episode buttons navigate with state.skipResume = true so
    // the new episode always starts from 0 regardless of saved position.
    const skipResume = (location.state as { skipResume?: boolean } | null)?.skipResume === true;
    const seek = () => {
      if (skipResume) {
        try { video.currentTime = 0; } catch { /* ignore */ }
        return;
      }
      const entry = readProgress()[id];
      if (!entry) return;
      const dur = video.duration;
      if (!Number.isFinite(dur) || dur <= 0) return;
      if (entry.t < RESUME_HEAD_SKIP || entry.t > dur - RESUME_TAIL_SKIP) return;
      try { video.currentTime = entry.t; } catch { /* ignore */ }
    };
    if (video.readyState >= 1 && Number.isFinite(video.duration)) {
      seek();
      return;
    }
    video.addEventListener('loadedmetadata', seek, { once: true });
    return () => video.removeEventListener('loadedmetadata', seek);
  }, [videoSrc, seriesId, episodeNumber]);

  // Persist position on a 4s heartbeat + at every pause and on unload. Clear
  // the entry once the user has effectively finished the episode (within the
  // tail window) so the next play starts fresh instead of jumping to credits.
  // Depend on videoSrc so we (re-)attach AFTER the <video> element actually
  // mounts — on the first render the player shows a loading shell with no
  // <video>, so videoRef.current is null and an empty-deps effect would bail
  // out for the entire lifetime of this mount.
  useEffect(() => {
    if (!videoSrc) return;
    const video = videoRef.current;
    if (!video) return;

    let lastSave = 0;
    const save = () => {
      const id = currentIdRef.current;
      if (!id) return;
      const t = video.currentTime;
      const d = video.duration;
      if (!Number.isFinite(t) || !Number.isFinite(d) || d <= 0) return;
      const map = readProgress();
      if (t >= d - RESUME_TAIL_SKIP) {
        if (map[id]) { delete map[id]; writeProgress(map); }
        if (seriesId && episodeNumber) {
          recordEpisodeCompleted(seriesId, Number(episodeNumber));
        }
      } else if (t > RESUME_HEAD_SKIP) {
        map[id] = { t, d, updated: Date.now() };
        writeProgress(map);
      }
    };
    const onTime = () => {
      const now = Date.now();
      if (now - lastSave < 4000) return;
      lastSave = now;
      save();
    };
    const onEnded = () => {
      const id = currentIdRef.current;
      if (!id) return;
      const map = readProgress();
      if (map[id]) { delete map[id]; writeProgress(map); }
      if (seriesId && episodeNumber) {
        recordEpisodeCompleted(seriesId, Number(episodeNumber));
      }
    };

    video.addEventListener('timeupdate', onTime);
    video.addEventListener('pause', save);
    video.addEventListener('ended', onEnded);
    window.addEventListener('beforeunload', save);

    return () => {
      save();
      video.removeEventListener('timeupdate', onTime);
      video.removeEventListener('pause', save);
      video.removeEventListener('ended', onEnded);
      window.removeEventListener('beforeunload', save);
    };
  }, [videoSrc, seriesId, episodeNumber]);

  // Mark the series as "viewed" once the user has accumulated >= 30s of
  // forward playback within this mount. Used by the Library "Last viewed"
  // sort. Accumulating real elapsed time (rather than reading currentTime
  // directly) means a scrub to ep credits doesn't count as a view, but
  // pause/resume sessions do — what the user would intuit by "I watched
  // some of this". Fires once per mount; navigating to a new episode
  // remounts the effect and re-arms the threshold.
  useEffect(() => {
    if (!videoSrc || !seriesId || !episodeNumber) return;
    const video = videoRef.current;
    if (!video) return;

    const epNum = Number(episodeNumber);
    if (!Number.isFinite(epNum)) return;

    let accumulated = 0;        // seconds of forward play observed
    let lastTime = video.currentTime;
    let marked = false;

    const VIEW_THRESHOLD_SEC = 30;
    // Cap per-tick advance — guards against seeks and tab-backgrounding
    // gaps. timeupdate normally fires ~4×/sec, so 2s is generous headroom
    // for legitimate playback while still rejecting jumps.
    const MAX_TICK_DELTA = 2;

    const onTime = () => {
      if (marked) return;
      const t = video.currentTime;
      const dt = t - lastTime;
      lastTime = t;
      if (dt > 0 && dt <= MAX_TICK_DELTA && !video.paused && !video.seeking) {
        accumulated += dt;
        if (accumulated >= VIEW_THRESHOLD_SEC) {
          marked = true;
          window.electronAPI.markEpisodeViewed?.({
            seriesId,
            episodeNumber: epNum,
            ts: Date.now(),
          }).catch((err: unknown) => {
            console.warn('[view-history] markEpisodeViewed failed:', err);
          });
        }
      }
    };
    const onSeeking = () => { lastTime = video.currentTime; };

    video.addEventListener('timeupdate', onTime);
    video.addEventListener('seeking', onSeeking);
    return () => {
      video.removeEventListener('timeupdate', onTime);
      video.removeEventListener('seeking', onSeeking);
    };
  }, [videoSrc, seriesId, episodeNumber]);

  // Derive the active episode's filePath from metadata. The memo only
  // returns a new value if `seriesId`, `episodeNumber`, or the resolved
  // filePath actually changes — keeps the open-video effect from re-firing
  // every time some unrelated metadata field updates mid-playback.
  const activeFilePath = useMemo<string | null>(() => {
    if (!seriesId || !episodeNumber || !metadata[seriesId]) return null;
    const seriesData = metadata[seriesId];
    const episodeNum = parseInt(episodeNumber, 10);
    if (isNaN(episodeNum)) return null;
    const fileEpisodes = seriesData.fileEpisodes || [];
    const episode = fileEpisodes.find((ep: FileEpisode) => ep.episodeNumber === episodeNum);
    return episode?.filePath ?? null;
  }, [seriesId, episodeNumber, metadata]);

  // Open the underlying video through main — main checks for a cached
  // pre-transcoded copy and returns whichever URL the <video> should use.
  // The `cancelled` flag guards against strict-mode double-mount races
  // where a stale openVideo() resolves AFTER the unmount.
  useEffect(() => {
    if (!activeFilePath) return;
    const filePath = activeFilePath;
    let cancelled = false;

    // Reset on episode change so the popup doesn't linger across switches.
    setUnsupported(null);
    setTranscodeProgress(null);
    setVideoSrc('');

    (async () => {
      try {
        const result = await window.electronAPI.openVideo(filePath);
        if (cancelled) return;
        if (result.kind === 'transcoding') {
          setUnsupported({ mode: 'transcoding', vCodec: result.vCodec, aCodec: result.aCodec });
          return;
        }
        if (result.kind === 'unsupported') {
          setUnsupported({ mode: 'decode-failed' });
          return;
        }
        setVideoSrc(result.url);
      } catch (err) {
        if (cancelled) return;
        console.warn('openVideo failed, falling back to direct:', err);
        setVideoSrc(`file://${filePath}`);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [activeFilePath]);

  // While a file is being transcoded, subscribe to status + progress IPC
  // events. When the file flips to 'ready' (cached MP4 has been published),
  // re-call openVideo so the renderer picks up the cached URL and playback
  // begins automatically — the user never has to click anything.
  useEffect(() => {
    if (!activeFilePath) return;
    const filePath = activeFilePath;

    const offStatus = window.electronAPI.onMetadataFileStatusChanged((payload) => {
      if (payload.filePath !== filePath) return;
      if (payload.status === 'ready') {
        setTranscodeProgress(null);
        // Re-open: the cached MP4 should now exist on disk, so video:open
        // returns kind='direct'. If for some reason it doesn't, we'll get
        // 'transcoding' back and the overlay just stays up.
        (async () => {
          try {
            const result = await window.electronAPI.openVideo(filePath);
            if (result.kind === 'direct') {
              setUnsupported(null);
              setVideoSrc(result.url);
            }
          } catch (err) {
            console.warn('post-transcode reopen failed:', err);
          }
        })();
      } else if (payload.status === 'stalled') {
        // The encode died (driver error, source corruption, ran out of
        // disk). Fall back to the decode-failed branch so the user gets
        // the mpv escape hatch instead of an indefinite progress bar.
        setTranscodeProgress(null);
        setUnsupported({ mode: 'decode-failed' });
      } else if (payload.status === 'transcoding') {
        // Edge case: file just started transcoding while the player was
        // already mounted (e.g. user opened it before the scanner queued
        // it). Make sure the overlay is up.
        setUnsupported((prev) => prev?.mode === 'transcoding' ? prev : { mode: 'transcoding' });
      }
    });

    const offProgress = window.electronAPI.onTranscodeProgress((payload) => {
      if (payload.filePath !== filePath) return;
      setTranscodeProgress({
        currentSec: payload.currentSec,
        totalSec: payload.totalSec,
        fraction: payload.fraction,
        speed: payload.speed,
        etaSec: payload.etaSec,
      });
    });

    return () => {
      offStatus();
      offProgress();
    };
  }, [activeFilePath]);

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
          assPlayResYRef.current = parsePlayResY(subContent);
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
            // SSAA: render at 2× display × DPR, browser downsamples on blit.
            // prescaleHeightLimit is the CAP under which prescaleFactor is
            // applied — it is NOT "0 = unlimited" (that branch becomes
            // `result <= 0` which never fires). Set huge so the upscale
            // branch always runs; maxRenderHeight: 0 means no upper cap.
            prescaleHeightLimit: 8640,
            prescaleFactor: 2.0,
            maxRenderHeight: 0,
          } as ConstructorParameters<typeof JASSUB>[0]);
          jassubRef.current = inst;
          await (inst as unknown as { ready?: Promise<unknown> }).ready;
          console.log('[subs] JASSUB ready for', sub.label);
          // Trigger dialogue-style detection / override-application effects
          // now that the renderer is reachable.
          setJassubReadyTick((t) => t + 1);

          // Render every rAF tick with the current mediaTime. JASSUB's
          // internal _demandRender skips redundant renders (same frame, no
          // change since last) so this is cheap. Letting it dedupe internally
          // avoids the trap of trying to be clever with frame counters that
          // don't update on the same cadence as the compositor.
          const renderRef = (inst as unknown as { manualRender: (m: { expectedDisplayTime: number; width: number; height: number; mediaTime: number }) => Promise<unknown> });
          const pump = () => {
            if (jassubRef.current !== inst) return;
            const v = videoRef.current;
            if (v && v.readyState >= 2) {
              try {
                void renderRef.manualRender({
                  expectedDisplayTime: performance.now(),
                  width: v.videoWidth,
                  height: v.videoHeight,
                  mediaTime: v.currentTime,
                });
              } catch { /* ignore */ }
            }
            requestAnimationFrame(pump);
          };
          requestAnimationFrame(pump);
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
      const linePct = Math.max(0, Math.min(100, 100 - vttStyle.positionBottom));
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
  }, [activeSubIdx, vttStyle.positionBottom, subtitleSrcs]);

  // When an ASS track activates AND JASSUB is ready: snapshot the file's
  // original styles, list the dialogue-named ones for the dropdown.
  useEffect(() => {
    if (activeSubIdx < 0) {
      setAssDialogueStyleNames([]);
      setSelectedAssStyle(null);
      assOriginalsRef.current = [];
      return;
    }
    const sub = subtitleSrcs[activeSubIdx];
    if (!sub || sub.format !== 'ass') {
      setAssDialogueStyleNames([]);
      setSelectedAssStyle(null);
      assOriginalsRef.current = [];
      return;
    }
    const inst = jassubRef.current;
    if (!inst) return;
    let cancelled = false;
    void (async () => {
      try {
        await (inst as unknown as { ready: Promise<void> }).ready;
        const r = (inst as unknown as { renderer: {
          getStyles: () => Promise<Array<Record<string, unknown>>>;
          getEvents: () => Promise<Array<Record<string, unknown>>>;
        } }).renderer;
        const [styles, events] = await Promise.all([r.getStyles(), r.getEvents()]);
        if (cancelled) return;
        assOriginalsRef.current = styles.map((s) => ({ ...s }));

        // Count how many events use each style index, then surface dialogue
        // styles ordered by usage. Styles with 0 uses (e.g. an unused
        // "Default" placeholder) drop out entirely.
        const usageByIdx = new Map<number, number>();
        for (const ev of events) {
          const idx = ev.Style as number;
          usageByIdx.set(idx, (usageByIdx.get(idx) ?? 0) + 1);
        }
        const ranked = styles
          .map((s, i) => ({ name: s.Name as string, count: usageByIdx.get(i) ?? 0 }))
          .filter((s) => isDialogueStyleName(s.name) && s.count > 0)
          .sort((a, b) => b.count - a.count)
          .map((s) => s.name);

        setAssDialogueStyleNames(ranked);
        // Default selection = most-used dialogue style. Keep prior selection
        // if it still exists in the new list.
        setSelectedAssStyle((prev) => (prev && ranked.includes(prev) ? prev : (ranked[0] ?? null)));
      } catch (err) {
        console.warn('[subs] could not list ASS styles', err);
      }
    })();
    return () => { cancelled = true; };
  }, [activeSubIdx, subtitleSrcs, jassubReadyTick]);

  // ASS style override application — DISABLED while the feature is W.I.P.
  // The dropdown still detects and lists dialogue styles for visibility,
  // but nothing gets written to JASSUB so the file's authored styling is
  // shown verbatim. Re-enable by removing the `return` below.
  useEffect(() => {
    return; // W.I.P. — see Style tab banner.
    // eslint-disable-next-line @typescript-eslint/no-unreachable-code, no-unreachable
    if (activeSubIdx < 0) return;
    const sub = subtitleSrcs[activeSubIdx];
    if (!sub || sub.format !== 'ass') return;
    const inst = jassubRef.current;
    if (!inst) return;
    const originals = assOriginalsRef.current;
    if (!originals.length) return;
    void (async () => {
      try {
        const r = (inst as unknown as { renderer: { setStyle: (s: Record<string, unknown>, idx: number) => Promise<unknown> } }).renderer;
        const playResY = assPlayResYRef.current || 288;
        for (let i = 0; i < originals.length; i++) {
          const orig = originals[i];
          const name = (orig.Name as string) || '';
          // Only touch dialogue styles; signs / typesetting are off-limits.
          if (!isDialogueStyleName(name)) continue;
          const o = assStyles[name];
          let patch: Record<string, unknown>;
          if (o) {
            const fontSizePx = Math.max(1, Math.round((o.fontSize / 100) * playResY));
            const marginVPx = Math.max(0, Math.round((o.positionBottom / 100) * playResY));
            const fontName = o.fontFamily.split(',')[0].trim().replace(/^['"]|['"]$/g, '');
            // ASS color/background are deliberately NOT touched. The author
            // hand-typeset every dialogue style's colors to fit the show;
            // overriding them just made things look wrong (and the wasm
            // round-trip kept producing weird red outlines anyway). Only
            // structural properties (size, position, font, outline thickness)
            // are exposed to the user for ASS.
            patch = {
              FontSize: fontSizePx,
              FontName: fontName,
              Outline: ASS_OUTLINE_THICKNESS[o.outline],
              MarginV: marginVPx,
            };
          } else {
            // No override → restore EVERY original field so a previous
            // override is fully wiped from wasm.
            patch = { ...orig };
          }
          await r.setStyle(patch, i);
        }
      } catch (err) {
        console.warn('[subs] failed to apply ASS overrides', err);
      }
    })();
  }, [activeSubIdx, subtitleSrcs, assStyles, jassubReadyTick]);

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

  // Resolve intro/outro skip windows. Two sources, in order:
  //   1. Embedded chapter markers in the file (Intro/Outro/Credits/etc.).
  //   2. AniSkip community DB keyed on MAL id.
  // Main handles the source decision; the renderer just calls and caches
  // the resolved values on the episode entry in metadata.json. We short-
  // circuit only when the cached entry was sourced from chapters — older
  // AniSkip-only caches still get a one-time chapter check next time.
  useEffect(() => {
    if (!seriesId || !episodeNumber || duration <= 0) return;
    const seriesData = metadata[seriesId];
    if (!seriesData) return;
    const epNum = parseInt(episodeNumber, 10);
    if (isNaN(epNum)) return;

    const epMeta = seriesData.episodes?.find((e) => e.episodeNumber === epNum);
    if (epMeta?.skipFetched && epMeta.skipSource === 'chapters') {
      setSkipTimes({
        op: epMeta.opStart != null && epMeta.opEnd != null ? { start: epMeta.opStart, end: epMeta.opEnd } : undefined,
        ed: epMeta.edStart != null && epMeta.edEnd != null ? { start: epMeta.edStart, end: epMeta.edEnd } : undefined,
      });
      return;
    }

    // Seed the UI with cached AniSkip values (if any) while we kick off
    // the IPC — chapters might add or replace them, but in the meantime
    // the skip bands don't flash empty.
    if (epMeta?.skipFetched) {
      setSkipTimes({
        op: epMeta.opStart != null && epMeta.opEnd != null ? { start: epMeta.opStart, end: epMeta.opEnd } : undefined,
        ed: epMeta.edStart != null && epMeta.edEnd != null ? { start: epMeta.edStart, end: epMeta.edEnd } : undefined,
      });
    }

    const malId = (seriesData as { malId?: number }).malId;
    // Without a malId AND no filePath, neither source can resolve — bail.
    if (!malId && !activeFilePath) return;

    let cancelled = false;
    void window.electronAPI
      .fetchSkipTimes(seriesId, malId ?? 0, epNum, duration, activeFilePath ?? undefined)
      .then((res) => {
        if (cancelled) return;
        // Only overwrite if main actually returned something — a network
        // failure with no chapters returns {} and we don't want to clear
        // the cached seed we just set.
        if (res.op || res.ed) setSkipTimes(res);
      });
    return () => { cancelled = true; };
  }, [seriesId, episodeNumber, duration, metadata, activeFilePath]);

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

  // ----- Tracker auto-mark -----
  // Resolves to the time at which we count the episode as "watched" — outro
  // start when AniSkip has data, otherwise duration minus 90 s as a coarse
  // "credits started" fallback. null when we have no signal yet.
  const epNumForMark = parseInt(episodeNumber ?? '', 10);
  const series = seriesId ? metadata[seriesId] : undefined;
  const seriesAnilistId = series?.anilistId;
  const seriesMalId = (series as { malId?: number } | undefined)?.malId;
  const seriesTotalEps = series?.totalEpisodes ?? null;

  // Fire the moment the user crosses whichever comes first: AniSkip's outro
  // start, or 85% through. The percentage cutoff matches AniList/MAL's own
  // "watched" heuristic and ensures the toast appears even when AniSkip has
  // no data, or when the outro is unusually short.
  const autoMarkAt = (() => {
    const candidates: number[] = [];
    if (skipTimes.ed?.start != null) candidates.push(skipTimes.ed.start);
    if (duration > 0) candidates.push(duration * 0.85);
    return candidates.length ? Math.min(...candidates) : null;
  })();

  const autoMarkedRef = useRef<string>('');
  const [trackerToast, setTrackerToast] = useState<string | null>(null);
  const trackerToastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  // When the auto-mark fires on the final episode, we surface a persistent
  // "Rate this show" prompt above the auto-dismiss toast. Null = not shown.
  const [ratingPrompt, setRatingPrompt] = useState<null | { seriesKey: string }>(null);
  const [ratingValue, setRatingValue] = useState<string>('8.0');
  const [ratingBusy, setRatingBusy] = useState(false);

  const showTrackerToast = useCallback((msg: string) => {
    setTrackerToast(msg);
    if (trackerToastTimer.current) clearTimeout(trackerToastTimer.current);
    trackerToastTimer.current = setTimeout(() => setTrackerToast(null), 4000);
  }, []);

  const triggerMark = useCallback(async (origin: 'auto' | 'manual') => {
    console.log('[tracker]', origin, 'fire requested', { seriesId, epNumForMark, seriesAnilistId, seriesMalId });
    if (!seriesId || !Number.isFinite(epNumForMark)) {
      console.warn('[tracker] bail: missing seriesId or episode number');
      return;
    }
    if (!seriesAnilistId && !seriesMalId) {
      // Surface the bail in both auto and manual cases — silent failures
      // here were the cause of "I watched a full episode and nothing
      // happened". Most common reason: poster auto-match didn't find a
      // confident hit, so the series has no anilistId/malId. Match it
      // manually from the Metadata tab.
      showTrackerToast('No AniList/MAL id on this series — match it from the Metadata tab.');
      return;
    }
    type MarkRes = Awaited<ReturnType<typeof window.electronAPI.trackerMarkEpisode>>;
    const calls: Promise<MarkRes>[] = [];
    if (seriesAnilistId) {
      calls.push(window.electronAPI.trackerMarkEpisode('anilist', seriesAnilistId, epNumForMark, seriesTotalEps));
    }
    if (seriesMalId) {
      calls.push(window.electronAPI.trackerMarkEpisode('mal', seriesMalId, epNumForMark, seriesTotalEps));
    }
    const results = await Promise.allSettled(calls);
    console.log('[tracker]', origin, 'results', results);
    const summary: string[] = [];
    let anyOk = false;
    let anyNotConnected = false;
    let anyError = false;
    let lastErrorMsg: string | null = null;
    for (const r of results) {
      if (r.status === 'rejected') {
        anyError = true;
        lastErrorMsg = (r.reason as Error)?.message ?? 'unknown error';
        continue;
      }
      const v = r.value;
      if (v.ok) {
        anyOk = true;
        summary.push(`${v.provider.toUpperCase()} → ep ${v.newProgress}`);
      } else if (v.reason === 'no-account') {
        anyNotConnected = true;
      } else if (v.reason === 'not-newer') {
        summary.push(`${v.provider.toUpperCase()} already at ${v.newProgress}`);
      } else if (v.reason === 'error') {
        anyError = true;
        lastErrorMsg = v.message ?? null;
      }
    }
    if (anyOk) {
      showTrackerToast(`Tracked · ${summary.join(' · ')}`);
      // Final-episode rating prompt. Only show on the last episode and only
      // if the user actually had a successful mark this turn (otherwise the
      // tracker is already complete and we shouldn't re-prompt every replay).
      if (
        seriesId
        && Number.isFinite(epNumForMark)
        && seriesTotalEps != null
        && epNumForMark >= seriesTotalEps
      ) {
        setRatingPrompt({ seriesKey: `${seriesId}::${epNumForMark}` });
      }
    } else if (anyError) {
      showTrackerToast(`Tracker error${lastErrorMsg ? ': ' + lastErrorMsg : ''}`);
    } else if (summary.length) {
      // Always show "already at N" — for the auto case this confirms the
      // fire actually happened, just nothing to bump.
      showTrackerToast(summary.join(' · '));
    } else if (anyNotConnected) {
      showTrackerToast('No tracker connected. Link AniList/MAL in Settings.');
    } else if (origin === 'manual') {
      showTrackerToast('Nothing to update.');
    }
  }, [seriesId, epNumForMark, seriesAnilistId, seriesMalId, seriesTotalEps, showTrackerToast]);

  // Reset the auto-mark guard whenever the episode changes.
  useEffect(() => {
    autoMarkedRef.current = '';
    setRatingPrompt(null);
  }, [seriesId, episodeNumber]);

  // Fire once when playback first crosses the auto-mark threshold (the
  // earlier of AniSkip's outro start and 85% of duration). Re-checks on every
  // currentTime update — cheap because a ref short-circuits after the first
  // hit. The console.log helps debug "why didn't it fire" reports.
  useEffect(() => {
    if (!seriesId || !episodeNumber) return;
    const key = `${seriesId}::${episodeNumber}`;
    if (autoMarkedRef.current === key) return;
    if (autoMarkAt == null) return;
    if (currentTime < autoMarkAt) return;
    console.log('[tracker] auto-mark threshold reached', {
      key, currentTime, autoMarkAt, duration, edStart: skipTimes.ed?.start,
    });
    autoMarkedRef.current = key;
    void triggerMark('auto');
  }, [currentTime, autoMarkAt, seriesId, episodeNumber, triggerMark, duration, skipTimes.ed?.start]);

  // Clear any pending toast when the player unmounts.
  useEffect(() => () => {
    if (trackerToastTimer.current) clearTimeout(trackerToastTimer.current);
  }, []);

  const submitRating = useCallback(async () => {
    if (!seriesId) return;
    const score = parseFloat(ratingValue);
    if (!Number.isFinite(score)) return;
    setRatingBusy(true);
    type ScoreRes = Awaited<ReturnType<typeof window.electronAPI.trackerSetScore>>;
    const calls: Promise<ScoreRes>[] = [];
    if (seriesAnilistId) {
      calls.push(window.electronAPI.trackerSetScore('anilist', seriesAnilistId, score, seriesTotalEps));
    }
    if (seriesMalId) {
      calls.push(window.electronAPI.trackerSetScore('mal', seriesMalId, score, seriesTotalEps));
    }
    const results = await Promise.allSettled(calls);
    const successes: string[] = [];
    let anyCompleted = false;
    let lastErr: string | null = null;
    for (const r of results) {
      if (r.status === 'rejected') { lastErr = (r.reason as Error)?.message ?? 'unknown error'; continue; }
      const v = r.value;
      if (v.ok) {
        successes.push(`${v.provider.toUpperCase()} ${score.toFixed(1)}`);
        if (v.completedToo) anyCompleted = true;
      } else if (v.reason === 'error') {
        lastErr = v.message ?? null;
      }
    }
    setRatingBusy(false);
    if (successes.length) {
      showTrackerToast(`Rated · ${successes.join(' · ')}${anyCompleted ? ' · completed' : ''}`);
      setRatingPrompt(null);
    } else {
      showTrackerToast(`Score failed${lastErr ? ': ' + lastErr : ''}`);
    }
  }, [seriesId, ratingValue, seriesAnilistId, seriesMalId, seriesTotalEps, showTrackerToast]);

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

  // Keep the captions-toggle closure fresh (no dep array → re-mirror every
  // render) so the keydown listener can call it without going stale.
  useEffect(() => {
    toggleCaptionsRef.current = () => {
      if (subtitleSrcs.length === 0) return;
      if (activeSubIdx >= 0) {
        // Currently showing a track → remember it and turn captions off.
        lastSubIdxRef.current = activeSubIdx;
        selectSubtitle(-1);
      } else {
        // Off → restore the last track, falling back to the default or first.
        let target = lastSubIdxRef.current;
        if (target == null || target < 0 || target >= subtitleSrcs.length) {
          const def = subtitleSrcs.findIndex((s) => s.default);
          target = def >= 0 ? def : 0;
        }
        selectSubtitle(target);
      }
    };
  });

  // Track ended-state for the center replay affordance. Cleared on play. Also
  // arms the auto-next overlay once the new video has actually loaded — never
  // on the stale frame left over from the previous episode.
  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const onEnded = () => setVideoEnded(true);
    const onPlay = () => setVideoEnded(false);
    const onLoaded = () => { autoNextArmedRef.current = true; };
    video.addEventListener('ended', onEnded);
    video.addEventListener('play', onPlay);
    video.addEventListener('loadeddata', onLoaded);
    return () => {
      video.removeEventListener('ended', onEnded);
      video.removeEventListener('play', onPlay);
      video.removeEventListener('loadeddata', onLoaded);
    };
  }, [videoSrc]);

  // Keyboard shortcuts
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const video = videoRef.current;
      if (!video) return;
      const el = e.target as HTMLElement | null;
      const tag = el?.tagName?.toLowerCase();
      // Let our own range sliders (scrubber, volume) fall through to the
      // shortcuts below — otherwise a focused slider eats Arrow keys as native
      // 0.1-step increments (feels like "seeking does nothing, just stutters").
      // The handled branches all preventDefault(), suppressing that native step.
      const isRange = tag === 'input' && (el as HTMLInputElement).type === 'range';
      if ((tag === 'input' && !isRange) || tag === 'textarea') return;
      // Suppress OS key auto-repeat. Every shortcut here is meant to fire
      // once per press — without this, holding Arrow Right floods <video>
      // with overlapping currentTime assignments; Chromium cancels the
      // in-flight seek for each new one and the picture freezes, then
      // snaps to a compounded target. Feels exactly like "jittery and
      // doesn't actually work" even though the file itself is fine.
      if (e.repeat) return;
      if (e.key === ' ' || e.key === 'k') {
        e.preventDefault();
        if (video.paused) void video.play(); else video.pause();
        showChrome();
      } else if (e.ctrlKey && e.key === 'ArrowRight') {
        // Ctrl+Right = jump to the end of the current intro/outro if we're
        // inside one, otherwise +90s. The intro/outro range comes from the
        // AniSkip skipTimes already in state.
        e.preventDefault();
        const t = video.currentTime;
        let target: number;
        if (skipTimes.op && t >= skipTimes.op.start && t < skipTimes.op.end) {
          target = skipTimes.op.end;
        } else if (skipTimes.ed && t >= skipTimes.ed.start && t < skipTimes.ed.end) {
          target = skipTimes.ed.end;
        } else {
          target = t + 90;
        }
        video.currentTime = Math.min(video.duration || Infinity, target);
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
      } else if (e.key === 'c' && !e.ctrlKey && !e.metaKey) {
        // Toggle captions off / back on to the last-used language. No cycling.
        e.preventDefault();
        toggleCaptionsRef.current();
        showChrome();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [showChrome, skipTimes]);

  const togglePlay = () => {
    const video = videoRef.current;
    if (!video) return;
    if (video.paused) void video.play(); else video.pause();
  };

  // Sorted episode-number list for prev/next nav. Built from the series's
  // fileEpisodes (only on-disk episodes are navigable). Integer numbers only —
  // skip decimals like 6.5 since those aren't really sequential.
  const epNumNumeric = episodeNumber ? parseInt(episodeNumber, 10) : NaN;
  const seriesEpisodeNumbers: number[] = (() => {
    if (!seriesId) return [];
    const series = metadata[seriesId];
    if (!series?.fileEpisodes) return [];
    const nums = series.fileEpisodes
      .map((f) => f.episodeNumber)
      .filter((n): n is number => typeof n === 'number' && Number.isInteger(n));
    return Array.from(new Set(nums)).sort((a, b) => a - b);
  })();
  const currentEpIdx = Number.isFinite(epNumNumeric) ? seriesEpisodeNumbers.indexOf(epNumNumeric) : -1;
  const prevEp = currentEpIdx > 0 ? seriesEpisodeNumbers[currentEpIdx - 1] : null;
  const nextEp = currentEpIdx >= 0 && currentEpIdx < seriesEpisodeNumbers.length - 1
    ? seriesEpisodeNumbers[currentEpIdx + 1]
    : null;

  const goToEpisode = (epNum: number) => {
    if (!seriesId) return;
    // skipResume = true forces a fresh start regardless of saved position.
    navigate(`/player/${seriesId}/${epNum}`, { state: { skipResume: true }, replace: false });
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

  // <video> decode failure → escalate to the unsupported-codec popup. Some
  // codec combos slip past our probe (e.g. exotic audio in an MP4 wrapper)
  // and the only signal is the media element's error event.
  const onVideoError = useCallback(() => {
    const video = videoRef.current;
    const code = video?.error?.code;
    // MEDIA_ERR_SRC_NOT_SUPPORTED (4) and MEDIA_ERR_DECODE (3) are the
    // "Chromium can't play this" cases. Aborts (1) and network errors (2)
    // are noise — let those re-try naturally.
    if (code !== 3 && code !== 4) return;
    if (unsupported) return;
    setUnsupported({ mode: 'decode-failed' });
  }, [unsupported]);

  const handleOpenInMpv = useCallback(async () => {
    if (!activeFilePath || mpvLaunching) return;
    setMpvLaunching(true);
    try {
      await window.electronAPI.openWithMpv(activeFilePath);
    } catch (err) {
      console.warn('openWithMpv failed:', err);
    } finally {
      setMpvLaunching(false);
    }
  }, [activeFilePath, mpvLaunching]);

  // Reset the auto-next overlay whenever the episode changes. Disarm
  // synchronously (ref, not state) so the driver effect below — which runs in
  // this same commit, before the new video has loaded — can't act on the old
  // episode's lingering end-of-stream currentTime/duration. Also clear those
  // to 0 so the scrubber and `remaining` math don't read stale values.
  useEffect(() => {
    autoNextArmedRef.current = false;
    setNextVisible(false);
    setNextCounting(false);
    setNextDismissed(false);
    setVideoEnded(false);
    setCurrentTime(0);
    setDuration(0);
  }, [seriesId, episodeNumber]);

  // Drive the auto-next overlay off the playhead. The pill appears the moment
  // the outro begins (or, with no outro data, 3s before the end). The 5s fill
  // starts either 3s before the end, or right when the outro ends if the
  // post-outro tail is under 20s (so short ED/preview tails get skipped while
  // longer next-episode previews are left to play out).
  useEffect(() => {
    if (nextEp == null || nextDismissed || !duration) return;
    // Bail until the current episode's video has actually loaded — guards
    // against the stale post-navigation frame re-triggering the countdown.
    if (!autoNextArmedRef.current) return;
    const remaining = duration - currentTime;
    const ed = skipTimes.ed;
    const hasOutro = ed != null;
    const tail = hasOutro ? duration - ed.end : Infinity;
    // With no outro data, show the idle pill a few seconds before the fill
    // starts so the CSS transition has a prior scaleX(0) frame to animate from.
    const shouldShow = hasOutro ? currentTime >= ed.start : remaining <= 8;
    if (shouldShow && !nextVisible) setNextVisible(true);
    const shouldCount = (hasOutro && tail < 20 && currentTime >= ed.end) || remaining <= 3;
    if (shouldCount && !nextCounting) setNextCounting(true);
  }, [currentTime, duration, skipTimes.ed, nextEp, nextDismissed, nextVisible, nextCounting]);

  // Once the fill begins, auto-advance after the fixed 5s timer. The CSS fill
  // animation runs for the same 5s; "Stay" clears nextCounting → cancels both.
  useEffect(() => {
    if (!nextCounting || nextEp == null) return;
    const id = window.setTimeout(() => goToEpisode(nextEp), 5000);
    return () => window.clearTimeout(id);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [nextCounting]);

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
  // Prefer the real AniList/MAL episode title (series-level `episodes[]`, keyed
  // by number) over the scanner's filename-derived `episodeData.title`. Mirrors
  // SeriesDetailPage's apiTitleByEp lookup.
  const episodeName = (() => {
    const eps = metadata[seriesId]?.episodes ?? [];
    const hit = eps.find((e) => e.episodeNumber === episodeNum);
    if (hit?.title && hit.title.length > 0) return hit.title;
    return episodeData.title || `Episode ${episodeNum}`;
  })();

  const cueCss = `
    .player-canvas video::cue {
      background-color: rgba(${hexToRgb(vttStyle.bgColor)}, ${vttStyle.bgOpacity});
      color: ${vttStyle.color};
      font-size: ${vttStyle.fontSize}vh;
      font-family: ${vttStyle.fontFamily};
      text-shadow: ${OUTLINE_PRESETS[vttStyle.outline]};
    }
  `;

  return (
    <div
      className={`player-wrap${!chrome && !subMenuOpen && !unsupported ? ' cursor-hidden' : ''}`}
      ref={wrapRef}
      onMouseMove={showChrome}
    >
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
            {episodeName}
          </h2>
          <div className="player-meta">{code}</div>
        </div>
      </div>
      <div className="player-canvas" onClick={togglePlay}>
        <video
          ref={videoRef}
          // Empty videoSrc has to be undefined (not "") so Chromium doesn't
          // try to refetch the host page as a media URL.
          src={videoSrc || undefined}
          autoPlay
          onError={onVideoError}
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
      {trackerToast && (
        <div className="player-toast" role="status">{trackerToast}</div>
      )}
      {ratingPrompt && (
        <div className="player-rating-prompt" role="dialog" aria-modal="false">
          <div className="player-rating-prompt-head">
            <CheckCheck size={14} strokeWidth={2.5} />
            <span>Tracked · final episode — rate this show?</span>
          </div>
          <div className="player-rating-prompt-body">
            <ScorePicker
              value={ratingValue}
              onChange={setRatingValue}
              disabled={ratingBusy}
              ariaLabel="Final-episode rating"
            />
            <button
              type="button"
              className="player-rating-submit"
              onClick={() => void submitRating()}
              disabled={ratingBusy}
            >
              {ratingBusy ? 'Saving…' : 'Submit'}
            </button>
            <button
              type="button"
              className="player-rating-dismiss"
              onClick={() => setRatingPrompt(null)}
              disabled={ratingBusy}
            >
              Skip
            </button>
          </div>
        </div>
      )}
      {unsupported && unsupported.mode === 'transcoding' && (
        <div className="codec-modal-backdrop" role="dialog" aria-modal="true">
          <div className="codec-modal codec-modal--transcoding">
            <div className="codec-modal-head">
              <Loader2 size={20} className="codec-modal-icon codec-modal-icon--spin" />
              <div>
                <div className="codec-modal-title">Re-encoding for in-app playback</div>
                <div className="codec-modal-sub">
                  {unsupported.vCodec
                    ? `Converting ${unsupported.vCodec}${unsupported.aCodec ? ` + ${unsupported.aCodec}` : ''} → h264 + aac. Plays automatically when ready.`
                    : 'Converting to a browser-playable codec. Plays automatically when ready.'}
                </div>
              </div>
            </div>
            <div className="codec-modal-progress">
              <div className="codec-modal-progress-track">
                <div
                  className="codec-modal-progress-fill"
                  style={{ width: `${Math.round((transcodeProgress?.fraction ?? 0) * 100)}%` }}
                />
              </div>
              <div className="codec-modal-progress-meta">
                <span className="codec-modal-progress-pct">
                  {transcodeProgress
                    ? `${Math.round(transcodeProgress.fraction * 100)}%`
                    : 'Starting…'}
                </span>
                <span className="codec-modal-progress-time">
                  {transcodeProgress && transcodeProgress.totalSec > 0
                    ? `${formatTime(transcodeProgress.currentSec)} / ${formatTime(transcodeProgress.totalSec)}`
                    : ''}
                </span>
                <span className="codec-modal-progress-eta">
                  {transcodeProgress?.etaSec != null
                    ? `~${formatTime(transcodeProgress.etaSec)} left${transcodeProgress.speed ? ` · ${transcodeProgress.speed.toFixed(1)}× speed` : ''}`
                    : transcodeProgress?.speed
                      ? `${transcodeProgress.speed.toFixed(1)}× speed`
                      : ''}
                </span>
              </div>
            </div>
            <div className="codec-modal-actions">
              <button
                className="codec-modal-btn ghost"
                onClick={() => navigate(`/series/${seriesId}`)}
              >Back to series</button>
              <Tooltip label="Skip the wait — play the original file in your system mpv">
                <button
                  className="codec-modal-btn"
                  onClick={() => void handleOpenInMpv()}
                  disabled={mpvLaunching || !activeFilePath}
                >
                  <ExternalLink size={14} />
                  {mpvLaunching ? 'Launching mpv…' : 'Play now in mpv'}
                </button>
              </Tooltip>
            </div>
          </div>
        </div>
      )}
      {unsupported && unsupported.mode === 'decode-failed' && (
        <div className="codec-modal-backdrop" role="dialog" aria-modal="true">
          <div className="codec-modal">
            <div className="codec-modal-head">
              <AlertTriangle size={22} className="codec-modal-icon" />
              <div>
                <div className="codec-modal-title">Can&rsquo;t play this in-app</div>
                <div className="codec-modal-sub">
                  Chromium failed to decode this file.
                </div>
              </div>
            </div>
            <div className="codec-modal-body">
              Launch the file in your system&rsquo;s <code>mpv</code> instead. AniBeam stays running — mpv opens in its own window.
            </div>
            <div className="codec-modal-actions">
              <button
                className="codec-modal-btn ghost"
                onClick={() => navigate(`/series/${seriesId}`)}
              >Back to series</button>
              <button
                className="codec-modal-btn primary"
                onClick={() => void handleOpenInMpv()}
                disabled={mpvLaunching || !activeFilePath}
              >
                <ExternalLink size={15} />
                {mpvLaunching ? 'Launching mpv…' : 'Open in system mpv'}
              </button>
            </div>
          </div>
        </div>
      )}
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
      {nextVisible && nextEp != null && (
        <div className="player-autonext">
          <button
            className="player-autonext-stay"
            onClick={(e) => {
              e.stopPropagation();
              setNextDismissed(true);
              setNextVisible(false);
              setNextCounting(false);
            }}
          >
            Stay
          </button>
          <button
            className={`player-autonext-next${nextCounting ? ' counting' : ''}`}
            onClick={(e) => {
              e.stopPropagation();
              goToEpisode(nextEp);
            }}
          >
            <span className="player-autonext-fill" />
            <span className="player-autonext-label">Next</span>
          </button>
        </div>
      )}
      {videoEnded && !nextCounting && (
        <div className="player-replay">
          <button
            className="player-replay-btn"
            onClick={(e) => {
              e.stopPropagation();
              const video = videoRef.current;
              if (!video) return;
              video.currentTime = 0;
              void video.play();
              setVideoEnded(false);
            }}
            aria-label="Replay episode"
          >
            <RotateCcw size={34} />
          </button>
        </div>
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
          style={{
            '--progress': `${duration > 0 ? (currentTime / duration) * 100 : 0}%`,
            // Build the track background dynamically: stacked gradients with
            // intro / outro tints overlaid on the played-vs-unplayed base.
            // First listed gradient renders on top.
            background: (() => {
              if (!duration) return undefined;
              const pct = (n: number) => `${Math.max(0, Math.min(100, (n / duration) * 100))}%`;
              const layers: string[] = [];
              const band = (start: number, end: number, rgba: string) =>
                `linear-gradient(to right, transparent 0, transparent ${pct(start)}, ${rgba} ${pct(start)}, ${rgba} ${pct(end)}, transparent ${pct(end)}, transparent 100%)`;
              if (skipTimes.op) layers.push(band(skipTimes.op.start, skipTimes.op.end, 'rgba(224, 192, 137, 0.7)')); // intro: warm amber
              if (skipTimes.ed) layers.push(band(skipTimes.ed.start, skipTimes.ed.end, 'rgba(96, 144, 208, 0.7)'));  // outro: cool blue
              const progress = Math.max(0, Math.min(100, (currentTime / duration) * 100));
              layers.push(`linear-gradient(to right, #d8d8e0 0%, #d8d8e0 ${progress}%, rgba(255,255,255,0.18) ${progress}%, rgba(255,255,255,0.18) 100%)`);
              return layers.join(', ');
            })(),
          } as React.CSSProperties}
        />
        <div className="player-controls-row">
          <Tooltip label={prevEp != null ? `Previous: episode ${prevEp}` : 'No previous episode'}>
            <button
              className="player-ctl-btn"
              onClick={() => prevEp != null && goToEpisode(prevEp)}
              disabled={prevEp == null}
              aria-label="Previous episode"
            >
              <SkipBack size={18} />
            </button>
          </Tooltip>
          <button className="player-ctl-btn" onClick={togglePlay} aria-label={isPlaying ? 'Pause' : 'Play'}>
            {isPlaying ? <Pause size={18} /> : <Play size={18} />}
          </button>
          <Tooltip label={nextEp != null ? `Next: episode ${nextEp}` : 'No next episode'}>
            <button
              className="player-ctl-btn"
              onClick={() => nextEp != null && goToEpisode(nextEp)}
              disabled={nextEp == null}
              aria-label="Next episode"
            >
              <SkipForward size={18} />
            </button>
          </Tooltip>
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
            {(seriesAnilistId || seriesMalId) && (
              <Tooltip label="Mark watched">
                <button
                  className="player-ctl-btn"
                  onClick={() => void triggerMark('manual')}
                  aria-label="Mark this episode as watched on linked trackers"
                >
                  <CheckCheck size={18} />
                </button>
              </Tooltip>
            )}
            {subtitleSrcs.length > 0 && (
              <div className="player-sub-anchor">
                <Tooltip label={activeSubIdx >= 0 ? subtitleSrcs[activeSubIdx]?.label ?? 'Subtitles' : 'Subtitles off'}>
                  <button
                    ref={ccBtnRef}
                    className={`player-ctl-btn${activeSubIdx >= 0 ? ' active' : ''}`}
                    onClick={() => setSubMenuOpen((v) => !v)}
                    aria-label="Subtitles menu"
                  >
                    <Subtitles size={18} />
                  </button>
                </Tooltip>
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
                    ) : (() => {
                      // Decide which style we're editing right now.
                      const activeFormat = activeSubIdx >= 0 ? subtitleSrcs[activeSubIdx]?.format : 'vtt';
                      const isAss = activeFormat === 'ass';
                      const editing: SubtitleStyle = isAss
                        ? (selectedAssStyle ? assStyles[selectedAssStyle] ?? DEFAULT_SUB_STYLE : DEFAULT_SUB_STYLE)
                        : vttStyle;
                      const update = (patch: Partial<SubtitleStyle>) => {
                        if (isAss) {
                          if (!selectedAssStyle) return;
                          setAssStyles((prev) => ({
                            ...prev,
                            [selectedAssStyle]: { ...(prev[selectedAssStyle] ?? DEFAULT_SUB_STYLE), ...patch },
                          }));
                        } else {
                          setVttStyle((s) => ({ ...s, ...patch }));
                        }
                      };
                      const reset = () => {
                        if (isAss) {
                          if (!selectedAssStyle) return;
                          setAssStyles((prev) => {
                            const next = { ...prev };
                            delete next[selectedAssStyle];
                            return next;
                          });
                        } else {
                          setVttStyle(DEFAULT_SUB_STYLE);
                        }
                      };
                      // ASS styling is disabled for now — libass's layout cache
                      // makes per-style edits behave inconsistently and the
                      // current code path was producing more frustration than
                      // value. Coming back to it later with a different approach.
                      const disabled = isAss || (isAss && !selectedAssStyle);
                      return (
                        <div className="sub-menu-body sub-menu-style">
                          {isAss && (
                            <div className="sub-wip-banner">
                              <span className="sub-wip-tag">W.I.P.</span>
                              <span>Style overrides for embedded ASS subs are temporarily disabled. The dropdown still shows the dialogue styles detected in this file.</span>
                            </div>
                          )}
                          {isAss && (
                            <label className="sub-style-row">
                              <span>Style</span>
                              {assDialogueStyleNames.length > 0 ? (
                                <select
                                  value={selectedAssStyle ?? ''}
                                  onChange={(e) => setSelectedAssStyle(e.target.value || null)}
                                >
                                  {assDialogueStyleNames.map((n) => (
                                    <option key={n} value={n}>{n}</option>
                                  ))}
                                </select>
                              ) : (
                                <span className="sub-style-val" style={{ fontStyle: 'italic', color: '#6a6a76' }}>none detected</span>
                              )}
                            </label>
                          )}
                          <label className="sub-style-row">
                            <span>Size</span>
                            <input
                              type="range" min={1} max={15} step={0.25} disabled={disabled}
                              value={editing.fontSize}
                              onChange={(e) => update({ fontSize: Number(e.target.value) })}
                            />
                            <span className="sub-style-val">{editing.fontSize.toFixed(2)}vh</span>
                          </label>
                          <label className="sub-style-row">
                            <span>Bottom %</span>
                            <input
                              type="range" min={0} max={50} step={1} disabled={disabled}
                              value={editing.positionBottom}
                              onChange={(e) => update({ positionBottom: Number(e.target.value) })}
                            />
                            <span className="sub-style-val">{Math.round(editing.positionBottom)}%</span>
                          </label>
                          {!isAss && (
                            <>
                              <label className="sub-style-row">
                                <span>Text color</span>
                                <input
                                  type="color"
                                  value={editing.color}
                                  onChange={(e) => update({ color: e.target.value })}
                                />
                              </label>
                              <label className="sub-style-row">
                                <span>Background</span>
                                <input
                                  type="color"
                                  value={editing.bgColor}
                                  onChange={(e) => update({ bgColor: e.target.value })}
                                />
                              </label>
                              <label className="sub-style-row">
                                <span>Bg opacity</span>
                                <input
                                  type="range" min={0} max={1} step={0.05}
                                  value={editing.bgOpacity}
                                  onChange={(e) => update({ bgOpacity: Number(e.target.value) })}
                                />
                                <span className="sub-style-val">{Math.round(editing.bgOpacity * 100)}%</span>
                              </label>
                            </>
                          )}
                          <label className="sub-style-row">
                            <span>Font</span>
                            <select
                              value={editing.fontFamily} disabled={disabled}
                              onChange={(e) => update({ fontFamily: e.target.value as SubtitleStyle['fontFamily'] })}
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
                              value={editing.outline} disabled={disabled}
                              onChange={(e) => update({ outline: e.target.value as SubtitleStyle['outline'] })}
                            >
                              <option value="none">None</option>
                              <option value="light">Light</option>
                              <option value="medium">Medium</option>
                              <option value="heavy">Heavy</option>
                            </select>
                          </label>
                          {isAss && (
                            <p style={{ margin: '4px 0 0', color: '#6a6a76', fontSize: 11, fontStyle: 'italic' }}>
                              Colors come from the file&rsquo;s author-typeset styling — left untouched on purpose.
                            </p>
                          )}
                          <button
                            className="sub-style-reset"
                            disabled={disabled}
                            onClick={reset}
                          >{isAss ? 'Clear override for this style' : 'Reset to defaults'}</button>
                        </div>
                      );
                    })()}
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
