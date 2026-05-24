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
  type ProgressSnapshot,
  type ProgressEntry,
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
  getMainProvider,
  setMainProvider,
  getProgressSnapshot,
  replaceProgress,
  setProgressEntry,
  setProgressScoreAndStatus,
  normalizeListStatus,
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

// ----- Error sanitization -----
//
// graphql-request's ClientError stringifies the entire response + request
// payload into `err.message`, so propagating it straight to the renderer
// dumps a ~600-char JSON blob in the rating popover. Axios behaves the same
// way for MAL — the user sees raw JSON instead of "Rate limited".
//
// Map the common HTTP statuses to short user-readable strings and fall back
// to a clamped first-GraphQL-error or "Tracker error." for anything else.
// The full original message is still logged at warn level so we can debug
// from the activity log, just not surfaced to the UI.
function providerLabel(p: TrackerProvider): string {
  return p === 'anilist' ? 'AniList' : 'MAL';
}

function sanitizeTrackerError(err: unknown, provider: TrackerProvider): string {
  const label = providerLabel(provider);
  // graphql-request: { response: { status, errors: [{ message }] } }
  // axios:           { response: { status, statusText, data } }
  const e = err as {
    response?: {
      status?: number;
      statusText?: string;
      errors?: Array<{ message?: string }>;
      data?: { message?: string };
    };
    message?: string;
  };
  const status = e.response?.status;
  if (status === 429) return `${label} rate limited — try again in a minute.`;
  if (status === 401 || status === 403) return `${label} auth expired — reconnect in Settings.`;
  if (status === 404) return `${label} entry not found.`;
  if (typeof status === 'number' && status >= 500) {
    return `${label} server error (${status}) — try again later.`;
  }
  const firstGqlMsg = e.response?.errors?.[0]?.message;
  if (firstGqlMsg && firstGqlMsg.length < 200) return firstGqlMsg;
  const malMsg = e.response?.data?.message;
  if (malMsg && malMsg.length < 200) return malMsg;
  const raw = e.message;
  if (raw && raw.length < 200) return raw;
  return `${label} error — see activity log.`;
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
      return await markAnilist(token, acct.userId, args.mediaId, ep, args.totalEpisodes ?? null);
    }
    return await markMal(token, args.mediaId, ep, args.totalEpisodes ?? null);
  } catch (err) {
    // Log the raw error for debugging, but return a clean short string to
    // the renderer so the toast/popover doesn't dump a JSON blob.
    logger.error('tracker', `${args.provider} mark failed: ${(err as Error).message}`);
    return {
      ok: false,
      provider: args.provider,
      newProgress: null,
      previousProgress: null,
      reason: 'error',
      message: sanitizeTrackerError(err, args.provider),
    };
  }
}

async function markAnilist(token: string, userId: number | null, mediaId: number, ep: number, total: number | null): Promise<MarkResult> {
  const headers = { Authorization: `Bearer ${token}` };
  // Read current entry so we can apply the monotonic guard. userId is required:
  // MediaList(mediaId) without userId ignores the bearer token and returns
  // some other user's entry, which spuriously trips the monotonic guard.
  const current = userId != null
    ? await request<{ MediaList: { progress: number; status: string } | null }>(
        ANILIST_API,
        gql`query ($userId: Int, $mediaId: Int) { MediaList(userId: $userId, mediaId: $mediaId) { progress status } }`,
        { userId, mediaId },
        headers,
      ).catch(() => ({ MediaList: null }))
    : { MediaList: null };

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
  await setProgressEntry('anilist', mediaId, ep, normalizeListStatus('anilist', newStatus));
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
  const newMalStatus = isComplete ? 'completed' : 'watching';
  await setProgressEntry('mal', mediaId, ep, normalizeListStatus('mal', newMalStatus));
  logger.info('tracker', `mal ${previousProgress} → ${ep} (mediaId ${mediaId})`);
  return { ok: true, provider: 'mal', newProgress: ep, previousProgress };
}

