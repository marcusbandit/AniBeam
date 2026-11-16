#!/usr/bin/env bash
# Fails when the tree spells the name with a capital B (#929, #1047). The app, its id, its
# files and its docs all say Anibeam. The migration that renames an old install's files
# (apps/linux/src/old_name.rs) builds the old spelling from two halves, so it needs no
# exception here.
set -euo pipefail
cd "$(dirname "$0")/.."

old="Ani""Beam"
set +e
hits="$(git grep -n -I "$old" -- . ':(exclude).github/one-spelling.sh')"
status=$?
set -e
[ $status -le 1 ] || { echo "one-spelling: git grep failed" >&2; exit 2; }
files="$(git ls-files | grep "$old" || true)"
if [ -n "$hits$files" ]; then
  printf '%s\n' "$hits" "$files" | sed '/^$/d'
  echo "one-spelling: the name is Anibeam, with a lower-case b" >&2
  exit 1
fi
