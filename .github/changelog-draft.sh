#!/usr/bin/env bash
# Prints commits merged since the last stable release for one app, as raw material when
# curating its entry in CHANGELOG.md before shipping.
#
#   .github/changelog-draft.sh desktop
#   .github/changelog-draft.sh phone
#   .github/changelog-draft.sh desktop v2.4.0
set -euo pipefail
app="${1:-desktop}"
since="${2:-}"

case "$app" in
  desktop)
    pattern='v[0-9]*.[0-9]*.[0-9]*'
    paths=(contract foundation net library releases metadata trackers downloads playback host core host-api room recs i18n apps/linux apps/cli)
    ;;
  phone)
    pattern='android-v[0-9]*.[0-9]*.[0-9]*'
    paths=(apps/phone contract foundation net library releases metadata trackers downloads playback host core host-api room recs i18n)
    ;;
  *)
    echo "usage: changelog-draft.sh [desktop|phone] [since-tag]" >&2
    exit 1
    ;;
esac

if [ -z "$since" ]; then
  since="$(git tag -l "$pattern" --sort=-v:refname | grep -v -- '-nightly\.' | grep -v -- '-pr\.' | head -n1 || true)"
fi

if [ -n "$since" ]; then
  echo "# Commits for $app since $since"
  range="$since..HEAD"
else
  echo "# Commits for $app (no previous stable tag found)"
  range="HEAD"
fi
echo
git log --no-merges --pretty=format:'%h %s' "$range" -- "${paths[@]}"
echo
