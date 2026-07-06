import { useEffect, useMemo, useRef, useState } from 'react';
import { Activity, X, Trash2, Copy, ChevronRight, ChevronDown } from 'lucide-react';
import { useActivityLog, ALL_STAGES, ALL_LEVELS } from '../contexts/ActivityLogContext';
import type { LogEvent, LogLevel, LogStage } from '../../shared/logTypes';
import { Tooltip } from './primitives';

function formatTime(ts: number): string {
  const d = new Date(ts);
  const hh = String(d.getHours()).padStart(2, '0');
  const mm = String(d.getMinutes()).padStart(2, '0');
  const ss = String(d.getSeconds()).padStart(2, '0');
  return `${hh}:${mm}:${ss}`;
}

function splitMessage(message: string): { header: string; detail: string | null } {
  const idx = message.indexOf(':');
  if (idx <= 0) return { header: message, detail: null };
  return { header: message.slice(0, idx).trim(), detail: message.slice(idx + 1).trim() };
}

type GroupedRow =
  | { kind: 'single'; event: LogEvent }
  | {
      kind: 'group';
      id: number;
      header: string;
      stage: LogStage;
      level: LogLevel;
      firstTs: number;
      events: LogEvent[];
    };

function groupEvents(events: LogEvent[]): GroupedRow[] {
  const out: GroupedRow[] = [];
  let i = 0;
  while (i < events.length) {
    const start = events[i];
    const { header } = splitMessage(start.message);
    const key = `${start.stage}|${start.level}|${header}`;
    let j = i + 1;
    while (j < events.length) {
      const e = events[j];
      const k = `${e.stage}|${e.level}|${splitMessage(e.message).header}`;
      if (k !== key) break;
      j++;
    }
    const run = events.slice(i, j);
    if (run.length >= 2) {
      out.push({
        kind: 'group',
        id: start.id,
        header,
        stage: start.stage,
        level: start.level,
        firstTs: start.ts,
        events: run,
      });
    } else {
      out.push({ kind: 'single', event: start });
    }
    i = j;
  }
  return out;
}

