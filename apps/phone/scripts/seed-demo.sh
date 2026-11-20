#!/usr/bin/env bash
# Dev only: seed the emulator with a realistic library, calendar and downloads for screenshots.
# Usage: scripts/seed-demo.sh
#
# Picks about eight TV shows that are airing this week from AniList, writes the app's seed list
# (empty library files, read at core open by src/local.rs seed) and a torrents.json for the
# downloads screen, then pushes both to the device. Relaunch the app after running this.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh

STAGE=build/demo
SEED_DEVICE=/data/local/tmp/anibeam-seed
DEMO_DEVICE=/data/local/tmp/anibeam-demo
mkdir -p "$STAGE"

for tool in curl jq; do
  command -v "$tool" >/dev/null || { echo "seed-demo.sh: $tool is not installed" >&2; exit 1; }
done

now="$(date +%s)"
week="$((now + 7 * 86400))"

# Step 1: releasing TV anime, most popular first. AniList cannot filter on the next airing
# time in this query, so the window is applied to the result. One weekday per show first, so
# the calendar has spread; the rest of the eight fill in by popularity.
query='query { Page(page: 1, perPage: 50) { media(type: ANIME, format: TV, status: RELEASING, sort: POPULARITY_DESC) { id title { romaji english } nextAiringEpisode { airingAt episode } } } }'
body="$(jq -cn --arg q "$query" '{query: $q}')"
raw="$(curl -fsS -X POST https://graphql.anilist.co \
  -H 'Content-Type: application/json' -H 'Accept: application/json' -d "$body")"

