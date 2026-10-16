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
  "$cli" --root "$root" call AddSource --json "{\"path\": \"$lib\"}" --wait > "$out/seed-add.log"
  "$cli" --root "$root" call AutoMatch --wait > "$out/seed-match.log" || true
  "$cli" --root "$root" list
  touch "$root/.seeded"
fi
[ -n "${SMOKE_SEED_ONLY:-}" ] && exit 0

echo "== the bundled app"
"$bin" --version

shoot() {
  local name="$1"; shift
  "$bin" --root "$root" --shoot "$out/$name.png" --width 1600 --height 1000 "$@" 2> "$out/$name.log" \
    || { echo "anibeam exited $? on $name"; tail -40 "$out/$name.log"; return 1; }
  file "$out/$name.png" | grep -q PNG || { echo "no PNG for $name"; tail -40 "$out/$name.log"; return 1; }
  echo "$name ok"
}
status=0
shoot home --page home || status=1
shoot library --page library || status=1
shoot series --page series || status=1
shoot settings --page settings || status=1

echo "== the frame smoke drive"
"$bin" --root "$root" --drive "$shell/scripts/drive/smoke.json" --out "$out/drive" \
  --width 1600 --height 1000 2> "$out/drive.log" || { echo "smoke drive failed"; tail -40 "$out/drive.log"; status=1; }

echo "== the player on a generated clip"
first_series="$("$cli" --root "$root" list | awk 'NR==1 {print $1}')"
file_id="$("$cli" --root "$root" show "$first_series" \
  | python3 -c 'import json,sys; d=json.load(sys.stdin); e=d["episodes"] or d["unmatched_files"]; print(e[0]["file"])')"
cat > "$out/player.json" <<JSON
[
  {"page": "player", "props": {"file": $file_id}},
  {"wait": 4000},
  {"eval": "frame.item.pageItem.loaded", "expect": true},
  {"eval": "frame.item.pageItem.duration > 10", "expect": true},
  {"shot": "player-playing"},
  {"hover": [800, 700]},
  {"wait": 400},
  {"shot": "player-chrome"}
]
JSON
"$bin" --root "$root" --drive "$out/player.json" --out "$out/drive" \
  --width 1600 --height 1000 2> "$out/player.log" || { echo "player drive failed"; tail -60 "$out/player.log"; status=1; }
ls -la "$out" "$out/drive" 2>/dev/null || true
exit "$status"
