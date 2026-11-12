#!/usr/bin/env bash
# usage: sign-rpm.sh <file.rpm>...
# Signs AniBeam's .rpm with the project key, in place, so the file the release carries is the
# one dnf checks (#788). The dnf repo on anibeam.app points at the release's own file rather
# than a copy, so the signature has to be on it before it is uploaded.
#   ANIBEAM_REPO_GPG_KEY      the armored private signing key (an Actions secret)
#   ANIBEAM_REPO_PASSPHRASE   its passphrase, when it has one
# Needs rpmsign and gpg; on ubuntu-24.04: sudo apt-get install -y rpm gnupg
set -euo pipefail

[ $# -gt 0 ] || { echo "usage: $0 <file.rpm>..." >&2; exit 2; }
: "${ANIBEAM_REPO_GPG_KEY:?ANIBEAM_REPO_GPG_KEY holds the armored private signing key}"
command -v rpmsign > /dev/null || { echo "sign-rpm.sh needs rpmsign: sudo apt-get install -y rpm" >&2; exit 1; }

GNUPGHOME="$(mktemp -d)"
export GNUPGHOME
trap 'rm -rf "$GNUPGHOME"' EXIT
gpg --batch --quiet --import <<< "$ANIBEAM_REPO_GPG_KEY"
key="$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr/ { print $10; exit }')"

extra=()
if [ -n "${ANIBEAM_REPO_PASSPHRASE:-}" ]; then
  printf '%s' "$ANIBEAM_REPO_PASSPHRASE" > "$GNUPGHOME/passphrase"
  chmod 600 "$GNUPGHOME/passphrase"
  extra=(-D "_gpg_sign_cmd_extra_args --pinentry-mode loopback --passphrase-file $GNUPGHOME/passphrase")
fi
rpmsign --addsign \
  -D "_gpg_name $key" \
  -D "_gpg_digest_algo sha256" \
  -D "__gpg $(command -v gpg)" \
  -D "_gpg_path $GNUPGHOME" \
  "${extra[@]}" \
  "$@"
for f in "$@"; do
  echo "signed $(basename "$f"): $(rpm -qp --qf '%{SIGPGP:pgpsig}%{RSAHEADER:pgpsig}' "$f" 2>/dev/null)"
done
