#!/usr/bin/env bash
# usage: bundle.sh <out-dir>
# Turns target/release/anibeam into <out-dir>/AniBeam.app and <out-dir>/AniBeam-<ver>-macos-arm64.dmg.
#   ANIBEAM_BIN   the binary to bundle (default: the workspace's target/release/anibeam)
#   DEPS_PREFIX   where deps.sh installed libmpv and MpvQt
# The app is ad-hoc signed: it runs on any Mac, and Gatekeeper asks once (Open Anyway).
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
shell="$(cd "$here/../.." && pwd)"
repo="$(cd "$shell/../.." && pwd)"
out="$1"
bin="${ANIBEAM_BIN:-$repo/target/release/anibeam}"
deps="${DEPS_PREFIX:?DEPS_PREFIX names the prefix deps.sh built into}"
min_macos="${MACOSX_DEPLOYMENT_TARGET:-15.0}"

# The PKGBUILD's pkgver rule: v2.0.0 -> 2.0.0, v2.0.0-21-gabc1234 -> 2.0.0.r21.gabc1234.
describe="$(git -C "$repo" describe --tags --dirty 2>/dev/null || echo v0.0.0)"
pkgver="$(sed -E 's/^v//; s/-([0-9]+)-g/.r\1.g/; s/-dirty$/.dirty/' <<<"$describe")"
short="${pkgver%%.r*}"
short="${short%.dirty}"

app="$out/AniBeam.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$bin" "$app/Contents/MacOS/anibeam"
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
macdeployqt "$app" -qmldir="$shell/qml" -libpath="$deps/lib" -no-codesign -verbose=1
if ! otool -l "$app/Contents/MacOS/anibeam" | grep -q '@executable_path/../Frameworks'; then
  install_name_tool -add_rpath @executable_path/../Frameworks "$app/Contents/MacOS/anibeam"
fi

echo "== the rest of the libraries"
python3 "$here/fix-dylibs.py" "$app"

echo "== signing"
codesign --force --deep --sign - --timestamp=none "$app"
codesign --verify --deep --strict --verbose=2 "$app"

echo "== dmg"
stage="$(mktemp -d)"
cp -R "$app" "$stage/"
ln -s /Applications "$stage/Applications"
dmg="$out/AniBeam-$pkgver-macos-arm64.dmg"
rm -f "$dmg"
hdiutil create -volname AniBeam -srcfolder "$stage" -ov -format UDZO "$dmg"
du -sh "$app" "$dmg"
echo "$dmg"
