#!/usr/bin/env bash
# usage: bundle.sh <out-dir>
# Turns target/release/anibeam into <out-dir>/AniBeam.app and <out-dir>/AniBeam-<ver>-macos-arm64.dmg.
#   ANIBEAM_BIN   the binary to bundle (default: the workspace's target/release/anibeam)
#   DEPS_PREFIX   where deps.sh installed libmpv and MpvQt
# The app carries two helpers the Downloads page starts: qbittorrent-nox beside the app's own
# binary (DEPS_PREFIX/bin, built by deps.sh) and anirss with its own Python under
# Resources/anirss (anirss.sh).
# The app is ad-hoc signed: it runs on any Mac, and Gatekeeper asks once (Open Anyway).
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
shell="$(cd "$here/../.." && pwd)"
repo="$(cd "$shell/../.." && pwd)"
out="$1"
bin="${ANIBEAM_BIN:-$repo/target/release/anibeam}"
deps="${DEPS_PREFIX:?DEPS_PREFIX names the prefix deps.sh built into}"
min_macos="${MACOSX_DEPLOYMENT_TARGET:-15.0}"

# CI tells a build what it ships as: 2.4.0, 2.4.0-nightly.20261030.12 or 2.4.0-pr.280.3.
# Otherwise the PKGBUILD's pkgver rule: v2.0.0 -> 2.0.0, v2.0.0-21-gabc1234 ->
# 2.0.0.r21.gabc1234, described from stable tags only.
describe="$(git -C "$repo" describe --tags --match 'v*' --exclude 'v*-*' --dirty 2>/dev/null || echo v0.0.0)"
pkgver="${ANIBEAM_RELEASE_VERSION:-$(sed -E 's/^v//; s/-([0-9]+)-g/.r\1.g/; s/-dirty$/.dirty/' <<<"$describe")}"
short="$(grep -oE '^[0-9]+\.[0-9]+\.[0-9]+' <<<"$pkgver")"

app="$out/AniBeam.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$bin" "$app/Contents/MacOS/anibeam"
cp "$deps/bin/qbittorrent-nox" "$app/Contents/MacOS/qbittorrent-nox"
sed -e "s/@SHORT_VERSION@/$short/" -e "s/@BUILD_VERSION@/$pkgver/" -e "s/@MIN_MACOS@/$min_macos/" \
  "$here/Info.plist.in" > "$app/Contents/Info.plist"
plutil -lint "$app/Contents/Info.plist"
printf 'APPL????' > "$app/Contents/PkgInfo"

# The 512 px PNG at every size an icns carries; the 1024 slot is left out rather than upscaled.
iconset="$(mktemp -d)/AniBeam.iconset"
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$shell/assets/icon.png" --out "$iconset/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  [ "$double" -le 512 ] && sips -z "$double" "$double" "$shell/assets/icon.png" \
    --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/AniBeam.icns"

# What /usr/share/anibeam holds on Arch; paths.rs looks for it in Resources on a Mac.
cp -R "$shell/themes" "$app/Contents/Resources/themes"
cp "$shell/mpv.conf" "$app/Contents/Resources/mpv.conf"

echo "== macdeployqt"
# -executable makes it deploy qbittorrent-nox's Qt frameworks (Core, Network, Sql, Xml) and
# the SQL driver plugin as well; the frameworks it shares with the app are copied once.
macdeployqt "$app" -qmldir="$shell/qml" -libpath="$deps/lib" -no-codesign -verbose=1 \
  -executable="$app/Contents/MacOS/qbittorrent-nox"
for exe in anibeam qbittorrent-nox; do
  if ! otool -l "$app/Contents/MacOS/$exe" | grep -q '@executable_path/../Frameworks'; then
    install_name_tool -add_rpath @executable_path/../Frameworks "$app/Contents/MacOS/$exe"
  fi
done
# Without the SQLite driver qbittorrent-nox starts, then fails on its first database.
test -e "$app/Contents/PlugIns/sqldrivers/libqsqlite.dylib" || {
  echo "macdeployqt left out the Qt SQLite driver qbittorrent-nox needs" >&2
  exit 1
}
# The other drivers want PostgreSQL, MySQL or ODBC libraries, none of which anyone here uses.
find "$app/Contents/PlugIns/sqldrivers" -type f ! -name libqsqlite.dylib -delete

echo "== anirss and its Python"
"$here/anirss.sh" "$app"

echo "== the rest of the libraries"
python3 "$here/fix-dylibs.py" "$app"

echo "== signing"
# A checkout on a Mac can carry extended attributes that cp brings along (Finder info and
# File Provider tags under an iCloud-synced folder), and codesign refuses a bundle with any.
# A runner's fresh clone has none. Out-dirs inside a synced folder can be re-tagged while
# this runs, so bundle into one outside it.
xattr -cr "$app"
# anirss.sh signed the Python tree already; --deep does not treat Mach-O files in Resources
# as nested code.
codesign --force --deep --sign - --timestamp=none "$app"
codesign --verify --deep --strict --verbose=2 "$app"
# Gatekeeper's verdict on an ad-hoc signed app is "rejected", and a runner may report the
# assessment as disabled; it is logged for the record, not checked.
spctl --assess --type execute --verbose=4 "$app" || true

echo "== dmg"
stage="$(mktemp -d)"
cp -R "$app" "$stage/"
ln -s /Applications "$stage/Applications"
dmg="$out/AniBeam-$pkgver-macos-arm64.dmg"
rm -f "$dmg"
hdiutil create -volname AniBeam -srcfolder "$stage" -ov -format UDZO "$dmg"
du -sh "$app" "$dmg"
echo "$dmg"
