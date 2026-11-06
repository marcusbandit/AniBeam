#!/usr/bin/env bash
# usage: publish.sh <version>          e.g. publish.sh 2.1.0
# Points the AUR's anibeam (built from the v<version> tag) and anibeam-bin (the release's
# AniBeam-<version>-linux-x86_64.tar.zst) at that version and pushes both, so the two never
# drift apart. Run it after the GitHub release is published: the tag's tarball and the release
# asset have to exist for their checksums. Needs an AUR account whose SSH key ~/.ssh/config
# hands to aur.archlinux.org.
#   AUR_DRY_RUN=1   do everything but the push, and print each commit
set -euo pipefail

version="${1:?usage: publish.sh <version>}"
here="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# publish <package>: the PKGBUILD in the repo, at this version, with the source's real
# checksum, committed to the package's AUR repo and pushed.
publish() {
  local name="$1" dir="$work/$1"
  mkdir -p "$dir"
  cp "$here/$name/PKGBUILD" "$dir/PKGBUILD"
  sed -i -e "s/^pkgver=.*/pkgver=$version/" -e "s/^pkgrel=.*/pkgrel=1/" "$dir/PKGBUILD"
  (cd "$dir" && updpkgsums && makepkg --printsrcinfo > .SRCINFO)
  grep -E '^\s*(pkgver|sha256sums) =' "$dir/.SRCINFO"

  git clone -q "ssh://aur@aur.archlinux.org/$name.git" "$dir/aur"
  cp "$dir/PKGBUILD" "$dir/.SRCINFO" "$dir/aur/"
  git -C "$dir/aur" add PKGBUILD .SRCINFO
  if git -C "$dir/aur" diff --cached --quiet; then
    echo "$name $version is already on the AUR"
    return 0
  fi
  git -C "$dir/aur" commit -q -m "$name $version"
  if [ -n "${AUR_DRY_RUN:-}" ]; then
    git -C "$dir/aur" show --stat HEAD
    return 0
  fi
  git -C "$dir/aur" push -q origin HEAD:master
  echo "https://aur.archlinux.org/packages/$name"
}

publish anibeam
publish anibeam-bin
