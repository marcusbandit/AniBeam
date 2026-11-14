#!/usr/bin/env bash
# Fails when the tree names a download site (#277, #606): Anibeam ships no sources, and a site
# exists only as a definition in an endpoint repo the person attaches. CONTRIBUTING.md has the
# rule under Sources.
#
# Skipped, as the owner decided on 2026-11-06: what is written history (each app's
# CHANGELOG.md and the dated plans and specs under docs/superpowers/), the SQL migrations
# already applied, lockfiles, and the one core module that maps the kinds' old names.
set -euo pipefail
cd "$(dirname "$0")/.."

names='nyaa|animetosho|tosho|erai|erai-raws|subsplease|sukebei|1337x|knaben|limetorrents|everythingmoe|ddos-guard|anirena|eztv|eztvx'

# A name counts unless a lower-case letter or a digit runs on into it: tracker_nyaa_kind,
# nyaaSource and SubspleaseQuirks are hits, surfaceRaised is another word. The name itself
# is matched in any case.
pattern="(?<!(?-i:[a-z0-9]))(${names})(?!(?-i:[a-z0-9]))"
skip=(
  ':(exclude)**/CHANGELOG.md'
  ':(exclude)CHANGELOG.md'
  ':(exclude)docs/superpowers/'
  ':(exclude)core/src/store/sql/'
  ':(exclude)core/src/releases/kinds.rs'
  ':(exclude)*.lock'
  ':(exclude).github/no-site-names.sh'
)

# git grep answers 1 for no match; anything else, such as a git built without PCRE, is an
# error and must not read as a clean tree.
set +e
hits=$(git grep -nIiP "$pattern" -- . "${skip[@]}")
rc=$?
set -e
if [ "$rc" -gt 1 ]; then
  echo "git grep failed (exit $rc); the check needs a git with PCRE." >&2
  exit 2
fi

# File and folder names count too.
named=$(git ls-files -- . "${skip[@]}" | grep -iP "$pattern" || true)

if [ -n "$hits" ] || [ -n "$named" ]; then
  echo "These name a download site. A site belongs in an endpoint repo, never here:"
  [ -n "$hits" ] && echo "$hits"
  [ -n "$named" ] && echo "$named"
  exit 1
fi
echo "No download site is named."
