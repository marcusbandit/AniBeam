#!/usr/bin/env bash
# Folds an app's change files into its CHANGELOG.md under the newest heading, then removes
# them (#504). A pull request writes its patch note in <app>/changes/<branch>.md instead of
# CHANGELOG.md, so two of them never touch the same lines. The apps build the change files
# in and show them under the newest version (core::changelog::parse_with), which this
# matches line for line: in each section the file's own lines come first, then each change
# file's in name order, and the sections go New, Changed, Fixed, Removed. A comment under
# the newest heading stays, and a section of any other name follows those four.
#
#   .github/changelog-fold.sh apps/linux          every change file in the checkout
#   .github/changelog-fold.sh apps/linux <ref>    only what <ref> had, as <ref> had it
#
# With a ref, a change file folds as the ref has it, and a line added to it since stays
# behind in the file for the next version. The Release workflow runs it with the commit
# it shipped, before main moves to the next version. A version bump made by hand (the
# phone's, or a hotfix branch's) runs it with no ref; see CONTRIBUTING.md, Patch notes.
set -euo pipefail
export LC_ALL=C
app="${1:?usage: changelog-fold.sh <app dir> [ref]}"
ref="${2:-}"
notes="$app/CHANGELOG.md"
[ -f "$notes" ] || { echo "changelog-fold.sh: no $notes" >&2; exit 1; }
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
files=()   # the change files in the checkout that fold
texts=()   # what each folds: the file itself, or the ref's copy of it
for f in "$app"/changes/*.md; do
  [ -e "$f" ] || continue
  name="${f##*/}"
  [ "$name" = README.md ] && continue
  if [ -n "$ref" ]; then
    git -C "$app" show "$ref:./changes/$name" > "$work/$name" 2> /dev/null || continue
    texts+=("$work/$name")
  else
    texts+=("$f")
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
    if (line ~ /^### /) {
      kind = trim(substr(line, 5))
      if (!(kind in seen)) { seen[kind] = 1; kinds[++nkinds] = kind }
      return
    }
    if (line ~ /^- / && kind != "") items[kind, ++count[kind]] = trim(substr(line, 3))
  }
  BEGIN { nkinds = split(order, kinds, " "); for (k = 1; k <= nkinds; k++) seen[kinds[k]] = 1 }
  FNR == 1 { file++; kind = ""; comment = 0 }
  {
    line = trim($0)
    # A comment is skipped, as the apps skip it; one under the newest heading is kept.
    if (comment || line ~ /^<!--/) {
      if (!comment) comment = 1
      if (index(line, "-->")) comment = 0
      if (file == 1) { if (start && !end) kept[++nkept] = $0; else text[++n] = $0 }
      next
    }
  }
  file == 1 {
    if (line ~ /^## /) { text[++n] = $0; if (!start) start = n; else if (!end) end = n }
    else if (start && !end) take(line)
    else text[++n] = $0
    next
  }
  { take(line) }
  END {
    if (!start) { print "changelog-fold.sh: no version heading" > "/dev/stderr"; exit 1 }
    if (!end) end = n + 1
    for (i = 1; i <= start; i++) print text[i]
    for (i = 1; i <= nkept; i++) { if (i == 1) print ""; print kept[i] }
    for (k = 1; k <= nkinds; k++) {
      if (!count[kinds[k]]) continue
      print ""; print "### " kinds[k]
      for (j = 1; j <= count[kinds[k]]; j++) print "- " items[kinds[k], j]
    }
    if (end <= n) print ""
    for (i = end; i <= n; i++) print text[i]
  }
' "$notes" "${texts[@]}" > "$work/CHANGELOG.md"
cp "$work/CHANGELOG.md" "$notes"
for i in "${!files[@]}"; do
  f="${files[$i]}"
  # A line added since the ref stays for the next version, under its section.
  if [ "${texts[$i]}" != "$f" ] && ! cmp -s "${texts[$i]}" "$f"; then
    awk '
      function trim(s) { sub(/^[ \t\r]+/, "", s); sub(/[ \t\r]+$/, "", s); return s }
      FNR == NR { if (trim($0) ~ /^- /) shipped[trim($0)]++; next }
      { line = trim($0) }
      line ~ /^### / { if (head != "" && body != "") printf "%s%s", head, body; head = $0 "\n"; body = ""; next }
      line ~ /^- / { if (shipped[line]-- > 0) next; body = body $0 "\n"; next }
      { if (head == "") print; else if (line != "") body = body $0 "\n" }
      END { if (head != "" && body != "") printf "%s", head body }
    ' "${texts[$i]}" "$f" > "$work/left"
    if grep -q '^[[:space:]]*- ' "$work/left"; then
      cp "$work/left" "$f"
      continue
    fi
  fi
  if git -C "$app" ls-files --error-unmatch "./changes/${f##*/}" > /dev/null 2>&1; then
    git -C "$app" rm -q -f "./changes/${f##*/}"
  else
    rm -f "$f"
  fi
done
echo "Folded ${#files[@]} change file(s) into $notes."
