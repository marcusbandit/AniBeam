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
# The build goes into the shared target dir (target-dir.sh) and is copied to this checkout's own
# target/debug/anibeam-staged, which is what runs. ANIBEAM_NO_BUILD=1 skips the build and reuses
# this checkout's last staged build: parallel reviewers capture without queueing on the cargo lock.
source "$(dirname "$0")/target-dir.sh"
# ANIBEAM_QML_DIR=<dir> (debug builds) loads the QML from disk. A relative dir is made
# absolute against the caller's directory; Qt's compile cache for it stays in the sandbox
# root, not ~/.cache.
if [ -n "${ANIBEAM_QML_DIR:-}" ]; then
  export ANIBEAM_QML_DIR="$(realpath -m "$ANIBEAM_QML_DIR")"
  export QML_DISK_CACHE_PATH="${QML_DISK_CACHE_PATH:-$root/qmlcache}"
fi
anibeam_stage
# The RHI backend only with a DISPLAY to make its GL context against; see shoot.sh.
rhi_env=()
[ -n "${DISPLAY:-}" ] && rhi_env=(QT_QUICK_BACKEND=rhi QSG_RHI_BACKEND=opengl)
set +e
env "${rhi_env[@]}" QT_QPA_PLATFORM=offscreen QT_FORCE_STDERR_LOGGING=1 ANIBEAM_THEMES_DIR="$here/themes" ANIBEAM_MPV_CONF="$here/mpv.conf" \
  "$ANIBEAM_BIN" --root "$root" --drive "$script" --out "$out" --width "${W:-1600}" --height "${H:-1000}" "$@" \
  2> "$log"
code=$?
set -e
grep -E '^DRIVE ' "$log" || true
if [ $code -ne 0 ] || grep -q '^DRIVE FAIL' "$log"; then
  echo "drive failed (exit $code); full log: $log"; exit 1
fi
grep -q '^DRIVE DONE' "$log" || { echo "drive never finished; full log: $log"; exit 1; }
