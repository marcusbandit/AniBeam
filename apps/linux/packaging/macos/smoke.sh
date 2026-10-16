#!/usr/bin/env bash
# usage: smoke.sh <AniBeam.app> <anibeam-cli> <out-dir>
# Runs the bundled app the way a Mac user gets it and leaves PNGs in <out-dir>: a small
# library of generated clips, matched against AniList, then the home, library, series and
# settings pages, the frame smoke drive and the player on a real file through libmpv.
#   SMOKE_SEED_ONLY=1   only build the sandbox (run before Homebrew is hidden)
#   SMOKE_ROOT          the sandbox root (default: <out-dir>/root)
set -euo pipefail

mkdir -p "$3"
# Absolute, because AddSource refuses a relative source path.
app="$(cd "$1" && pwd)"
cli="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
out="$(cd "$3" && pwd)"
here="$(cd "$(dirname "$0")" && pwd)"
shell="$(cd "$here/../.." && pwd)"
root="${SMOKE_ROOT:-$out/root}"
lib="$(dirname "$root")/lib"
bin="$app/Contents/MacOS/anibeam"

# A run that dies on a signal prints nothing of its own; the crash report says where.
crashed() {
  echo "anibeam exited $1 on $2"
  tail -40 "$out/$2.log"
  sleep 3 # ReportCrash writes the .ips a moment after the process is gone
  python3 "$here/crash-report.py" anibeam
}

if [ ! -f "$root/.seeded" ]; then
  echo "== a library of generated clips"
  for show in "Sousou no Frieren" "Mushishi" "Cowboy Bebop"; do
    mkdir -p "$lib/$show"
    for ep in 01 02; do
      ffmpeg -nostdin -loglevel error -y \
        -f lavfi -i "testsrc2=size=1280x720:rate=24" -f lavfi -i "sine=frequency=330" \
        -t 15 -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest "$lib/$show/$show - $ep.mkv"
    done
  done
  "$cli" --root "$root" call AddSource --json "{\"path\": \"$lib\"}" > "$out/seed-add.log"
  "$cli" --root "$root" scan --wait > "$out/seed-scan.log"
  "$cli" --root "$root" call AutoMatch --wait > "$out/seed-match.log" || true
  "$cli" --root "$root" list
  touch "$root/.seeded"
fi
if [ -n "${SMOKE_SEED_ONLY:-}" ]; then
  # One page with Homebrew still in place, so a failure later can be told apart from
  # a library the bundle is missing. Informational only.
  echo "== probe with Homebrew visible"
  QT_FORCE_STDERR_LOGGING=1 "$bin" --root "$root" --shoot "$out/probe.png" --page home \
    --width 1600 --height 1000 2> "$out/probe.log" && echo "probe ok" || crashed $? probe || true
  exit 0
fi

echo "== the bundled app"
"$bin" --version

shoot() {
  local name="$1"; shift
  QT_FORCE_STDERR_LOGGING=1 "$bin" --root "$root" --shoot "$out/$name.png" --width 1600 --height 1000 "$@" \
    2> "$out/$name.log" || { crashed $? "$name"; return 1; }
  file "$out/$name.png" | grep -q PNG || { echo "no PNG for $name"; tail -40 "$out/$name.log"; return 1; }
  echo "$name ok"
}
status=0
shoot home --page home || status=1
shoot library --page library || status=1
shoot series --page series || status=1
shoot settings --page settings || status=1

echo "== the frame smoke drive"
# A runner's screen is about 1024x768, so macOS shrinks the window to fit; the one click
# the Linux script aims below that (a press outside a dropdown) goes to the rail instead.
sed 's/"click": \[1000, 950\]/"click": [44, 280]/' "$shell/scripts/drive/smoke.json" > "$out/smoke-mac.json"
QT_FORCE_STDERR_LOGGING=1 "$bin" --root "$root" --drive "$out/smoke-mac.json" --out "$out/drive" \
  --width 1600 --height 1000 2> "$out/drive.log" || { crashed $? drive; status=1; }

echo "== the player on a generated clip"
first_series="$("$cli" --root "$root" list | awk 'NR==1 {print $1}')"
file_id="$("$cli" --root "$root" show "$first_series" \
  | python3 -c 'import json,sys; d=json.load(sys.stdin); e=d["episodes"] or d["unmatched_files"]; print(e[0]["file"])')"
# Twice: as shipped, then with hardware decoding off, each with mpv's own log and a
# screencapture of the real screen beside the window grab, so a black picture can be told
# apart between the decoder, the renderer and the grab.
for run in player player-swdec; do
  {
    echo "log-file=$out/$run-mpv.log"
    echo "msg-level=all=v"
    if [ "$run" = player-swdec ]; then echo "hwdec=no"; fi
  } > "$root/config/mpv.conf"
  cat > "$out/$run.json" <<JSON
[
  {"page": "player", "props": {"file": $file_id}},
  {"wait": 4000},
  {"eval": "frame.item.pageItem.loaded", "expect": true},
  {"eval": "frame.item.pageItem.duration > 10", "expect": true},
  {"shot": "$run-playing"},
  {"hover": [500, 500]},
  {"wait": 400},
  {"shot": "$run-chrome"}
]
JSON
  ( sleep 4; screencapture -x "$out/$run-screen.png"; sleep 2; screencapture -x "$out/$run-screen-later.png" ) &
  QSG_INFO=1 QT_LOGGING_RULES="qt.scenegraph.renderloop=true;qt.qpa.window=true" \
    QT_FORCE_STDERR_LOGGING=1 "$bin" --root "$root" --drive "$out/$run.json" --out "$out/drive" \
    --width 1600 --height 1000 2> "$out/$run.log" || { crashed $? "$run"; status=1; }
  wait
  grep -E 'hwdec|Using|VO:|error|rror' "$out/$run-mpv.log" | head -20 || true
  grep -c 'not being called' "$out/$run-mpv.log" || true
  grep -i -E 'render loop|GL_|OpenGL|exposed|expose|renderWindow|occlu' "$out/$run.log" | sort | uniq -c | sort -rn | head -30 || true
done
rm -f "$root/config/mpv.conf"
ls -la "$out" "$out/drive" 2>/dev/null || true
exit "$status"
