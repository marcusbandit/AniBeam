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
