#!/usr/bin/env bash
# Fetch every Lucide glyph the shell names into assets/icons/, stroke set to black so QtSvg
# reads it and ColorImage tints it. Re-run to add a name; then list it in build.rs.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
tag="1.41.0"
out="$here/assets/icons"; mkdir -p "$out"
cp -n "$here/../../spikes/home-grid-qml/assets/icons/"*.svg "$out"/ 2>/dev/null || true
cp "$here/../../spikes/home-grid-qml/assets/icons/LICENSE" "$out/LICENSE"
names=(search arrow-left arrow-right arrow-down arrow-up chevron-down chevron-up chevron-left chevron-right chevrons-right
  external-link link star eye-off plus pause skip-back skip-forward volume-2 volume-x maximize minimize rotate-ccw rotate-cw
  audio-lines languages clock film tv layers list-filter keyboard scan ban circle-check circle-x clapperboard image
  sliders-horizontal triangle-alert square-check square calendar-clock book-open users case-sensitive chart-pie check-check
  badge-check bell circle-question-mark step-back step-forward file-down file-up circle-play list-video bookmark sparkles
  radio folder-search check pencil trash)
for n in "${names[@]}"; do
  [ -f "$out/$n.svg" ] && continue
  curl -fsSL "https://raw.githubusercontent.com/lucide-icons/lucide/$tag/icons/$n.svg" \
    | sed 's/stroke="currentColor"/stroke="#000"/' > "$out/$n.svg"
done
ls "$out"/*.svg | wc -l
