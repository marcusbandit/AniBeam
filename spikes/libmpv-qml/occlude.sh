#!/bin/bash
# occlude.sh: play on the active workspace, switch away for ~12 s, come back, quit. Counts drops while hidden.
export XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-1 DISPLAY=:1
export HYPRLAND_INSTANCE_SIGNATURE=$(ls /run/user/1000/hypr | head -1)
S=$HOME/spike-libmpv; OUT=$S/runs/occlude; rm -rf "$OUT"; mkdir -p "$OUT"
F=$S/media/gup03.mkv
HOME_WS=$(hyprctl monitors -j | jq -r '.[0].activeWorkspace.id')
AWAY_WS=${AWAY_WS:-2}
echo "=== occlude: home ws $HOME_WS, away ws $AWAY_WS"
(cd "$OUT" && env QSG_INFO=1 QT_LOGGING_TO_CONSOLE=1 QT_FORCE_STDERR_LOGGING=1 nohup "$S/build/mpvspike" "$F" --out="$OUT" --hwdec=auto > "$OUT/stdout.log" 2> "$OUT/stderr.log" &)
for i in $(seq 1 50); do sleep 0.2; hyprctl clients -j | jq -e '.[] | select(.class=="mpvspike")' >/dev/null 2>&1 && break; done
sleep 10
echo "t=10 switching to ws $AWAY_WS"; hyprctl dispatch "hl.dsp.focus({ workspace = $AWAY_WS })"
sleep 1; echo "now: $(hyprctl monitors -j | jq -c '.[0] | {ws: .activeWorkspace.id}')"
sleep 11
echo "t=22 back to ws $HOME_WS"; hyprctl dispatch "hl.dsp.focus({ workspace = $HOME_WS })"
sleep 1; echo "now: $(hyprctl monitors -j | jq -c '.[0] | {ws: .activeWorkspace.id}')"
sleep 10
echo "t=33 quitting"; pkill -x mpvspike; sleep 1
echo "=== drop events (player) ==="
grep '^SPIKE frame-drop-count' "$OUT/stdout.log" | sed 's/^SPIKE //' | while read -r tag json; do jq -r 'select(.who=="player") | "\(.t_ms) ms  drops=\(.value) at video t=\(."time-pos")"' <<<"$json"; done | tail -40
echo "=== stuck warnings: $(grep -c 'not being called' "$OUT/mpv-player.log")"
grep 'not being called' "$OUT/mpv-player.log" | sed -n '1p;$p'
grep -E "render loop" "$OUT/stderr.log"
