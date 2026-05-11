import { spawn } from 'child_process';
import { homedir } from 'os';
import { logger } from '../services/logger';

export interface SubscriptionFeed {
  name: string;
  feedUrl: string;
  savePath: string;
  ruleEnabled: boolean;
  torrentCount: number;
}

export type SubscriptionsResult =
  | { ok: true; items: SubscriptionFeed[] }
  | { ok: false; error: string; needsAuth?: boolean };

const ANIRSS_TIMEOUT_MS = 15_000;

// Keep `~/.local/bin` reachable when the app is launched from a .desktop
// entry whose PATH is sparse. Belt and braces: include both the user's
// installed-binaries dir and whatever PATH inherited from systemd/dex.
function spawnEnv(): NodeJS.ProcessEnv {
  const userBin = `${homedir()}/.local/bin`;
  const path = process.env.PATH ?? '';
  const merged = path.split(':').includes(userBin) ? path : `${userBin}:${path}`;
  return { ...process.env, PATH: merged };
}

interface AnirssJsonItem {
  name?: unknown;
  feed_url?: unknown;
  save_path?: unknown;
  rule_enabled?: unknown;
  torrent_count?: unknown;
}

function coerce(item: AnirssJsonItem): SubscriptionFeed | null {
  if (typeof item.name !== 'string' || !item.name) return null;
  return {
    name: item.name,
    feedUrl: typeof item.feed_url === 'string' ? item.feed_url : '',
    savePath: typeof item.save_path === 'string' ? item.save_path : '',
    ruleEnabled: item.rule_enabled !== false,
    torrentCount: typeof item.torrent_count === 'number' ? item.torrent_count : 0,
  };
}

// Closing stdin makes anirss's getpass.getpass() raise EOFError, which it
// turns into `die("cancelled")` (exit 1). That's how we detect "needs auth"
// — the cached qbt.sid was missing or dead and there's no terminal to
// prompt on. The user fixes it with `anirss -Sy` once.
export async function listSubscriptions(): Promise<SubscriptionsResult> {
  return new Promise((resolve) => {
    const child = spawn('anirss', ['-Qj'], {
      env: spawnEnv(),
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';
    let settled = false;

    const finish = (result: SubscriptionsResult) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      try { child.kill('SIGTERM'); } catch { /* already exited */ }
      resolve(result);
    };

    const timer = setTimeout(() => {
      logger.warn('system', `anirss -Qj timed out after ${ANIRSS_TIMEOUT_MS}ms`);
      finish({ ok: false, error: 'anirss timed out — is qBittorrent reachable?' });
    }, ANIRSS_TIMEOUT_MS);

    child.stdout.on('data', (chunk: Buffer) => { stdout += chunk.toString('utf8'); });
    child.stderr.on('data', (chunk: Buffer) => { stderr += chunk.toString('utf8'); });

    child.on('error', (err) => {
      const msg = (err as NodeJS.ErrnoException).code === 'ENOENT'
        ? 'anirss is not installed or not on PATH'
        : `failed to launch anirss: ${(err as Error).message}`;
      logger.error('system', msg);
      finish({ ok: false, error: msg });
    });

    child.on('close', (code) => {
      if (code === 0) {
        try {
          const parsed = JSON.parse(stdout) as AnirssJsonItem[];
          if (!Array.isArray(parsed)) {
            finish({ ok: false, error: 'anirss returned non-array JSON' });
            return;
          }
          const items = parsed.map(coerce).filter((x): x is SubscriptionFeed => x !== null);
          finish({ ok: true, items });
          return;
        } catch (err) {
          finish({ ok: false, error: `couldn't parse anirss output: ${(err as Error).message}` });
          return;
        }
      }
      // Strip ANSI; the password-prompt path writes red "error: cancelled"
      // to stderr. Detecting it lets us point the user at `anirss -Sy`.
      const cleaned = stderr.replace(/\[[0-9;]*m/g, '').toLowerCase();
      const needsAuth = cleaned.includes('cancelled') || cleaned.includes("can't reach qbittorrent");
      finish({
        ok: false,
        error: needsAuth
          ? 'No qBittorrent session — run `anirss -Sy` in a terminal once to authenticate.'
          : `anirss exited with code ${code}: ${stderr.trim() || 'no output'}`,
        needsAuth,
      });
    });
  });
}

export const subscriptionsHandler = {
  list: listSubscriptions,
};

export default subscriptionsHandler;
