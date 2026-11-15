#!/usr/bin/env bash
# usage: deps.sh <prefix>
# Builds what Ubuntu 24.04 does not ship the way Anibeam needs it into <prefix>:
#   libva 2.24.1, which the AppImage bundles. A newer libva loads older drivers and never
#     the reverse, and noble ships 2.20, so the build that leaves the machine carries the
#     newest. The driver directory is left to the host (see appimage.sh's apprun hook).
#   FFmpeg 6.1.6, lean, the same 6.1 line and sonames noble ships, so mpv sees the ABI it
#     always had. Noble's own build links every encoder and filter library Ubuntu carries,
#     and linuxdeploy copies whatever libavcodec and libavfilter name, so the AppImage held
#     x265, SVT-AV1, rav1e, aom, x264, vpx, jxl, codec2 and librsvg, the flite voices,
#     pocketsphinx, lapack, gfortran and fftw, and libxml2 with a second ICU (74 beside Qt's
#     73): 125 MB unpacked that a player never runs (#850). This build keeps FFmpeg's own
#     decoders, demuxers, parsers, protocols, muxers, encoders and filters, all of which
#     need no outside library, and links only what playback uses:
#       dav1d             AV1, as noble's did
#       VA-API, libdrm    hardware decoding on Intel and AMD, against the libva built here
#       nvdec, cuvid      hardware decoding on NVIDIA, through noble's ffnvcodec headers
#       gnutls            https
#       libwebp           webp screenshots; jpg (mpv's default) and png are FFmpeg's own
#       zlib, bzlib       compressed Matroska tracks and png
#     Left out on purpose: every outside encoder, the filters that need an outside library
#     (mpv draws subtitles with its own libass and never asks lavfi for them), libxml2, which
#     only the DASH demuxer wants, libavdevice and libpostproc, which mpv here does not
#     link, and VDPAU, which mpv here is built without. A few outside decoders go with them,
#     ones FFmpeg has no version of its own for: libzvbi's DVB teletext subtitles, speex,
#     codec2, game music (gme) and tracker modules (openmpt), and lzma-packed TIFF. Xlib is
#     off too: mpv hands FFmpeg the VA-API display it opened itself, on X11 and Wayland alike.
#   libmpv 0.41.0, lean: the OpenGL render API, VA-API, ALSA and PulseAudio, libass, and
#     none of Lua, JavaScript, VapourSynth, Vulkan or libarchive. It builds against the
#     FFmpeg above and noble's libplacebo 6.338.2.
#   extra-cmake-modules 6.20.0, since noble's is 5.115 and MpvQt wants 6.15 or newer. The
#     6.31 line needs a newer cmake than noble's.
#   MpvQt 1.2.0, which Ubuntu does not package at all.
# Qt itself comes from install-qt-action: noble's 6.4 is too old, MpvQt needs 6.9 or newer.
# The caller exports QMAKE (the Qt root's bin/qmake). The workflow caches <prefix> on a key
# made from this file, which names FFmpeg's version and flags, and the apt versions of what
# FFmpeg and libmpv link, libplacebo among them.
set -euo pipefail

prefix="$1"
: "${QMAKE:?QMAKE names the qmake of the Qt that install-qt-action put in place}"
FFMPEG_VERSION=6.1.6
FFMPEG_SHA=d4fcb164028dd3beee5d92c0ac72e46aac6973c75ea12dc14de07bf8f407370a
MPV_VERSION=0.41.0
LIBVA_VERSION=2.24.1
MPVQT_VERSION=v1.2.0
ECM_VERSION=6.20.0
export PKG_CONFIG_PATH="$prefix/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$prefix"

echo "== libva $LIBVA_VERSION"
curl -fsSL "https://github.com/intel/libva/releases/download/$LIBVA_VERSION/libva-$LIBVA_VERSION.tar.bz2" | tar -xj -C "$work"
meson setup "$work/libva-build" "$work/libva-$LIBVA_VERSION" --prefix="$prefix" --libdir=lib --buildtype=release \
  -Dwith_x11=yes -Dwith_wayland=yes -Dwith_glx=no -Ddriverdir=/usr/lib/x86_64-linux-gnu/dri
meson install -C "$work/libva-build"

