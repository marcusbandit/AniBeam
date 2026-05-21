import { readFile, writeFile, mkdir } from 'fs/promises';
import { join } from 'path';
import { app, safeStorage } from 'electron';
import { logger } from './logger';

export type TrackerProvider = 'anilist' | 'mal';

// Canonical list-status value used across the app. Provider-specific values
// (AniList CURRENT, MAL watching, etc.) are normalized into this set by
// normalizeListStatus() below so the renderer never has to know about either
// vocabulary. `null` means the provider doesn't have a list entry for this
// media id (or returned an unrecognized value).
export type ListStatus =
  | 'watching'
  | 'planning'
  | 'completed'
  | 'paused'
  | 'dropped'
  | 'repeating';

export interface ProgressEntry {
  progress: number;
  status: ListStatus | null;
  // Normalised to a 0-10 scale (AniList POINT_10_DECIMAL, MAL native 0-10).
  // Null means "no score" — both providers use 0 to signal unrated and we
  // collapse that to null here so consumers don't have to special-case it.
  score: number | null;
}

const ANILIST_TO_CANONICAL: Record<string, ListStatus> = {
  CURRENT: 'watching',
  PLANNING: 'planning',
  COMPLETED: 'completed',
  PAUSED: 'paused',
  DROPPED: 'dropped',
  REPEATING: 'repeating',
};

const MAL_TO_CANONICAL: Record<string, ListStatus> = {
  watching: 'watching',
  plan_to_watch: 'planning',
  completed: 'completed',
  on_hold: 'paused',
  dropped: 'dropped',
};

export function normalizeListStatus(
  provider: TrackerProvider,
  raw: string | null | undefined,
  // MAL has no separate "rewatching" list status; it uses an is_rewatching
  // flag while keeping `status` as "watching". Pass the flag in so we can
  // promote those entries to "repeating" to match AniList's vocabulary.
  isRewatching: boolean = false,
): ListStatus | null {
  if (!raw) return null;
  if (provider === 'anilist') {
    return ANILIST_TO_CANONICAL[raw] ?? null;
  }
  const mapped = MAL_TO_CANONICAL[raw];
  if (mapped === 'watching' && isRewatching) return 'repeating';
  return mapped ?? null;
}

export interface TrackerAccount {
  // Public bits — safe to send to renderer
  username: string | null;
  userId: number | null;
  expiresAt: number | null;     // ms epoch, null = no expiry (AniList)
  lastSync: number | null;      // ms epoch of last successful progress update
  clientId: string;             // OAuth app id user registered

  // Secret bits — never leave main. Persisted as base64 of safeStorage cipher
  // when available, or base64 of utf-8 plaintext otherwise (with a flag).
  accessTokenCipher: string;
  refreshTokenCipher: string | null;
  cipherEncrypted: boolean;     // false = plaintext fallback, warn the user
}

interface TrackerStore {
  anilist: TrackerAccount | null;
  mal: TrackerAccount | null;
  // Client IDs the user has entered, kept even after disconnect so they don't
  // have to paste it again on reconnect. Plaintext — these aren't secrets.
  clientIds: { anilist: string; mal: string };
  // Per-user client SECRETS (only MAL needs one). Encrypted at rest via the
  // same safeStorage flow as access tokens. Persisted across disconnects so
  // the user doesn't have to paste it again on reconnect.
  clientSecretCiphers: { anilist: string; mal: string };
  // Did we encrypt those ciphertexts? Same flag semantics as accounts.
  clientSecretsEncrypted: boolean;
  // Which provider's progress is the source of truth for UI surfaces (e.g.
  // the watched count on show cards). Falls back to the other provider on
  // a per-series basis when the main one has no entry for that series.
  mainProvider: TrackerProvider;
  // Last-known watched-episode count + list status per provider, keyed by
  // provider's media id. Persisted so the UI shows numbers immediately on
  // launch; refreshed in the background after app ready and after every
  // successful mark. Schema v1 stored bare numbers — loadStore() migrates
  // those to {progress, status: null} on read.
  progress: { anilist: Record<number, ProgressEntry>; mal: Record<number, ProgressEntry> };
  progressFetchedAt: { anilist: number | null; mal: number | null };
  version: 2;
}

const DEFAULT_STORE: TrackerStore = {
  anilist: null,
  mal: null,
  clientIds: { anilist: '', mal: '' },
  clientSecretCiphers: { anilist: '', mal: '' },
  clientSecretsEncrypted: true,
  mainProvider: 'anilist',
  progress: { anilist: {}, mal: {} },
  progressFetchedAt: { anilist: null, mal: null },
  version: 2,
};

