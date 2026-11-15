#!/usr/bin/env bash
# usage: publish.sh <version>          e.g. publish.sh 2.1.0
# Points the AUR's anibeam (built from the v<version> tag) and anibeam-bin (the release's
# AniBeam-<version>-linux-x86_64.tar.zst) at that version and pushes both, so the two never
# drift apart. Run it after the GitHub release is published: the tag's tarball and the release
# asset have to exist for their checksums. Needs an AUR account whose SSH key ~/.ssh/config
# hands to aur.archlinux.org.
#   AUR_DRY_RUN=1   do everything but the push, and print each commit
#
# Each PKGBUILD comes from the tag being published, not from this checkout, so the recipe
# matches the source it builds: a hotfix of an older version gets that version's package().
# A tag from before anibeam-bin existed (#235) has no anibeam-bin PKGBUILD, and only anibeam
# is published. Both packages are prepared, checksums and all, before either is pushed, so a
# missing release asset stops the run with the AUR untouched.
set -euo pipefail

version="${1:?usage: publish.sh <version>}"
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(git -C "$here" rev-parse --show-toplevel)"
tag="v$version"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

git -C "$repo" fetch -q origin tag "$tag" 2> /dev/null || true
git -C "$repo" rev-parse -q --verify "refs/tags/$tag^{commit}" > /dev/null \
  || { echo "publish.sh: no tag $tag; publish after the release is out" >&2; exit 1; }

# prepare <package>: that tag's PKGBUILD at this version with the source's real checksum and
# its .SRCINFO, in $work/<package>. Returns 1 when the tag has no PKGBUILD for it and 2 when a
# step fails. Chained by hand: set -e does not hold inside a function its caller tests.
prepare() {
  local name="$1" dir="$work/$1"
  mkdir -p "$dir"
  git -C "$repo" show "$tag:apps/linux/packaging/aur/$name/PKGBUILD" > "$dir/PKGBUILD" 2> /dev/null \
    || return 1
  if ! { sed -i -e "s/^pkgver=.*/pkgver=$version/" -e "s/^pkgrel=.*/pkgrel=1/" "$dir/PKGBUILD" \
    && (cd "$dir" && updpkgsums && makepkg --printsrcinfo > .SRCINFO) \
    && grep -E '^\s*(pkgver|sha256sums) =' "$dir/.SRCINFO"; }; then
    echo "publish.sh: could not prepare $name $version" >&2
    return 2
  fi
}

# push <package>: the prepared files, committed to the package's AUR repo and pushed.
push() {
  local name="$1" dir="$work/$1"
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

packages=(anibeam)
rc=0; prepare anibeam || rc=$?
[ "$rc" = 0 ] || { [ "$rc" = 1 ] && echo "publish.sh: $tag has no anibeam PKGBUILD" >&2; exit 1; }
rc=0; prepare anibeam-bin || rc=$?
case "$rc" in
  0) packages+=(anibeam-bin) ;;
  1) echo "publish.sh: $tag predates anibeam-bin; publishing anibeam only" ;;
  *) exit 1 ;;
esac
for name in "${packages[@]}"; do push "$name"; done
