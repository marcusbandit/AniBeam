#!/usr/bin/env bash
# usage: smoke.sh <AniBeam.AppImage> <out-dir>
# Runs the AppImage the way a user gets it and leaves PNGs and logs in <out-dir>: the
# version, the home and settings pages on an empty sandbox root, and the two helper
# programs it carries. Needs xvfb and a Mesa that draws on the CPU.
#   SMOKE_CLI   the anibeam-cli to seed a library with. When set, the script also plays a
#               generated clip through libmpv (needs ffmpeg), the check that the picture
#               reaches the window.
#   SMOKE_DOWNLOADS   set to 1 to open the Downloads page on a fresh install: no anirss on
#               PATH, no qBittorrent running, so the image has to start its own of both.
# The host's Qt is scrubbed first, so a plugin the bundle forgot cannot hide behind it.
set -uo pipefail

unset QT_PLUGIN_PATH QML2_IMPORT_PATH QT_ROOT_DIR LD_LIBRARY_PATH QMAKE
mkdir -p "$2"
img="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
out="$(cd "$2" && pwd)"
root="$out/root"
XDG_RUNTIME_DIR="$(mktemp -d)"
export XDG_RUNTIME_DIR
chmod 700 "$XDG_RUNTIME_DIR"
export APPIMAGE_EXTRACT_AND_RUN=1 # a runner and a container have no FUSE
status=0

# A PNG, and one with something in it: an empty frame compresses to a few kilobytes.
is_picture() {
  [ "$(head -c 4 "$1" 2>/dev/null | tail -c 3)" = PNG ] && [ "$(wc -c < "$1")" -gt 20000 ]
}

echo "== the version"
"$img" --version || status=1

for page in home settings; do
  echo "== $page"
  QT_FORCE_STDERR_LOGGING=1 xvfb-run -a -s "-screen 0 1920x1080x24" \
    "$img" --root "$root" --shoot "$out/$page.png" --page "$page" --width 1600 --height 1000 \
    2> "$out/$page.log"
  code=$?
  if [ "$code" -ne 0 ] || ! is_picture "$out/$page.png"; then
    echo "$page failed (exit $code)"; tail -30 "$out/$page.log"; status=1
  else
    echo "$page ok"
  fi
done

echo "== the helpers"
# Straight out of the unpacked image: no AppRun, no environment, so each one has to stand
# on what the bundle holds.
helpers="$(mktemp -d)"
( cd "$helpers" && "$img" --appimage-extract > /dev/null ) || { echo "could not unpack the image"; exit 1; }
unpacked="$helpers/squashfs-root"
# A scratch HOME, since anirss writes a default config on its first run.
anirss="$(env -i HOME="$helpers" "$unpacked/usr/share/anibeam/anirss/bin/anirss" --version 2>/dev/null)"
echo "anirss: $anirss"
[ "$anirss" = "anirss 0.4.2" ] || { echo "anirss did not report 0.4.2"; status=1; }
qbt="$(env -i "$unpacked/usr/bin/qbittorrent-nox" --version 2>/dev/null)"
echo "qbittorrent-nox: $qbt"
case "$qbt" in
  "qBittorrent v5.2.4") ;;
  *) echo "qbittorrent-nox did not report 5.2.4"; status=1 ;;
esac
rm -rf "$helpers"

