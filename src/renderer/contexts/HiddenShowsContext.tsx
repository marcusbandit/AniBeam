import { createContext, useContext, useState, type ReactNode } from 'react';

interface HiddenShowsValue {
  showHidden: boolean;
  setShowHidden: (v: boolean) => void;
}

const HiddenShowsContext = createContext<HiddenShowsValue | undefined>(undefined);

// Session-only by design: `showHidden` is plain React state seeded to `false`,
// so it ALWAYS boots OFF and resets on every app launch. Deliberately NOT
// persisted to localStorage or config.json — the user re-enables it manually.
export function HiddenShowsProvider({ children }: { children: ReactNode }) {
  const [showHidden, setShowHidden] = useState(false);
  return (
    <HiddenShowsContext.Provider value={{ showHidden, setShowHidden }}>
      {children}
    </HiddenShowsContext.Provider>
  );
}

export function useHiddenShows(): HiddenShowsValue {
  const ctx = useContext(HiddenShowsContext);
  if (!ctx) throw new Error('useHiddenShows must be used within HiddenShowsProvider');
  return ctx;
}