echo "== FFmpeg $FFMPEG_VERSION"
curl -fsSL "https://ffmpeg.org/releases/ffmpeg-$FFMPEG_VERSION.tar.xz" -o "$work/ffmpeg.tar.xz"
echo "$FFMPEG_SHA  $work/ffmpeg.tar.xz" | sha256sum -c --quiet -
tar -xJf "$work/ffmpeg.tar.xz" -C "$work"
mkdir -p "$work/ffmpeg-build"
# --disable-autodetect keeps configure from picking up whatever else the runner has
# installed; each library below is asked for by name. --disable-debug matters: appimage.sh
# bundles libraries unstripped, and FFmpeg builds with -g unless told not to.
( cd "$work/ffmpeg-build" && "$work/ffmpeg-$FFMPEG_VERSION/configure" \
    --prefix="$prefix" --libdir="$prefix/lib" \
    --enable-shared --disable-static --enable-pic --disable-debug \
    --disable-programs --disable-doc --disable-avdevice --disable-postproc \
    --disable-autodetect \
    --enable-zlib --enable-bzlib --enable-iconv \
    --enable-vaapi --enable-libdrm \
    --enable-ffnvcodec --enable-cuda --enable-cuvid --enable-nvdec \
    --enable-libdav1d --enable-gnutls --enable-libwebp )
make -C "$work/ffmpeg-build" -j"$(nproc)"
make -C "$work/ffmpeg-build" install

echo "== libmpv $MPV_VERSION"
curl -fsSL "https://github.com/mpv-player/mpv/archive/refs/tags/v$MPV_VERSION.tar.gz" | tar -xz -C "$work"
meson setup "$work/mpv-build" "$work/mpv-$MPV_VERSION" \
  --prefix="$prefix" --libdir=lib --buildtype=release \
  -Dlibmpv=true -Dcplayer=false -Dtests=false -Dbuild-date=false \
  -Dmanpage-build=disabled -Dhtml-build=disabled -Dpdf-build=disabled \
  -Dgl=enabled -Degl=enabled -Degl-x11=enabled -Degl-wayland=enabled -Dplain-gl=enabled \
  -Dx11=enabled -Dwayland=enabled -Dvaapi=enabled -Dvdpau=disabled -Dvulkan=disabled \
  -Dalsa=enabled -Dpulse=enabled -Dpipewire=disabled -Djack=disabled -Dopenal=disabled -Dsdl2-audio=disabled \
  -Dlcms2=enabled -Duchardet=enabled -Dzimg=enabled -Diconv=enabled -Dzlib=enabled \
  -Dlua=disabled -Djavascript=disabled -Dvapoursynth=disabled -Dlibarchive=disabled \
  -Drubberband=disabled -Dlibbluray=disabled -Ddvdnav=disabled -Dcdda=disabled -Ddvbin=disabled \
  -Dlibavdevice=disabled -Djpeg=disabled -Dsixel=disabled -Dcaca=disabled -Dshaderc=disabled -Dspirv-cross=disabled \
  -Ddrm=enabled -Dgbm=enabled -Dsdl2-video=disabled
meson compile -C "$work/mpv-build"
meson install -C "$work/mpv-build"

echo "== extra-cmake-modules $ECM_VERSION"
curl -fsSL "https://download.kde.org/stable/frameworks/${ECM_VERSION%.*}/extra-cmake-modules-$ECM_VERSION.tar.xz" | tar -xJ -C "$work"
cmake -S "$work/extra-cmake-modules-$ECM_VERSION" -B "$work/ecm-build" -G Ninja \
  -DCMAKE_INSTALL_PREFIX="$prefix" -DBUILD_TESTING=OFF -DBUILD_HTML_DOCS=OFF -DBUILD_MAN_DOCS=OFF -DBUILD_QTHELP_DOCS=OFF
cmake --install "$work/ecm-build"

echo "== MpvQt $MPVQT_VERSION"
# FindLibmpv hands CMake <prefix>/include/mpv while the sources say <mpv/client.h>; on Arch
# /usr/include covers the difference, here the flag below does.
git clone --quiet --depth 1 --branch "$MPVQT_VERSION" https://invent.kde.org/libraries/mpvqt.git "$work/mpvqt"
qtroot="$("$QMAKE" -query QT_INSTALL_PREFIX)"
cmake -S "$work/mpvqt" -B "$work/mpvqt-build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$prefix" \
  -DCMAKE_INSTALL_LIBDIR=lib \
  -DCMAKE_PREFIX_PATH="$prefix;$qtroot" \
  -DCMAKE_CXX_FLAGS="-isystem $prefix/include" \
  -DBUILD_TESTING=OFF
cmake --build "$work/mpvqt-build"
cmake --install "$work/mpvqt-build"

echo "== installed"
ls -la "$prefix/lib" "$prefix/include"
LD_LIBRARY_PATH="$prefix/lib" ldd "$prefix/lib/libmpv.so" | grep -E "va|drm|wayland|X11|GL|EGL|placebo|ass|avcodec"
echo "== what FFmpeg links"
for lib in avcodec avformat avfilter avutil; do
  echo "lib$lib: $(readelf -d "$prefix/lib/lib$lib.so" | sed -n 's/.*NEEDED.*\[\(.*\)\]/\1/p' | tr '\n' ' ')"
done
