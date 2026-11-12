#!/usr/bin/env bash
# usage: publish-repos.sh <site-dir>
# Adds the apt and dnf repos to an assembled website (#788): takes the .deb and .rpm off the
# newest stable release, signs their metadata with the project key and writes
# <site-dir>/apt and <site-dir>/rpm through repos.sh, so https://anibeam.app/apt and /rpm
# always serve the current stable version. site.yml runs it on every deploy, and release.yml
# deploys right after a stable release goes public.
#   ANIBEAM_REPO_GPG_KEY   the armored private signing key (an Actions secret). Without it,
#                          or without packages on the newest stable release, the site ships
#                          without repos and says why.
#   GH_TOKEN               for gh to read the releases.
# Runs on an ubuntu-24.04 runner; installs apt-utils, createrepo-c and rpm for repos.sh.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
site="$(cd "$1" && pwd)"
repo="${GITHUB_REPOSITORY:-marcusbandit/AniBeam}"

if [ -z "${ANIBEAM_REPO_GPG_KEY:-}" ]; then
  echo "::notice::No signing key here, so the site ships without the apt and dnf repos"
  exit 0
fi

tag="$(gh release list -R "$repo" --exclude-drafts --exclude-pre-releases --limit 20 \
  --json tagName,isLatest --jq '[.[] | select(.tagName | test("^v[0-9]+\\.[0-9]+\\.[0-9]+$"))][0].tagName // empty')"
if [ -z "$tag" ]; then
  echo "::notice::No stable release yet, so no repos"
  exit 0
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/packages"
gh release download "$tag" -R "$repo" -D "$work/packages" -p '*.deb' -p '*.rpm' || true
if ! ls "$work/packages"/*.deb "$work/packages"/*.rpm > /dev/null 2>&1; then
  echo "::notice::$tag carries no .deb or .rpm yet, so no repos"
  exit 0
fi

sudo apt-get update -qq
sudo apt-get install -y -qq apt-utils createrepo-c rpm gnupg > /dev/null

export GNUPGHOME="$work/gnupg"
mkdir -m 700 "$GNUPGHOME"
gpg --batch --quiet --import <<< "$ANIBEAM_REPO_GPG_KEY"
ANIBEAM_REPO_KEY="$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr/ { print $10; exit }')"
export ANIBEAM_REPO_KEY
export ANIBEAM_RPM_BASEURL="https://github.com/$repo/releases/download/$tag/"

"$here/repos.sh" "$work/packages" "$site" https://anibeam.app
echo "Repos for $tag:"
find "$site/apt" "$site/rpm" -maxdepth 1 | sed "s|$site/||"
