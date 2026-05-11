import { useCallback, useEffect, useRef } from 'react';

// Trailing-edge debounce. Repeated calls inside `delayMs` reset the timer;
// the wrapped function fires once with the *last* arguments after the
// burst settles. The fn ref is updated on every render so the latest
// closure (capturing fresh state) runs when the timer fires — without
// this, a stale callback could be invoked.
export function useDebouncedCallback<Args extends unknown[]>(
  fn: (...args: Args) => void,
  delayMs: number,
): (...args: Args) => void {
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const fnRef = useRef(fn);
  useEffect(() => { fnRef.current = fn; }, [fn]);
  useEffect(() => () => {
    if (timer.current) clearTimeout(timer.current);
  }, []);
  return useCallback((...args: Args) => {
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => fnRef.current(...args), delayMs);
  }, [delayMs]);
}
