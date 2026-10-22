#!/usr/bin/env bash
# Screenshot the device framebuffer into captures/ (guest pixels, no compositor involved).
# Usage: scripts/shot.sh [name] [--size]
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh
mkdir -p captures

name=shot
size=0
for arg in "$@"; do
  case "$arg" in
    --size) size=1 ;;
    *) name="$arg" ;;
  esac
done
out="captures/${name}.png"

# Ask which display is powered: a bare screencap on a multi-display device prints a warning
# into the PNG and falls back to display 0.
disp="$("$ADB" shell dumpsys display \
  | grep -oE 'DisplayViewport\{[^}]*isActive=true[^}]*\}' \
  | sed -n "s/.*uniqueId='local:\([0-9]*\)'.*/\1/p" | head -1)"
if [ -n "$disp" ]; then
  "$ADB" exec-out screencap -d "$disp" -p > "$out"
else
  "$ADB" exec-out screencap -p > "$out"
fi
# screencap exits 0 with an empty file while the surface is not up.
if ! file "$out" | grep -q "PNG image"; then
  echo "shot.sh: $out is not a PNG; the screen is probably not up yet." >&2
  exit 1
fi
echo "$out"
if [ "$size" = 1 ]; then
  "$ADB" shell wm size | tr -d '\r'
fi