// ----- Set progress to an exact value (no monotonic guard) -----
//
// markEpisode() only ever moves progress UP. This is the corrective path: set
// the watched count to any value, including a lower one, to undo an
// over-counted episode (e.g. an auto-advance that marked one too many). Status
// is derived from the target: 0 → planning, >= total → completed, else current.

interface SetProgressArgs {
  provider: TrackerProvider;
  mediaId: number;
  progress: number;
  totalEpisodes?: number | null;
}

export async function setEpisodeProgress(args: SetProgressArgs): Promise<MarkResult> {
  const acct = await getAccount(args.provider);
  const token = await getAccessToken(args.provider);
  if (!acct || !token) {
    return { ok: false, provider: args.provider, newProgress: null, previousProgress: null, reason: 'no-account' };
  }
  if (!args.mediaId) {
    return { ok: false, provider: args.provider, newProgress: null, previousProgress: null, reason: 'no-id' };
  }
  const target = Math.floor(args.progress);
  if (!Number.isFinite(target) || target < 0) {
    return { ok: false, provider: args.provider, newProgress: null, previousProgress: null, reason: 'no-id', message: 'invalid progress' };
  }

  try {
    if (args.provider === 'anilist') {
      return await setAnilistProgress(token, acct.userId, args.mediaId, target, args.totalEpisodes ?? null);
    }
    return await setMalProgress(token, args.mediaId, target, args.totalEpisodes ?? null);
  } catch (err) {
    logger.error('tracker', `${args.provider} set-progress failed: ${(err as Error).message}`);
    return {
      ok: false,
      provider: args.provider,
      newProgress: null,
      previousProgress: null,
      reason: 'error',
      message: sanitizeTrackerError(err, args.provider),
    };
  }
}

async function setAnilistProgress(token: string, userId: number | null, mediaId: number, progress: number, total: number | null): Promise<MarkResult> {
  const headers = { Authorization: `Bearer ${token}` };
  const current = userId != null
    ? await request<{ MediaList: { progress: number; status: string } | null }>(
        ANILIST_API,
        gql`query ($userId: Int, $mediaId: Int) { MediaList(userId: $userId, mediaId: $mediaId) { progress status } }`,
        { userId, mediaId },
        headers,
      ).catch(() => ({ MediaList: null }))
    : { MediaList: null };
  const previousProgress = current.MediaList?.progress ?? 0;

  const newStatus = progress <= 0 ? 'PLANNING' : (total != null && progress >= total ? 'COMPLETED' : 'CURRENT');
  await request(
    ANILIST_API,
    gql`mutation ($mediaId: Int, $progress: Int, $status: MediaListStatus) {
      SaveMediaListEntry(mediaId: $mediaId, progress: $progress, status: $status) {
        id progress status
      }
    }`,
    { mediaId, progress, status: newStatus },
    headers,
  );

  await markSync('anilist');
  await setProgressEntry('anilist', mediaId, progress, normalizeListStatus('anilist', newStatus));
  logger.info('tracker', `anilist set ${previousProgress} → ${progress} (mediaId ${mediaId})`);
  return { ok: true, provider: 'anilist', newProgress: progress, previousProgress };
}

async function setMalProgress(token: string, mediaId: number, progress: number, total: number | null): Promise<MarkResult> {
  const headers = { Authorization: `Bearer ${token}` };
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

  const newMalStatus = progress <= 0 ? 'plan_to_watch' : (total != null && progress >= total ? 'completed' : 'watching');
  const body = new URLSearchParams();
  body.set('num_watched_episodes', String(progress));
  body.set('status', newMalStatus);
  await axios.patch(`${MAL_API}/anime/${mediaId}/my_list_status`, body.toString(), {
    headers: { ...headers, 'Content-Type': 'application/x-www-form-urlencoded' },
  });

  await markSync('mal');
  await setProgressEntry('mal', mediaId, progress, normalizeListStatus('mal', newMalStatus));
  logger.info('tracker', `mal set ${previousProgress} → ${progress} (mediaId ${mediaId})`);
  return { ok: true, provider: 'mal', newProgress: progress, previousProgress };
}

