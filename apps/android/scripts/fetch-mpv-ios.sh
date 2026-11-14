#!/usr/bin/env bash
# Fetch libmpv, its FFmpeg and the libraries under them for the iPhone's player into
# vendor/mpv-ios/{device,sim}/, one static archive per library.
#
# They come out of MPVKit's LGPL release (its xcframework zips, no samba, no GPL parts), pinned
# by tag and checked by hash, so every build links the same mpv. Each zip holds a device
# slice (ios-arm64) and a simulator slice (arm64 and x86_64); the device archive is copied as
# it is and the simulator's thinned to arm64. vendor/ is not in git: about 350 MB of zips,
# kept in vendor/mpv-ios/zips so a --force run does not download them again.
#
# Runs on the bash 3.2 macOS ships, with shasum, unzip and lipo.
#
#   scripts/fetch-mpv-ios.sh          fetch what is missing
#   scripts/fetch-mpv-ios.sh --force  unpack again
set -euo pipefail
cd "$(dirname "$0")/.."

TAG=1.1.0-n9.0.2
MPVKIT=https://github.com/mpvkit/MPVKit/releases/download/$TAG
GH=https://github.com/mpvkit
# name, url, sha256 of the zip (SwiftPM's checksum in MPVKit's Package.swift at the tag).
LIST="
Libmpv $MPVKIT/Libmpv.xcframework.zip 1d8651bebfe467bd5408751647de692cdacb1d19ba825e11785d770bc88c3225
Libavcodec $MPVKIT/Libavcodec.xcframework.zip d516e904490c711c2875dd238d7d8d9f4f46e335829cb8ee7e0e1623e952e372
Libavdevice $MPVKIT/Libavdevice.xcframework.zip f29bde9d8f788337996f8b2759645a6e22e9056e4bc1e3494458193405f50a5d
Libavfilter $MPVKIT/Libavfilter.xcframework.zip e764dc3b3d36abb26d6d0ae2f0c6a1de0b80a453f3ca4bf2bb6596c0f6464b19
Libavformat $MPVKIT/Libavformat.xcframework.zip 0a840424d7a2d971687d6e96d995e21420cce1b27dae74687ff4f5f31dc36e73
Libavutil $MPVKIT/Libavutil.xcframework.zip 0ca518d2a3eab22763310d8bcdc977494ddb3daf3dc3b73a2ed84c5eefc13f75
Libswresample $MPVKIT/Libswresample.xcframework.zip 49edabb3c2e7fb97e6458f2e6a2fea66ab567b55258141e598fecfcd29cd9eeb
Libswscale $MPVKIT/Libswscale.xcframework.zip 5aebc8107219d518d8e89f0af97365ff9758f91b3096a7d5c1a16e092745de39
Libcrypto $GH/openssl-build/releases/download/3.3.5/Libcrypto.xcframework.zip 593283be2a90f7fd66f6e6ed331b2f099cf403e0926fe3b4ac09a7062b793965
Libssl $GH/openssl-build/releases/download/3.3.5/Libssl.xcframework.zip ff5ffd43d015d7285fd37e4a3145b25cbd8d2842740bd629a711c299a20e226a
gmp $GH/gnutls-build/releases/download/3.8.11/gmp.xcframework.zip ad33c7a08f4cdcb9924c8f0e6d9a054dad33d7794b97667bf8b6fb2b236ae585
nettle $GH/gnutls-build/releases/download/3.8.11/nettle.xcframework.zip 0fdf3ebf8bd7b8bc8eee837cf27261cb4c52ae520b6576a2f468656aa1691e02
hogweed $GH/gnutls-build/releases/download/3.8.11/hogweed.xcframework.zip 25727c9fa67287fa0a4f4722f88bb8be669b23cd7e837e2d00870eb8a25d3f27
gnutls $GH/gnutls-build/releases/download/3.8.11/gnutls.xcframework.zip 3dbec5809339189bf9679e218c6cff387ebf8fb72745927835afc2678f5c9f4d
Libunibreak $GH/libass-build/releases/download/0.17.5/Libunibreak.xcframework.zip 940d9833cf4477d0a260d9f2b4066125bc0ff7bbc111ac3c90e774765b77a559
Libfreetype $GH/libass-build/releases/download/0.17.5/Libfreetype.xcframework.zip 496ca62488530e14b1e4624d20ee2b237c0bd675cd70c19da578a5768302d02d
Libfribidi $GH/libass-build/releases/download/0.17.5/Libfribidi.xcframework.zip bc15e097b892f2f90424e4a27ba287070cc2f98a74a4da10e6d2481d15cf5ff9
Libharfbuzz $GH/libass-build/releases/download/0.17.5/Libharfbuzz.xcframework.zip aa8e0b9ca0387dac74e3e93c86e34d11982bb013b28022d0e6966a8427a35b2e
Libass $GH/libass-build/releases/download/0.17.5/Libass.xcframework.zip 3f4c576d2818ceb4544aa2a20e1f55846511c5e706fd19adc3ea9fd842270498
Libbluray $GH/libbluray-build/releases/download/1.4.0/Libbluray.xcframework.zip bc037d34e2b0b5ab7f202fb371f5fb298136cc66fdf406c2172185d06f53f18d
Libuavs3d $GH/libuavs3d-build/releases/download/1.2.1-fix/Libuavs3d.xcframework.zip bd5256081486d16c51c868d755bf70266c424b54c895269580de44ec6707f789
Libdovi $GH/libdovi-build/releases/download/3.3.2/Libdovi.xcframework.zip e693e239808350868e79c5448ef9f02e2716bc822dd8632a41a368a1eae5ca7d
MoltenVK $GH/moltenvk-build/releases/download/1.4.2/MoltenVK.xcframework.zip aee189c54ad7c62bf734a3dc51eb4cfad5685d1d63b0ec519ecd1b437c332418
Libshaderc_combined $GH/libshaderc-build/releases/download/2025.5.0/Libshaderc_combined.xcframework.zip 758047b615708575b580eb960a2d083f760a29dc462d6eaa360416c946ce433b
lcms2 $GH/lcms2-build/releases/download/2.17.0/lcms2.xcframework.zip dc0dce0606f6ab6841a8ec5a6bd4448e2f3ef00661a050460f806c9393dc6982
Libplacebo $GH/libplacebo-build/releases/download/7.360.1/Libplacebo.xcframework.zip 2fa3d54cb81f302d6f11c7b2f509af30944381c3b11ee9d35096eb4637a6e2dd
Libdav1d $GH/libdav1d-build/releases/download/1.5.3/Libdav1d.xcframework.zip d1a32ae6a1f0193e9f05c44c9176844af7f6d2a58cb33843f6f1b8dfd9224083
Libuchardet $GH/libuchardet-build/releases/download/0.0.8/Libuchardet.xcframework.zip ea4f548a230a755e059144657cc9e2ff563c1cdeae03974c38f8b6e1a40303fb
"

