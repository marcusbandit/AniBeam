#!/usr/bin/env bash
# usage: appimage.sh <out-dir>
# Turns target/release/anibeam into <out-dir>/AniBeam-<pkgver>-x86_64.AppImage.
#   ANIBEAM_BIN   the binary to bundle (default: the workspace's target/release/anibeam)
#   DEPS_PREFIX   where deps.sh installed libva, libmpv and MpvQt
#   QMAKE         the qmake of the Qt the binary was built against
# Two helpers ride along, each pinned by version and sha256 below:
#   usr/share/anibeam/anirss/    Python 3.13 (python-build-standalone) and anirss, which needs 3.11
#   usr/bin/qbittorrent-nox      the fully static build from userdocs
# linuxdeploy runs first and sees only the shell; the helpers are added after it, so it
# never rewrites the Python tree's rpaths or goes looking for a static binary's libraries.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
shell="$(cd "$here/../.." && pwd)"
repo="$(cd "$shell/../.." && pwd)"
mkdir -p "$1"
out="$(cd "$1" && pwd)"
bin="${ANIBEAM_BIN:-$repo/target/release/anibeam}"
deps="${DEPS_PREFIX:?DEPS_PREFIX names the prefix deps.sh built into}"
: "${QMAKE:?QMAKE names the qmake of the Qt the binary was built against}"
downloads="${ANIBEAM_DOWNLOADS:-$out/downloads}"

LINUXDEPLOY_TAG=1-alpha-20251107-1
LINUXDEPLOY_SHA=c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d
QT_PLUGIN_TAG=1-alpha-20250213-1
QT_PLUGIN_SHA=15106be885c1c48a021198e7e1e9a48ce9d02a86dd0a1848f00bdbf3c1c92724
APPIMAGE_PLUGIN_TAG=1-alpha-20250213-1
APPIMAGE_PLUGIN_SHA=992d502a248e14ab185448ddf6f6e7d25558cb84d4623c354c3af350c25fccb3
PYTHON_TAG=20261013
PYTHON_FILE=cpython-3.13.16+20261013-x86_64-unknown-linux-gnu-install_only_stripped.tar.gz
PYTHON_SHA=d0080766b6b74aeb81e1949ef0d9a20e6c17f7b160fb295a69b936ccb17660be
ANIRSS_VERSION=0.4.1
ANIRSS_SHA=41abedaa11ae5955081d2883c8e0d6723334ba08f2c5d1d1c882ab94e903d2f3
QBT_TAG=release-5.2.4_v2.0.15
QBT_SHA=14d29323f1c12c1e8892ac69496f8b51ddebb74b4b590fc13e343aa213fecaf4

# The PKGBUILD's pkgver rule: v2.0.0 -> 2.0.0, v2.0.0-21-gabc1234 -> 2.0.0.r21.gabc1234.
describe="$(git -C "$repo" describe --tags --dirty 2>/dev/null || echo v0.0.0)"
pkgver="$(sed -E 's/^v//; s/-([0-9]+)-g/.r\1.g/; s/-dirty$/.dirty/' <<<"$describe")"

# fetch <url> <sha256> <name>: into the downloads directory, checked every time.
fetch() {
  local file="$downloads/$3"
  mkdir -p "$downloads"
  [ -f "$file" ] || curl -fsSL "$1" -o "$file"
  echo "$2  $file" | sha256sum -c --quiet - || { echo "checksum mismatch for $3" >&2; rm -f "$file"; exit 1; }
}

echo "== tools"
tools="$downloads"
fetch "https://github.com/linuxdeploy/linuxdeploy/releases/download/$LINUXDEPLOY_TAG/linuxdeploy-x86_64.AppImage" \
  "$LINUXDEPLOY_SHA" linuxdeploy-x86_64.AppImage
fetch "https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/$QT_PLUGIN_TAG/linuxdeploy-plugin-qt-x86_64.AppImage" \
  "$QT_PLUGIN_SHA" linuxdeploy-plugin-qt-x86_64.AppImage
fetch "https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/$APPIMAGE_PLUGIN_TAG/linuxdeploy-plugin-appimage-x86_64.AppImage" \
  "$APPIMAGE_PLUGIN_SHA" linuxdeploy-plugin-appimage-x86_64.AppImage
