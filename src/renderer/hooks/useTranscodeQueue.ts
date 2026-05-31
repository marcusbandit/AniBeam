import { useSyncExternalStore } from 'react';

export type TranscodeQueueStatus = 'encoding' | 'queued';

// The new queue-snapshot IPC is modeled on the existing onTranscodeProgress /
// ensureSeriesTranscoded surface and is added to the preload type by the
// backend. Narrow through a local typed accessor (rather than redeclaring the
// global) so this compiles whether or not the methods have landed on the
// canonical ElectronAPI type yet — and so a missing impl degrades gracefully.
type QueueSnapshot = Record<string, TranscodeQueueStatus>;
const api = window.electronAPI as typeof window.electronAPI & {
  getTranscodeQueueSnapshot?: () => Promise<QueueSnapshot>;
  onTranscodeQueueChanged?: (cb: (snap: QueueSnapshot) => void) => () => void;
};

// One IPC subscription for the whole app — never one per card. The store is a
// tiny external store wired into React via useSyncExternalStore. `snapshot`
// holds a STABLE object reference between updates: it's only swapped when new
// data arrives from main, so getSnapshot() stays Object.is-equal across
// unrelated re-renders and useSyncExternalStore never tears or loops.
let snapshot: QueueSnapshot = {};
const listeners = new Set<() => void>();
let started = false;
let unsubscribe: (() => void) | null = null;

function emit(): void {
  for (const l of listeners) l();
}

// Lazily wire up IPC on the first subscriber. Kept alive for the app lifetime
// once started (it's a single subscription) — simpler and robust vs. ref-count
// teardown, with no leak of note.
function ensureStarted(): void {
  if (started) return;
  started = true;
  void api.getTranscodeQueueSnapshot?.().then((snap) => {
    snapshot = snap ?? {};
    emit();
  }).catch(() => { /* best-effort — the live channel still fills it in */ });
  unsubscribe = api.onTranscodeQueueChanged?.((snap) => {
    snapshot = snap ?? {};
    emit();
  }) ?? null;
}

function subscribe(listener: () => void): () => void {
  ensureStarted();
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}

// Returns the current snapshot. Stable reference between updates (see above).
function getSnapshot(): QueueSnapshot {
  return snapshot;
}

// Touch `unsubscribe` so the lint/ts "assigned but unused" check is satisfied
// while keeping the handle around for the app lifetime.
void unsubscribe;

/**
 * Per-series re-encode queue status, sourced from a single shared IPC
 * subscription. Returns 'encoding' while any of the series' episodes is being
 * transcoded, 'queued' while one is waiting, and null otherwise.
 *
 * The selector reads a PRIMITIVE (string | null) so it's referentially stable
 * under Object.is — re-renders only fire when THIS series' status actually
 * changes, not on every unrelated queue mutation.
 */
export function useSeriesTranscodeStatus(seriesId: string): TranscodeQueueStatus | null {
  return useSyncExternalStore(
    subscribe,
    () => getSnapshot()[seriesId] ?? null,
    () => null,
  );
}
