#!/usr/bin/env bash
# The name a release goes out under: "Anibeam 2.5.0", or "Anibeam 2.5.0 · Quality of life"
# when that version's heading in apps/linux/CHANGELOG.md carries a title. A nightly or test
# build takes the title of the version it leads up to. The app reads the title back out of
# the name for its update notice (core/src/update.rs). With phone, the phone's release:
# "Anibeam for Android 2.4.3", titled from apps/android/CHANGELOG.md.
#
#   .github/release-name.sh 2.5.0-nightly.20261102.40
#   .github/release-name.sh 2.4.3-nightly.20261113.41 phone
set -euo pipefail
version="$1"
app="${2:-desktop}"
base="${version%%-*}"
case "$app" in
  desktop) notes=apps/linux/CHANGELOG.md name="Anibeam" ;;
  phone) notes=apps/android/CHANGELOG.md name="Anibeam for Android" ;;
  *) echo "release-name.sh: the app is desktop or phone, not $app" >&2; exit 1 ;;
esac
title="$(awk -F ' · ' -v v="$base" '
  /^## / { sub(/^## /, "", $1); if ($1 == v) { if (NF == 3) print $2; exit } }
' "$(dirname "$0")/../$notes")"
if [ -n "$title" ]; then echo "$name $version · $title"; else echo "$name $version"; fi
