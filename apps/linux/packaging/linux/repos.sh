#!/usr/bin/env bash
# usage: repos.sh <packages-dir> <out-dir> <base-url>
# Builds the signed apt and dnf repositories the website serves, from every anibeam .deb and
# .rpm in <packages-dir>:
#   <out-dir>/apt/    pool/, dists/stable/ (Release, InRelease, Release.gpg) and anibeam.sources
#   <out-dir>/rpm/    repodata/ with a detached repomd.xml.asc, and anibeam.repo. The .rpm itself
#                     stays on the GitHub release: the metadata names it by an absolute URL
#                     (ANIBEAM_RPM_BASEURL), so the site never carries a 230 MB file twice.
#                     The .rpm must be signed already (sign-rpm.sh, when the release builds it).
#   <out-dir>/anibeam.asc   the public key; apt/ and rpm/ carry a copy as well
# <base-url> is where <out-dir> is served, such as https://anibeam.app; the generated
# .sources and .repo files point at <base-url>/apt and <base-url>/rpm.
#   ANIBEAM_REPO_KEY          fingerprint of the signing key, already in the gpg keyring
#                             (GNUPGHOME is honoured)
#   ANIBEAM_REPO_PASSPHRASE   the key's passphrase, when it has one
#   ANIBEAM_RPM_BASEURL       where the .rpm files are downloaded from, the release's download
#                             URL ending in /
#
# Written for an ubuntu-24.04 GitHub runner:
#   sudo apt-get install -y apt-utils createrepo-c rpm gnupg
# apt-ftparchive comes from apt-utils, createrepo_c from createrepo-c and rpmsign from rpm.
set -euo pipefail

[ $# -eq 3 ] || { echo "usage: $0 <packages-dir> <out-dir> <base-url>" >&2; exit 2; }
pkgs="$(cd "$1" && pwd)"
mkdir -p "$2"
out="$(cd "$2" && pwd)"
base="${3%/}"
key="${ANIBEAM_REPO_KEY:?ANIBEAM_REPO_KEY names the fingerprint of the signing key}"
rpm_base="${ANIBEAM_RPM_BASEURL:?ANIBEAM_RPM_BASEURL is where the .rpm files are downloaded from}"

missing=0
for tool in apt-ftparchive createrepo_c rpm gpg gzip; do
  command -v "$tool" > /dev/null || { echo "repos.sh needs $tool; on ubuntu-24.04: sudo apt-get install -y apt-utils createrepo-c rpm gnupg" >&2; missing=1; }
done
[ "$missing" -eq 0 ] || exit 1

shopt -s nullglob
debs=("$pkgs"/anibeam_*_amd64.deb)
rpms=("$pkgs"/anibeam-*.x86_64.rpm)
[ "${#debs[@]}" -gt 0 ] || { echo "no anibeam_*_amd64.deb in $pkgs" >&2; exit 1; }
[ "${#rpms[@]}" -gt 0 ] || { echo "no anibeam-*.x86_64.rpm in $pkgs" >&2; exit 1; }
gpg --list-secret-keys "$key" > /dev/null 2>&1 || { echo "the gpg keyring has no secret key $key" >&2; exit 1; }

# gpg with the project key, no prompts: loopback pinentry, so a passphrase comes from the env.
sign() {
  local pass=()
  if [ -n "${ANIBEAM_REPO_PASSPHRASE:-}" ]; then
    pass=(--passphrase-fd 3)
  fi
  gpg --batch --yes --pinentry-mode loopback --local-user "$key" --digest-algo SHA512 \
    "${pass[@]}" "$@" 3<<<"${ANIBEAM_REPO_PASSPHRASE:-}"
}

rm -rf "$out/apt" "$out/rpm" "$out/anibeam.asc"
mkdir -p "$out/apt" "$out/rpm"
gpg --batch --yes --armor --export-options export-minimal --export "$key" > "$out/anibeam.asc"
[ -s "$out/anibeam.asc" ] || { echo "could not export the public key" >&2; exit 1; }
cp "$out/anibeam.asc" "$out/apt/anibeam.asc"
cp "$out/anibeam.asc" "$out/rpm/anibeam.asc"

echo "== apt"
apt="$out/apt"
mkdir -p "$apt/pool/main" "$apt/dists/stable/main/binary-amd64"
cp "${debs[@]}" "$apt/pool/main/"
(
  cd "$apt"
  apt-ftparchive packages pool > dists/stable/main/binary-amd64/Packages
  gzip -9nk dists/stable/main/binary-amd64/Packages
  apt-ftparchive \
    -o APT::FTPArchive::Release::Origin=AniBeam \
    -o APT::FTPArchive::Release::Label=AniBeam \
    -o APT::FTPArchive::Release::Suite=stable \
    -o APT::FTPArchive::Release::Codename=stable \
    -o APT::FTPArchive::Release::Architectures=amd64 \
    -o APT::FTPArchive::Release::Components=main \
    -o "APT::FTPArchive::Release::Description=AniBeam" \
    release dists/stable > dists/stable/Release
)
sign --armor --detach-sign --output "$apt/dists/stable/Release.gpg" "$apt/dists/stable/Release"
sign --clearsign --output "$apt/dists/stable/InRelease" "$apt/dists/stable/Release"

# deb822 Signed-By holds the key inline: each line indented by one space, a blank line as " .".
{
  echo "Types: deb"
  echo "URIs: $base/apt"
  echo "Suites: stable"
  echo "Components: main"
  echo "Architectures: amd64"
  echo "Signed-By:"
  sed -e 's/^$/ ./' -e '/^ \./!s/^/ /' "$out/anibeam.asc"
} > "$apt/anibeam.sources"

echo "== rpm"
rpm="$out/rpm"
for f in "${rpms[@]}"; do
  sig="$(rpm -qp --qf '%{SIGPGP:pgpsig}%{RSAHEADER:pgpsig}' "$f" 2>/dev/null)"
  case "$sig" in *"(none)(none)"*|"") echo "$(basename "$f") is not signed; sign-rpm.sh signs it as the release builds it" >&2; exit 1 ;; esac
done
# The metadata is made over a copy and written to the site; only repodata/ stays.
staging="$(mktemp -d)"
cp "${rpms[@]}" "$staging/"
createrepo_c --quiet --baseurl "$rpm_base" --outputdir "$rpm" "$staging"
rm -rf "$staging"
sign --armor --detach-sign --output "$rpm/repodata/repomd.xml.asc" "$rpm/repodata/repomd.xml"
cat > "$rpm/anibeam.repo" <<REPO
[anibeam]
name=AniBeam
baseurl=$base/rpm
enabled=1
gpgcheck=1
repo_gpgcheck=1
gpgkey=$base/rpm/anibeam.asc
REPO

echo "== done"
(cd "$out" && find apt rpm -maxdepth 4 -type f | sort)
