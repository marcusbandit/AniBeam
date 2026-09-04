#!/bin/bash
# usage: shoot-main.sh <name> <preset> <workspace> [keep]   (keep: leave the window running)
# Launches a preset, moves the window silently to the given workspace of the main monitor,
# shows that workspace, and captures the window's own rectangle with grim into captures/ (or
# $OUT). Hyprland 0.56 Lua dispatch throughout; no windowrule, the running compositor rejects
# them. The caller switches the monitor back to the workspace it wants when it is done.
name="$1"; preset="$2"; ws="$3"; keep="$4"
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
  hyprctl dispatch "hl.dsp.window.move({ workspace = $ws, silent = true, window = \"class:anibeam-proto\" })" >/dev/null
  nap 0.3
  at=$(hyprctl clients -j | jq -r '[.[] | select(.class=="anibeam-proto")] | .[0].workspace.id')
  [ "$at" = "$ws" ] && break
done
hyprctl dispatch "hl.dsp.focus({ workspace = $ws })" >/dev/null
nap 2
geom=$(hyprctl clients -j | jq -r '[.[] | select(.class=="anibeam-proto")] | .[0] | "\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"')
grim -g "$geom" "$SB/$name.png" && echo "$name ok (ws $at, $geom)"
[ -z "$keep" ] && pkill -x anibeam-proto
true
