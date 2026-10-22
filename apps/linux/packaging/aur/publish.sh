#!/usr/bin/env bash
# usage: publish.sh <version>          e.g. publish.sh 2.1.0
# Points the AUR's anibeam at the v<version> tag and pushes it. Run it after the GitHub
# release is published: the tag's tarball has to exist for its checksum. Needs an AUR
# account whose SSH key ~/.ssh/config hands to aur.archlinux.org.
#   AUR_DRY_RUN=1   do everything but the push, and print the commit
set -euo pipefail

version="${1:?usage: publish.sh <version>}"
here="$(cd "$(dirname "$0")/anibeam" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# The PKGBUILD in the repo, at this version, with the tarball's real checksum.
cp "$here/PKGBUILD" "$work/PKGBUILD"
sed -i -e "s/^pkgver=.*/pkgver=$version/" -e "s/^pkgrel=.*/pkgrel=1/" "$work/PKGBUILD"
(cd "$work" && updpkgsums && makepkg --printsrcinfo > .SRCINFO)
grep -E '^\s*(pkgver|sha256sums) =' "$work/.SRCINFO"

git clone -q ssh://aur@aur.archlinux.org/anibeam.git "$work/aur"
cp "$work/PKGBUILD" "$work/.SRCINFO" "$work/aur/"
cd "$work/aur"
git add PKGBUILD .SRCINFO
if git diff --cached --quiet; then
  echo "anibeam $version is already on the AUR"
  exit 0
fi
git commit -q -m "anibeam $version"
if [ -n "${AUR_DRY_RUN:-}" ]; then
  git show --stat HEAD
  exit 0
fi
git push -q origin HEAD:master
echo "https://aur.archlinux.org/packages/anibeam"
