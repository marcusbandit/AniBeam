#!/bin/bash
# quality.sh NAME FILE [mpvspike args...]   env: FULL=1 to run fullscreen, MON=eDP-1
export XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-1 DISPLAY=:1
export HYPRLAND_INSTANCE_SIGNATURE=$(ls /run/user/1000/hypr | head -1)
export QSG_RENDER_LOOP=threaded
S=$HOME/spike-libmpv
NAME=$1; F=$2; shift 2
OUT=$S/qruns/$NAME; rm -rf "$OUT"; mkdir -p "$OUT"
MON=${MON:-eDP-1}
GPU=/sys/class/drm/card1/device
HW=$(echo $GPU/hwmon/hwmon*)

echo "=== run $NAME  file=$(basename "$F")  args=$*"

# GPU sampler: epoch_ms busy% power_uW sclk_MHz temp_mC
( while :; do
    printf '%s %s %s %s %s\n' "$(date +%s%3N)" "$(cat $GPU/gpu_busy_percent)" \
      "$(cat $HW/power1_average 2>/dev/null || echo 0)" "$(cat $HW/freq1_input 2>/dev/null || echo 0)" \
      "$(cat $HW/temp1_input 2>/dev/null || echo 0)"
    sleep 0.25
  done ) > "$OUT/gpu.log" 2>/dev/null &
SAMP=$!

ENVV=(QT_QPA_PLATFORM=wayland QT_LOGGING_TO_CONSOLE=1 QT_FORCE_STDERR_LOGGING=1 QSG_RENDER_LOOP=threaded)
FULLARG=(); [ -n "$FULL" ] && FULLARG=(--fullscreen)
(cd "$OUT" && env "${ENVV[@]}" nohup "$S/build/mpvspike" "$F" --out="$OUT" "${FULLARG[@]}" "$@" > "$OUT/stdout.log" 2> "$OUT/stderr.log" &)

# marker watcher: grim on every still-hold, stamp the measurement window
( tail -n +1 -F "$OUT/stdout.log" 2>/dev/null | while read -r line; do
    case "$line" in
      *'SPIKE measure-begin'*) echo "$(date +%s%3N) measure-begin" >> "$OUT/marks.log" ;;
      *'SPIKE measure-end'*)   echo "$(date +%s%3N) measure-end"   >> "$OUT/marks.log" ;;
      *'SPIKE still-hold'*)
          n=$(sed 's/.*"n":\([0-9]*\).*/\1/' <<<"$line")
          grim -o $MON "$OUT/still-$n.png"
          echo "$(date +%s%3N) still-$n" >> "$OUT/marks.log" ;;
    esac
  done ) &
WATCH=$!

for i in $(seq 1 200); do pgrep -x mpvspike >/dev/null && break; sleep 0.2; done
for i in $(seq 1 200); do pgrep -x mpvspike >/dev/null || break; sleep 1; done
pgrep -x mpvspike >/dev/null && { echo "still running, killing"; pkill -x mpvspike; sleep 1; }
sleep 1
pkill -P $WATCH 2>/dev/null; kill $SAMP $WATCH 2>/dev/null; sleep 0.5

python3 "$S/summarise.py" "$OUT" | tee "$OUT/summary.txt"
