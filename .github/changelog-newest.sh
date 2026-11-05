#!/usr/bin/env bash
# Refuses a patch note added under any version heading but the newest one. A merge of main
# into an old branch moves the branch's lines under a heading that has since shipped, and the
# format check in core/tests/changelog.rs cannot tell (#480). CI runs it on every pull request
# against the base it merges into, and on main against the commit before; run it by hand
# against any ref. It checks committed work, HEAD, never the working tree:
#
#   .github/changelog-newest.sh origin/main
#
# A line changed in place, a typo fixed in a shipped note, is an edit, not a new line, and
# passes: within one hunk, as many added lines as were removed count as edits. So does a
# line folded in from a change file the same diff deletes (.github/changelog-fold.sh): a
# hotfix branch folds the shipped version's change files under it, then adds its own heading.
#
# A pull request writes its notes in <app>/changes/, which always ship under the newest
# version, so this only ever catches a line written straight into CHANGELOG.md.
set -euo pipefail
base="${1:-origin/main}"
root="$(git rev-parse --show-toplevel)"
status=0
for file in apps/linux/CHANGELOG.md apps/android/CHANGELOG.md; do
  git -C "$root" cat-file -e "HEAD:$file" 2>/dev/null || continue
  # The lines of the change files this diff deletes, which may land in the file as they are.
  moved="$(git -C "$root" diff -U0 --no-color --diff-filter=D "$base...HEAD" -- "${file%CHANGELOG.md}changes" |
    awk '/^---/ { next } /^-- / { s = substr($0, 3); sub(/^ +/, "", s); sub(/[ \t\r]+$/, "", s); print s }')"
  git -C "$root" diff -U0 --no-color "$base...HEAD" -- "$file" | MOVED="$moved" awk -v file="$file" '
    BEGIN {
      count_moved = split(ENVIRON["MOVED"], m, "\n")
      for (i = 1; i <= count_moved; i++) if (m[i] != "") folded[m[i]]++
      cmd = "git -C \"'"$root"'\" show \"HEAD:" file "\""
      while ((cmd | getline l) > 0) {
        text[++count] = l
        if (l ~ /^## /) { if (!newest) newest = count; else if (!from) from = count }
      }
      close(cmd)
      if (!from) exit
    }
    /^@@/ {
      hunk = 1
      split($2, o, ","); edits = (o[2] == "") ? 1 : o[2] + 0
      split($3, n, ","); line = substr(n[1], 2) + 0
      next
    }
    !hunk || /^-/ || /^\\/ { next }
    /^\+/ {
      item = $0; sub(/^\+- */, "", item); sub(/[ \t\r]+$/, "", item)
      if (edits > 0) edits--
      else if ($0 ~ /^\+- / && folded[item] > 0) folded[item]--
      else if (line >= from && $0 !~ /^\+[[:space:]]*$/) {
        for (i = line; i > 0; i--) if (text[i] ~ /^## /) break
        note = substr($0, 2); gsub(/%/, "%25", note); gsub(/\r/, "%0D", note)
        printf "::error file=%s,line=%d::%s:%d adds \"%s\" under \"%s\", which has shipped. Move it under the newest heading, \"%s\".\n", file, line, file, line, note, text[i], text[newest]
        bad = 1
      }
      line++
    }
    END { exit bad }
  ' || status=1
done
exit "$status"
