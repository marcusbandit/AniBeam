import { useCallback, useEffect, useState } from "react";
import { Rss, RefreshCw, ExternalLink, FolderOpen } from "lucide-react";
import type { SubscriptionFeed, SubscriptionsResult } from "../../main/preload";
import { Page, Inline, Pill, Tooltip } from "../components/primitives";

function decodeNyaaQuery(feedUrl: string): string | null {
  if (!feedUrl) return null;
  try {
    const u = new URL(feedUrl);
    const q = u.searchParams.get("q");
    return q ? q.replace(/\+/g, " ") : null;
  } catch {
    return null;
  }
}

function SubscriptionsPage() {
  const [result, setResult] = useState<SubscriptionsResult | null>(null);
  const [loading, setLoading] = useState(true);

  const reload = useCallback(async () => {
    setLoading(true);
    try {
      const r = await window.electronAPI.listSubscriptions();
      setResult(r);
    } catch (err) {
      setResult({ ok: false, error: (err as Error).message });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  return (
    <Page
      head={
        <Inline gap="s4" justify="space-between" align="flex-start">
          <div>
            <h1 className="page-title">Subscriptions</h1>
            <p className="page-sub">RSS feeds anirss is watching for you.</p>
          </div>
          <button
            type="button"
            className="btn-secondary subscriptions-refresh"
            onClick={() => void reload()}
            disabled={loading}
            aria-label="Refresh subscriptions"
          >
            <RefreshCw size={14} className={loading ? "spin" : undefined} />
            <span>Refresh</span>
          </button>
        </Inline>
      }
    >
      {loading && !result ? (
        <div className="loading">Reading anirss…</div>
      ) : result?.ok ? (
        result.items.length === 0 ? (
          <EmptyState title="No subscriptions yet" hint="Run anirss in a terminal to subscribe to a feed." />
        ) : (
          <div className="subscriptions-list">
            {result.items.map((item) => (
              <SubscriptionRow key={item.name} item={item} />
            ))}
          </div>
        )
      ) : result ? (
        <ErrorState message={result.error} needsAuth={result.needsAuth ?? false} />
      ) : null}
    </Page>
  );
}

function SubscriptionRow({ item }: { item: SubscriptionFeed }) {
  const query = decodeNyaaQuery(item.feedUrl);
  return (
    <div className={`subscription-row${item.ruleEnabled ? "" : " disabled"}`}>
      <div className="subscription-head">
        <div className="subscription-title">
          <Rss size={14} className="subscription-icon" />
          <Tooltip label={item.name}>
            <span className="subscription-name">{item.name}</span>
          </Tooltip>
        </div>
        <div className="subscription-meta">
          <Pill tone={item.ruleEnabled ? "teal" : "muted"}>
            {item.ruleEnabled ? "active" : "paused"}
          </Pill>
          <Tooltip label="torrents in qBittorrent">
            <span className="subscription-count">
              {item.torrentCount} {item.torrentCount === 1 ? "torrent" : "torrents"}
            </span>
          </Tooltip>
        </div>
      </div>
      {query && (
        <Tooltip label={item.feedUrl}>
          <div className="subscription-query">
            <span className="subscription-label">query</span>
            <code>{query}</code>
          </div>
        </Tooltip>
      )}
      {item.savePath && (
        <Tooltip label={item.savePath}>
          <div className="subscription-path">
            <FolderOpen size={12} />
            <span>{item.savePath}</span>
          </div>
        </Tooltip>
      )}
      {item.feedUrl && (
        <button
          type="button"
          className="subscription-link"
          onClick={() => void window.electronAPI.openExternal(item.feedUrl)}
        >
          <ExternalLink size={12} />
          <span>open feed</span>
        </button>
      )}
    </div>
  );
}

function EmptyState({ title, hint }: { title: string; hint: string }) {
  return (
    <div className="empty">
      <div className="empty-icon"><Rss size={48} /></div>
      <div className="empty-title">{title}</div>
      <div className="empty-text">{hint}</div>
    </div>
  );
}

function ErrorState({ message, needsAuth }: { message: string; needsAuth: boolean }) {
  return (
    <div className="empty subscriptions-error">
      <div className="empty-icon"><Rss size={48} /></div>
      <div className="empty-title">{needsAuth ? "qBittorrent session needed" : "Couldn't read subscriptions"}</div>
      <div className="empty-text">
        {needsAuth ? (
          <>
            Run <code>anirss -Sy</code> in a terminal once to log in to qBittorrent. AniBeam will pick up the cached session afterwards.
          </>
        ) : (
          message
        )}
      </div>
    </div>
  );
}

export default SubscriptionsPage;