// Coerce a stored progress map from v1 (numbers) or v2 ({progress,status})
// into the canonical v2 shape. Anything we can't recognize becomes a fresh
// {progress: 0, status: null} so the rest of the codebase never has to
// branch on the old shape.
function migrateProgressMap(raw: unknown): Record<number, ProgressEntry> {
  if (!raw || typeof raw !== 'object') return {};
  const out: Record<number, ProgressEntry> = {};
  for (const [k, v] of Object.entries(raw as Record<string, unknown>)) {
    const id = Number(k);
    if (!Number.isFinite(id)) continue;
    if (typeof v === 'number') {
      out[id] = { progress: v, status: null, score: null };
    } else if (v && typeof v === 'object') {
      const obj = v as { progress?: unknown; status?: unknown; score?: unknown };
      const progress = typeof obj.progress === 'number' ? obj.progress : 0;
      const status = typeof obj.status === 'string' ? (obj.status as ListStatus) : null;
      const score = typeof obj.score === 'number' && obj.score > 0 ? obj.score : null;
      out[id] = { progress, status, score };
    }
  }
  return out;
}

function storePath(): string {
  return join(app.getPath('userData'), 'trackers.json');
}

async function ensureDir(): Promise<void> {
  try {
    await mkdir(app.getPath('userData'), { recursive: true });
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code !== 'EEXIST') throw err;
  }
}

let cache: TrackerStore | null = null;

export async function loadStore(): Promise<TrackerStore> {
  if (cache) return cache;
  try {
    await ensureDir();
    const raw = await readFile(storePath(), 'utf-8');
    const parsed = JSON.parse(raw) as Partial<TrackerStore> & {
      progress?: { anilist?: unknown; mal?: unknown };
    };
    cache = {
      ...DEFAULT_STORE,
      ...parsed,
      clientIds: { ...DEFAULT_STORE.clientIds, ...(parsed.clientIds ?? {}) },
      clientSecretCiphers: { ...DEFAULT_STORE.clientSecretCiphers, ...(parsed.clientSecretCiphers ?? {}) },
      progress: {
        anilist: migrateProgressMap(parsed.progress?.anilist),
        mal: migrateProgressMap(parsed.progress?.mal),
      },
      progressFetchedAt: { ...DEFAULT_STORE.progressFetchedAt, ...(parsed.progressFetchedAt ?? {}) },
      version: 2,
    };
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code === 'ENOENT') {
      cache = { ...DEFAULT_STORE };
    } else {
      logger.error('tracker', `failed to load tracker store: ${(err as Error).message}`);
      cache = { ...DEFAULT_STORE };
    }
  }
  return cache;
}

async function saveStore(store: TrackerStore): Promise<void> {
  await ensureDir();
  await writeFile(storePath(), JSON.stringify(store, null, 2), 'utf-8');
  cache = store;
}

function encrypt(plain: string): { cipher: string; encrypted: boolean } {
  if (!plain) return { cipher: '', encrypted: false };
  if (safeStorage.isEncryptionAvailable()) {
    return { cipher: safeStorage.encryptString(plain).toString('base64'), encrypted: true };
  }
  logger.warn('tracker', 'safeStorage unavailable — token persisted as plaintext');
  return { cipher: Buffer.from(plain, 'utf-8').toString('base64'), encrypted: false };
}

function decrypt(cipher: string, encrypted: boolean): string {
  if (!cipher) return '';
  const buf = Buffer.from(cipher, 'base64');
  if (encrypted) return safeStorage.decryptString(buf);
  return buf.toString('utf-8');
}

export interface SetTokenOpts {
  username: string | null;
  userId: number | null;
  accessToken: string;
  refreshToken?: string | null;
  expiresAt?: number | null;
  clientId: string;
}

export async function setAccount(provider: TrackerProvider, opts: SetTokenOpts): Promise<TrackerAccount> {
  const store = await loadStore();
  const access = encrypt(opts.accessToken);
  const refresh = opts.refreshToken ? encrypt(opts.refreshToken) : null;
  const account: TrackerAccount = {
    username: opts.username,
    userId: opts.userId,
    expiresAt: opts.expiresAt ?? null,
    lastSync: null,
    clientId: opts.clientId,
    accessTokenCipher: access.cipher,
    refreshTokenCipher: refresh?.cipher ?? null,
    cipherEncrypted: access.encrypted,
  };
  store[provider] = account;
  store.clientIds[provider] = opts.clientId;
  await saveStore(store);
  return account;
}

export async function clearAccount(provider: TrackerProvider): Promise<void> {
  const store = await loadStore();
  store[provider] = null;
  // Drop cached progress too — a reconnect to a different account would
  // otherwise serve numbers from the previous user.
  store.progress[provider] = {};
  store.progressFetchedAt[provider] = null;
  await saveStore(store);
}