if [ -n "${SMOKE_CLI:-}" ]; then
  echo "== the player on a generated clip"
  cli="$(cd "$(dirname "$SMOKE_CLI")" && pwd)/$(basename "$SMOKE_CLI")"
  proot="$out/player-root"
  lib="$out/player-lib"
  mkdir -p "$lib/Mushishi"
  ffmpeg -nostdin -loglevel error -y \
    -f lavfi -i "testsrc2=size=1280x720:rate=24" -f lavfi -i "sine=frequency=330" \
    -t 15 -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest "$lib/Mushishi/Mushishi - 01.mkv" || status=1
  "$cli" --root "$proot" call AddSource --json "{\"path\": \"$lib\"}" > "$out/player-add.log" 2>&1 || status=1
  "$cli" --root "$proot" scan --wait > "$out/player-scan.log" 2>&1 || status=1
  series="$("$cli" --root "$proot" list | awk 'NR==1 {print $1}')"
  file_id="$("$cli" --root "$proot" show "$series" \
    | python3 -c 'import json,sys; d=json.load(sys.stdin); e=d["episodes"] or d["unmatched_files"]; print(e[0]["file"])')"
  echo "series=$series file=$file_id"
  # A runner draws on the CPU, so mpv gets the cheap settings; the point is that frames arrive.
  mkdir -p "$proot/config"
  printf 'log-file=%s/player-mpv.log\nmsg-level=all=v\nhwdec=no\nprofile=fast\n' "$out" > "$proot/config/mpv.conf"
  cat > "$out/player.json" <<JSON
[
  {"page": "player", "props": {"file": $file_id}},
  {"wait": 6000},
  {"eval": "frame.item.pageItem.loaded", "expect": true},
  {"eval": "frame.item.pageItem.duration > 10", "expect": true},
  {"shot": "playing"}
]
JSON
  QT_FORCE_STDERR_LOGGING=1 xvfb-run -a -s "-screen 0 1920x1080x24" \
    "$img" --root "$proot" --drive "$out/player.json" --out "$out/player" --width 1600 --height 1000 \
    2> "$out/player.log"
  code=$?
  grep -E 'DRIVE (EVAL|FAIL)' "$out/player.log" || true
  if [ "$code" -ne 0 ] || ! is_picture "$out/player/playing.png"; then
    echo "the player failed (exit $code)"; tail -30 "$out/player.log"; status=1
  else
    echo "player ok"
  fi
  grep -E 'VO:|Using|hwdec|vaapi|Failed|rror' "$out/player-mpv.log" 2>/dev/null | head -12 || true
fi

if [ -n "${SMOKE_DOWNLOADS:-}" ]; then
  echo "== the Downloads page on a fresh install"
  droot="$out/downloads-root"
  dhome="$(mktemp -d)"
  cat > "$out/downloads.json" <<'JSON'
[
  {"page": "downloads"},
  {"wait": 20000},
  {"shot": "downloads"},
  {"eval": "frame.item.pageItem.anirss.state", "expect": "Ready"},
  {"eval": "frame.item.pageItem.anirss.version", "expect": "0.4.2"},
  {"eval": "frame.item.pageItem.torrentOn", "expect": true},
  {"eval": "frame.item.pageItem.status.settings.torrent_mode", "expect": "Managed"}
]
JSON
  # A fresh HOME and a PATH that holds no anirss and no qbittorrent-nox: whatever answers
  # on the page came out of the image.
  QT_FORCE_STDERR_LOGGING=1 xvfb-run -a -s "-screen 0 1920x1080x24" \
    env -u XDG_CONFIG_HOME -u XDG_DATA_HOME -u XDG_CACHE_HOME HOME="$dhome" PATH=/usr/bin:/bin \
    "$img" --root "$droot" --drive "$out/downloads.json" --out "$out/downloads" --width 1600 --height 1000 \
    2> "$out/downloads.log"
  code=$?
  grep -E 'DRIVE (EVAL|FAIL)' "$out/downloads.log" || true
  if [ "$code" -ne 0 ] || ! is_picture "$out/downloads/downloads.png"; then
    echo "the Downloads page failed (exit $code)"; tail -40 "$out/downloads.log"; status=1
  else
    echo "downloads ok"
  fi
  # The managed qBittorrent is meant to outlive the app, so stop it here.
  pkill -f "[q]bittorrent-nox --profile" || true
  rm -rf "$dhome"
fi

ls -la "$out"
exit "$status"
