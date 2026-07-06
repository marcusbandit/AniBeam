import { useState, useEffect, useCallback } from 'react';
import { Link2, Link2Off, ExternalLink, Copy, Check } from 'lucide-react';
import type { TrackerProvider, TrackerStatus } from '../../main/preload';
import { LOOPBACK_REDIRECT_URI, DEFAULT_CLIENT_IDS, DEFAULT_CLIENT_SECRETS } from '../../shared/trackerConstants';
import { useTrackerProgress } from '../contexts/TrackerProgressContext';
import { Section, Tooltip, SegmentedSwitch } from './primitives';

interface TrackerRowProps {
  provider: TrackerProvider;
  label: string;
  registerUrl: string;
  registerHelp: string;
  status: TrackerStatus | null;
  onChange: () => Promise<void>;
}

function relTime(ts: number | null): string {
  if (!ts) return 'never';
  const diff = Date.now() - ts;
  if (diff < 60_000) return 'just now';
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} min ago`;
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} h ago`;
  return `${Math.floor(diff / 86_400_000)} d ago`;
}

function CopyableUri({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);
  const onCopy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    } catch { /* ignore */ }
  };
  return (
    <Tooltip label={copied ? 'Copied' : 'Copy to clipboard'}>
      <button type="button" className={`tracker-uri${copied ? ' is-copied' : ''}`} onClick={() => void onCopy()}>
        <code>{value}</code>
        {copied ? <Check size={12} /> : <Copy size={12} />}
      </button>
    </Tooltip>
  );
}

function TrackerRow({ provider, label, registerUrl, registerHelp, status, onChange }: TrackerRowProps) {
  const bundledId = DEFAULT_CLIENT_IDS[provider].trim();
  const bundledSecret = DEFAULT_CLIENT_SECRETS[provider].trim();
  const requiresSecret = provider === 'mal'; // AniList implicit grant has no secret
  const savedHasSecret = status?.hasClientSecret ?? false;
  const savedHasId = !!status?.clientId;
  const credsReady =
    !!bundledId || (savedHasId && (!requiresSecret || (savedHasSecret || !!bundledSecret)));

  const [clientId, setClientId] = useState(status?.clientId ?? bundledId);
  const [clientSecret, setClientSecret] = useState('');
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setClientId(status?.clientId ?? bundledId);
  }, [status?.clientId, bundledId]);

  const handleConnect = async () => {
    setError(null);
    const id = (bundledId || clientId).trim();
    if (!id) {
      setError('No Client ID configured.');
      return;
    }
    if (requiresSecret && !bundledSecret && !savedHasSecret && !clientSecret.trim()) {
      setError('MAL requires a Client Secret. Paste yours below.');
      return;
    }
    setConnecting(true);
    try {
      await window.electronAPI.trackerSetClientId(provider, id);
      await window.electronAPI.trackerConnect(provider, id, bundledSecret || clientSecret);
      setClientSecret('');
      await onChange();
    } catch (err) {
      setError((err as Error).message || 'Connect failed');
    } finally {
      setConnecting(false);
    }
  };

  const handleCancel = async () => {
    try {
      await window.electronAPI.trackerCancelConnect();
    } catch { /* ignore */ }
    setConnecting(false);
  };

  const handleDisconnect = async () => {
    if (!confirm(`Disconnect ${label}? Your access token will be removed.`)) return;
    try {
      await window.electronAPI.trackerDisconnect(provider);
      await onChange();
    } catch (err) {
      setError((err as Error).message || 'Disconnect failed');
    }
  };

  const connected = status?.connected ?? false;

  return (
    <div className="tracker-row">
      <div className="tracker-head">
        <div className="tracker-name">{label}</div>
        <div className={`tracker-state ${connected ? 'on' : 'off'}`}>
          {connected
            ? <>Connected as <strong>{status?.username ?? '?'}</strong> · last sync {relTime(status?.lastSync ?? null)}</>
            : 'Not connected'}
        </div>
      </div>

      {!connected && credsReady && (
        <div className="tracker-input-row">
          {connecting ? (
            <>
              <span className="tracker-help">Waiting for browser authorization…</span>
              <button type="button" className="btn btn-secondary" onClick={() => void handleCancel()}>
                <span>Cancel</span>
              </button>
            </>
          ) : (
            <button type="button" className="btn btn--accent" onClick={() => void handleConnect()}>
              <Link2 size={14} />
              <span>Log in to {label}</span>
            </button>
          )}
        </div>
      )}

      {!connected && !credsReady && (
        <>
          <div className="tracker-help">{registerHelp}</div>
          <div className="tracker-uri-row">
            <span className="tracker-uri-label">Redirect URL</span>
            <CopyableUri value={LOOPBACK_REDIRECT_URI} />
            <button
              type="button"
              className="tracker-link"
              onClick={() => void window.electronAPI.openExternal(registerUrl)}
            >
              Open API config <ExternalLink size={11} />
            </button>
          </div>
          <div className="tracker-input-row">
            <input
              type="text"
              className="tracker-input"
              placeholder="Client ID"
              value={clientId}
              onChange={(e) => setClientId(e.target.value)}
              spellCheck={false}
              autoComplete="off"
            />
          </div>
          {requiresSecret && !bundledSecret && (
            <div className="tracker-input-row">
              <input
                type="password"
                className="tracker-input"
                placeholder={savedHasSecret ? 'Client Secret (saved, leave empty to reuse)' : 'Client Secret'}
                value={clientSecret}
                onChange={(e) => setClientSecret(e.target.value)}
                spellCheck={false}
                autoComplete="off"
              />
            </div>
          )}
          <div className="tracker-input-row">
            {connecting ? (
              <button type="button" className="btn btn-secondary" onClick={() => void handleCancel()}>
                <span>Cancel</span>
              </button>
            ) : (
              <button
                type="button"
                className="btn btn--accent"
                onClick={() => void handleConnect()}
              >
                <Link2 size={14} />
                <span>Connect</span>
              </button>
            )}
          </div>
          {connecting && (
            <div className="tracker-state off">Waiting for browser authorization…</div>
          )}
        </>
      )}

      {connected && (
        <div className="tracker-input-row">
          <button
            type="button"
            className="btn btn-secondary"
            onClick={() => void handleDisconnect()}
          >
            <Link2Off size={14} />
            <span>Disconnect</span>
          </button>
        </div>
      )}

      {error && <div className="tracker-error">{error}</div>}
    </div>
  );
}

