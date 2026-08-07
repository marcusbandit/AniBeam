import { useEffect } from 'react';
import { Play, X, RotateCw } from 'lucide-react';

/**
 * Shown when an episode the user has ruled out of re-encoding is opened.
 *
 * The in-window player needs an h.264 copy; mpv doesn't. Rather than silently
 * starting the encode the user said they never wanted — or worse, dropping
 * them on a dead player — this puts the choice up front and leads with the one
 * that just works. It reads as a play prompt, not an error: the failure here is
 * a codec detail, and the user's actual intent was "watch this episode".
 */
interface Props {
  open: boolean;
  /** e.g. "Episode 3" — the line the user identifies the episode by. */
  code: string;
  title: string;
  /** Source video codec, named so the reason is concrete rather than vague. */
  vCodec: string | null;
  busy?: boolean;
  onPlayInMpv: () => void;
  onReencode: () => void;
  onClose: () => void;
}

function ExternalPlayPrompt({
  open, code, title, vCodec, busy = false, onPlayInMpv, onReencode, onClose,
}: Props) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !busy) onClose();
      // Enter takes the primary action, so the whole thing is one keypress
      // away from playing.
      if (e.key === 'Enter' && !busy) onPlayInMpv();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, busy, onClose, onPlayInMpv]);

  if (!open) return null;

  return (
    <div
      className="play-prompt-backdrop"
      onMouseDown={(e) => { if (e.target === e.currentTarget && !busy) onClose(); }}
    >
      <div
        className="play-prompt"
        data-liquid-glass=""
        data-lg-bezel="16"
        role="dialog"
        aria-modal="true"
        aria-labelledby="play-prompt-title"
      >
        <button className="play-prompt__close icon-btn" onClick={onClose} disabled={busy} aria-label="Close">
          <X size={16} />
        </button>

        <div className="play-prompt__mark" aria-hidden="true">
          <Play size={26} strokeWidth={2.5} />
        </div>

        <div className="play-prompt__code">{code}</div>
        <div id="play-prompt-title" className="play-prompt__title">{title}</div>

        <p className="play-prompt__body">
          You&apos;ve turned re-encoding off for this episode
          {vCodec ? <>, and its {vCodec.toUpperCase()} video won&apos;t decode in the app window</> : null}.
          mpv plays the original file as it is.
        </p>

        <button className="play-prompt__primary" onClick={onPlayInMpv} disabled={busy} autoFocus>
          <Play size={18} strokeWidth={2.5} />
          <span>Play in mpv</span>
        </button>

        <button className="play-prompt__secondary" onClick={onReencode} disabled={busy}>
          <RotateCw size={14} />
          <span>{busy ? 'Starting re-encode…' : 'Re-encode and play here'}</span>
        </button>
      </div>
    </div>
  );
}

export default ExternalPlayPrompt;
