#!/usr/bin/env bash
# Refuses a patch note added under any version heading but the newest one. A merge of main
# into an old branch moves the branch's lines under a heading that has since shipped, and the
# format check in core/tests/changelog.rs cannot tell (#480). CI runs this on every pull
# request against the base it merges into; run it by hand against any ref:
#
#   .github/changelog-newest.sh origin/main
set -euo pipefail
base="${1:-origin/main}"
root="$(git rev-parse --show-toplevel)"
status=0
for file in apps/linux/CHANGELOG.md apps/android/CHANGELOG.md; do
  [ -f "$root/$file" ] || continue
  # The newest heading, and the line where the next one starts: anything added from there
  # on sits under a version that has shipped.
  read -r newest_line second_line < <(awk '
    /^## / { if (!n) n = NR; else { print n, NR; found = 1; exit } }
    END { if (!found) print n + 0, 0 }
  ' "$root/$file")
  [ "$second_line" -gt 0 ] || continue
  newest="$(sed -n "${newest_line}p" "$root/$file")"
  git -C "$root" diff -U0 --no-color "$base" HEAD -- "$file" | awk -v file="$file" \
    -v from="$second_line" -v newest="$newest" -v src="$root/$file" '
    BEGIN { while ((getline l < src) > 0) text[++count] = l }
    /^@@/ { split($3, p, ","); line = substr(p[1], 2) + 0; next }
    /^\+\+\+/ || /^---/ { next }
    /^\+/ {
      if (line >= from && $0 !~ /^\+[[:space:]]*$/) {
        heading = ""
        for (i = line; i > 0; i--) if (text[i] ~ /^## /) { heading = text[i]; break }
        printf "::error file=%s,line=%d::%s:%d adds \"%s\" under \"%s\", which has shipped. Move it under the newest heading, \"%s\".\n", file, line, file, line, substr($0, 2), heading, newest
        bad = 1
      }
      line++
    }
    END { exit bad }
  ' || status=1
done
exit "$status"