out=vendor/mpv-ios
# The tag file holds the list's own hash, so a changed pin fetches again.
stamp="$TAG $(printf '%s' "$LIST" | shasum -a 256 | cut -d' ' -f1)"
if [ "${1:-}" != "--force" ] && [ -f "$out/.tag" ] && [ "$(cat "$out/.tag")" = "$stamp" ]; then
  exit 0
fi

zips=$out/zips
mkdir -p "$zips"
rm -rf "$out/device" "$out/sim" "$out/.tag"
mkdir -p "$out/device" "$out/sim"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# The archive inside one slice: a framework's binary, or a bare .a.
archive() {
  local slice=$1 name=$2
  if [ -f "$slice/$name.framework/$name" ]; then
    echo "$slice/$name.framework/$name"
    return
  fi
  local fw
  fw=$(find "$slice" -maxdepth 1 -name '*.framework' -type d | head -1)
  if [ -n "$fw" ] && [ -f "$fw/$(basename "$fw" .framework)" ]; then
    echo "$fw/$(basename "$fw" .framework)"
    return
  fi
  local found
  found=$(find "$slice" -maxdepth 1 -name '*.a' -type f | head -1)
  [ -n "$found" ] || { echo "fetch-mpv-ios.sh: no archive in $slice" >&2; exit 1; }
  echo "$found"
}

echo "$LIST" | while read -r name url sha; do
  [ -n "$name" ] || continue
  zip=$zips/$name.xcframework.zip
  if [ ! -f "$zip" ] || ! echo "$sha  $zip" | shasum -a 256 -c --quiet >/dev/null 2>&1; then
    echo "fetch-mpv-ios.sh: $name"
    curl -fsSL --retry 3 -o "$zip.part" "$url"
    mv "$zip.part" "$zip"
  fi
  echo "$sha  $zip" | shasum -a 256 -c --quiet || { rm -f "$zip"; exit 1; }
  rm -rf "${tmp:?}"/*
  unzip -q "$zip" -d "$tmp"
  xc=$(find "$tmp" -maxdepth 2 -name '*.xcframework' -type d | head -1)
  [ -n "$xc" ] || { echo "fetch-mpv-ios.sh: no xcframework in $zip" >&2; exit 1; }
  device=$(archive "$xc/ios-arm64" "$name")
  cp "$device" "$out/device/lib$name.a"
  simdir=$(find "$xc" -maxdepth 1 -type d -name 'ios-*-simulator' | head -1)
  [ -n "$simdir" ] || { echo "fetch-mpv-ios.sh: no simulator slice in $zip" >&2; exit 1; }
  sim=$(archive "$simdir" "$name")
  case "$(lipo -archs "$sim")" in
    *" "*) lipo -thin arm64 "$sim" -output "$out/sim/lib$name.a" ;;
    *) cp "$sim" "$out/sim/lib$name.a" ;;
  esac
done

echo "$stamp" > "$out/.tag"
echo "fetch-mpv-ios.sh: $(ls "$out/device" | wc -l | tr -d ' ') libraries in $out"
