import { createServer, type IncomingMessage, type ServerResponse, type Server } from 'http';
import { AddressInfo } from 'net';
import { randomBytes } from 'crypto';
import { shell } from 'electron';
import axios from 'axios';
import { request, gql } from 'graphql-request';
import { logger } from '../services/logger';
import {
  type TrackerProvider,
  type TrackerStatus,
  setAccount,
  clearAccount,
  getAccount,
  getAccessToken,
  getStatus,
  markSync,
  setClientId,
  getClientId,
  setClientSecret,
  getClientSecret,
} from '../services/trackerStore';

import { LOOPBACK_HOST, LOOPBACK_PORT, LOOPBACK_REDIRECT_URI } from '../../shared/trackerConstants';

// Per-OAuth-attempt state. Held on the module while a flow is in flight so
// the handlers in main.ts can ask "are we currently authenticating?".
type PendingFlow = {
  provider: TrackerProvider;
  state: string;
  codeVerifier: string;       // PKCE — only used by MAL
  redirectUri: string;        // includes the chosen port
  clientId: string;
  clientSecret: string;       // empty for AniList; required for MAL
  server: Server;
  resolve: (token: ProviderTokens) => void;
  reject: (err: Error) => void;
  timeoutHandle: ReturnType<typeof setTimeout>;
};
let pending: PendingFlow | null = null;
const FLOW_TIMEOUT_MS = 5 * 60 * 1000;

interface ProviderTokens {
  accessToken: string;
  refreshToken?: string | null;
  expiresAt: number | null;
}

const ANILIST_API = 'https://graphql.anilist.co';
const MAL_OAUTH = 'https://myanimelist.net/v1/oauth2';
const MAL_API = 'https://api.myanimelist.net/v2';

const SUCCESS_PAGE = (provider: string) => `<!doctype html><html><head><meta charset="utf-8">
<title>AniBeam — connected</title>
<style>html,body{margin:0;height:100%;background:#0b0b10;color:#f1f5f9;font-family:'JetBrains Mono',ui-monospace,monospace;display:flex;align-items:center;justify-content:center;flex-direction:column;gap:0.5rem}h1{font-weight:500;font-size:1rem}p{color:#94a3b8;font-size:0.85rem}</style></head>
<body><h1>${provider} connected</h1><p>You can close this tab and return to AniBeam.</p></body></html>`;

const ERROR_PAGE = (msg: string) => `<!doctype html><html><head><meta charset="utf-8">
<title>AniBeam — auth failed</title>
<style>html,body{margin:0;height:100%;background:#0b0b10;color:#f1f5f9;font-family:'JetBrains Mono',ui-monospace,monospace;display:flex;align-items:center;justify-content:center;flex-direction:column;gap:0.5rem}h1{font-weight:500;font-size:1rem;color:#f43f5e}p{color:#94a3b8;font-size:0.85rem}</style></head>
<body><h1>Authentication failed</h1><p>${msg}</p></body></html>`;

// AniList uses implicit grant: the token comes back in the URL fragment, which
// the server doesn't see. We serve a tiny page that reads window.location.hash
// and re-issues the request as a query string our server CAN read.
const FRAGMENT_FORWARDER = `<!doctype html><html><head><meta charset="utf-8"><title>Connecting…</title>
<style>html,body{margin:0;height:100%;background:#0b0b10;color:#94a3b8;font-family:'JetBrains Mono',ui-monospace,monospace;display:flex;align-items:center;justify-content:center}</style></head>
<body>Connecting…<script>
(function(){
  var h=window.location.hash;
  if(!h){document.body.textContent='No token in URL.';return;}
  // Replace # with ? and reload so the params land in the query string.
  window.location.replace(window.location.pathname+'?'+h.slice(1));
})();
</script></body></html>`;

function randomToken(bytes = 32): string {
  return randomBytes(bytes).toString('base64url');
}

function pkceVerifier(): string {
  return randomBytes(32).toString('base64url');
}

// MAL accepts "plain" PKCE (verifier == challenge). Documented quirk: they
// don't support S256, so we don't even hash. Spec-compliant for plain method.
function pkceChallenge(verifier: string): { challenge: string; method: 'plain' | 'S256' } {
  return { challenge: verifier, method: 'plain' };
}

// Bind to the fixed loopback port. EADDRINUSE bubbles up so the connect call
// rejects with a useful message — almost always means another AniBeam instance
// is mid-connect, or some other app is squatting on the port.
function bindLoopback(handler: (req: IncomingMessage, res: ServerResponse) => void): Promise<{ server: Server; port: number }> {
  return new Promise((resolve, reject) => {
    const server = createServer(handler);
    server.once('error', (err: NodeJS.ErrnoException) => {
      if (err.code === 'EADDRINUSE') {
        reject(new Error(`Port ${LOOPBACK_PORT} is in use — close any other AniBeam instance and try again.`));
      } else {
        reject(err);
      }
    });
    server.listen(LOOPBACK_PORT, LOOPBACK_HOST, () => {
      const addr = server.address() as AddressInfo;
      resolve({ server, port: addr.port });
    });
  });
}

