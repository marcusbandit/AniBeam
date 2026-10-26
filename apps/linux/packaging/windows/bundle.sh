#!/usr/bin/env bash
# usage: bundle.sh <out-dir>
# Turns target/release/anibeam.exe into <out-dir>/AniBeam (the folder the installer copies),
# <out-dir>/AniBeam-<ver>-windows-x64-setup.exe and a portable <out-dir>/AniBeam-<ver>-windows-x64.zip.
#   ANIBEAM_BIN   the binary to bundle (default: the workspace's target/release/anibeam.exe)
#   DEPS_PREFIX   where deps.sh put libmpv, MpvQt and qbittorrent-nox
# Runs in Git Bash on the runner, with Qt's bin (windeployqt) on PATH. The folder holds the
# app, Qt, the two Downloads helpers (qbittorrent-nox.exe beside the app, anirss with its own
# Python under anirss\) and what /usr/share/anibeam holds on Arch: themes\ and mpv.conf.
set -euo pipefail
export MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*'

here="$(cd "$(dirname "$0")" && pwd)"
shell="$(cd "$here/../.." && pwd)"
repo="$(cd "$shell/../.." && pwd)"
mkdir -p "$1"
out="$(cd "$1" && pwd)"
bin="${ANIBEAM_BIN:-$repo/target/release/anibeam.exe}"
deps="${DEPS_PREFIX:?DEPS_PREFIX names the prefix deps.sh built into}"

# The PKGBUILD's pkgver rule: v2.0.0 -> 2.0.0, v2.0.0-21-gabc1234 -> 2.0.0.r21.gabc1234.
describe="$(git -C "$repo" describe --tags --match 'v*' --dirty 2>/dev/null || echo v0.0.0)"
pkgver="$(sed -E 's/^v//; s/-([0-9]+)-g/.r\1.g/; s/-dirty$/.dirty/' <<<"$describe")"
short="${pkgver%%.r*}"
short="${short%.dirty}"

app="$out/AniBeam"
rm -rf "$app"
mkdir -p "$app"
cp "$bin" "$app/anibeam.exe"
cp "$deps/bin/qbittorrent-nox.exe" "$app/"
cp "$deps"/bin/*.dll "$app/"
cp "$here/anibeam.ico" "$app/anibeam.ico"
cp -R "$shell/themes" "$app/themes"
cp "$shell/mpv.conf" "$app/mpv.conf"

echo "== windeployqt"
# The app's own QML module is compiled into the exe; --qmldir is how windeployqt learns which
# Qt Quick modules those files import. qbittorrent-nox adds Sql, Xml and the TLS and SQLite
# plugins; the libraries it shares with the app are copied once.
windeployqt --release --no-translations --no-system-d3d-compiler --compiler-runtime \
  --qmldir "$shell/qml" --dir "$app" "$app/anibeam.exe"
windeployqt --release --no-translations --no-system-d3d-compiler --compiler-runtime \
  --dir "$app" "$app/qbittorrent-nox.exe"
test -e "$app/plugins/sqldrivers/qsqlite.dll" || test -e "$app/sqldrivers/qsqlite.dll" || {
  echo "windeployqt left out the Qt SQLite driver qbittorrent-nox needs" >&2
  exit 1
}
# windeployqt's --compiler-runtime drops the installer for the runtime, not the DLLs; the DLLs
# beside the exe are what a machine without the redistributable needs.
rm -f "$app"/vc_redist*.exe
redist="$(cygpath -u "${VCToolsRedistDir:?run inside an MSVC developer environment}")"
cp "$redist"/x64/Microsoft.VC*.CRT/*.dll "$app/"

echo "== anirss and its Python"
"$here/anirss.sh" "$app"

echo "== the app starts"
"$app/anibeam.exe" --version

echo "== zip"
zip="$out/AniBeam-$pkgver-windows-x64.zip"
rm -f "$zip"
(cd "$out" && 7z a -tzip -mx=7 "$zip" AniBeam > /dev/null)

echo "== installer"
iscc="${ISCC:-C:/Program Files (x86)/Inno Setup 6/ISCC.exe}"
"$iscc" /Q "/DAppVersion=$short" "/DFullVersion=$pkgver" "/DSourceDir=$(cygpath -w "$app")" \
  "/DOutputDir=$(cygpath -w "$out")" "/DOutputName=AniBeam-$pkgver-windows-x64-setup" \
  "$(cygpath -w "$here/anibeam.iss")"
du -sh "$app" "$zip" "$out/AniBeam-$pkgver-windows-x64-setup.exe"
