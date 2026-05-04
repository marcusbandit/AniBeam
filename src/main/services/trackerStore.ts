import { readFile, writeFile, mkdir } from 'fs/promises';
import { join } from 'path';
import { app, safeStorage } from 'electron';
import { logger } from './logger';

export type TrackerProvider = 'anilist' | 'mal';

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
  version: 1;
}

const DEFAULT_STORE: TrackerStore = {
  anilist: null,
  mal: null,
  clientIds: { anilist: '', mal: '' },
  clientSecretCiphers: { anilist: '', mal: '' },
  clientSecretsEncrypted: true,
  version: 1,
};

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
    const parsed = JSON.parse(raw) as Partial<TrackerStore>;
    cache = {
      ...DEFAULT_STORE,
      ...parsed,
      clientIds: { ...DEFAULT_STORE.clientIds, ...(parsed.clientIds ?? {}) },
      clientSecretCiphers: { ...DEFAULT_STORE.clientSecretCiphers, ...(parsed.clientSecretCiphers ?? {}) },
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
