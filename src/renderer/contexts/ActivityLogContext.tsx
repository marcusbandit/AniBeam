import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import type { LogEvent, LogStage, LogLevel } from '../../shared/logTypes';

interface ActivityLogContextValue {
  events: LogEvent[];
  stageFilter: Set<LogStage>;
  levelFilter: Set<LogLevel>;
  toggleStage: (stage: LogStage) => void;
  toggleLevel: (level: LogLevel) => void;
  clear: () => void;
  visibleEvents: LogEvent[];
  unseenErrorCount: number;
  markErrorsSeen: () => void;
}

const ALL_STAGES: LogStage[] = ['folder', 'metadata', 'image', 'thumbnail', 'watch', 'probe', 'system'];
const ALL_LEVELS: LogLevel[] = ['info', 'warn', 'error'];

const ActivityLogContext = createContext<ActivityLogContextValue | null>(null);

export function ActivityLogProvider({ children }: { children: ReactNode }) {
  const [events, setEvents] = useState<LogEvent[]>([]);
  const [stageFilter, setStageFilter] = useState<Set<LogStage>>(() => new Set(ALL_STAGES));
  const [levelFilter, setLevelFilter] = useState<Set<LogLevel>>(() => new Set(ALL_LEVELS));
  const [lastSeenErrorId, setLastSeenErrorId] = useState(0);

  useEffect(() => {
    let cancelled = false;
    window.electronAPI.getLogBuffer().then((buf) => {
      if (!cancelled) setEvents(buf);
    });
    const unsubscribe = window.electronAPI.onLogEvent((event) => {
      setEvents((prev) => {
        const next = prev.length >= 5000 ? prev.slice(prev.length - 4999) : prev.slice();
        next.push(event);
        return next;
      });
    });
    return () => {
      cancelled = true;
      unsubscribe();
    };
  }, []);

  const toggleStage = (stage: LogStage) => {
    setStageFilter((prev) => {
      const next = new Set(prev);
      if (next.has(stage)) next.delete(stage);
      else next.add(stage);
      return next;
    });
  };

  const toggleLevel = (level: LogLevel) => {
    setLevelFilter((prev) => {
      const next = new Set(prev);
      if (next.has(level)) next.delete(level);
      else next.add(level);
      return next;
    });
  };

  const clear = async () => {
    await window.electronAPI.clearLog();
    setEvents([]);
    setLastSeenErrorId(0);
  };

  const visibleEvents = useMemo(
    () => events.filter((e) => stageFilter.has(e.stage) && levelFilter.has(e.level)),
    [events, stageFilter, levelFilter],
  );

  const unseenErrorCount = useMemo(
    () => events.reduce((n, e) => (e.level === 'error' && e.id > lastSeenErrorId ? n + 1 : n), 0),
    [events, lastSeenErrorId],
  );

  const markErrorsSeen = () => {
    const lastId = events.length > 0 ? events[events.length - 1].id : 0;
    setLastSeenErrorId(lastId);
  };

  return (
    <ActivityLogContext.Provider
      value={{
        events,
        stageFilter,
        levelFilter,
        toggleStage,
        toggleLevel,
        clear,
        visibleEvents,
        unseenErrorCount,
        markErrorsSeen,
      }}
    >
      {children}
    </ActivityLogContext.Provider>
  );
}

export function useActivityLog(): ActivityLogContextValue {
  const ctx = useContext(ActivityLogContext);
  if (!ctx) throw new Error('useActivityLog must be used inside ActivityLogProvider');
  return ctx;
}

export { ALL_STAGES, ALL_LEVELS };
