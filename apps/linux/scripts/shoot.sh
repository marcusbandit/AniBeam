#!/usr/bin/env bash
# usage: shoot.sh <name> [anibeam args...]     writes apps/linux/captures/<name>.png
# Renders one page under the offscreen platform and grabs it; no window lands anywhere.
# ANIBEAM_ROOT sandboxes the run (default: a copy-free empty root under captures/root).
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
name="$1"; shift
out="$here/captures"; mkdir -p "$out"
root="${ANIBEAM_ROOT:-$out/root}"; mkdir -p "$root"
cargo build -p anibeam --quiet
# The software scene graph cannot paint a Shapes fillItem, so every poster comes back white
# under plain offscreen; forcing the RHI backend fixes that, but RHI's GL backend needs GLX,
# which needs a DISPLAY. Without one, forcing it aborts (no context, no PNG) instead of
# falling back, so only force it when there is an X display to make the context against.
rhi_env=()
[ -n "${DISPLAY:-}" ] && rhi_env=(QT_QUICK_BACKEND=rhi QSG_RHI_BACKEND=opengl)
env "${rhi_env[@]}" QT_QPA_PLATFORM=offscreen QT_FORCE_STDERR_LOGGING=1 ANIBEAM_THEMES_DIR="$here/themes" ANIBEAM_MPV_CONF="$here/mpv.conf" \
  "$here/../../target/debug/anibeam" --root "$root" --shoot "$out/$name.png" --width "${W:-1600}" --height "${H:-1000}" "$@" \
  2> "$out/$name.log" || { echo "anibeam exited $?; see $out/$name.log"; exit 1; }
file "$out/$name.png" | grep -q PNG && echo "$name ok" || { echo "no PNG written; see $out/$name.log"; exit 1; }