function closeFlow(): void {
  if (!pending) return;
  try { clearTimeout(pending.timeoutHandle); } catch { /* ignore */ }
  try { pending.server.close(); } catch { /* ignore */ }
  pending = null;
}

export async function startConnect(provider: TrackerProvider, clientId: string, clientSecret: string): Promise<TrackerStatus> {
  // If a previous flow stalled (e.g. the user closed the browser tab, or the
  // provider returned an error page that never redirected back), tear it
  // down and start fresh. Holding a hard guard here just frustrates users.
  if (pending) {
    logger.info('tracker', `cancelling stale ${pending.provider} flow before starting ${provider}`);
    pending.reject(new Error('superseded by a new connect'));
    closeFlow();
  }
  if (!clientId.trim()) throw new Error(`No client ID set for ${provider}.`);
  // Persist any new credentials BEFORE starting OAuth so a kill mid-flow
  // doesn't lose them. Empty values are intentionally ignored — we use the
  // already-stored secret (if any) on retries.
  await setClientId(provider, clientId.trim());
  if (clientSecret.trim()) {
    await setClientSecret(provider, clientSecret.trim());
  }
  // Resolve the secret we'll actually send: caller-provided > stored.
  const effectiveSecret = clientSecret.trim() || (await getClientSecret(provider));
  if (provider === 'mal' && !effectiveSecret) {
    throw new Error('MAL requires a client secret. Paste it in the Trackers tab, or set ANIBEAM_MAL_CLIENT_SECRET in .env.local for build-time bundling.');
  }

  const state = randomToken(16);
  const verifier = pkceVerifier();

  const tokens = await new Promise<ProviderTokens>((resolve, reject) => {
    const handler = (req: IncomingMessage, res: ServerResponse) => {
      const url = new URL(req.url ?? '/', `http://${LOOPBACK_HOST}:0`);
      if (url.pathname !== '/callback') {
        res.writeHead(404).end('not found');
        return;
      }

      // Implicit-grant fragment forwarder hop. AniList lands here first with a
      // hash fragment we can't read — page reloads with the params as a query.
      if (provider === 'anilist' && !url.searchParams.has('access_token') && !url.searchParams.has('error')) {
        res.writeHead(200, { 'Content-Type': 'text/html' });
        res.end(FRAGMENT_FORWARDER);
        return;
      }

      const error = url.searchParams.get('error');
      if (error) {
        res.writeHead(400, { 'Content-Type': 'text/html' });
        res.end(ERROR_PAGE(error));
        reject(new Error(`Provider returned error: ${error}`));
        return;
      }

      // MAL passes state back; AniList doesn't (we don't send it). Only
      // verify when we actually sent one.
      if (provider !== 'anilist') {
        const returnedState = url.searchParams.get('state');
        if (returnedState !== state) {
          res.writeHead(400, { 'Content-Type': 'text/html' });
          res.end(ERROR_PAGE('state mismatch — possible CSRF, aborted.'));
          reject(new Error('OAuth state mismatch'));
          return;
        }
      }

      if (provider === 'anilist') {
        const accessToken = url.searchParams.get('access_token');
        const expiresIn = url.searchParams.get('expires_in');
        if (!accessToken) {
          res.writeHead(400, { 'Content-Type': 'text/html' });
          res.end(ERROR_PAGE('no access_token in callback'));
          reject(new Error('AniList returned no access_token'));
          return;
        }
        res.writeHead(200, { 'Content-Type': 'text/html' });
        res.end(SUCCESS_PAGE('AniList'));
        const expiresAt = expiresIn ? Date.now() + parseInt(expiresIn, 10) * 1000 : null;
        resolve({ accessToken, expiresAt, refreshToken: null });
        return;
      }

      // MAL — exchange the code for a token.
      const code = url.searchParams.get('code');
      if (!code) {
        res.writeHead(400, { 'Content-Type': 'text/html' });
        res.end(ERROR_PAGE('no code in callback'));
        reject(new Error('MAL returned no code'));
        return;
      }
      // Respond to browser before doing the network exchange — feels snappier.
      res.writeHead(200, { 'Content-Type': 'text/html' });
      res.end(SUCCESS_PAGE('MyAnimeList'));
      void exchangeMalCode(code, verifier, clientId, effectiveSecret, pending!.redirectUri).then(resolve).catch(reject);
    };

    void bindLoopback(handler).then(({ server }) => {
      const timeoutHandle = setTimeout(() => {
        if (pending?.server === server) {
          logger.warn('tracker', `${provider} OAuth flow timed out after ${FLOW_TIMEOUT_MS / 1000}s`);
          reject(new Error('Authorization timed out — try again.'));
          closeFlow();
        }
      }, FLOW_TIMEOUT_MS);
      pending = { provider, state, codeVerifier: verifier, redirectUri: LOOPBACK_REDIRECT_URI, clientId, clientSecret: effectiveSecret, server, resolve, reject, timeoutHandle };
      const authUrl = buildAuthUrl(provider, clientId, state, verifier, LOOPBACK_REDIRECT_URI);
      logger.info('tracker', `${provider} OAuth started, redirect uri = ${LOOPBACK_REDIRECT_URI}`);
      void shell.openExternal(authUrl);
    }).catch(reject);
  }).finally(() => {
    closeFlow();
  });

  // Hit the provider's "me" endpoint so we can store the username.
  const profile = await fetchProfile(provider, tokens.accessToken);
  const account = await setAccount(provider, {
    username: profile.username,
    userId: profile.userId,
    accessToken: tokens.accessToken,
    refreshToken: tokens.refreshToken,
    expiresAt: tokens.expiresAt,
    clientId,
  });
  logger.info('tracker', `${provider} connected as ${account.username ?? '?'}`);
  return getStatus(provider);
}

