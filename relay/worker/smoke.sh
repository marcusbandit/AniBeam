#!/usr/bin/env bash
# Runs the Worker locally under `wrangler dev` (local mode, no Cloudflare login)
# and drives it with smoke.mjs: rooms, a restart that must reload the room from
# Durable Object storage, the alarm that hands the host role over (about two
# minutes), and the creation limit.
#
#   ./smoke.sh                 rooms, reload, timers, limits
#   SMOKE_QUICK=1 ./smoke.sh   skip the two minute timer phase
#   SMOKE_CLOSE=1 ./smoke.sh   also wait out the 10 minute empty room close
#
# Needs node 22 or later, worker-build on PATH and the wasm32-unknown-unknown
# target. PORT picks the local port (default 8797).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
port="${PORT:-8797}"
base="http://127.0.0.1:${port}"
wrangler="${WRANGLER:-npx --yes wrangler@4.161.0}"
work="$(mktemp -d)"
pid=""

stop() {
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
    fi
    pid=""
}
trap 'stop; rm -rf "$work"' EXIT

start() {
    (cd "$here" && exec $wrangler dev --local --ip 127.0.0.1 --port "$port" \
        --persist-to "$work/state" --show-interactive-dev-session=false) \
        >>"$work/wrangler.log" 2>&1 &
    pid=$!
    for _ in $(seq 1 300); do
        if curl -fsS "$base/health" >/dev/null 2>&1; then
            return 0
        fi
        if ! kill -0 "$pid" 2>/dev/null; then
            break
        fi
        sleep 1
    done
    echo "wrangler dev did not come up; its log:" >&2
    cat "$work/wrangler.log" >&2
    return 1
}

state="$work/smoke-state.json"
start
node "$here/smoke.mjs" "$base" rooms "$state"
stop
start
node "$here/smoke.mjs" "$base" reload "$state"
if [[ -z "${SMOKE_QUICK:-}" ]]; then
    node "$here/smoke.mjs" "$base" timers
fi
node "$here/smoke.mjs" "$base" limits
if [[ -n "${SMOKE_CLOSE:-}" ]]; then
    closed="$work/smoke-closed.json"
    node "$here/smoke.mjs" "$base" close "$closed"
    stop
    start
    node "$here/smoke.mjs" "$base" gone "$closed"
fi
echo "smoke: all phases passed"