// ----- Score (user rating) mutations -----
//
// 0–10 scale across both providers. Internally we still send AniList a
// POINT_10_DECIMAL so the value isn't rounded when the user's display format
// is POINT_100. Score === 0 clears the rating on both sides.
//
// If the user already has `progress >= totalEpisodes`, we also flip the list
// status to "completed" so rating a show on the final episode (or after
// watching every episode without the auto-mark catching) snaps the list to
// the right state.

export interface ScoreResult {
  ok: boolean;
  provider: TrackerProvider;
  newScore: number | null;
  reason?: 'no-account' | 'no-id' | 'error';
  message?: string;
  /** Whether the entry was also marked completed by this call. */
  completedToo?: boolean;
}

interface ScoreArgs {
  provider: TrackerProvider;
  mediaId: number;
  /** 0–10. 0 clears the rating. Decimals are sent as-is. */
  score: number;
  /** When known, used to decide whether to also set status to "completed". */
  totalEpisodes?: number | null;
}

export async function setScore(args: ScoreArgs): Promise<ScoreResult> {
  const acct = await getAccount(args.provider);
  const token = await getAccessToken(args.provider);
  if (!acct || !token) {
    return { ok: false, provider: args.provider, newScore: null, reason: 'no-account' };
  }
  if (!args.mediaId) {
    return { ok: false, provider: args.provider, newScore: null, reason: 'no-id' };
  }
  const score = Math.max(0, Math.min(10, Number(args.score)));
  if (!Number.isFinite(score)) {
    return { ok: false, provider: args.provider, newScore: null, reason: 'error', message: 'invalid score' };
  }
  try {
    if (args.provider === 'anilist') {
      return await scoreAnilist(token, acct.userId, args.mediaId, score, args.totalEpisodes ?? null);
    }
    return await scoreMal(token, args.mediaId, score, args.totalEpisodes ?? null);
  } catch (err) {
    logger.error('tracker', `${args.provider} score failed: ${(err as Error).message}`);
    return {
      ok: false,
      provider: args.provider,
      newScore: null,
      reason: 'error',
      message: sanitizeTrackerError(err, args.provider),
    };
  }
}

async function scoreAnilist(
  token: string,
  userId: number | null,
  mediaId: number,
  score: number,
  total: number | null,
): Promise<ScoreResult> {
  const headers = { Authorization: `Bearer ${token}` };
  // Read current progress so we know whether to also complete the entry.
  const current = userId != null
    ? await request<{ MediaList: { progress: number; status: string } | null }>(
        ANILIST_API,
        gql`query ($userId: Int, $mediaId: Int) { MediaList(userId: $userId, mediaId: $mediaId) { progress status } }`,
        { userId, mediaId },
        headers,
      ).catch(() => ({ MediaList: null }))
    : { MediaList: null };
  const currentProgress = current.MediaList?.progress ?? 0;
  const shouldComplete = total != null && total > 0 && currentProgress >= total;
  const newStatus = shouldComplete ? 'COMPLETED' : undefined;
  // scoreRaw is always 0–100 regardless of the user's display format
  // (POINT_100 / POINT_10 / POINT_10_DECIMAL / POINT_5 / POINT_3), so we
  // can write a single number without branching per format.
  const scoreRaw = Math.round(score * 10);
  await request(
    ANILIST_API,
    gql`mutation ($mediaId: Int, $scoreRaw: Int, $status: MediaListStatus) {
      SaveMediaListEntry(mediaId: $mediaId, scoreRaw: $scoreRaw, status: $status) {
        id score status
      }
    }`,
    { mediaId, scoreRaw, status: newStatus },
    headers,
  );
  await markSync('anilist');
  const stored = score > 0 ? score : null;
  await setProgressScoreAndStatus('anilist', mediaId, stored, shouldComplete ? 'completed' : null);
  logger.info('tracker', `anilist score → ${score} (mediaId ${mediaId})${shouldComplete ? ' + completed' : ''}`);
  return { ok: true, provider: 'anilist', newScore: stored, completedToo: shouldComplete };
}