export function cancelConnect(): void {
  if (!pending) return;
  pending.reject(new Error('cancelled'));
  closeFlow();
}

function buildAuthUrl(provider: TrackerProvider, clientId: string, state: string, verifier: string, redirectUri: string): string {
  if (provider === 'anilist') {
    // AniList's authorize endpoint runs Laravel Passport with strict param
    // validation — sending redirect_uri (even when it matches the registered
    // one) and/or state/scope makes it return a generic
    // "unsupported_grant_type" JSON. The docs' own example omits redirect_uri
    // entirely; AniList uses the URL registered on the developer panel. So
    // we send only the two params the docs show.
    const u = new URL('https://anilist.co/api/v2/oauth/authorize');
    u.searchParams.set('client_id', clientId);
    u.searchParams.set('response_type', 'token');
    return u.toString();
  }
  // MAL OAuth 2.0 + PKCE.
  const ch = pkceChallenge(verifier);
  const u = new URL(`${MAL_OAUTH}/authorize`);
  u.searchParams.set('response_type', 'code');
  u.searchParams.set('client_id', clientId);
  u.searchParams.set('redirect_uri', redirectUri);
  u.searchParams.set('state', state);
  u.searchParams.set('code_challenge', ch.challenge);
  u.searchParams.set('code_challenge_method', ch.method);
  return u.toString();
}

async function exchangeMalCode(code: string, verifier: string, clientId: string, clientSecret: string, redirectUri: string): Promise<ProviderTokens> {
  const body = new URLSearchParams({
    client_id: clientId,
    client_secret: clientSecret,
    code,
    code_verifier: verifier,
    grant_type: 'authorization_code',
    redirect_uri: redirectUri,
  });
  const resp = await axios.post(`${MAL_OAUTH}/token`, body.toString(), {
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
  });
  const data = resp.data as { access_token: string; refresh_token: string; expires_in: number };
  return {
    accessToken: data.access_token,
    refreshToken: data.refresh_token,
    expiresAt: Date.now() + data.expires_in * 1000,
  };
}

interface ProfileInfo {
  username: string | null;
  userId: number | null;
}