export function ActivityLogDrawer() {
  const {
    visibleEvents,
    stageFilter,
    levelFilter,
    toggleStage,
    toggleLevel,
    clear,
    unseenErrorCount,
    markErrorsSeen,
  } = useActivityLog();
  const [open, setOpen] = useState(false);
  const listRef = useRef<HTMLDivElement>(null);
  const [stickToBottom, setStickToBottom] = useState(true);
  const [expandedIds, setExpandedIds] = useState<Set<number>>(() => new Set());
  const [expandedGroups, setExpandedGroups] = useState<Set<number>>(() => new Set());

  const toggleExpanded = (id: number) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const toggleGroup = (id: number) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const groupedRows = useMemo(() => groupEvents(visibleEvents), [visibleEvents]);

  useEffect(() => {
    if (open && stickToBottom && listRef.current) {
      listRef.current.scrollTop = listRef.current.scrollHeight;
    }
  }, [visibleEvents, open, stickToBottom]);

  useEffect(() => {
    if (open) markErrorsSeen();
  }, [open, markErrorsSeen]);

  const handleScroll = () => {
    const el = listRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 8;
    setStickToBottom(atBottom);
  };

  const handleCopy = async () => {
    const text = visibleEvents
      .map((e) => `${formatTime(e.ts)} [${e.level}] [${e.stage}] ${e.ctx?.series ? `(${e.ctx.series}) ` : ''}${e.message}`)
      .join('\n');
    await navigator.clipboard.writeText(text);
  };

  return (
    <>
      <button
        className={`activity-log-toggle${unseenErrorCount > 0 ? ' has-errors' : ''}`}
        onClick={() => setOpen((v) => !v)}
        aria-label="Toggle activity log"
      >
        <Activity size={16} />
        <span>Activity</span>
        {unseenErrorCount > 0 && <span className="chip chip--sm chip--rose">{unseenErrorCount}</span>}
      </button>
      {open && (
        <aside className="activity-log-drawer">
          <header className="activity-log-header">
            <span className="activity-log-title">Activity</span>
            <div className="activity-log-actions">
              <Tooltip label="Copy log">
                <button className="icon-btn" onClick={handleCopy} aria-label="Copy log">
                  <Copy size={14} />
                </button>
              </Tooltip>
              <Tooltip label="Clear log">
                <button className="icon-btn" onClick={clear} aria-label="Clear log">
                  <Trash2 size={14} />
                </button>
              </Tooltip>
              <Tooltip label="Close">
                <button className="icon-btn" onClick={() => setOpen(false)} aria-label="Close">
                  <X size={14} />
                </button>
              </Tooltip>
            </div>
          </header>
          <div className="activity-log-filters">
            {ALL_STAGES.map((stage) => (
              <button
                key={stage}
                className={`chip chip--sm chip--toggle stage-${stage}${stageFilter.has(stage) ? ' is-on' : ''}`}
                onClick={() => toggleStage(stage)}
                aria-pressed={stageFilter.has(stage)}
              >
                {stage}
              </button>
            ))}
            <span className="activity-log-sep" />
            {ALL_LEVELS.map((level) => (
              <button
                key={level}
                className={`chip chip--sm chip--toggle level-${level}${levelFilter.has(level) ? ' is-on' : ''}`}
                onClick={() => toggleLevel(level)}
                aria-pressed={levelFilter.has(level)}
              >
                {level}
              </button>
            ))}
          </div>
          <div className="activity-log-list" ref={listRef} onScroll={handleScroll}>
            {groupedRows.length === 0 && <div className="activity-log-empty">No events.</div>}
            {groupedRows.map((row) => {
              if (row.kind === 'single') {
                const e = row.event;
                const ctxText = e.ctx?.series ?? e.ctx?.file;
                const fullLine = `${e.message}${ctxText ? ` · ${ctxText}` : ''}`;
                const expanded = expandedIds.has(e.id);
                const rowEl = (
                  <div
                    className={`activity-log-row level-${e.level}${expanded ? ' expanded' : ''}`}
                    onClick={() => toggleExpanded(e.id)}
                    role="button"
                  >
                    <span className="activity-log-ts">{formatTime(e.ts)}</span>
                    <span className={`activity-log-stage stage-${e.stage}`}>{e.stage}</span>
                    <span className="activity-log-msg">{e.message}</span>
                    {ctxText && <span className="activity-log-ctx">{ctxText}</span>}
                  </div>
                );
                return expanded
                  ? <div key={`s-${e.id}`}>{rowEl}</div>
                  : <Tooltip key={`s-${e.id}`} label={fullLine}>{rowEl}</Tooltip>;
              }

              const expanded = expandedGroups.has(row.id);
              return (
                <div
                  key={`g-${row.id}`}
                  className={`activity-log-group level-${row.level}${expanded ? ' expanded' : ''}`}
                >
                  <div
                    className={`activity-log-row activity-log-group-head level-${row.level}`}
                    onClick={() => toggleGroup(row.id)}
                    role="button"
                    aria-expanded={expanded}
                  >
                    <span className="activity-log-ts">{formatTime(row.firstTs)}</span>
                    <span className={`activity-log-stage stage-${row.stage}`}>{row.stage}</span>
                    <span className="activity-log-msg activity-log-group-msg">
                      <span className="activity-log-chev">
                        {expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                      </span>
                      {row.header}
                    </span>
                    <span className="chip chip--sm activity-log-count">{row.events.length}</span>
                  </div>
                  {expanded && (
                    <div className="activity-log-group-children">
                      {row.events.map((e) => {
                        const { detail } = splitMessage(e.message);
                        const ctxText = e.ctx?.series ?? e.ctx?.file;
                        const text = detail ?? e.message;
                        const fullLine = `${text}${ctxText ? ` · ${ctxText}` : ''}`;
                        return (
                          <Tooltip key={e.id} label={fullLine}>
                            <div className={`activity-log-child-row level-${e.level}`}>
                              <span className="activity-log-ts">{formatTime(e.ts)}</span>
                              <span className="activity-log-msg">{text}</span>
                              {ctxText && <span className="activity-log-ctx">{ctxText}</span>}
                            </div>
                          </Tooltip>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </aside>
      )}
    </>
  );
}