async function scoreMal(
  token: string,
  mediaId: number,
  score: number,
  total: number | null,
): Promise<ScoreResult> {
  const headers = { Authorization: `Bearer ${token}` };
  let currentProgress = 0;
  try {
    const resp = await axios.get(`${MAL_API}/anime/${mediaId}`, {
      params: { fields: 'my_list_status' },
      headers,
    });
    const status = resp.data?.my_list_status as { num_episodes_watched?: number } | undefined;
    currentProgress = status?.num_episodes_watched ?? 0;
  } catch (err) {
    if (!(axios.isAxiosError(err) && err.response?.status === 404)) throw err;
  }
  const shouldComplete = total != null && total > 0 && currentProgress >= total;
  // MAL wants an integer 0-10. Round to nearest so 8.7 → 9 (matches the MAL
  // UI's own rounding when the user types a decimal).
  const malScore = Math.round(score);
  const body = new URLSearchParams();
  body.set('score', String(malScore));
  if (shouldComplete) body.set('status', 'completed');
  await axios.patch(`${MAL_API}/anime/${mediaId}/my_list_status`, body.toString(), {
    headers: { ...headers, 'Content-Type': 'application/x-www-form-urlencoded' },
  });
  await markSync('mal');
  const stored = malScore > 0 ? malScore : null;
  await setProgressScoreAndStatus('mal', mediaId, stored, shouldComplete ? 'completed' : null);
  logger.info('tracker', `mal score → ${malScore} (mediaId ${mediaId})${shouldComplete ? ' + completed' : ''}`);
  return { ok: true, provider: 'mal', newScore: stored, completedToo: shouldComplete };
}

// ----- Bulk progress fetch (one call per provider, cached on disk) -----

const PROGRESS_FETCH_TIMEOUT_MS = 15_000;

function withTimeout<T>(p: Promise<T>, ms: number, label: string): Promise<T> {
  // Hand-rolled because graphql-request and axios don't expose the same
  // signal API. Rejecting after `ms` lets a stuck request bubble up to
  // refreshProgress's catch instead of holding the IPC open forever.
  return new Promise((resolve, reject) => {
    const t = setTimeout(() => reject(new Error(`${label} timed out after ${ms}ms`)), ms);
    p.then((v) => { clearTimeout(t); resolve(v); }, (err) => { clearTimeout(t); reject(err); });
  });
}

// AniList exposes the user's whole anime list via MediaListCollection. One
// query → every entry's progress. We key by the AniList media id so the
// renderer can resolve a series record's `anilistId` directly.
async function fetchAnilistProgressMap(): Promise<Record<number, ProgressEntry>> {
  const acct = await getAccount('anilist');
  const token = await getAccessToken('anilist');
  if (!acct || !token || !acct.userId) return {};
  const headers = { Authorization: `Bearer ${token}` };
  // `score(format: POINT_10_DECIMAL)` makes AniList normalise the value to a
  // 0–10 decimal regardless of the user's chosen display format, so we don't
  // have to branch on POINT_100 / POINT_10 / POINT_5 / POINT_3 on the client.
  const data = await withTimeout(
    request<{
      MediaListCollection: {
        lists: Array<{ entries: Array<{ progress: number; status: string | null; score: number | null; repeat: number | null; media: { id: number } }> }>;
      };
    }>(
      ANILIST_API,
      gql`query ($userId: Int) {
        MediaListCollection(userId: $userId, type: ANIME) {
          lists { entries { progress status score(format: POINT_10_DECIMAL) repeat media { id } } }
        }
      }`,
      { userId: acct.userId },
      headers,
    ),
    PROGRESS_FETCH_TIMEOUT_MS,
    'AniList MediaListCollection',
  );
  const map: Record<number, ProgressEntry> = {};
  for (const list of data.MediaListCollection?.lists ?? []) {
    for (const entry of list.entries ?? []) {
      if (entry.media?.id != null) {
        map[entry.media.id] = {
          progress: entry.progress ?? 0,
          status: normalizeListStatus('anilist', entry.status),
          score: typeof entry.score === 'number' && entry.score > 0 ? entry.score : null,
          rewatch: typeof entry.repeat === 'number' && entry.repeat > 0 ? entry.repeat : null,
        };
      }
    }
  }
  return map;
}

