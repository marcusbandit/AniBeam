#!/usr/bin/env bash
# usage: deps.sh <prefix>
# Builds what Homebrew does not ship the way AniBeam needs it into <prefix>:
#   libmpv 0.41.0, lean: the OpenGL render API, VideoToolbox, CoreAudio, libass, and none of
#     Lua, JavaScript, VapourSynth, Vulkan or libarchive. Homebrew's mpv enables VapourSynth,
#     which links a whole Python framework into libmpv. The Swift glue stays on: 0.41's
#     macOS sources include its header whatever else is off. Its media player half (Now
#     Playing, remote commands) does not, since the shell owns media keys.
#   MpvQt 1.2.0, which Homebrew does not package at all.
#   qbittorrent-nox 5.1.4, the headless daemon the Downloads page drives. Homebrew's
#     qbittorrent is the GUI app and has no nox formula, so this builds the pinned source
#     against Homebrew's Qt (the one the app deploys, so the frameworks are shared),
#     libtorrent-rasterbar, boost and openssl@3.
# Everything else (Qt, FFmpeg, libass, libplacebo) comes from Homebrew; bundle.sh copies it
# into the app. The workflow caches <prefix> on a key that includes those Homebrew versions,
# since libmpv records their install names.
set -euo pipefail

prefix="$1"
MPV_VERSION=0.41.0
MPVQT_VERSION=v1.2.0
QBT_VERSION=5.1.4
QBT_SHA256=ab37deeb364049a308127c572f139bd5eb0535fa355db8821e5aa31990b5625e
brew_prefix="$(brew --prefix)"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-15.0}"
export PKG_CONFIG_PATH="$prefix/lib/pkgconfig:$brew_prefix/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$prefix"

echo "== libmpv $MPV_VERSION"
curl -fsSL "https://github.com/mpv-player/mpv/archive/refs/tags/v$MPV_VERSION.tar.gz" | tar -xz -C "$work"
meson setup "$work/mpv-build" "$work/mpv-$MPV_VERSION" \
  --prefix="$prefix" --libdir=lib --buildtype=release \
  -Dlibmpv=true -Dcplayer=false -Dtests=false -Dbuild-date=false \
  -Dmanpage-build=disabled -Dhtml-build=disabled -Dpdf-build=disabled \
  -Dswift-build=enabled -Dmacos-cocoa-cb=disabled -Dmacos-media-player=disabled -Dmacos-touchbar=disabled \
  -Dcocoa=enabled -Dgl=enabled -Dgl-cocoa=enabled -Dplain-gl=enabled -Dvideotoolbox-gl=enabled \
  -Dcoreaudio=enabled -Dlcms2=enabled -Duchardet=enabled -Dzimg=enabled -Diconv=enabled -Dzlib=enabled \
  -Dlua=disabled -Djavascript=disabled -Dvapoursynth=disabled -Dlibarchive=disabled \
  -Drubberband=disabled -Dlibbluray=disabled -Ddvdnav=disabled -Dcdda=disabled -Ddvbin=disabled \
  -Dlibavdevice=disabled -Djpeg=disabled -Dsixel=disabled -Dcaca=disabled \
  -Dvulkan=disabled -Dshaderc=disabled -Dspirv-cross=disabled -Dsdl2-audio=disabled -Dsdl2-video=disabled
meson compile -C "$work/mpv-build"
meson install -C "$work/mpv-build"

echo "== MpvQt $MPVQT_VERSION"
# FindLibmpv hands CMake <prefix>/include/mpv while the sources say <mpv/client.h>; on Arch
# /usr/include covers the difference, here the flag below does.
git clone --quiet --depth 1 --branch "$MPVQT_VERSION" https://invent.kde.org/libraries/mpvqt.git "$work/mpvqt"
cmake -S "$work/mpvqt" -B "$work/mpvqt-build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$prefix" \
  -DCMAKE_INSTALL_LIBDIR=lib \
  -DCMAKE_PREFIX_PATH="$prefix;$brew_prefix" \
  -DCMAKE_CXX_FLAGS="-isystem $prefix/include" \
  -DBUILD_TESTING=OFF
cmake --build "$work/mpvqt-build"
cmake --install "$work/mpvqt-build"

echo "== qBittorrent $QBT_VERSION (nox)"
curl -fsSL -o "$work/qbittorrent.tar.gz" "https://github.com/qbittorrent/qBittorrent/archive/refs/tags/release-$QBT_VERSION.tar.gz"
echo "$QBT_SHA256  $work/qbittorrent.tar.gz" | shasum -a 256 -c -
tar -xz -C "$work" -f "$work/qbittorrent.tar.gz"
# The stacktrace support is boost's and only helps with crash reports nobody reads here. The
# install rpath keeps the binary runnable from the prefix, since CMake strips the build rpath
# that finds Qt's frameworks; bundle.sh and fix-dylibs.py replace it with the bundle's.
cmake -S "$work/qBittorrent-release-$QBT_VERSION" -B "$work/qbt-build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$prefix" \
  -DCMAKE_PREFIX_PATH="$brew_prefix;$(brew --prefix openssl@3)" \
  -DCMAKE_INSTALL_RPATH="$brew_prefix/lib" \
  -DOPENSSL_ROOT_DIR="$(brew --prefix openssl@3)" \
  -DGUI=OFF -DWEBUI=ON -DSTACKTRACE=OFF -DTESTING=OFF
cmake --build "$work/qbt-build"
cmake --install "$work/qbt-build"

echo "== installed"
ls -la "$prefix/lib" "$prefix/include" "$prefix/bin"
otool -L "$prefix/lib/libmpv.dylib" "$prefix/lib/libMpvQt.dylib" "$prefix/bin/qbittorrent-nox"
"$prefix/bin/qbittorrent-nox" --version
