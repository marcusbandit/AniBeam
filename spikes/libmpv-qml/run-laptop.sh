#!/bin/bash
# run-laptop.sh NAME [mpvspike args...]   env: QPA=wayland|xcb, MON=eDP-1, RENDER_LOOP=basic|threaded
export XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-1 DISPLAY=:1
export HYPRLAND_INSTANCE_SIGNATURE=$(ls /run/user/1000/hypr | head -1)
S=$HOME/spike-libmpv
NAME=$1; shift
OUT=$S/runs/$NAME; rm -rf "$OUT"; mkdir -p "$OUT"
F=$S/media/gup03.mkv
MON=${MON:-eDP-1}; QPA=${QPA:-wayland}
ENV=(QT_QPA_PLATFORM=$QPA QSG_INFO=1 QT_LOGGING_TO_CONSOLE=1 QT_FORCE_STDERR_LOGGING=1)
[ -n "$RENDER_LOOP" ] && ENV+=(QSG_RENDER_LOOP=$RENDER_LOOP)
echo "=== run $NAME: QPA=$QPA RENDER_LOOP=${RENDER_LOOP:-default} args=$*"
echo "=== before: $(hyprctl monitors -j | jq -c '.[] | {name, dpms: .dpmsStatus, ws: .activeWorkspace.name, special: .specialWorkspace.name, focused}')"
(cd "$OUT" && env "${ENV[@]}" nohup "$S/build/mpvspike" "$F" --out="$OUT" --script "$@" > "$OUT/stdout.log" 2> "$OUT/stderr.log" &)
T0=$(date +%s%N)
for i in $(seq 1 50); do sleep 0.2; hyprctl clients -j | jq -e '.[] | select(.class=="mpvspike")' >/dev/null 2>&1 && break; done
echo "mapped after $(( ($(date +%s%N)-T0)/1000000 )) ms: $(hyprctl clients -j | jq -c '.[] | select(.class=="mpvspike") | {ws: .workspace.id, monitor, floating, fullscreen, size, at}')"
( for i in $(seq 1 50); do echo "$(( ($(date +%s%N)-T0)/1000000 )) $(hyprctl clients -j | jq -c '.[] | select(.class=="mpvspike") | {ws: .workspace.id, monitor, floating, fullscreen, size, at}')"; sleep 1.5; done ) > "$OUT/hypr-poll.log" 2>&1 &
POLL=$!
sleep 14; grim -o $MON "$OUT/grim-op.png"
sleep 13; grim -o $MON "$OUT/grim-part-a.png"
sleep 17; grim -o $MON "$OUT/grim-fullscreen.png"
for i in $(seq 1 45); do pgrep -x mpvspike >/dev/null || break; sleep 1; done
pgrep -x mpvspike >/dev/null && { echo "still running, killing"; pkill -x mpvspike; }
kill $POLL 2>/dev/null
echo "=== hypr poll (dedup) ==="; awk '{ $1=""; print }' "$OUT/hypr-poll.log" | uniq -c
echo "=== stderr ==="; grep -vE '^\s*$' "$OUT/stderr.log" | head -40
echo "=== events ==="
grep '^SPIKE' "$OUT/stdout.log" | sed 's/^SPIKE //' | while read -r tag json; do
  case "$tag" in
    file-loaded) echo "$tag: $(jq -c '{fps: ."container-fps", chapters: [."chapter-list"[] | .title + "@" + (.time|tostring)], subs: [."track-list"[] | select(.type=="sub") | {id, codec, lang, selected}], video: [."track-list"[] | select(.type=="video") | {codec, "codec-profile", "demux-w", "demux-h"}], vp: ."video-params", who}' <<<"$json")";;
    ready|chapter|vo-configured|video-reconfig|shot) ;;
    final|preview-final) echo "$tag: $(jq -c '{report_ms, props, t: ."time-pos", pause, hw: ."hwdec-current", interop: ."hwdec-interop", pixfmt: ."video-params/pixelformat", hwfmt: ."video-params/hw-pixelformat", drops: ."frame-drop-count", decdrops: ."decoder-frame-drop-count", delayed: ."vo-delayed-frame-count", mistimed: ."mistimed-frame-count", vffps: ."estimated-vf-fps", dfps: ."display-fps", frame: ."estimated-frame-number", ch: .chapter, sub: ."sub-text", vo: ."current-vo", gpuctx: ."gpu-context", gpuapi: ."gpu-api", avsync, sync: ."video-sync", who}' <<<"$json")";;
    *) echo "$tag: $json";;
  esac
done
echo "=== mpv log highlights ==="
grep -E -i "hwdec|vaapi|libva|VA-API|radeonsi|GL_VERSION|GLSL|version 3|es\b|fbo|drop|not being called|libass|fontconfig|Using .* font|screenshot|Error|error" "$OUT/mpv-player.log" | grep -v -E "^\[cplayer\] +(Command|Set property|Run command)" | head -60
