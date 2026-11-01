#!/usr/bin/env bash
# Drive the hinge of the fold AVD (scripts/create-avd.sh --fold) and report what changed.
# Usage: ANIBEAM_AVD=anibeam_fold scripts/fold.sh fold | unfold | half [degrees] | state
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh

config="$ANDROID_AVD_HOME/$AVD.avd/config.ini"
if ! grep -q '^hw.sensor.hinge *= *yes' "$config" 2>/dev/null; then
  echo "fold.sh: AVD $AVD has no hinge. Create one with: scripts/create-avd.sh --fold" >&2
  exit 1
fi

angle() { "$ADB" emu sensor set hinge-angle0 "$1" >/dev/null; }

case "${1:-state}" in
  fold) angle 0 ;;
  unfold) angle 180 ;;
  half) angle "${2:-110}" ;;
  state) ;;
  *) echo "fold.sh: fold, unfold, half [degrees] or state" >&2; exit 1 ;;
esac
# The hinge settles and the display resizes a moment after the angle is set.
[ "${1:-state}" = state ] || sleep 1.5
echo "hinge $("$ADB" emu sensor get hinge-angle0 | tr -d '\r' | sed -n 's/.*= *//p' | head -1)"
"$ADB" shell wm size | tr -d '\r' | tail -1
"$ADB" shell dumpsys device_state | tr -d '\r' | grep -m1 -E 'mCommittedState|Committed' || true
