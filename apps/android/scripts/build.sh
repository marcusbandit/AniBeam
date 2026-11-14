#!/usr/bin/env bash
# Build the Anibeam APK inside a hard memory cap; a Rust + Skia + NDK build can take the
# desktop into swap otherwise.
#
#   scripts/build.sh                  APK for the emulator (x86_64), build/AniBeam-debug.apk
#   scripts/build.sh --abi all        APK for phones and the emulator (arm64-v8a + x86_64)
#   scripts/build.sh --dist           --abi all, then copy to dist/AniBeam-<versionName>.apk
#   scripts/build.sh cargo <args>     any cargo command, capped (cargo test -p anibeam-companion)
#
# The version is Cargo.toml's, 2.4.3, unless CI says what it ships the build as in
# ANIBEAM_RELEASE_VERSION: 2.4.3 for a stable release, 2.4.3-nightly.20261113.41 for a
# nightly. ANIBEAM_KEYSTORE, ANIBEAM_KEYSTORE_PASS and ANIBEAM_KEY_ALIAS pick the signing key;
# the default is the debug key in ~/.android/debug.keystore.
set -euo pipefail
cd "$(dirname "$0")/.."

# Re-enter inside a capped scope. Two Rust compile jobs plus the Slint compiler peak near
# 4.5 GB on this crate graph; 6 GB leaves room for a thin-LTO link. Swap is forbidden, so the
# kernel kills the build before the desktop starts thrashing.
if [ -z "${ANIBEAM_BUILD_SCOPED:-}" ]; then
  MIN_FREE_MB=7000
  avail=$(awk '/MemAvailable/ {print int($2/1024)}' /proc/meminfo)
  if (( avail < MIN_FREE_MB )); then
    echo "build.sh: only ${avail} MB available (< ${MIN_FREE_MB} MB). Close something first." >&2
    exit 1
  fi
  export ANIBEAM_BUILD_SCOPED=1
  exec systemd-run --user --scope -q \
    -p MemoryMax="${ANIBEAM_BUILD_MEM:-6000}M" -p MemorySwapMax=0 -p CPUQuota=600% \
    "$0" "$@"
fi

SDK="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
NDK="${ANDROID_NDK_ROOT:-$(ls -d "$SDK"/ndk/* | sort -V | tail -1)}"
BT="$(ls -d "$SDK"/build-tools/* | sort -V | tail -1)"
PLATFORM_JAR="$SDK/platforms/android-36/android.jar"
TC="$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin"
MIN_API=26
# Slint compiles its small IME helper with javac and d8; d8 and JDK 21+ have disagreed
# before (slint#4973), so pin the JDK it gets.
export JAVA_HOME="${JAVA_HOME:-/usr/lib/jvm/java-17-openjdk}"
export ANDROID_HOME="$SDK" ANDROID_NDK_ROOT="$NDK" ANDROID_NDK_HOME="$NDK" ANDROID_NDK="$NDK"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-6}"

for t in x86_64-linux-android aarch64-linux-android; do
  up=$(echo "$t" | tr 'a-z-' 'A-Z_')
  low=${t//-/_}
  export "CARGO_TARGET_${up}_LINKER=$TC/${t}${MIN_API}-clang"
  export "CC_${low}=$TC/${t}${MIN_API}-clang" "CXX_${low}=$TC/${t}${MIN_API}-clang++" "AR_${low}=$TC/llvm-ar"
done

crate_version=$(sed -n '0,/^version = "\(.*\)"/s//\1/p' Cargo.toml)
version_name="${ANIBEAM_RELEASE_VERSION:-$crate_version}"
base="${version_name%%-*}"
if [ "$base" != "$crate_version" ]; then
  echo "build.sh: ANIBEAM_RELEASE_VERSION $version_name is not a build of $crate_version (Cargo.toml)." >&2
  exit 1
fi
# Android installs an update only over a lower versionCode, so the code follows the train:
# every build of 2.4.3 sits below every build of 2.5.0, and within 2.4.3 a local build comes
# first, then the nightlies by run number, then the stable release they led up to.
#   2.4.3 local 204030000   2.4.3-nightly.<day>.41 204030041   2.4.3 stable 204039999
IFS=. read -r major minor patch <<<"$base"
case "$version_name" in
  "$base") step=$([ -n "${ANIBEAM_RELEASE_VERSION:-}" ] && echo 9999 || echo 0) ;;
  "$base"-nightly.*.*) step="${version_name##*.}"; [[ "$step" =~ ^[1-9][0-9]*$ ]] || step=bad ;;
  *) echo "build.sh: $version_name is neither a stable version nor a nightly." >&2; exit 1 ;;
esac
if [ "$step" = bad ] || (( minor > 99 || patch > 99 || step > 9999 )); then
  echo "build.sh: $version_name does not fit the versionCode scheme." >&2
  exit 1
