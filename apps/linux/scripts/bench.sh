#!/usr/bin/env bash
# usage: bench.sh <name> <workspace> [keep] [anibeam args...]
# Launches the shell on the main monitor's workspace, captures the window's own rectangle
# with grim into apps/linux/captures/<name>.png, and closes it unless keep is given.
set -uo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
name="$1"; ws="$2"; keep="${3:-}"; shift 3 2>/dev/null || shift $#
out="$here/captures"; mkdir -p "$out"
nap() { python3 -c "import time; time.sleep($1)"; }
pkill -x anibeam 2>/dev/null; nap 0.4
hyprctl dispatch "hl.dsp.focus({ workspace = $ws })" >/dev/null; nap 0.3
QT_FORCE_STDERR_LOGGING=1 ANIBEAM_THEMES_DIR="$here/themes" ANIBEAM_MPV_CONF="$here/mpv.conf" \
  setsid nohup "$here/../../target/release/anibeam" "$@" > "$out/$name.log" 2>&1 &
for i in $(seq 1 60); do
  hyprctl clients -j | jq -e '.[] | select(.class=="com.marcusrosado.AniBeam" and .mapped==true)' >/dev/null 2>&1 && break
  nap 0.2
done
nap 2.5
geom=$(hyprctl clients -j | jq -r '[.[] | select(.class=="com.marcusrosado.AniBeam")] | .[0] | "\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"')
grim -g "$geom" "$out/$name.png" && echo "$name ok ($geom)"
[ "$keep" = keep ] || pkill -x anibeam
true
