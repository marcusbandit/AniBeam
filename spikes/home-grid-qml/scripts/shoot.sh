#!/bin/bash
# usage: shoot.sh <name> <preset> [keep]   (keep: leave the window running)
name="$1"; preset="$2"; keep="$3"
SB="${OUT:-$(pwd)/captures}"; mkdir -p "$SB"
BIN="$(dirname "$0")/../target/release/anibeam-proto"
nap() { python3 -c "import time; time.sleep($1)"; }
pkill -x anibeam-proto 2>/dev/null; nap 0.4
QT_FORCE_STDERR_LOGGING=1 QSG_RENDER_LOOP=threaded setsid nohup "$BIN" --preset "$preset" > "$SB/$name.log" 2>&1 &
for i in $(seq 1 60); do
  if hyprctl clients -j | jq -e '.[] | select(.class=="anibeam-proto" and .mapped==true)' >/dev/null 2>&1; then break; fi
  nap 0.2
done
for i in $(seq 1 10); do
  hyprctl dispatch 'hl.dsp.window.move({ workspace = 6, silent = true, window = "class:anibeam-proto" })' >/dev/null
  nap 0.3
  ws=$(hyprctl clients -j | jq -r '[.[] | select(.class=="anibeam-proto")] | .[0].workspace.id')
  [ "$ws" = "6" ] && break
done
hyprctl dispatch 'hl.dsp.focus({ monitor = "HDMI-A-1" })' >/dev/null
nap 2.5
ws=$(hyprctl clients -j | jq -r '[.[] | select(.class=="anibeam-proto")] | .[0].workspace.id')
grim -o DP-1 "$SB/$name.png" && echo "$name ok (ws $ws)"
[ -z "$keep" ] && pkill -x anibeam-proto
true
