#!/usr/bin/env bash
# usage: deps.sh <prefix>
# Puts what the Windows build links and ships into <prefix>, the counterpart of the Mac's
# deps.sh. Runs in Git Bash on a windows-2025 runner, inside an MSVC developer environment
# (cl, link, lib and dumpbin on PATH) and with Qt's bin on PATH:
#   libmpv       shinchiro's mpv-winbuild-cmake dev package: one libmpv-2.dll with FFmpeg and
#                libass inside, plus the headers. It carries no MSVC import library, so mpv.lib
#                is made here from the DLL's own exports.
#   MpvQt        built from its tag against that libmpv, with extra-cmake-modules first.
#   qbittorrent-nox  qBittorrent has no headless build for Windows, so it is built here the
#                way its own Windows CI builds the GUI one, with the GUI off: OpenSSL and zlib
#                static from vcpkg, boost headers, libtorrent static.
set -euo pipefail
# Git Bash rewrites anything that looks like a path, /nologo and /def:x included. Its tar
# reads C:/... as a remote host, hence --force-local below.
export MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*'

MPV_RELEASE=20261021
MPV_ASSET=mpv-dev-x86_64-20261021-git-e471496c56.7z
MPV_SHA256=efc719eff3dc5954b4a5efc800632808bcec01cc9034940dd14e5eb7fe9ea633
ECM_VERSION=v6.31.0
MPVQT_VERSION=v1.2.0
BOOST_VERSION=1.91.0
LIBTORRENT_VERSION=2.0.15
LIBTORRENT_SHA256=5e2e79129823b7ea48721164c32b5aaf83d3fd733b5502100f6705b29f27bb02
QBT_VERSION=5.2.4
QBT_SHA256=226a6ad1aa7810fcbfa58b0d28b14feb8e207726cbe774d4d267ce769c19dd6c
VCPKG="${VCPKG_INSTALLATION_ROOT:-C:/vcpkg}"
TRIPLET=x64-windows-static-md-release

mkdir -p "$1"
prefix="$(cygpath -m -l "$(cd "$1" && pwd)")"
# The long form: the runner's temp is C:/Users/RUNNER~1/..., and CMake then fails to match
# a file's OUTPUT_LOCATION to its glob, so qBittorrent's .qm files land in the wrong place.
work="$(cygpath -m -l "$(mktemp -d)")"
mkdir -p "$prefix/bin" "$prefix/lib" "$prefix/include"
qt="$(cygpath -m "${QT_ROOT_DIR:?QT_ROOT_DIR names the Qt install-qt-action set up}")"

fetch() { # url sha256 out
  curl -fsSL --retry 3 -o "$3" "$1"
  echo "$2  $3" | sha256sum -c -
}

echo "== libmpv ($MPV_ASSET)"
fetch "https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/$MPV_RELEASE/$MPV_ASSET" \
  "$MPV_SHA256" "$work/mpv.7z"
7z x -y -o"$work/mpv" "$work/mpv.7z" > /dev/null
cp -R "$work/mpv/include/mpv" "$prefix/include/"
cp "$work/mpv/libmpv-2.dll" "$prefix/bin/"
# Only the mpv_ names: the DLL also exports libbluray's Java_ entry points.
{
  echo "LIBRARY libmpv-2.dll"
  echo "EXPORTS"
  dumpbin /nologo /exports "$prefix/bin/libmpv-2.dll" | awk '$4 ~ /^mpv_/ {print "    " $4}'
} > "$work/mpv.def"
grep -q mpv_render_context_create "$work/mpv.def"
lib /nologo /def:"$work/mpv.def" /machine:x64 /out:"$prefix/lib/mpv.lib"

echo "== extra-cmake-modules $ECM_VERSION"
git clone --quiet --depth 1 --branch "$ECM_VERSION" https://invent.kde.org/frameworks/extra-cmake-modules.git "$work/ecm"
cmake -S "$work/ecm" -B "$work/ecm-build" -G Ninja -DCMAKE_INSTALL_PREFIX="$prefix" \
  -DBUILD_TESTING=OFF -DBUILD_HTML_DOCS=OFF -DBUILD_MAN_DOCS=OFF -DBUILD_QTHELP_DOCS=OFF
cmake --install "$work/ecm-build"

echo "== MpvQt $MPVQT_VERSION"
# FindLibmpv hands CMake <prefix>/include/mpv while the sources say <mpv/client.h>, so the
# prefix's include directory goes in by hand, as on the Mac.
git clone --quiet --depth 1 --branch "$MPVQT_VERSION" https://invent.kde.org/libraries/mpvqt.git "$work/mpvqt"
cmake -S "$work/mpvqt" -B "$work/mpvqt-build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$prefix" \
  -DCMAKE_INSTALL_LIBDIR=lib \
  -DCMAKE_PREFIX_PATH="$qt;$prefix" \
  -DCMAKE_CXX_FLAGS="/EHsc /I$prefix/include" \
  -DBUILD_TESTING=OFF
