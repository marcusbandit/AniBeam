#!/usr/bin/env bash
# usage: drive.sh <script.json> [anibeam args...]     shots land in apps/linux/captures/
# Runs a --drive script under the offscreen platform: synthetic clicks, keys and wheel
# through the real event delivery, a PNG per {"shot": name} step. The step log streams to
# stdout and is kept in captures/<script>.drive.log; exits non-zero on DRIVE FAIL.
# ANIBEAM_ROOT sandboxes the run (default: an empty root under captures/root);
# ANIBEAM_CAPTURES moves the shots; W and H size the window.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
script="$(realpath "$1")"; shift
out="${ANIBEAM_CAPTURES:-$here/captures}"; mkdir -p "$out"
root="${ANIBEAM_ROOT:-$here/captures/root}"; mkdir -p "$root"
log="$out/$(basename "$script" .json).drive.log"
cargo build -p anibeam --quiet
# The RHI backend only with a DISPLAY to make its GL context against; see shoot.sh.
rhi_env=()
[ -n "${DISPLAY:-}" ] && rhi_env=(QT_QUICK_BACKEND=rhi QSG_RHI_BACKEND=opengl)
set +e
env "${rhi_env[@]}" QT_QPA_PLATFORM=offscreen QT_FORCE_STDERR_LOGGING=1 ANIBEAM_THEMES_DIR="$here/themes" ANIBEAM_MPV_CONF="$here/mpv.conf" \
  "$here/../../target/debug/anibeam" --root "$root" --drive "$script" --out "$out" --width "${W:-1600}" --height "${H:-1000}" "$@" \
  2> "$log"
code=$?
set -e
grep -E '^DRIVE ' "$log" || true
if [ $code -ne 0 ] || grep -q '^DRIVE FAIL' "$log"; then
  echo "drive failed (exit $code); full log: $log"; exit 1
fi
grep -q '^DRIVE DONE' "$log" || { echo "drive never finished; full log: $log"; exit 1; }
