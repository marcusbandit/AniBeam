#!/usr/bin/env bash
# usage: smoke.sh <AniBeam-dir> <anibeam-cli.exe> <out-dir>
# Runs the bundled app the way a Windows user gets it and leaves PNGs in <out-dir>: a small
# library of generated clips, matched against AniList, then the home, library, series and
# settings pages, the player on a real file through libmpv, and the Downloads page starting
# the bundled qbittorrent-nox and anirss on a fresh install. The Windows counterpart of the
# Mac's smoke.sh. A runner has no GPU, so Qt draws through the opengl32sw.dll windeployqt
# ships; mpv runs with its cheap settings there for the same reason.
set -euo pipefail
export MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*'

mkdir -p "$3"
app="$(cd "$1" && pwd)"
cli="$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"
out="$(cd "$3" && pwd)"
here="$(cd "$(dirname "$0")" && pwd)"
shell="$(cd "$here/../.." && pwd)"
root="$out/root"
lib="$out/lib"
bin="$app/anibeam.exe"
win() { cygpath -m "$1"; }

status=0

echo "== a library of generated clips"
for show in "Sousou no Frieren" "Mushishi" "Cowboy Bebop"; do
  mkdir -p "$lib/$show"
  for ep in 01 02; do
    ffmpeg -nostdin -loglevel error -y \
      -f lavfi -i "testsrc2=size=1280x720:rate=24" -f lavfi -i "sine=frequency=330" \
      -t 15 -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest "$(win "$lib/$show/$show - $ep.mkv")"
  done
done
"$cli" --root "$(win "$root")" call AddSource --json "{\"path\": \"$(win "$lib")\"}" > "$out/seed-add.log"
"$cli" --root "$(win "$root")" scan --wait > "$out/seed-scan.log"
"$cli" --root "$(win "$root")" call AutoMatch --wait > "$out/seed-match.log" || true
"$cli" --root "$(win "$root")" list | tee "$out/seed-list.log"

echo "== the two helpers, started for real"
free_port() {
  python -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'
}
# serves <name> <log> <url> <command...>: runs the command in the background, waits up to
# 30 s for the URL to answer, prints what it said, then stops the process.
serves() {
  local name="$1" log="$2" url="$3"; shift 3
  "$@" > "$log" 2>&1 &
  local pid=$! answer="" i
  for i in $(seq 1 60); do
    answer="$(curl -s --max-time 2 -w ' [HTTP %{http_code}]' "$url" || true)"
    case "$answer" in *"[HTTP 000]"*|"") ;; *) break ;; esac
    kill -0 "$pid" 2>/dev/null || break
    sleep 0.5
  done
  kill "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  case "$answer" in *"[HTTP 000]"*|"") echo "$name did not answer on $url"; tail -30 "$log"; return 1 ;; esac
  echo "$name answered: ${answer:0:200}"
}
helpers() {
  local state port
  state="$(mktemp -d)"
  echo "-- qbittorrent-nox"
  port="$(free_port)"
  serves qbittorrent-nox "$out/qbittorrent-nox.log" "http://127.0.0.1:$port/" \
    "$app/qbittorrent-nox.exe" --confirm-legal-notice \
    "--profile=$(win "$state/qbt")" "--webui-port=$port" || return 1
  taskkill /F /IM qbittorrent-nox.exe > /dev/null 2>&1 || true
  echo "-- anirss"
  "$app/anirss/python/python.exe" "$(win "$app/anirss/app/anirss")" --version || return 1
  port="$(free_port)"
  serves anirss "$out/anirss.log" "http://127.0.0.1:$port/api/v1/health" \
    env HOME="$(win "$state")" USERPROFILE="$(win "$state")" \
    "$app/anirss/python/python.exe" "$(win "$app/anirss/app/anirss")" --serve --port "$port" || return 1
}
helpers || status=1

