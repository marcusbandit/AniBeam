#!/usr/bin/env bash
# Start the project's emulator headless, inside a hard memory and CPU cap.
# Usage: scripts/emulator.sh [--window]
set -euo pipefail
source "$(dirname "$0")/env.sh"

if [ ! -d "$ANDROID_AVD_HOME/$AVD.avd" ]; then
  echo "emulator.sh: no AVD named $AVD. Create it with: scripts/create-avd.sh" >&2
  exit 1
fi
MIN_FREE_MB=8000
avail=$(awk '/MemAvailable/ {print int($2/1024)}' /proc/meminfo)
if (( avail < MIN_FREE_MB )); then
  echo "emulator.sh: only ${avail} MB available (< ${MIN_FREE_MB} MB). Not starting; close something first." >&2
  exit 1
fi
if pgrep -f "qemu-system-x86_64.* -avd ${AVD}( |$)" >/dev/null; then
  echo "emulator.sh: AVD ${AVD} is already running." >&2
  exit 1
fi

# Headless by default so no window lands on a workspace someone is using.
extra=(-no-window)
gpu=swiftshader_indirect
if [[ "${1:-}" == "--window" ]]; then
  extra=(); gpu=host; shift
fi

# Guest RAM is 2 GB, but a first boot of the 1280x2856 panel on software GL peaked past
# 4.8 GB on the host and a 5 GB cap killed it, so the cap is 7 GB. Swap is forbidden, so
# the guest dies before the desktop does.
mem="${ANIBEAM_EMU_MEM:-7000}"
exec systemd-run --user --scope -q \
  -p "MemoryMax=${mem}M" -p MemorySwapMax=0 -p CPUQuota=400% \
  "$SDK/emulator/emulator" -avd "$AVD" -port "$EMU_PORT" -memory 2048 -no-audio \
    -no-boot-anim -no-snapshot -gpu "$gpu" "${extra[@]}" "$@"
