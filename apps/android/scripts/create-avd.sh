#!/usr/bin/env bash
# Create the project's phone emulator, the same way on any machine.
# Usage: scripts/create-avd.sh          AVD anibeam_pixel, Pixel 10 Pro profile, Android 16
#        scripts/create-avd.sh --fold   AVD anibeam_fold, Pixel 10 Pro Fold with a hinge
# Use the fold one with ANIBEAM_AVD=anibeam_fold in front of the other scripts.
set -euo pipefail
if [[ "${1:-}" == "--fold" ]]; then
  export ANIBEAM_AVD="${ANIBEAM_AVD:-anibeam_fold}"
fi
source "$(dirname "$0")/env.sh"
IMAGE="system-images;android-36;google_apis_playstore;x86_64"
DEVICE=pixel_10_pro
[[ "${1:-}" == "--fold" ]] && DEVICE=pixel_10_pro_fold
export JAVA_HOME="${JAVA_HOME:-/usr/lib/jvm/java-21-openjdk}"
mkdir -p "$ANDROID_AVD_HOME"

SDKMANAGER="$SDK/cmdline-tools/latest/bin/sdkmanager"
AVDMANAGER="$SDK/cmdline-tools/latest/bin/avdmanager"
for tool in "$SDKMANAGER" "$AVDMANAGER"; do
  [ -x "$tool" ] || { echo "Missing $tool. Install the Android command line tools first." >&2; exit 1; }
done
if [ ! -d "$SDK/${IMAGE//;//}" ]; then
  echo "Downloading $IMAGE ..."
  yes | "$SDKMANAGER" "$IMAGE" >/dev/null
fi

if [ -d "$ANDROID_AVD_HOME/$AVD.avd" ]; then
  echo "AVD $AVD already exists."
else
  echo no | "$AVDMANAGER" create avd -n "$AVD" -k "$IMAGE" -d "$DEVICE" >/dev/null
  echo "Created AVD $AVD."
fi

CONFIG="$ANDROID_AVD_HOME/$AVD.avd/config.ini"
set_prop() {
  if grep -q "^$1 *=" "$CONFIG"; then sed -i "s|^$1 *=.*|$1=$2|" "$CONFIG"; else echo "$1=$2" >> "$CONFIG"; fi
}
set_prop hw.keyboard yes
set_prop hw.ramSize 2048
set_prop hw.cpu.ncore 4
set_prop disk.dataPartition.size 4G
echo "Configured $CONFIG (keyboard on, 2 GB guest RAM). Start it with: scripts/emulator.sh"
