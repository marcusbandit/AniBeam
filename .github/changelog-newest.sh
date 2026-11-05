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
# passes: within one hunk, as many added lines as were removed count as edits.
set -euo pipefail
base="${1:-origin/main}"
root="$(git rev-parse --show-toplevel)"
status=0
for file in apps/linux/CHANGELOG.md apps/android/CHANGELOG.md; do
  git -C "$root" cat-file -e "HEAD:$file" 2>/dev/null || continue
  git -C "$root" diff -U0 --no-color "$base...HEAD" -- "$file" | awk -v file="$file" '
    BEGIN {
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
      if (edits > 0) edits--
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