// MAL paginates at 1000 entries. Most users are well under that; loop just
// in case. Keys are MAL anime ids so the renderer matches against malId.
async function fetchMalProgressMap(): Promise<Record<number, ProgressEntry>> {
  const token = await getAccessToken('mal');
  if (!token) return {};
  const headers = { Authorization: `Bearer ${token}` };
  const map: Record<number, ProgressEntry> = {};
  let offset = 0;
  const limit = 1000;
  // Hard upper bound prevents an infinite loop if MAL returns a malformed
  // paging cursor; 50k entries is far past the largest real list.
  for (let i = 0; i < 50; i++) {
    const resp = await withTimeout(
      axios.get(`${MAL_API}/users/@me/animelist`, {
        params: { fields: 'list_status{status,num_episodes_watched,is_rewatching,num_times_rewatched,score}', limit, offset },
        headers,
      }),
      PROGRESS_FETCH_TIMEOUT_MS,
      `MAL animelist page ${i}`,
    );
    const data = resp.data as {
      data: Array<{
        node: { id: number };
        list_status?: { num_episodes_watched?: number; status?: string; is_rewatching?: boolean; num_times_rewatched?: number; score?: number };
      }>;
      paging?: { next?: string };
    };
    for (const item of data.data ?? []) {
      if (item.node?.id != null) {
        const ls = item.list_status;
        map[item.node.id] = {
          progress: ls?.num_episodes_watched ?? 0,
          status: normalizeListStatus('mal', ls?.status, ls?.is_rewatching ?? false),
          score: typeof ls?.score === 'number' && ls.score > 0 ? ls.score : null,
          rewatch: typeof ls?.num_times_rewatched === 'number' && ls.num_times_rewatched > 0 ? ls.num_times_rewatched : null,
        };
      }
    }
    if (!data.paging?.next || (data.data?.length ?? 0) < limit) break;
    offset += limit;
  }
  return map;
}

// 5 minutes is short enough that a watching session keeps fresh data, long
// enough that opening + closing the app a few times doesn't spam AniList.
const PROGRESS_FRESHNESS_MS = 5 * 60_000;

export async function refreshProgress(provider: TrackerProvider): Promise<Record<number, ProgressEntry>> {
  logger.info('tracker', `${provider} progress refresh starting`);
  try {
    const map = provider === 'anilist'
      ? await fetchAnilistProgressMap()
      : await fetchMalProgressMap();
    await replaceProgress(provider, map);
    logger.info('tracker', `${provider} progress cache refreshed (${Object.keys(map).length} entries)`);
    return map;
  } catch (err) {
    logger.warn('tracker', `${provider} progress refresh failed: ${(err as Error).message}`);
    return (await getProgressSnapshot())[provider];
  }
}

export async function refreshAllProgress(opts: { force?: boolean } = {}): Promise<void> {
  const snap = await getProgressSnapshot();
  const tasks: Promise<unknown>[] = [];
  for (const provider of ['anilist', 'mal'] as const) {
    if (!(await getAccount(provider))) continue;
    const fetchedAt = snap.fetchedAt[provider];
    if (!opts.force && fetchedAt != null && Date.now() - fetchedAt < PROGRESS_FRESHNESS_MS) {
      logger.info('tracker', `${provider} progress is fresh, skipping refresh`);
      continue;
    }
    tasks.push(refreshProgress(provider));
  }
  await Promise.allSettled(tasks);
}

// ----- Public surface for IPC handlers -----

export const trackerHandler = {
  startConnect: (provider: TrackerProvider, clientId: string, clientSecret: string) =>
    startConnect(provider, clientId, clientSecret),
  cancelConnect,
  markEpisode,
  setEpisodeProgress,
  setScore,
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
  async getProgress(): Promise<ProgressSnapshot> {
    return getProgressSnapshot();
  },
  refreshProgress,
  refreshAllProgress,
  async getMainProvider(): Promise<TrackerProvider> {
    return getMainProvider();
  },
  async setMainProvider(provider: TrackerProvider): Promise<TrackerProvider> {
    await setMainProvider(provider);
    return provider;
  },
};

export default trackerHandler;