async function fetchProfile(provider: TrackerProvider, accessToken: string): Promise<ProfileInfo> {
  if (provider === 'anilist') {
    const data = await request<{ Viewer: { id: number; name: string } }>(
      ANILIST_API,
      gql`query { Viewer { id name } }`,
      {},
      { Authorization: `Bearer ${accessToken}` },
    );
    return { username: data.Viewer.name, userId: data.Viewer.id };
  }
  const resp = await axios.get(`${MAL_API}/users/@me`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
  const data = resp.data as { id: number; name: string };
  return { username: data.name, userId: data.id };
}

// ----- Progress mutations -----

export interface MarkResult {
  ok: boolean;
  provider: TrackerProvider;
  newProgress: number | null;
  previousProgress: number | null;
  reason?: 'no-account' | 'no-id' | 'not-newer' | 'error';
  message?: string;
}

interface MarkArgs {
  provider: TrackerProvider;
  // Provider-specific media id from our metadata.
  mediaId: number;
  episodeNumber: number;
  // Total episodes — used to set the list status to "completed" when we hit
  // the last one. Not strictly required.
  totalEpisodes?: number | null;
}

export async function markEpisode(args: MarkArgs): Promise<MarkResult> {
  const acct = await getAccount(args.provider);
  const token = await getAccessToken(args.provider);
  if (!acct || !token) {
    return { ok: false, provider: args.provider, newProgress: null, previousProgress: null, reason: 'no-account' };
  }
  if (!args.mediaId) {
    return { ok: false, provider: args.provider, newProgress: null, previousProgress: null, reason: 'no-id' };
  }
  // Round down decimals (e.g. ep 12.5 → keep at 12 for tracker purposes).
  const ep = Math.floor(args.episodeNumber);
  if (!Number.isFinite(ep) || ep <= 0) {
    return { ok: false, provider: args.provider, newProgress: null, previousProgress: null, reason: 'no-id', message: 'invalid episode' };
  }

  try {
    if (args.provider === 'anilist') {
      return await markAnilist(token, args.mediaId, ep, args.totalEpisodes ?? null);
    }
    return await markMal(token, args.mediaId, ep, args.totalEpisodes ?? null);
  } catch (err) {
    const message = (err as Error).message;
    logger.error('tracker', `${args.provider} mark failed: ${message}`);
    return { ok: false, provider: args.provider, newProgress: null, previousProgress: null, reason: 'error', message };
  }
}

async function markAnilist(token: string, mediaId: number, ep: number, total: number | null): Promise<MarkResult> {
  const headers = { Authorization: `Bearer ${token}` };
  // Read current entry so we can apply the monotonic guard.
  const current = await request<{ MediaList: { progress: number; status: string } | null }>(
    ANILIST_API,
    gql`query ($mediaId: Int) { MediaList(mediaId: $mediaId) { progress status } }`,
    { mediaId },
    headers,
  ).catch(() => ({ MediaList: null }));

  const previousProgress = current.MediaList?.progress ?? 0;
  if (previousProgress >= ep) {
    return { ok: false, provider: 'anilist', newProgress: previousProgress, previousProgress, reason: 'not-newer' };
  }

  const isComplete = total != null && ep >= total;
  const newStatus = isComplete ? 'COMPLETED' : 'CURRENT';
  await request(
    ANILIST_API,
    gql`mutation ($mediaId: Int, $progress: Int, $status: MediaListStatus) {
      SaveMediaListEntry(mediaId: $mediaId, progress: $progress, status: $status) {
        id progress status
      }
    }`,
    { mediaId, progress: ep, status: newStatus },
    headers,
  );

  await markSync('anilist');
  logger.info('tracker', `anilist ${previousProgress} → ${ep} (mediaId ${mediaId})`);
  return { ok: true, provider: 'anilist', newProgress: ep, previousProgress };
}

async function markMal(token: string, mediaId: number, ep: number, total: number | null): Promise<MarkResult> {
  const headers = { Authorization: `Bearer ${token}` };
  // Read current status to enforce monotonic. MAL returns 404 if the user has
  // never added the anime — treat that as previousProgress 0.
  let previousProgress = 0;
  try {
    const resp = await axios.get(`${MAL_API}/anime/${mediaId}`, {
      params: { fields: 'my_list_status' },
      headers,
    });
    const status = resp.data?.my_list_status as { num_episodes_watched?: number } | undefined;
    previousProgress = status?.num_episodes_watched ?? 0;
  } catch (err) {
    if (axios.isAxiosError(err) && err.response?.status === 404) {
      previousProgress = 0;
    } else {
      throw err;
    }
  }

  if (previousProgress >= ep) {
    return { ok: false, provider: 'mal', newProgress: previousProgress, previousProgress, reason: 'not-newer' };
  }

  const isComplete = total != null && ep >= total;
  const body = new URLSearchParams();
  body.set('num_watched_episodes', String(ep));
  body.set('status', isComplete ? 'completed' : 'watching');
  await axios.patch(`${MAL_API}/anime/${mediaId}/my_list_status`, body.toString(), {
    headers: { ...headers, 'Content-Type': 'application/x-www-form-urlencoded' },
  });

  await markSync('mal');
  logger.info('tracker', `mal ${previousProgress} → ${ep} (mediaId ${mediaId})`);
  return { ok: true, provider: 'mal', newProgress: ep, previousProgress };
}

// ----- Public surface for IPC handlers -----

export const trackerHandler = {
  startConnect: (provider: TrackerProvider, clientId: string, clientSecret: string) =>
    startConnect(provider, clientId, clientSecret),
  cancelConnect,
  markEpisode,
  async disconnect(provider: TrackerProvider): Promise<TrackerStatus> {
    await clearAccount(provider);
    return getStatus(provider);
  },
  async status(provider: TrackerProvider): Promise<TrackerStatus> {
    return getStatus(provider);
  },
  async setClientId(provider: TrackerProvider, clientId: string): Promise<void> {
    await setClientId(provider, clientId);
  },
  async getClientId(provider: TrackerProvider): Promise<string> {
    return getClientId(provider);
  },
};

export default trackerHandler;
