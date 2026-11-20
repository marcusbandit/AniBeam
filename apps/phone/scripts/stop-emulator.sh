#!/usr/bin/env bash
# Stop the project's emulator. sync first: emu kill is a hard QEMU kill and loses
# whatever the guest still had in its page cache.
set -euo pipefail
source "$(dirname "$0")/env.sh"
"$ADB" shell sync && "$ADB" emu kill
