#!/usr/bin/env bash
# usage: publish-repos.sh <site-dir>
# Adds the apt and dnf repos to an assembled website (#788): takes the .deb and .rpm off the
# newest stable release, signs their metadata with the project key and writes
# <site-dir>/apt and <site-dir>/rpm through repos.sh, so https://anibeam.app/apt and /rpm
# always serve the current stable version. site.yml runs it on every deploy, and release.yml
# deploys right after a stable release goes public.
#   ANIBEAM_REPO_GPG_KEY   the armored private signing key (an Actions secret). Without it,
#                          or with no stable release carrying both packages, the deploy fails.
# It also writes <site-dir>/repos.json, the version the repos carry, which the app reads
# before it offers an apt or dnf install an update.
#   GH_TOKEN               for gh to read the releases.
# Runs on an ubuntu-24.04 runner; installs apt-utils, createrepo-c and rpm for repos.sh.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
site="$(cd "$1" && pwd)"
repo="${GITHUB_REPOSITORY:-marcusbandit/AniBeam}"

# Every way this can go wrong fails the deploy: Pages then keeps serving the last good site,
# repos included, instead of replacing it with one that has none.
: "${ANIBEAM_REPO_GPG_KEY:?ANIBEAM_REPO_GPG_KEY is not set; the deploy would drop the apt and dnf repos}"

# The highest stable version that carries both packages. Creation order is not version order:
# a hotfix cut for an older line after a newer release must not roll the repos back.
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
gh release list -R "$repo" --exclude-drafts --exclude-pre-releases --limit 100 --json tagName \
  --jq '.[].tagName | select(test("^v[0-9]+\\.[0-9]+\\.[0-9]+$"))' | sort -rV > "$work/tags"
tag=""
while read -r candidate; do
  assets="$(gh release view "$candidate" -R "$repo" --json assets --jq '.assets[].name')"
  if grep -q '_amd64\.deb$' <<< "$assets" && grep -q '\.x86_64\.rpm$' <<< "$assets"; then
    tag="$candidate"
    break
  fi
done < "$work/tags"
[ -n "$tag" ] || { echo "::error::No stable release carries both a .deb and an .rpm"; exit 1; }

mkdir -p "$work/packages"
gh release download "$tag" -R "$repo" -D "$work/packages" -p '*_amd64.deb' -p '*.x86_64.rpm'

sudo apt-get update -qq
sudo apt-get install -y -qq apt-utils createrepo-c rpm gnupg > /dev/null

export GNUPGHOME="$work/gnupg"
mkdir -m 700 "$GNUPGHOME"
gpg --batch --quiet --import <<< "$ANIBEAM_REPO_GPG_KEY"
ANIBEAM_REPO_KEY="$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr/ { print $10; exit }')"
export ANIBEAM_REPO_KEY
export ANIBEAM_RPM_BASEURL="https://github.com/$repo/releases/download/$tag/"

"$here/repos.sh" "$work/packages" "$site" https://anibeam.app
# What the repos carry, for the app: an apt or dnf install is offered this version only.
printf '{"version": "%s", "tag": "%s"}\n' "${tag#v}" "$tag" > "$site/repos.json"
echo "Repos for $tag:"
find "$site/apt" "$site/rpm" -maxdepth 1 | sed "s|$site/||"
