'use strict';

const { spawn } = require('child_process');
const { mkdir } = require('fs/promises');
const path = require('path');

// Drop-in replacement for extract-zip. Upstream stalls forever on Node 26 due
// to an fd-slicer + zlib.createInflateRaw backpressure deadlock; this version
// shells out to /usr/bin/unzip which is unaffected. The only consumer in this
// project is @electron/packager extracting the prebuilt Electron zip.

async function extract(zipPath, opts) {
  const dir = path.resolve(opts && opts.dir);
  if (!dir) throw new Error('extract-zip-shim: opts.dir is required');
  await mkdir(dir, { recursive: true });

  return new Promise((resolve, reject) => {
    const proc = spawn('unzip', ['-oq', zipPath, '-d', dir], { stdio: ['ignore', 'pipe', 'pipe'] });
    let stderr = '';
    proc.stderr.on('data', d => { stderr += d.toString(); });
    proc.on('error', reject);
    proc.on('close', code => {
      if (code === 0) {
        if (opts && typeof opts.onEntry === 'function') {
          // Best-effort: upstream callers in @electron/packager don't rely on
          // onEntry, so we skip per-entry callbacks rather than re-walk the dir.
        }
        resolve();
      } else {
        reject(new Error(`unzip exited ${code}${stderr ? ': ' + stderr.trim() : ''}`));
      }
    });
  });
}

module.exports = extract;
module.exports.default = extract;
