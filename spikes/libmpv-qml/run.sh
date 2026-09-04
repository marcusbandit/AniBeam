#!/bin/bash
# run.sh NAME [mpvspike args...]   env: QPA=wayland|xcb, WS=6 (target workspace), MON=DP-1
S=/tmp/claude-1000/-home-bandit-Projects-WebApps-AniBeam/ac4c58c2-a7af-415a-a976-7738705656c0/scratchpad
NAME=$1; shift
OUT=$S/runs/$NAME; rm -rf "$OUT"; mkdir -p "$OUT"
F='/mnt/wd_general/media/Anime/Girls und Panzer/Girls und Panzer/[ak-Submarines] Girls und Panzer - 03 [BD 1080p][6DDB5621].mkv'
WS=${WS:-6}; MON=${MON:-DP-1}; QPA=${QPA:-wayland}
SP=$(hyprctl monitors -j | jq -r ".[] | select(.name==\"$MON\") | .specialWorkspace.name")
if [ "$SP" = "special:communication" ]; then
  hyprctl --batch "dispatch hl.dsp.focus({ monitor = \"$MON\" }); dispatch hl.dsp.workspace.toggle_special(\"communication\"); dispatch hl.dsp.focus({ monitor = \"HDMI-A-1\" })" >/dev/null
fi
(cd "$OUT" && env QT_QPA_PLATFORM=$QPA QSG_INFO=1 QT_LOGGING_TO_CONSOLE=1 nohup "$S/spike/build/mpvspike" "$F" --out="$OUT" --script "$@" > "$OUT/stdout.log" 2> "$OUT/stderr.log" &)
T0=$(date +%s%N)
for i in $(seq 1 50); do sleep 0.2; hyprctl clients -j | jq -e '.[] | select(.class=="mpvspike")' >/dev/null 2>&1 && break; done
echo "mapped after $(( ($(date +%s%N)-T0)/1000000 )) ms on ws $(hyprctl clients -j | jq -c '.[] | select(.class=="mpvspike") | .workspace.id')"
hyprctl dispatch "hl.dsp.window.move({ workspace = $WS, silent = true, window = \"class:mpvspike\" })" >/dev/null
hyprctl dispatch 'hl.dsp.focus({ monitor = "HDMI-A-1" })' >/dev/null
( for i in $(seq 1 50); do echo "$(( ($(date +%s%N)-T0)/1000000 )) $(hyprctl clients -j | jq -c '.[] | select(.class=="mpvspike") | {ws: .workspace.id, monitor, floating, fullscreen, size, at}')"; sleep 1.5; done ) > "$OUT/hypr-poll.log" 2>&1 &
POLL=$!
sleep 26; grim -o $MON "$OUT/grim-part-a.png"
sleep 18; grim -o $MON "$OUT/grim-fullscreen.png"
for i in $(seq 1 45); do pgrep -x mpvspike >/dev/null || break; sleep 1; done
pgrep -x mpvspike >/dev/null && { echo "still running, killing"; pkill -x mpvspike; }
kill $POLL 2>/dev/null
hyprctl dispatch 'hl.dsp.focus({ monitor = "HDMI-A-1" })' >/dev/null
echo "=== focus now: $(hyprctl monitors -j | jq -c '[.[] | {name, focused}]')"
echo "=== hypr poll (dedup) ==="; awk '{ $1=""; print }' "$OUT/hypr-poll.log" | uniq -c
echo "=== stderr ==="; grep -vE '^\s*$' "$OUT/stderr.log" | head -30
echo "=== events ==="
grep '^SPIKE' "$OUT/stdout.log" | sed 's/^SPIKE //' | while read -r tag json; do
  case "$tag" in
    file-loaded) echo "$tag: $(jq -c '{fps: ."container-fps", chapters: [."chapter-list"[] | .title + "@" + (.time|tostring)], subs: [."track-list"[] | select(.type=="sub") | {id, codec, lang, selected}], video: [."track-list"[] | select(.type=="video") | {codec, "codec-profile", "demux-w", "demux-h"}], who}' <<<"$json")";;
    ready|chapter|vo-configured|video-reconfig|shot) ;;
    final|preview-final) echo "$tag: $(jq -c '{report_ms, props, t: ."time-pos", pause, hw: ."hwdec-current", hwfmt: ."video-params/hw-pixelformat", drops: ."frame-drop-count", decdrops: ."decoder-frame-drop-count", delayed: ."vo-delayed-frame-count", vffps: ."estimated-vf-fps", frame: ."estimated-frame-number", ch: .chapter, sub: ."sub-text", vo: ."current-vo", avsync, who}' <<<"$json")";;
    *) echo "$tag: $json";;
  esac
done
