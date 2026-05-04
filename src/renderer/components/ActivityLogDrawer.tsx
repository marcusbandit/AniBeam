import { useEffect, useRef, useState } from 'react';
import { Activity, X, Trash2, Copy } from 'lucide-react';
import { useActivityLog, ALL_STAGES, ALL_LEVELS } from '../contexts/ActivityLogContext';

function formatTime(ts: number): string {
  const d = new Date(ts);
  const hh = String(d.getHours()).padStart(2, '0');
  const mm = String(d.getMinutes()).padStart(2, '0');
  const ss = String(d.getSeconds()).padStart(2, '0');
  return `${hh}:${mm}:${ss}`;
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
        {unseenErrorCount > 0 && <span className="activity-log-badge">{unseenErrorCount}</span>}
      </button>
      {open && (
        <aside className="activity-log-drawer">
          <header className="activity-log-header">
            <span className="activity-log-title">Activity</span>
            <div className="activity-log-actions">
              <button className="activity-log-action" onClick={handleCopy} aria-label="Copy log">
                <Copy size={14} />
              </button>
              <button className="activity-log-action" onClick={clear} aria-label="Clear log">
                <Trash2 size={14} />
              </button>
              <button className="activity-log-action" onClick={() => setOpen(false)} aria-label="Close">
                <X size={14} />
              </button>
            </div>
          </header>
          <div className="activity-log-filters">
            {ALL_STAGES.map((stage) => (
              <button
                key={stage}
                className={`activity-log-chip${stageFilter.has(stage) ? ' active' : ''}`}
                onClick={() => toggleStage(stage)}
              >
                {stage}
              </button>
            ))}
            <span className="activity-log-sep" />
            {ALL_LEVELS.map((level) => (
              <button
                key={level}
                className={`activity-log-chip level-${level}${levelFilter.has(level) ? ' active' : ''}`}
                onClick={() => toggleLevel(level)}
              >
                {level}
              </button>
            ))}
          </div>
          <div className="activity-log-list" ref={listRef} onScroll={handleScroll}>
            {visibleEvents.length === 0 && <div className="activity-log-empty">No events.</div>}
            {visibleEvents.map((e) => (
              <div key={e.id} className={`activity-log-row level-${e.level}`}>
                <span className="activity-log-ts">{formatTime(e.ts)}</span>
                <span className="activity-log-stage">{e.stage}</span>
                <span className="activity-log-msg">{e.message}</span>
                {e.ctx?.series && <span className="activity-log-ctx">{e.ctx.series}</span>}
                {e.ctx?.file && !e.ctx.series && <span className="activity-log-ctx">{e.ctx.file}</span>}
              </div>
            ))}
          </div>
        </aside>
      )}
    </>
  );
}
