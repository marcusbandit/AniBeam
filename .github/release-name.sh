#!/usr/bin/env bash
# The name a release goes out under: "AniBeam 2.5.0", or "AniBeam 2.5.0 · Quality of life"
# when that version's heading in apps/linux/CHANGELOG.md carries a title. A nightly or test
# build takes the title of the version it leads up to. The app reads the title back out of
# the name for its update notice (core/src/update.rs).
#
#   .github/release-name.sh 2.5.0-nightly.20261102.40
set -euo pipefail
version="$1"
base="${version%%-*}"
title="$(awk -F ' · ' -v v="$base" '
  /^## / { sub(/^## /, "", $1); if ($1 == v) { if (NF == 3) print $2; exit } }
' "$(dirname "$0")/../apps/linux/CHANGELOG.md")"
if [ -n "$title" ]; then echo "AniBeam $version · $title"; else echo "AniBeam $version"; fi
