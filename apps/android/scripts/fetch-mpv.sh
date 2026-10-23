#!/usr/bin/env bash
# Fetch libmpv and its ffmpeg for the phone's player into vendor/mpv/<abi>/.
#
# The libraries come out of mpv-android's release APKs, pinned by tag and checked by
# hash, so every build links the same mpv. vendor/ is not in git: about 30 MB per ABI.
#
#   scripts/fetch-mpv.sh          fetch what is missing
#   scripts/fetch-mpv.sh --force  fetch again
set -euo pipefail
cd "$(dirname "$0")/.."

TAG=2026-09-17
declare -A SHA=(
  [arm64-v8a]=0b0e0a534dd6b8d2b117f06952e0600064ad97702e48bff02543e3e32107803d
  [x86_64]=347cef5d0c2fe76ec95e19685597ad5d208e5309cc0642d51ae656b125ce1d07
)
# libplayer.so is mpv-android's own JNI glue; the app has its own.
LIBS=(libmpv.so libavcodec.so libavdevice.so libavfilter.so libavformat.so libavutil.so
      libswresample.so libswscale.so libc++_shared.so)

force=0
[[ "${1:-}" == "--force" ]] && force=1

for abi in arm64-v8a x86_64; do
  out=vendor/mpv/$abi
  if (( !force )) && [[ -f $out/.tag ]] && [[ "$(cat "$out/.tag")" == "$TAG" ]]; then
    continue
  fi
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  apk=app-default-$abi-release.apk
  echo "fetch-mpv.sh: mpv-android $TAG $apk"
  curl -fsSL -o "$tmp/$apk" "https://github.com/mpv-android/mpv-android/releases/download/$TAG/$apk"
  echo "${SHA[$abi]}  $tmp/$apk" | sha256sum -c --quiet
  rm -rf "$out" && mkdir -p "$out"
  for lib in "${LIBS[@]}"; do
    unzip -q -j "$tmp/$apk" "lib/$abi/$lib" -d "$out"
  done
  echo "$TAG" > "$out/.tag"
  rm -rf "$tmp"
  trap - EXIT
done
