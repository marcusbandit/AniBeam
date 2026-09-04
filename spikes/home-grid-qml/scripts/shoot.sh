#!/bin/bash
# usage: shoot.sh <name> <preset> [keep]   (keep: leave the window running)
# Shows DP-1's workspace 6, launches a preset straight onto it, captures the output with grim
# into captures/ (or $OUT), and hands the focus back to the main monitor. The window is never
# moved: it tiles once, where it is captured.
name="$1"; preset="$2"; keep="$3"
SB="${OUT:-$(pwd)/captures}"; mkdir -p "$SB"
BIN="$(dirname "$0")/../target/release/anibeam-proto"
nap() { python3 -c "import time; time.sleep($1)"; }
pkill -x anibeam-proto 2>/dev/null; nap 0.4
hyprctl dispatch 'hl.dsp.focus({ workspace = 6 })' >/dev/null; nap 0.3
QT_FORCE_STDERR_LOGGING=1 QSG_RENDER_LOOP=threaded setsid nohup "$BIN" --preset "$preset" > "$SB/$name.log" 2>&1 &
for i in $(seq 1 60); do
  if hyprctl clients -j | jq -e '.[] | select(.class=="anibeam-proto" and .mapped==true)' >/dev/null 2>&1; then break; fi
  nap 0.2
done
hyprctl dispatch 'hl.dsp.focus({ monitor = "HDMI-A-1" })' >/dev/null
nap 2.5
ws=$(hyprctl clients -j | jq -r '[.[] | select(.class=="anibeam-proto")] | .[0].workspace.id')
grim -o DP-1 "$SB/$name.png" && echo "$name ok (ws $ws)"
[ -z "$keep" ] && pkill -x anibeam-proto
true