cmake --build "$work/mpvqt-build"
cmake --install "$work/mpvqt-build"

echo "== OpenSSL and zlib from vcpkg ($TRIPLET)"
mkdir -p "$work/triplets"
cat > "$work/triplets/$TRIPLET.cmake" <<CMAKE
set(VCPKG_TARGET_ARCHITECTURE x64)
set(VCPKG_LIBRARY_LINKAGE static)
set(VCPKG_CRT_LINKAGE dynamic)
set(VCPKG_BUILD_TYPE release)
CMAKE
"$VCPKG/vcpkg.exe" install --clean-after-build --overlay-triplets="$work/triplets" \
  "openssl:$TRIPLET" "zlib:$TRIPLET"
toolchain="$VCPKG/scripts/buildsystems/vcpkg.cmake"

echo "== boost $BOOST_VERSION headers"
boost_tar="boost_${BOOST_VERSION//./_}"
curl -fsSL --retry 3 -o "$work/boost.tar.gz" \
  "https://archives.boost.io/release/$BOOST_VERSION/source/$boost_tar.tar.gz"
tar --force-local -xzf "$work/boost.tar.gz" -C "$work"
boost="$work/$boost_tar"

echo "== libtorrent-rasterbar $LIBTORRENT_VERSION"
fetch "https://github.com/arvidn/libtorrent/releases/download/v$LIBTORRENT_VERSION/libtorrent-rasterbar-$LIBTORRENT_VERSION.tar.gz" \
  "$LIBTORRENT_SHA256" "$work/libtorrent.tar.gz"
tar --force-local -xzf "$work/libtorrent.tar.gz" -C "$work"
cmake -S "$work/libtorrent-rasterbar-$LIBTORRENT_VERSION" -B "$work/libtorrent-build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_CXX_STANDARD=20 \
  -DCMAKE_INSTALL_PREFIX="$work/libtorrent" \
  -DCMAKE_TOOLCHAIN_FILE="$toolchain" \
  -DVCPKG_TARGET_TRIPLET="$TRIPLET" \
  -DVCPKG_OVERLAY_TRIPLETS="$work/triplets" \
  -DBOOST_ROOT="$boost" -DBoost_INCLUDE_DIR="$boost" \
  -DBUILD_SHARED_LIBS=OFF -Ddeprecated-functions=OFF -Dstatic_runtime=OFF \
  -Dpython-bindings=OFF -Dbuild_tests=OFF -Dbuild_examples=OFF -Dbuild_tools=OFF
cmake --build "$work/libtorrent-build"
cmake --install "$work/libtorrent-build"

echo "== qBittorrent $QBT_VERSION (nox)"
fetch "https://github.com/qbittorrent/qBittorrent/archive/refs/tags/release-$QBT_VERSION.tar.gz" \
  "$QBT_SHA256" "$work/qbittorrent.tar.gz"
tar --force-local -xzf "$work/qbittorrent.tar.gz" -C "$work"
cmake -S "$work/qBittorrent-release-$QBT_VERSION" -B "$work/qbt-build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_PREFIX_PATH="$qt" \
  -DCMAKE_TOOLCHAIN_FILE="$toolchain" \
  -DVCPKG_TARGET_TRIPLET="$TRIPLET" \
  -DVCPKG_OVERLAY_TRIPLETS="$work/triplets" \
  -DBOOST_ROOT="$boost" -DBoost_INCLUDE_DIR="$boost" \
  -DLibtorrentRasterbar_DIR="$work/libtorrent/lib/cmake/LibtorrentRasterbar" \
  -DMSVC_RUNTIME_DYNAMIC=ON \
  -DGUI=OFF -DWEBUI=ON -DSTACKTRACE=OFF -DTESTING=OFF
# The .qm files the two .qrc lists name are made here with lrelease, at the paths the build
# reads them from. CMake's own rule for them puts them elsewhere on Windows, and the build
# then stops on "webui_uk.qm ... missing and no known rule to make it".
qbt_src="$work/qBittorrent-release-$QBT_VERSION"
for pair in "src/lang:src/lang" "src/webui/www/translations:src/webui/www/translations"; do
  from="$qbt_src/${pair%%:*}" to="$work/qbt-build/${pair##*:}"
  mkdir -p "$to"
  for ts in "$from"/*.ts; do
    name="$(basename "$ts" .ts)"
    "$qt/bin/lrelease.exe" -removeidentical -silent "$ts" -qm "$to/$name.qm"
  done
done
cmake --build "$work/qbt-build"
cp "$work/qbt-build/qbittorrent-nox.exe" "$prefix/bin/"

echo "== installed"
ls -la "$prefix/bin" "$prefix/lib" "$prefix/include"
