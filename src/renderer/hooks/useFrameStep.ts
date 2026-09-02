import { useEffect, useMemo, useRef, type RefObject } from 'react';
import { FrameStepper, type StepDirection } from '../../shared/frameStep';

// Frame stepping for the <video> element: `step(1)` shows the next frame,
// `step(-1)` the previous one, pausing first if the video was playing (every
// player pauses on a frame step). The stepping logic itself lives in
// shared/frameStep.ts; this hook only feeds it the element's seek and paint
// events and hands the landed frame back for the HUD.
//
// `mountKey` is whatever changes when the <video> element (re)mounts, the
// player's videoSrc. The element is absent on the loading shell, so an effect
// keyed on the ref alone would attach to nothing and never try again.

export function useFrameStep(
  videoRef: RefObject<HTMLVideoElement | null>,
  onLanded: (pts: number, frameDuration: number) => void,
  mountKey: unknown,
): { step: (direction: StepDirection) => void } {
  const onLandedRef = useRef(onLanded);
  useEffect(() => { onLandedRef.current = onLanded; });

  const stepper = useMemo(() => new FrameStepper({
    seek: (t) => {
      const video = videoRef.current;
      if (video) video.currentTime = t;
    },
    getCurrentTime: () => videoRef.current?.currentTime ?? 0,
    getDuration: () => videoRef.current?.duration ?? NaN,
    isSeeking: () => videoRef.current?.seeking ?? false,
    schedule: (fn, ms) => {
      const id = setTimeout(fn, ms);
      return () => clearTimeout(id);
    },
    onLanded: (pts, d) => onLandedRef.current(pts, d),
  }), [videoRef]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const onSeeking = () => stepper.onSeeking();
    const onReset = () => stepper.reset();
    const onPlay = () => stepper.cancelPending();
    video.addEventListener('seeking', onSeeking);
    video.addEventListener('emptied', onReset);
    video.addEventListener('loadstart', onReset);
    video.addEventListener('play', onPlay);

    // Persistent frame callback loop: each callback fires once, so it
    // re-registers itself. Costs nothing while paused (no frames are
    // presented) and a couple of arithmetic ops per frame while playing.
    let handle: number | null = null;
    const hasFrameCallback = typeof video.requestVideoFrameCallback === 'function';
    if (hasFrameCallback) {
      const onFrame: VideoFrameRequestCallback = (_now, metadata) => {
        stepper.onFramePresented(metadata.mediaTime);
        handle = video.requestVideoFrameCallback(onFrame);
      };
      handle = video.requestVideoFrameCallback(onFrame);
    }

    return () => {
      video.removeEventListener('seeking', onSeeking);
      video.removeEventListener('emptied', onReset);
      video.removeEventListener('loadstart', onReset);
      video.removeEventListener('play', onPlay);
      if (hasFrameCallback && handle != null) video.cancelVideoFrameCallback(handle);
      stepper.reset();
    };
  }, [videoRef, stepper, mountKey]);

  const step = useMemo(() => (direction: StepDirection) => {
    const video = videoRef.current;
    if (!video || video.readyState < 1) return;
    if (!video.paused) {
      video.pause();
      // The frame on screen may not have reported itself yet: its frame
      // callback runs in the next rendering step, ahead of rAF callbacks.
      // Requesting straight away anchors on the previous frame about a
      // quarter of the time (measured), which shows as a dead first press
      // forward or a two-frame jump back. One rendering step of latency is
      // invisible; a wrong first step is not.
      requestAnimationFrame(() => stepper.request(direction));
      return;
    }
    stepper.request(direction);
  }, [videoRef, stepper]);

  return { step };
}
