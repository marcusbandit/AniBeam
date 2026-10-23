#!/usr/bin/env bash
# usage: bump.sh <version>          e.g. bump.sh 2.1.0
# Points Casks/anibeam.rb at the v<version> release's dmg. Run it after the GitHub release
# is published: the dmg has to be downloadable for its checksum. The change then goes up on
# a branch and a PR like any other.
set -euo pipefail

version="${1:?usage: bump.sh <version>}"
root="$(cd "$(dirname "$0")/../../../.." && pwd)"
cask="$root/Casks/anibeam.rb"
url="https://github.com/marcusbandit/AniBeam/releases/download/v$version/AniBeam-$version-macos-arm64.dmg"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

curl -fsSL -o "$work/anibeam.dmg" "$url"
if command -v sha256sum >/dev/null; then
  sha="$(sha256sum "$work/anibeam.dmg" | cut -d' ' -f1)"
else
  sha="$(shasum -a 256 "$work/anibeam.dmg" | cut -d' ' -f1)"
fi

# perl rather than sed -i, which takes its arguments differently on Linux and macOS.
perl -pi -e "s/^  version \".*\"/  version \"$version\"/; s/^  sha256 .*/  sha256 \"$sha\"/" "$cask"
grep -E '^  (version|sha256) ' "$cask"
