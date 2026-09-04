#!/bin/bash
# quality-nv.sh NAME FILE [mpvspike args...]   env: FULL=1 fullscreen, MON=HEADLESS-1, WS=11
# The desktop twin of spikes/libmpv-qml/quality.sh: launches mpvspike straight onto a workspace
# on a headless Hyprland output through hl.dsp.exec_cmd, samples the GPU with nvidia-smi, grabs
# that output with grim on every still-hold, and summarises with the spike's summarise.py.
S=${SPIKE:-$HOME/spike-libmpv}
NAME=$1; F=$2; shift 2
OUT=$S/qruns/$NAME; rm -rf "$OUT"; mkdir -p "$OUT"
MON=${MON:-HEADLESS-1}; WS=${WS:-11}
echo "=== run $NAME  file=$(basename "$F")  args=$*  full=${FULL:-0}"
python3 "$S/nvsample.py" "$OUT" & SAMP=$!
FULLARG=(); [ -n "$FULL" ] && FULLARG=(--fullscreen)
{ echo '#!/bin/bash'; echo "cd '$OUT'"
  echo 'export QT_QPA_PLATFORM=wayland QT_LOGGING_TO_CONSOLE=1 QT_FORCE_STDERR_LOGGING=1 QSG_RENDER_LOOP=threaded QSG_INFO=1'
  printf 'exec %q %q --out=%q' "$S/build/mpvspike" "$F" "$OUT"
  for a in "${FULLARG[@]}" "$@"; do printf ' %q' "$a"; done
  echo " > '$OUT/stdout.log' 2> '$OUT/stderr.log'"; } > "$OUT/cmd.sh"; chmod +x "$OUT/cmd.sh"
touch "$OUT/stdout.log"
( tail -n +1 -F "$OUT/stdout.log" 2>/dev/null | while read -r line; do
    case "$line" in
      *'SPIKE measure-begin'*) echo "$(date +%s%3N) measure-begin" >> "$OUT/marks.log" ;;
      *'SPIKE measure-end'*)   echo "$(date +%s%3N) measure-end"   >> "$OUT/marks.log" ;;
      *'SPIKE still-hold'*)
          n=$(sed 's/.*"n":\([0-9]*\).*/\1/' <<<"$line")
          grim -o "$MON" "$OUT/still-$n.png"
          echo "$(date +%s%3N) still-$n" >> "$OUT/marks.log" ;;
    esac
  done ) & WATCH=$!
hyprctl dispatch "hl.dsp.exec_cmd(\"$OUT/cmd.sh\", { workspace = \"$WS silent\" })" >/dev/null
for i in $(seq 1 100); do pgrep -x mpvspike >/dev/null && break; sleep 0.2; done
sleep 4; echo "window:     $(hyprctl clients -j | jq -c '.[] | select(.class=="mpvspike") | {ws: .workspace.id, monitor, fullscreen, size, at}')"
for i in $(seq 1 240); do pgrep -x mpvspike >/dev/null || break; sleep 1; done
pgrep -x mpvspike >/dev/null && { echo "still running, killing"; pkill -x mpvspike; sleep 1; }
sleep 1
pkill -P $WATCH 2>/dev/null; kill $SAMP $WATCH 2>/dev/null; sleep 0.5
python3 "$S/summarise.py" "$OUT" | tee "$OUT/summary.txt"
