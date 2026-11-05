#!/usr/bin/env bash
# Folds an app's change files into its CHANGELOG.md under the newest heading, then removes
# them (#504). A pull request writes its patch note in <app>/changes/<branch>.md instead of
# CHANGELOG.md, so two of them never touch the same lines. The apps build the change files
# in and show them under the newest version (core::changelog::parse_with), which this
# matches line for line: in each section the file's own lines come first, then each change
# file's in name order, and the sections go New, Changed, Fixed, Removed.
#
#   .github/changelog-fold.sh apps/linux          every change file in the checkout
#   .github/changelog-fold.sh apps/linux <ref>    only those <ref> has too
#
# The Release workflow runs it with the commit it shipped, before main moves to the next
# version. The phone's version bump runs it by hand, before it adds the new heading.
set -euo pipefail
export LC_ALL=C
app="${1:?usage: changelog-fold.sh <app dir> [ref]}"
ref="${2:-}"
notes="$app/CHANGELOG.md"
[ -f "$notes" ] || { echo "changelog-fold.sh: no $notes" >&2; exit 1; }
files=()
for f in "$app"/changes/*.md; do
  [ -e "$f" ] || continue
  name="${f##*/}"
  [ "$name" = README.md ] && continue
  if [ -n "$ref" ] && ! git -C "$app" cat-file -e "$ref:./changes/$name" 2> /dev/null; then
    continue
  fi
  files+=("$f")
done
if [ ${#files[@]} -eq 0 ]; then
  echo "No change files to fold in $app/changes."
  exit 0
fi
awk -v order="New Changed Fixed Removed" '
  function trim(s) { sub(/^[ \t\r]+/, "", s); sub(/[ \t\r]+$/, "", s); return s }
  # One line of a version or a change file: a section starts, or a line joins the open one.
  function take(line) {
    if (line ~ /^### /) { kind = trim(substr(line, 5)); return }
    if (line ~ /^- / && kind != "") items[kind, ++count[kind]] = trim(substr(line, 3))
  }
  FNR == 1 { file++; kind = ""; comment = 0 }
  {
    line = trim($0)
    if (comment) { comment = (index(line, "-->") == 0); if (file == 1) text[++n] = $0; next }
    if (line ~ /^<!--/) { comment = (index(line, "-->") == 0); if (file == 1) text[++n] = $0; next }
  }
  file == 1 {
    text[++n] = $0
    if (line ~ /^## /) { if (!start) start = n; else if (!end) end = n }
    else if (start && !end) take(line)
    next
  }
  { take(line) }
  END {
    if (!start) { print "changelog-fold.sh: no version heading" > "/dev/stderr"; exit 1 }
    if (!end) end = n + 1
    for (i = 1; i <= start; i++) print text[i]
    split(order, kinds, " ")
    for (k = 1; k <= 4; k++) {
      if (!count[kinds[k]]) continue
      print ""; print "### " kinds[k]
      for (j = 1; j <= count[kinds[k]]; j++) print "- " items[kinds[k], j]
    }
    if (end <= n) print ""
    for (i = end; i <= n; i++) print text[i]
  }
' "$notes" "${files[@]}" > "$notes.folded"
mv "$notes.folded" "$notes"
for f in "${files[@]}"; do
  if git -C "$app" ls-files --error-unmatch "./changes/${f##*/}" > /dev/null 2>&1; then
    git -C "$app" rm -q "./changes/${f##*/}"
  else
    rm -f "$f"
  fi
done
echo "Folded ${#files[@]} change file(s) into $notes."
