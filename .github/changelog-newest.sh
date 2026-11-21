#!/usr/bin/env bash
# Refuses lines added under any version heading in CHANGELOG.md except the newest one. A
# merge of main into an old branch can land lines under a heading that has since shipped,
# and the format check in core/tests/changelog.rs cannot tell (#480). CI runs it on every
# pull request against the base it merges into, and on main against the commit before; run
# it by hand against any ref. It checks committed work, HEAD, never the working tree:
#
#   .github/changelog-newest.sh origin/main
#
# Fixing a typo or tightening a shipped version in place passes: every version below the
# newest heading may keep at most as many non-blank lines as it had at the base. A file
# the diff moves is compared with its old path, so a renamed app folder adds nothing.
set -euo pipefail
base="${1:-origin/main}"
root="$(git rev-parse --show-toplevel)"
mb="$(git -C "$root" merge-base "$base" HEAD 2>/dev/null || echo "$base")"
status=0
for file in apps/linux/CHANGELOG.md apps/phone/CHANGELOG.md; do
  git -C "$root" cat-file -e "HEAD:$file" 2>/dev/null || continue
  # Where the file was at the base, when this diff moved it there.
  old="$(git -C "$root" diff --name-status -M --diff-filter=R "$mb" HEAD |
    awk -v f="$file" '$3 == f { print $2 }')"
  old="${old:-$file}"
  git -C "$root" cat-file -e "$mb:$old" 2>/dev/null || continue
  awk -v file="$file" '
    function ver(line,    a) {
      sub(/^## /, "", line)
      split(line, a, /[[:space:]]+/)
      sub(/^v/, "", a[1])
      return a[1]
    }
    FNR == 1 { comment = 0 }
    comment || /^[[:space:]]*<!--/ {
      if (!comment) comment = 1
      if (index($0, "-->")) comment = 0
      next
    }
    FNR == NR {
      if ($0 ~ /^## /) { v = ver($0); base_seen[v] = 1; next }
      if (v != "" && $0 !~ /^[[:space:]]*$/) base_lines[v]++
      next
    }
    $0 ~ /^## / {
      if (cur != "" && idx > 1) check(cur, cur_head, cur_line)
      idx++
      cur = ver($0)
      cur_head = $0
      cur_line = FNR
      if (idx == 1) newest_head = $0
      next
    }
    cur != "" && $0 !~ /^[[:space:]]*$/ { head_lines[cur]++ }
    END {
      if (cur != "" && idx > 1) check(cur, cur_head, cur_line)
      exit bad
    }
    function check(v, head, line) {
      if (!(v in base_seen)) {
        printf "::error file=%s,line=%d::%s:%d adds a shipped version \"%s\" below \"%s\".\n", file, line, file, line, head, newest_head
        bad = 1
      } else if (head_lines[v] + 0 > base_lines[v] + 0) {
        printf "::error file=%s,line=%d::%s:%d adds %d line(s) under \"%s\", which has shipped. Move new notes under \"%s\".\n", file, line, file, line, head_lines[v] - base_lines[v], head, newest_head
        bad = 1
      }
    }
  ' <(git -C "$root" show "$mb:$old") <(git -C "$root" show "HEAD:$file") || status=1
done
exit "$status"