crashed() {
  echo "anibeam exited $1 on $2"
  tail -40 "$out/$2.log"
}
shoot() {
  local name="$1"; shift
  QT_FORCE_STDERR_LOGGING=1 "$bin" --root "$(win "$root")" --shoot "$(win "$out/$name.png")" \
    --width 1600 --height 1000 "$@" 2> "$out/$name.log" || { crashed $? "$name"; return 1; }
  head -c 8 "$out/$name.png" | grep -q PNG || { echo "no PNG for $name"; tail -40 "$out/$name.log"; return 1; }
  echo "$name ok"
}
echo "== the pages"
shoot home --page home || status=1
shoot library --page library || status=1
shoot series --page series || status=1
shoot settings --page settings || status=1

echo "== the player on a generated clip"
first_series="$("$cli" --root "$(win "$root")" list | awk 'NR==1 {print $1}')"
file_id="$("$cli" --root "$(win "$root")" show "$first_series" \
  | python -c 'import json,sys; d=json.load(sys.stdin); e=d["episodes"] or d["unmatched_files"]; print(e[0]["file"])')"
{
  echo "log-file=$(win "$out/player-mpv.log")"
  echo "msg-level=all=v"
  printf '%s\n' hwdec=no profile=fast fbo-format=rgba8 dither=no deband=no
} > "$root/config/mpv.conf"
mkdir -p "$out/drive"
cat > "$out/player.json" <<JSON
[
  {"page": "player", "props": {"file": $file_id}},
  {"wait": 8000},
  {"eval": "frame.item.pageItem.loaded", "expect": true},
  {"eval": "frame.item.pageItem.duration > 10", "expect": true},
  {"shot": "player-playing"},
  {"hover": [500, 500]},
  {"wait": 400},
  {"shot": "player-chrome"}
]
JSON
QT_FORCE_STDERR_LOGGING=1 "$bin" --root "$(win "$root")" --drive "$(win "$out/player.json")" \
  --out "$(win "$out/drive")" --width 1600 --height 1000 2> "$out/player.log" || { crashed $? player; status=1; }
grep -E 'DRIVE (EVAL|FAIL)' "$out/player.log" || true
grep -E 'Using|VO:|error|rror' "$out/player-mpv.log" | head -20 || true
rm -f "$root/config/mpv.conf"

echo "== the Downloads page on the bundled pair"
# A fresh install: its own root and profile, nothing listening on 8080. The core finds no
# qBittorrent, starts the bundled qbittorrent-nox on a loopback port, then the bundled anirss
# against it. The managed qBittorrent is meant to outlive the app, so it is killed here.
taskkill /F /IM qbittorrent-nox.exe > /dev/null 2>&1 || true
fresh="$(mktemp -d)"
mkdir -p "$fresh/home" "$out/downloads-drive"
cat > "$out/downloads.json" <<'JSON'
[
  {"page": "downloads"},
  {"wait": 25000},
  {"shot": "downloads"},
  {"eval": "frame.item.pageItem.anirss.state", "expect": "Ready"},
  {"eval": "frame.item.pageItem.anirss.version", "expect": "0.6.1"},
  {"eval": "frame.item.pageItem.torrentOn", "expect": true},
  {"eval": "frame.item.pageItem.status.settings.torrent_mode", "expect": "Managed"}
]
JSON
USERPROFILE="$(win "$fresh/home")" HOME="$(win "$fresh/home")" QT_FORCE_STDERR_LOGGING=1 \
  "$bin" --root "$(win "$fresh/root")" --drive "$(win "$out/downloads.json")" \
  --out "$(win "$out/downloads-drive")" --width 1600 --height 1000 2> "$out/downloads.log" \
  || { crashed $? downloads; status=1; }
if grep -q 'DRIVE FAIL' "$out/downloads.log"; then status=1; fi
grep -E 'DRIVE (EVAL|FAIL)' "$out/downloads.log" || true
mkdir -p "$out/downloads-state"
cp "$fresh/root/data/qbittorrent/qbittorrent-nox.log" "$out/downloads-state/" 2>/dev/null || true
cp "$fresh/root/data/qbittorrent/qBittorrent/config/qBittorrent.ini" "$out/downloads-state/" 2>/dev/null || true
tasklist | grep -i -E 'anibeam|qbittorrent|python' || true
taskkill /F /IM qbittorrent-nox.exe > /dev/null 2>&1 || true
ls -la "$out" "$out/drive" 2>/dev/null || true
exit "$status"
