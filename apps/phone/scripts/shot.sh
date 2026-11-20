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

# From the powered display, checked to be a PNG (env.sh, shared with ctl.sh shot).
shot_to "$out" || { echo "shot.sh: failed" >&2; exit 1; }
echo "$out"
if [ "$size" = 1 ]; then
  "$ADB" shell wm size | tr -d '\r'
fi
