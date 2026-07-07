// Tail the AniBeam subtitle debug log, following across the session rotation
// the app performs on startup (subtitles.log -> subtitles.prev.log). Works
// against the packaged .desktop-launched build: the log lives under Electron's
// userData dir, which resolves from package.json "name" (no productName), so
// the expected location is <config-home>/anibeam/logs/subtitles.log.
import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const configHome = process.env.XDG_CONFIG_HOME || join(homedir(), '.config');
const candidates = ['anibeam', 'AniBeam'].map((dir) => join(configHome, dir, 'logs', 'subtitles.log'));
const target = candidates.find((p) => existsSync(p)) ?? candidates[0];

if (!existsSync(target)) {
  console.log(`No log file yet at ${target}`);
  console.log('Waiting for the app to write one (start AniBeam and play something with subs)...');
}
console.log(`Tailing ${target}\n`);

// -F (not -f) retries on rotation/creation, so this survives app restarts.
const tail = spawn('tail', ['-n', '100', '-F', target], { stdio: 'inherit' });
tail.on('close', (code) => process.exit(code ?? 0));