shows="$(printf '%s' "$raw" | jq -c --argjson now "$now" --argjson week "$week" '
  [ .data.Page.media[]
    | select(.nextAiringEpisode != null)
    | select(.nextAiringEpisode.airingAt >= $now and .nextAiringEpisode.airingAt <= $week)
    | select(.nextAiringEpisode.episode >= 2)
    | {
        id,
        romaji: .title.romaji,
        english: (.title.english // ""),
        next: .nextAiringEpisode.episode,
        airingAt: .nextAiringEpisode.airingAt,
        weekday: (.nextAiringEpisode.airingAt | strftime("%a"))
      }
  ] as $all
  | (reduce $all[] as $m ({picked: [], days: []};
      if (.picked | length) < 8 and ((.days | index($m.weekday)) == null)
      then .picked += [$m] | .days += [$m.weekday]
      else . end) | .picked) as $spread
  | ($spread + ($all - $spread)) | .[0:8]
')"

count="$(printf '%s' "$shows" | jq 'length')"
if [ "$count" -eq 0 ]; then
  echo "seed-demo.sh: AniList returned no releasing TV shows airing in the next 7 days" >&2
  exit 1
fi

# Folder is the English title, romaji when there is none, with filesystem-unsafe characters
# removed. Adds the folder name to each show.
shows="$(printf '%s' "$shows" | jq -c '
  map(. + {folder: (((if .english != "" then .english else .romaji end)
    | gsub("[\\\\/:*?\"<>|]"; "") | gsub("^ +| +$"; "")))})')"

echo "Shows airing in the next 7 days (UTC):"
printf '%s' "$shows" | jq -r '.[] | "  \(.airingAt | strftime("%Y-%m-%d %a %H:%M")) ep \(.next)  \(.folder)"'

# Seed list: the app creates each line as an empty file under its library folder. Episodes
# 1 to next-1, at most three per show.
: > "$STAGE/anibeam-seed"
printf '%s' "$shows" | jq -r '.[] as $s
  | [range(1; ([$s.next - 1, 3] | min) + 1)][]
  | "\($s.folder)/[Demo] \($s.folder) - \(if . < 10 then "0" + (tostring) else tostring end) (1080p).mkv"' \
  >> "$STAGE/anibeam-seed"
echo "Seed list: $(wc -l < "$STAGE/anibeam-seed") files in $STAGE/anibeam-seed"

# Step 4: torrents for the downloads screen, built from the first two shows. Hashes are made up
# and tracker fields stay empty.
a="$(printf '%s' "$shows" | jq -c '.[0]')"
b="$(printf '%s' "$shows" | jq -c '.[1] // .[0]')"
torrents="$(jq -n --argjson a "$a" --argjson b "$b" --argjson now "$now" '
  def torrent($hash; $name; $state; $progress; $size; $dl; $eta; $episode; $batch; $files; $done):
    {
      hash: $hash, name: $name, state: $state, progress: $progress, size: $size,
      dlspeed: $dl, upspeed: 0, eta: $eta, num_seeds: 42, num_leechs: 3, ratio: 0,
      save_path: "/storage/emulated/0/Download/anibeam", content_path: "/storage/emulated/0/Download/anibeam/\($name)",
      added_on: ($now - 86400), completion_on: $done, tags: "", category: "anibeam",
      series_id: null, episode: $episode, batch: $batch, files: $files,
      seq_dl: false, f_l_piece_prio: false, tracker: "", group: "Demo",
      resolution: "1080p", provider: "", roll: null
    };
  def ep($n): if $n < 10 then "0\($n)" else "\($n)" end;
  def file($name; $size; $progress; $episode):
    { name: $name, size: $size, progress: $progress, priority: 1, episode: $episode };
  ($a.next - 1 | if . < 1 then 1 else . end) as $ea
  | [
      torrent("1a2b3c4d5e6f708192a3b4c5d6e7f80910111201";
        "[Demo] \($a.folder) - \(ep($ea)) (1080p) [ABCD1234].mkv";
        "downloading"; 0.42; 1450000000; 3400000; 380; $ea; false; []; -1),
      torrent("1a2b3c4d5e6f708192a3b4c5d6e7f80910111202";
        "[Demo] \($b.folder) - 01-06 (1080p) [BATCH]";
        "downloading"; 0.51; 8700000000; 5200000; 900; null; true;
        [ range(1; 7) as $i
          | file("[Demo] \($b.folder) - \(ep($i)) (1080p) [EF01AB2\($i)].mkv";
              1100000000;
              ([1.0, 1.0, 0.73, 0.2, 0.0, 0.0][$i - 1]);
              $i) ];
        -1),
      torrent("1a2b3c4d5e6f708192a3b4c5d6e7f80910111203";
        "[Demo] \($b.folder) - 01 (1080p) [CD56EF78].mkv";
        "pausedDL"; 0.12; 1320000000; 0; 0; 1; false; []; -1),
      torrent("1a2b3c4d5e6f708192a3b4c5d6e7f80910111204";
        "[Demo] \($a.folder) - 01 (1080p) [9A8B7C6D].mkv";
        "stoppedUP"; 1.0; 1380000000; 0; 0; 1; false; []; ($now - 3600))
    ]')"

printf '%s\n' "$torrents" | jq . > "$STAGE/torrents.json"
echo "Torrents: $(jq 'length' "$STAGE/torrents.json") in $STAGE/torrents.json"

# Device steps. A missing emulator stops here with the local files already written.
if ! "$ADB" devices | grep -q "^${ANDROID_SERIAL}[[:space:]]*device$"; then
  echo "seed-demo.sh: $ANDROID_SERIAL is not attached. Start it with: scripts/emulator.sh" >&2
  echo "The local files are in $STAGE; run again once it is up." >&2
  exit 1
fi

"$ADB" push "$STAGE/anibeam-seed" "$SEED_DEVICE" >/dev/null
"$ADB" shell chmod a+r "$SEED_DEVICE"
"$ADB" shell mkdir -p "$DEMO_DEVICE"
"$ADB" push "$STAGE/torrents.json" "$DEMO_DEVICE/torrents.json" >/dev/null
"$ADB" shell chmod -R a+rX "$DEMO_DEVICE"
echo "Pushed the seed list and torrents to the device. Relaunch the app to load them."
