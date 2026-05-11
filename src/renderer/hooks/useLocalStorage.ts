import { useCallback, useState } from 'react';

// Tiny wrapper around localStorage with JSON serialization. Defaults are
// merged shallowly into the stored value so adding a new field to the
// shape doesn't break loading older saved blobs. Writes are best-effort
// — quota errors don't propagate to the caller.
//
// Returns a tuple matching React's useState signature.
export function useLocalStorage<T extends object>(
  key: string,
  defaults: T,
): [T, (next: T | ((prev: T) => T)) => void] {
  const [value, setValue] = useState<T>(() => {
    try {
      const raw = localStorage.getItem(key);
      if (!raw) return defaults;
      return { ...defaults, ...(JSON.parse(raw) as Partial<T>) };
    } catch {
      return defaults;
    }
  });

  const update = useCallback(
    (next: T | ((prev: T) => T)) => {
      setValue((prev) => {
        const resolved = typeof next === 'function' ? (next as (p: T) => T)(prev) : next;
        try {
          localStorage.setItem(key, JSON.stringify(resolved));
        } catch {
          // Quota / private mode — best-effort, in-memory state still updates.
        }
        return resolved;
      });
    },
    [key],
  );

  return [value, update];
}

// Variant for shapes where the stored value is an arbitrary record (no
// shallow-merge of defaults — the entire object is replaced on read).
// Used by the per-style ASS settings in the player.
export function useLocalStorageRecord<V>(
  key: string,
  defaults: Record<string, V>,
): [Record<string, V>, (next: Record<string, V> | ((prev: Record<string, V>) => Record<string, V>)) => void] {
  const [value, setValue] = useState<Record<string, V>>(() => {
    try {
      const raw = localStorage.getItem(key);
      if (!raw) return defaults;
      return JSON.parse(raw) as Record<string, V>;
    } catch {
      return defaults;
    }
  });

  const update = useCallback(
    (next: Record<string, V> | ((prev: Record<string, V>) => Record<string, V>)) => {
      setValue((prev) => {
        const resolved = typeof next === 'function' ? (next as (p: Record<string, V>) => Record<string, V>)(prev) : next;
        try {
          localStorage.setItem(key, JSON.stringify(resolved));
        } catch { /* best-effort */ }
        return resolved;
      });
    },
    [key],
  );

  return [value, update];
}
