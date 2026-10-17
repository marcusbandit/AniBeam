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

# The two helpers the Downloads page starts, run from inside the bundle with the same bare
# environment a Finder launch has: qbittorrent-nox and anirss on the Python that ships with
# it. Each is started for real, not just asked for its version, so a missing Qt plugin or a
# Python module left out of the tree shows up here and not on a Mac.
free_port() {
  /usr/bin/python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'
}
# serves <name> <log> <url> <command...>: runs the command in the background, waits up to
# 20 s for the URL to answer, prints what it said, then stops the process.
serves() {
  local name="$1" log="$2" url="$3"; shift 3
  "$@" > "$log" 2>&1 &
  local pid=$! answer="" i
  for i in $(seq 1 40); do
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
  clean=(env -i HOME="$state" TMPDIR="${TMPDIR:-/tmp}" PATH=/usr/bin:/bin:/usr/sbin:/sbin)
  echo "-- qbittorrent-nox"
  "${clean[@]}" "$app/Contents/MacOS/qbittorrent-nox" --version || return 1
  port="$(free_port)"
  serves qbittorrent-nox "$out/qbittorrent-nox.log" "http://127.0.0.1:$port/" \
    "${clean[@]}" "$app/Contents/MacOS/qbittorrent-nox" --confirm-legal-notice \
    --profile="$state/qbt" --webui-port="$port" || return 1
  echo "-- anirss"
  "${clean[@]}" "$app/Contents/Resources/anirss/bin/anirss" --version || return 1
  port="$(free_port)"
  serves anirss "$out/anirss.log" "http://127.0.0.1:$port/api/v1/health" \
    "${clean[@]}" "$app/Contents/Resources/anirss/bin/anirss" --serve --port "$port" || return 1
}
status=0
helpers || status=1

shoot() {
  local name="$1"; shift
  QT_FORCE_STDERR_LOGGING=1 "$bin" --root "$root" --shoot "$out/$name.png" --width 1600 --height 1000 "$@" \
    2> "$out/$name.log" || { crashed $? "$name"; return 1; }
  file "$out/$name.png" | grep -q PNG || { echo "no PNG for $name"; tail -40 "$out/$name.log"; return 1; }
  echo "$name ok"
}
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
# Twice, each with mpv's own log and a screencapture beside the window grab: as shipped,
# then as cheap as mpv renders. A runner is a VM whose only OpenGL is Apple's software
# renderer, on the CPU, and mpv's default scalers into an rgba16f FBO take longer there
# than mpv waits for a frame, so the shipped run plays with every frame dropped. The cheap
# run is what shows the picture really reaches the window; a Mac's GPU needs none of it.
for run in player player-cheap; do
  {
    echo "log-file=$out/$run-mpv.log"
    echo "msg-level=all=v"
    if [ "$run" = player-cheap ]; then
      printf '%s\n' hwdec=no profile=fast fbo-format=rgba8 dither=no deband=no
    fi
  } > "$root/config/mpv.conf"
  cat > "$out/$run.json" <<JSON
[
  {"page": "player", "props": {"file": $file_id}},
  {"wait": 7000},
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

echo "== the Downloads page on the bundled pair"
# A fresh install: its own root and HOME, launchd's PATH, no anirss from Homebrew and nothing
# listening on 8080. The core finds no qBittorrent, starts the bundled qbittorrent-nox on a
# loopback port, then the bundled anirss against it. The managed qBittorrent is meant to
# outlive the app, so it is killed here.
pkill -f "[q]bittorrent-nox --profile" || true  # one the pages above started in their root
fresh="$(mktemp -d)"
mkdir -p "$fresh/home" "$out/downloads-drive"
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
mkdir -p "$out/downloads-state"
# When each helper first answers, so a slow start can be told apart from one that never comes.
(
  conf="$fresh/root/data/qbittorrent/qBittorrent/config/qBittorrent.ini"
  for i in $(seq 1 90); do
    port="$(sed -n 's/^WebUI\\Port=//p' "$conf" 2>/dev/null || true)"
    if [ -n "$port" ]; then
      code="$(curl -s -o /dev/null --max-time 2 -w '%{http_code}' "http://127.0.0.1:$port/api/v2/app/version" || true)"
      echo "$(date +%T) qbt port $port api/v2/app/version -> $code  anirss: $(pgrep -f 'anirss' | tr '\n' ' ')"
    else
      echo "$(date +%T) no qbt config yet"
    fi
    sleep 1
  done
) > "$out/downloads-state/poll.log" 2>&1 &
poller=$!
env -i HOME="$fresh/home" TMPDIR="${TMPDIR:-/tmp}" PATH=/usr/bin:/bin:/usr/sbin:/sbin QT_FORCE_STDERR_LOGGING=1 \
  "$bin" --root "$fresh/root" --drive "$out/downloads.json" --out "$out/downloads-drive" \
  --width 1600 --height 1000 2> "$out/downloads.log" || { crashed $? downloads; status=1; }
grep -E 'DRIVE (EVAL|FAIL)' "$out/downloads.log" || true
kill "$poller" 2>/dev/null || true
echo "date at the end: $(date +%T)"
# What the managed pair left behind, for a run that did not come up.
cp "$fresh/root/data/qbittorrent/qbittorrent-nox.log" "$out/downloads-state/" 2>/dev/null || true
cp "$fresh/root/data/qbittorrent/qBittorrent/config/qBittorrent.ini" "$out/downloads-state/" 2>/dev/null || true
cp -R "$fresh/home/.config/anirss" "$fresh/home/.local/state/anirss" "$out/downloads-state/" 2>/dev/null || true
rm -f "$out/downloads-state/anirss/api.token"
ps ax | grep -E '[a]nirss|[q]bittorrent-nox' | cut -c1-200 || true
ps ax | grep '[q]bittorrent-nox --profile' || echo "no managed qbittorrent-nox is running"
pkill -f "[q]bittorrent-nox --profile" || true
ls -la "$out" "$out/drive" 2>/dev/null || true
exit "$status"
