#!/usr/bin/env bash
# usage: packages.sh <AniBeam.AppImage> <version> <out-dir>
# Turns the AppImage appimage.sh built into the two packages the apt and dnf repos serve:
#   <out-dir>/anibeam_<pkgver>_amd64.deb
#   <out-dir>/anibeam-<pkgver>-1.x86_64.rpm
# <version> is what the build ships as: 2.4.0, or 2.4.0-nightly.20261112.40 for a
# pre-release. A package version has no hyphen, so the first becomes a tilde: apt and rpm
# both sort 2.4.0~nightly.20261112.40 before 2.4.0.
# The AppImage is unpacked (--appimage-extract needs no FUSE) into /opt/anibeam, and
# /usr/bin/anibeam (anibeam.sh) launches it. nfpm.yaml holds the metadata and the
# dependency lists. nfpm is pinned by version and sha256 below, fetched the way
# appimage.sh fetches its tools.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
[ $# -eq 3 ] || { echo "usage: $0 <AniBeam.AppImage> <version> <out-dir>" >&2; exit 2; }
image="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
version="${2#v}"
mkdir -p "$3"
out="$(cd "$3" && pwd)"
downloads="${ANIBEAM_DOWNLOADS:-$out/downloads}"
[ -f "$image" ] || { echo "no AppImage at $image" >&2; exit 1; }

NFPM_VERSION=2.49.0
NFPM_SHA=2afaa5ffb5e3493402e31e7c67a5a1fb42133e3f66fc315ca0051934125561c7

pkgver="${version//-/\~}"

# fetch <url> <sha256> <name>: into the downloads directory, checked every time.
fetch() {
  local file="$downloads/$3"
  mkdir -p "$downloads"
  [ -f "$file" ] || curl -fsSL "$1" -o "$file"
  echo "$2  $file" | sha256sum -c --quiet - || { echo "checksum mismatch for $3" >&2; rm -f "$file"; exit 1; }
}

echo "== nfpm $NFPM_VERSION"
fetch "https://github.com/goreleaser/nfpm/releases/download/v$NFPM_VERSION/nfpm_${NFPM_VERSION}_Linux_x86_64.tar.gz" \
  "$NFPM_SHA" "nfpm-$NFPM_VERSION.tar.gz"
nfpm_dir="$downloads/nfpm-$NFPM_VERSION"
if [ ! -x "$nfpm_dir/nfpm" ]; then
  rm -rf "$nfpm_dir"
  mkdir -p "$nfpm_dir"
  tar -xzf "$downloads/nfpm-$NFPM_VERSION.tar.gz" -C "$nfpm_dir" nfpm
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

echo "== unpacking $(basename "$image")"
( cd "$work" && "$image" --appimage-extract > /dev/null ) || { echo "could not unpack the image" >&2; exit 1; }
appdir="$work/squashfs-root"
[ -x "$appdir/usr/bin/anibeam" ] || { echo "no usr/bin/anibeam in the image" >&2; exit 1; }

tree="$work/tree"
mkdir -p "$tree/opt" "$tree/usr/bin" "$tree/usr/share/applications" \
  "$tree/usr/share/icons/hicolor/512x512/apps"
mv "$appdir" "$tree/opt/anibeam"
app="$tree/opt/anibeam"
# The AppImage's own entry points: AppRun and the top-level desktop file and icon links are
# replaced by /usr/bin/anibeam and the files under /usr/share.
cp "$app/com.marcusrosado.AniBeam.desktop" "$tree/usr/share/applications/"
cp -L "$app/usr/share/icons/hicolor/512x512/apps/com.marcusrosado.AniBeam.png" \
  "$tree/usr/share/icons/hicolor/512x512/apps/"
rm -f "$app/AppRun" "$app/AppRun.wrapped" "$app/.DirIcon" \
  "$app/com.marcusrosado.AniBeam.desktop" "$app/com.marcusrosado.AniBeam.png"
# The desktop file runs the command on PATH, with the anibeam:// link it was opened for; the
# version stamp is the AppImage tooling's.
desktop="$tree/usr/share/applications/com.marcusrosado.AniBeam.desktop"
sed -i -e '/^X-AppImage-/d' -e 's/^Exec=.*/Exec=anibeam %u/' "$desktop"
install -m 755 "$here/anibeam.sh" "$tree/usr/bin/anibeam"
# squashfs-root comes out drwx------; every user has to read the install.
chmod -R u+rwX,go+rX,go-w "$tree"

echo "== packages $pkgver"
export ANIBEAM_PKG_VERSION="$pkgver"
cd "$tree" # nfpm.yaml names the files relative to the tree
rm -f "$out"/anibeam_*_amd64.deb "$out"/anibeam-*.x86_64.rpm
ANIBEAM_PKG_RELEASE="" "$nfpm_dir/nfpm" package --config "$here/nfpm.yaml" --packager deb \
  --target "$out/anibeam_${pkgver}_amd64.deb" > /dev/null
ANIBEAM_PKG_RELEASE=1 "$nfpm_dir/nfpm" package --config "$here/nfpm.yaml" --packager rpm \
  --target "$out/anibeam-${pkgver}-1.x86_64.rpm" > /dev/null
ls -la "$out"/anibeam_"${pkgver}"_amd64.deb "$out/anibeam-${pkgver}-1.x86_64.rpm"