export async function getAccount(provider: TrackerProvider): Promise<TrackerAccount | null> {
  const store = await loadStore();
  return store[provider];
}

export async function getAccessToken(provider: TrackerProvider): Promise<string | null> {
  const acct = await getAccount(provider);
  if (!acct) return null;
  try {
    return decrypt(acct.accessTokenCipher, acct.cipherEncrypted);
  } catch (err) {
    logger.error('tracker', `failed to decrypt ${provider} token: ${(err as Error).message}`);
    return null;
  }
}

export async function setClientId(provider: TrackerProvider, clientId: string): Promise<void> {
  const store = await loadStore();
  store.clientIds[provider] = clientId.trim();
  await saveStore(store);
}

export async function getClientId(provider: TrackerProvider): Promise<string> {
  const store = await loadStore();
  return store.clientIds[provider] ?? '';
}

export async function setClientSecret(provider: TrackerProvider, plain: string): Promise<void> {
  const store = await loadStore();
  if (!plain.trim()) {
    store.clientSecretCiphers[provider] = '';
  } else {
    const enc = encrypt(plain.trim());
    store.clientSecretCiphers[provider] = enc.cipher;
    store.clientSecretsEncrypted = enc.encrypted;
  }
  await saveStore(store);
}

export async function getClientSecret(provider: TrackerProvider): Promise<string> {
  const store = await loadStore();
  const cipher = store.clientSecretCiphers[provider];
  if (!cipher) return '';
  try {
    return decrypt(cipher, store.clientSecretsEncrypted);
  } catch (err) {
    logger.error('tracker', `failed to decrypt ${provider} secret: ${(err as Error).message}`);
    return '';
  }
}

export async function hasClientSecret(provider: TrackerProvider): Promise<boolean> {
  const store = await loadStore();
  return !!store.clientSecretCiphers[provider];
}

export async function markSync(provider: TrackerProvider): Promise<void> {
  const store = await loadStore();
  const acct = store[provider];
  if (!acct) return;
  acct.lastSync = Date.now();
  await saveStore(store);
}

// Public-safe shape for the renderer — never includes ciphertext.
export interface TrackerStatus {
  connected: boolean;
  username: string | null;
  expiresAt: number | null;
  lastSync: number | null;
  clientId: string;
  hasClientSecret: boolean;     // true if a stored per-user secret exists
  cipherEncrypted: boolean;
}

export async function getMainProvider(): Promise<TrackerProvider> {
  const store = await loadStore();
  return store.mainProvider;
}

export async function setMainProvider(provider: TrackerProvider): Promise<void> {
  const store = await loadStore();
  store.mainProvider = provider;
  await saveStore(store);
}

export interface ProgressSnapshot {
  mainProvider: TrackerProvider;
  anilist: Record<number, ProgressEntry>;
  mal: Record<number, ProgressEntry>;
  fetchedAt: { anilist: number | null; mal: number | null };
}

export async function getProgressSnapshot(): Promise<ProgressSnapshot> {
  const store = await loadStore();
  return {
    mainProvider: store.mainProvider,
    anilist: { ...store.progress.anilist },
    mal: { ...store.progress.mal },
    fetchedAt: { ...store.progressFetchedAt },
  };
}

export async function replaceProgress(provider: TrackerProvider, map: Record<number, ProgressEntry>): Promise<void> {
  const store = await loadStore();
  store.progress[provider] = map;
  store.progressFetchedAt[provider] = Date.now();
  await saveStore(store);
}

// Merge a single (mediaId, progress, status) update into the cached map.
// Status is optional — a mark mutation knows the new progress for sure but
// may not always know the resulting status. Pass null to leave status as-is.
// Score is preserved across episode marks (mark mutations don't touch it).
export async function setProgressEntry(
  provider: TrackerProvider,
  mediaId: number,
  progress: number,
  status: ListStatus | null = null,
): Promise<void> {
  const store = await loadStore();
  const prev = store.progress[provider][mediaId];
  store.progress[provider][mediaId] = {
    progress,
    status: status ?? prev?.status ?? null,
    score: prev?.score ?? null,
  };
  await saveStore(store);
}

export async function getStatus(provider: TrackerProvider): Promise<TrackerStatus> {
  const store = await loadStore();
  const acct = store[provider];
  return {
    connected: !!acct,
    username: acct?.username ?? null,
    expiresAt: acct?.expiresAt ?? null,
    lastSync: acct?.lastSync ?? null,
    clientId: acct?.clientId ?? store.clientIds[provider] ?? '',
    hasClientSecret: !!store.clientSecretCiphers[provider],
    cipherEncrypted: acct?.cipherEncrypted ?? true,
  };
}
