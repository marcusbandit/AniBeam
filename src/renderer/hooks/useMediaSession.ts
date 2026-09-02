import { useEffect, useRef, useState } from 'react';

// Publishes what the player is showing through the Media Session API. On
// Linux Chromium bridges the session to MPRIS, so the metadata and action
// handlers set here are what desktop media widgets and playerctl see.
// Position and duration flow from the <video> element on their own, so this
// hook never calls setPositionState.

export interface MediaSessionSpec {
  title: string;
  artist: string;
  artworkUrl: string | null;
  onPlay: () => void;
  onPause: () => void;
  onPrevious: (() => void) | null;
  onNext: (() => void) | null;
  onSeekTo: (seconds: number) => void;
}

const ACTIONS: MediaSessionAction[] = ['play', 'pause', 'previoustrack', 'nexttrack', 'seekto'];

// Chromium throws a TypeError for an action it does not support, so every
// install and teardown goes through this guard.
function setHandler(
  session: MediaSession,
  action: MediaSessionAction,
  handler: MediaSessionActionHandler | null,
): void {
  try {
    session.setActionHandler(action, handler);
  } catch {
    // Unsupported action on this Chromium build; nothing to install.
  }
}

// Chromium fetches media-session artwork through its own image downloader,
// not through the page, and that downloader cannot load the app's media://
// scheme: the metadata goes out with an empty mpris:artUrl even though a
// plain fetch() of the same URL succeeds. https, data: and blob: sources all
// work (checked on D-Bus), so the local poster is fetched here once and
// republished as a blob: URL. The object URL is revoked when the source
// changes or the caller unmounts.
export function useBlobUrl(sourceUrl: string | null): string | null {
  const [blobUrl, setBlobUrl] = useState<string | null>(null);
  useEffect(() => {
    if (!sourceUrl) {
      setBlobUrl(null);
      return;
    }
    let cancelled = false;
    let created: string | null = null;
    fetch(sourceUrl)
      .then((res) => (res.ok ? res.blob() : Promise.reject(new Error(`HTTP ${res.status}`))))
      .then((blob) => {
        if (cancelled) return;
        created = URL.createObjectURL(blob);
        setBlobUrl(created);
      })
      .catch(() => { if (!cancelled) setBlobUrl(null); });
    return () => {
      cancelled = true;
      if (created) URL.revokeObjectURL(created);
      setBlobUrl(null);
    };
  }, [sourceUrl]);
  return blobUrl;
}

export function useMediaSession(spec: MediaSessionSpec | null): void {
  // Callers may build a fresh spec every render. The ref always holds the
  // latest one, so the handlers below read live callbacks while the effects
  // only re-run when something the OS can see actually changed.
  const specRef = useRef(spec);
  specRef.current = spec;

  const title = spec?.title ?? null;
  const artist = spec?.artist ?? null;
  const artworkUrl = spec?.artworkUrl ?? null;
  const active = spec !== null;
  const hasPrevious = spec?.onPrevious != null;
  const hasNext = spec?.onNext != null;

  useEffect(() => {
    const session = navigator.mediaSession as MediaSession | undefined;
    if (!session) return;
    session.metadata = title === null || artist === null
      ? null
      : new MediaMetadata({
        title,
        artist,
        album: '',
        artwork: artworkUrl ? [{ src: artworkUrl }] : [],
      });
  }, [title, artist, artworkUrl]);

  useEffect(() => () => {
    const session = navigator.mediaSession as MediaSession | undefined;
    if (session) session.metadata = null;
  }, []);

  useEffect(() => {
    const session = navigator.mediaSession as MediaSession | undefined;
    if (!session || !active) return;
    setHandler(session, 'play', () => { specRef.current?.onPlay(); });
    setHandler(session, 'pause', () => { specRef.current?.onPause(); });
    setHandler(session, 'previoustrack', hasPrevious
      ? () => { specRef.current?.onPrevious?.(); }
      : null);
    setHandler(session, 'nexttrack', hasNext
      ? () => { specRef.current?.onNext?.(); }
      : null);
    setHandler(session, 'seekto', (details: MediaSessionActionDetails) => {
      const seekTime = details.seekTime;
      if (typeof seekTime === 'number' && Number.isFinite(seekTime)) {
        specRef.current?.onSeekTo(seekTime);
      }
    });
    return () => {
      for (const action of ACTIONS) setHandler(session, action, null);
    };
  }, [active, hasPrevious, hasNext]);
}
