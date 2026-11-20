# Sourced by the other scripts: one place for the SDK paths and the device they talk to.
SDK="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
export ANDROID_AVD_HOME="${ANDROID_AVD_HOME:-$HOME/.config/.android/avd}"
export ANDROID_SDK_HOME="${ANDROID_SDK_HOME:-$HOME/.config/.android}"
ADB="$SDK/platform-tools/adb"
[ -x "$ADB" ] || ADB=adb
AVD="${ANIBEAM_AVD:-anibeam_pixel}"
# A fixed console port inside adb's scanned range (5555 to 5585), so this project's emulator
# always has the same serial and the scripts never act on another device on the adb server.
EMU_PORT="${ANIBEAM_EMU_PORT:-5580}"
export ANDROID_SERIAL="${ANDROID_SERIAL:-emulator-$EMU_PORT}"
PKG=com.marcusrosado.anibeam
ACT=android.app.NativeActivity

# The id of the display that is powered, empty when the device does not say: a bare
# screencap on a foldable prints a warning into the PNG and falls back to display 0.
# DisplayViewport has no isActive before Android 10, so finding none is not an error.
active_display() {
  "$ADB" shell dumpsys display \
    | grep -oE 'DisplayViewport\{[^}]*isActive=true[^}]*\}' \
    | sed -n "s/.*uniqueId='local:\([0-9]*\)'.*/\1/p" | head -1 || true
}

# shot_to <file.png>: the device's own pixels, from the powered display.
shot_to() {
  local disp
  disp="$(active_display)"
  mkdir -p "$(dirname "$1")"
  # Callers run this as `shot_to ... || ...`, which turns errexit off in here.
  if ! "$ADB" exec-out screencap ${disp:+-d "$disp"} -p > "$1"; then
    echo "adb could not take the screenshot (is $ANDROID_SERIAL attached?)" >&2
    return 1
  fi
  # screencap exits 0 with an empty file while the surface is not up.
  if ! file "$1" | grep -q "PNG image"; then
    echo "$1 is not a PNG; the screen is probably not up yet." >&2
    return 1
  fi
}
