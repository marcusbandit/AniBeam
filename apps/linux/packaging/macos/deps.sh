#!/usr/bin/env bash
# usage: deps.sh <prefix>
# Builds what Homebrew does not ship the way AniBeam needs it into <prefix>:
#   libmpv 0.41.0, lean: the OpenGL render API, VideoToolbox, CoreAudio, libass, and none of
#     Lua, JavaScript, VapourSynth, Vulkan or libarchive. Homebrew's mpv enables VapourSynth,
#     which links a whole Python framework into libmpv.
#   MpvQt 1.2.0, which Homebrew does not package at all.
# Everything else (Qt, FFmpeg, libass, libplacebo) comes from Homebrew; bundle.sh copies it
# into the app. The workflow caches <prefix> on a key that includes those Homebrew versions,
# since libmpv records their install names.
set -euo pipefail

prefix="$1"
MPV_VERSION=0.41.0
MPVQT_VERSION=v1.2.0
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
  -Dswift-build=disabled -Dmacos-cocoa-cb=disabled -Dmacos-media-player=disabled -Dmacos-touchbar=disabled \
  -Dcocoa=enabled -Dgl=enabled -Dgl-cocoa=enabled -Dplain-gl=enabled -Dvideotoolbox-gl=enabled \
  -Dcoreaudio=enabled -Dlcms2=enabled -Duchardet=enabled -Dzimg=enabled -Diconv=enabled -Dzlib=enabled \
  -Dlua=disabled -Djavascript=disabled -Dvapoursynth=disabled -Dlibarchive=disabled \
  -Drubberband=disabled -Dlibbluray=disabled -Ddvdnav=disabled -Dcdda=disabled -Ddvbin=disabled \
  -Dlibavdevice=disabled -Djpeg=disabled -Dsixel=disabled -Dcaca=disabled \
  -Dvulkan=disabled -Dshaderc=disabled -Dspirv-cross=disabled -Dsdl2-audio=disabled -Dsdl2-video=disabled
meson compile -C "$work/mpv-build"
meson install -C "$work/mpv-build"

echo "== MpvQt $MPVQT_VERSION"
git clone --quiet --depth 1 --branch "$MPVQT_VERSION" https://invent.kde.org/libraries/mpvqt.git "$work/mpvqt"
cmake -S "$work/mpvqt" -B "$work/mpvqt-build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$prefix" \
  -DCMAKE_INSTALL_LIBDIR=lib \
  -DCMAKE_PREFIX_PATH="$prefix;$brew_prefix" \
  -DBUILD_TESTING=OFF
cmake --build "$work/mpvqt-build"
cmake --install "$work/mpvqt-build"

echo "== installed"
ls -la "$prefix/lib" "$prefix/include"
otool -L "$prefix/lib/libmpv.dylib" "$prefix/lib/libMpvQt.dylib"