fi
version_code=$(( ((major * 100 + minor) * 100 + patch) * 10000 + step ))
# build.rs puts both into the app, which shows them in More, About. Worked out before the
# cargo passthrough too, so a cargo test between two APK builds does not change them and
# rebuild the app crate each time.
export ANIBEAM_PHONE_VERSION="$version_name" ANIBEAM_VERSION_CODE="$version_code"
if [[ "${1:-}" == "cargo" ]]; then
  shift
  exec cargo "$@"
fi

abis=(x86_64)
dist=0
while (( $# )); do
  case "$1" in
    --abi) shift; case "${1:-}" in
             all) abis=(arm64-v8a x86_64) ;;
             x86_64|arm64-v8a) abis=("$1") ;;
             *) echo "build.sh: --abi takes x86_64, arm64-v8a or all" >&2; exit 1 ;;
           esac ;;
    --dist) dist=1; abis=(arm64-v8a x86_64) ;;
    *) echo "build.sh: unknown argument $1" >&2; exit 1 ;;
  esac
  shift
done

# What's new reads CHANGELOG.md into the APK: a build ships with its own version's notes.
notes_version=$(sed -n 's/^## \([^ ]*\).*/\1/p' CHANGELOG.md | head -1)
if [ "$notes_version" != "$base" ]; then
  echo "build.sh: the newest heading in CHANGELOG.md is ${notes_version:-missing}, but the version is $base. Add its notes first." >&2
  exit 1
fi

triple() { case "$1" in x86_64) echo x86_64-linux-android ;; arm64-v8a) echo aarch64-linux-android ;; esac; }

scripts/fetch-mpv.sh
# rust-toolchain.toml pins the version for the whole repo, and that toolchain starts with only
# the host's standard library, so add the targets this build needs to it. A no-op once they
# are there; a desktop checkout never downloads Android's standard library.
if command -v rustup > /dev/null; then
  targets=()
  for abi in "${abis[@]}"; do targets+=("$(triple "$abi")"); done
  rustup target add "${targets[@]}"
fi

for abi in "${abis[@]}"; do
  echo "build.sh: cargo build --release for $abi"
  cargo build --release --lib --target "$(triple "$abi")"
done

stage=build/apk
rm -rf "$stage"
mkdir -p "$stage"
for abi in "${abis[@]}"; do
  mkdir -p "$stage/lib/$abi"
  cp "target/$(triple "$abi")/release/libanibeam.so" "$stage/lib/$abi/"
  # The player: libmpv, its ffmpeg and the C++ runtime they share.
  cp vendor/mpv/"$abi"/*.so "$stage/lib/$abi/"
done

manifest=android/AndroidManifest.xml
"$BT/aapt2" compile --dir android/res -o "$stage/res.zip"
"$BT/aapt2" link -o "$stage/base.apk" -I "$PLATFORM_JAR" --manifest "$manifest" \
  --version-code "$version_code" --version-name "$version_name" "$stage/res.zip"
# The Java side, the download service's notification and the player's screen calls:
# javac, then d8, into classes.dex at the APK root.
rm -rf "$stage/java" && mkdir -p "$stage/java/classes" "$stage/java/dex"
"$JAVA_HOME/bin/javac" --release 8 -Xlint:-options -classpath "$PLATFORM_JAR" -d "$stage/java/classes" \
  $(find android/java -name '*.java')
"$BT/d8" --lib "$PLATFORM_JAR" --min-api "$MIN_API" --output "$stage/java/dex" \
  $(find "$stage/java/classes" -name '*.class')
(cd "$stage/java/dex" && zip -q ../../base.apk classes.dex)
# Native libraries go in stored, not deflated, and zipalign puts them on 16 KB page
# boundaries, so the loader maps them straight out of the APK (extractNativeLibs=false).
(cd "$stage" && zip -q -0 -r base.apk lib)
"$BT/zipalign" -P 16 -f 4 "$stage/base.apk" "$stage/aligned.apk"

keystore="${ANIBEAM_KEYSTORE:-$HOME/.android/debug.keystore}"
pass="${ANIBEAM_KEYSTORE_PASS:-android}"
apk=build/AniBeam-debug.apk
"$BT/apksigner" sign --ks "$keystore" --ks-pass "pass:$pass" --key-pass "pass:$pass" \
  --ks-key-alias "${ANIBEAM_KEY_ALIAS:-androiddebugkey}" --out "$apk" "$stage/aligned.apk"
rm -f "$apk.idsig"
echo "build.sh: $apk (versionCode $version_code, versionName $version_name, ${abis[*]})"
[[ " ${abis[*]} " == *" arm64-v8a "* ]] || echo "build.sh: emulator only, a phone cannot run this APK; use --dist for one"

if (( dist )); then
  # A phone runs arm64 only. An APK without it installs and then dies on launch.
  if ! unzip -l "$apk" "lib/arm64-v8a/libanibeam.so" >/dev/null 2>&1; then
    echo "build.sh: $apk has no arm64-v8a code; refusing to put it in dist/" >&2
    exit 1
  fi
  mkdir -p dist
  cp "$apk" "dist/AniBeam-$version_name.apk"
  echo "build.sh: dist/AniBeam-$version_name.apk"
fi
