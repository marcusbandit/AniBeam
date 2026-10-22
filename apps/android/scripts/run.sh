#!/usr/bin/env bash
# Build, install and launch AniBeam on the project's running emulator.
# Usage: scripts/run.sh [--no-build] [build.sh args]
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh

if ! "$ADB" devices | grep -q "^${ANDROID_SERIAL}[[:space:]]*device$"; then
  echo "run.sh: $ANDROID_SERIAL is not attached. Start it with: scripts/emulator.sh" >&2
  exit 1
fi
until [ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do
  sleep 2
done

if [[ "${1:-}" == "--no-build" ]]; then
  shift
else
  scripts/build.sh "$@"
fi
"$ADB" install -r build/AniBeam-debug.apk
"$ADB" shell am start -W -n "$PKG/$ACT"