function TrackersSection() {
  const [anilistStatus, setAnilistStatus] = useState<TrackerStatus | null>(null);
  const [malStatus, setMalStatus] = useState<TrackerStatus | null>(null);
  const { snapshot, setMainProvider } = useTrackerProgress();

  const refresh = useCallback(async () => {
    try {
      const [a, m] = await Promise.all([
        window.electronAPI.trackerStatus('anilist'),
        window.electronAPI.trackerStatus('mal'),
      ]);
      setAnilistStatus(a);
      setMalStatus(m);
    } catch (err) {
      console.error('Error loading tracker status:', err);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const main = snapshot?.mainProvider ?? 'anilist';

  return (
    <Section title="Trackers">
      <p className="section-sub">Sync watched-episode count to AniList and MyAnimeList. Updates fire when you reach the outro or hit "Mark watched", and only ever count up. Each service needs you to register a personal API client once.</p>
      <div className="tracker-list">
        <TrackerRow
          provider="anilist"
          label="AniList"
          registerUrl="https://anilist.co/settings/developer"
          registerHelp={'Create a new client. Paste the redirect URL below into AniList\'s "Redirect URL" field exactly, port and trailing /callback included.'}
          status={anilistStatus}
          onChange={refresh}
        />
        <TrackerRow
          provider="mal"
          label="MyAnimeList"
          registerUrl="https://myanimelist.net/apiconfig"
          registerHelp={'Create an app (App Type: "Web"). Paste the redirect URL below into MAL\'s "App Redirect URL" field. Save the Client ID; there is no client secret used here (PKCE flow).'}
          status={malStatus}
          onChange={refresh}
        />
      </div>

      <div className="pref-row tracker-main-row">
        <div>
          <div className="pref-label">Main tracker</div>
          <div className="pref-help">Source of truth for the watched count shown on each card. The other tracker still receives updates when both are connected.</div>
        </div>
        <SegmentedSwitch
          value={main}
          onChange={(p) => void setMainProvider(p)}
          ariaLabel="Main tracker"
          options={[
            { value: 'anilist', label: 'AniList' },
            { value: 'mal', label: 'MyAnimeList' },
          ]}
        />
      </div>
    </Section>
  );
}

export default TrackersSection;