# linuxdeploy finds its plugins by name on PATH.
chmod +x "$tools"/*.AppImage

qtroot="$("$QMAKE" -query QT_INSTALL_PREFIX)"
appdir="$out/AppDir"
rm -rf "$appdir"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/anibeam"

# The build carries its debug info, which is hundreds of megabytes the user never needs.
strip -o "$appdir/usr/bin/anibeam" "$bin"
# What /usr/share/anibeam holds on Arch; paths.rs finds it beside the binary's bin.
cp -r "$shell/themes" "$appdir/usr/share/anibeam/themes"
cp "$shell/mpv.conf" "$appdir/usr/share/anibeam/mpv.conf"

# linuxdeploy-plugin-qt does not copy these three, and a Wayland session needs them.
mkdir -p "$appdir/usr/plugins"
cp -r "$qtroot"/plugins/wayland-{decoration-client,graphics-integration-client,shell-integration} "$appdir/usr/plugins/"

# Sourced by AppRun. The bundled libva looks for drivers in the Debian multiarch directory
# only, so hand it every driver directory the host has (Fedora lib64, Arch lib).
mkdir -p "$appdir/apprun-hooks"
cat > "$appdir/apprun-hooks/anibeam-hook.sh" <<'HOOK'
if [ -z "${LIBVA_DRIVERS_PATH:-}" ]; then
  for d in /usr/lib/x86_64-linux-gnu/dri /usr/lib64/dri /usr/lib/dri; do
    [ -d "$d" ] && LIBVA_DRIVERS_PATH="${LIBVA_DRIVERS_PATH:+$LIBVA_DRIVERS_PATH:}$d"
  done
  export LIBVA_DRIVERS_PATH
fi
HOOK

echo "== linuxdeploy"
export APPIMAGE_EXTRACT_AND_RUN=1 # a CI runner and a container have no FUSE
export PATH="$tools:$PATH"
export LD_LIBRARY_PATH="$deps/lib:$qtroot/lib"
export QML_SOURCES_PATH="$shell/qml" QML_SOURCES_PATHS="$shell/qml"
export EXTRA_PLATFORM_PLUGINS='libqwayland.so'
export EXTRA_QT_MODULES='waylandclient'
export NO_STRIP=1 # the bundled strip is older than the toolchains that built the libraries
export LINUXDEPLOY_OUTPUT_VERSION="$pkgver"
cd "$out"
"$tools/linuxdeploy-x86_64.AppImage" --appdir "$appdir" \
  -e "$appdir/usr/bin/anibeam" \
  -d "$shell/com.marcusrosado.AniBeam.desktop" \
  -i "$shell/assets/icon.png" --icon-filename com.marcusrosado.AniBeam \
  --deploy-deps-only "$appdir/usr/plugins/wayland-decoration-client" \
  --deploy-deps-only "$appdir/usr/plugins/wayland-graphics-integration-client" \
  --deploy-deps-only "$appdir/usr/plugins/wayland-shell-integration" \
  --plugin qt > "$out/linuxdeploy.log" 2>&1 || { tail -60 "$out/linuxdeploy.log"; exit 1; }

echo "== qbittorrent-nox $QBT_TAG"
fetch "https://github.com/userdocs/qbittorrent-nox-static/releases/download/$QBT_TAG/x86_64-qbittorrent-nox" \
  "$QBT_SHA" qbittorrent-nox
install -m 755 "$downloads/qbittorrent-nox" "$appdir/usr/bin/qbittorrent-nox"

echo "== anirss $ANIRSS_VERSION on Python 3.13"
anirss="$appdir/usr/share/anibeam/anirss"
mkdir -p "$anirss/app"
fetch "https://github.com/astral-sh/python-build-standalone/releases/download/$PYTHON_TAG/$PYTHON_FILE" "$PYTHON_SHA" python.tar.gz
tar -xzf "$downloads/python.tar.gz" -C "$anirss"
# anirss imports the standard library and nothing else. Leave out what only a developer
# of Python itself, or a Tk window, would touch.
py="$anirss/python"
rm -rf "$py/include" "$py/share" "$py"/lib/tcl* "$py"/lib/tk* "$py"/lib/itcl* "$py"/lib/thread* \
  "$py"/lib/python3.13/{test,idlelib,tkinter,turtledemo,ensurepip,config-3.13-x86_64-linux-gnu} \
  "$py"/lib/python3.13/lib-dynload/_tkinter* "$py"/lib/python3.13/turtle.py \
  "$py"/lib/libpython3.13.a "$py"/bin/idle* "$py"/bin/pip* "$py"/bin/pydoc* "$py"/bin/python3*-config
find "$py" -name __pycache__ -type d -prune -exec rm -rf {} +
fetch "https://github.com/marcusbandit/anirss/archive/refs/tags/v$ANIRSS_VERSION.tar.gz" "$ANIRSS_SHA" anirss.tar.gz
src="$(mktemp -d)"
trap 'rm -rf "$src"' EXIT
tar -xzf "$downloads/anirss.tar.gz" -C "$src"
cp "$src/anirss-$ANIRSS_VERSION/anirss" "$anirss/app/anirss"
cp -r "$src/anirss-$ANIRSS_VERSION/anirss_lib" "$anirss/app/anirss_lib"
find "$anirss/app" -name __pycache__ -type d -prune -exec rm -rf {} +
mkdir -p "$anirss/bin"
cat > "$anirss/bin/anirss" <<'LAUNCHER'
#!/bin/sh
# AniBeam's own anirss: the release it was built with, on the Python beside it.
here="$(cd "$(dirname "$0")/.." && pwd)"
exec "$here/python/bin/python3" "$here/app/anirss" "$@"
LAUNCHER
chmod 755 "$anirss/bin/anirss"

echo "== appimage"
# The plugin linuxdeploy's --output appimage calls, run by hand now the helpers are in.
rm -f "$out"/*.AppImage
"$tools/linuxdeploy-plugin-appimage-x86_64.AppImage" --appdir "$appdir" > "$out/appimage-plugin.log" 2>&1 \
  || { tail -40 "$out/appimage-plugin.log"; exit 1; }
image="$out/AniBeam-$pkgver-x86_64.AppImage"
[ -f "$image" ] || { echo "no $image; the plugin wrote:" >&2; ls -la "$out" >&2; tail -20 "$out/appimage-plugin.log" >&2; exit 1; }
chmod +x "$image"
ls -la "$image"
echo "$image"
