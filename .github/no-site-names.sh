#!/usr/bin/env bash
# Fails when the tree names a download site (#277, #606): AniBeam ships no sources, and a site
# exists only as a definition in an endpoint repo the person attaches. CONTRIBUTING.md has the
# rule under Sources.
#
# Skipped, as the owner decided on 2026-11-06: what is written history (each app's
# CHANGELOG.md and the dated plans and specs under docs/superpowers/), the SQL migrations
# already applied, lockfiles, and the one core module that maps the kinds' old names.
set -euo pipefail
cd "$(dirname "$0")/.."

names='nyaa|animetosho|tosho|erai|erai-raws|subsplease|sukebei|1337x|knaben|limetorrents|everythingmoe|ddos-guard|anirena|eztv|eztvx'

# A letter or digit on either side makes it another word; an underscore does not, so a test
# named tracker_nyaa_kind still counts.
hits=$(git grep -nIiP "(?<![A-Za-z0-9])(${names})(?![A-Za-z0-9])" -- . \
  ':(exclude)**/CHANGELOG.md' \
  ':(exclude)CHANGELOG.md' \
  ':(exclude)docs/superpowers/' \
  ':(exclude)core/src/store/sql/' \
  ':(exclude)core/src/releases/kinds.rs' \
  ':(exclude)*.lock' \
  ':(exclude).github/no-site-names.sh' || true)

if [ -n "$hits" ]; then
  echo "These lines name a download site. A site belongs in an endpoint repo, never here:"
  echo "$hits"
  exit 1
fi
echo "No download site is named."
