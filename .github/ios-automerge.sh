#!/usr/bin/env bash
# Merges the open PRs of the iPhone and Mac contributors (#909). ios-automerge.yml runs it.
#
# A PR by one of AUTHORS, ready for review and aimed at main, is squash merged once every
# check and workflow run on its head commit has passed, when every file it changes is
# iPhone or Mac only (ONLY below). A PR that changes anything else is its author's to merge,
# and this leaves it alone (#916).
#
#   GH_REPO   owner/name
#   AUTHORS   GitHub logins, separated by spaces or commas
#   PRS       the PR numbers to look at, from the event; empty looks at every open one
#   DRY_RUN   1 to say what it would do without merging
#
# It reads PRs through the API only and never runs their code. Anything it cannot read
# leaves the PR waiting: an API error never counts as a pass.
set -euo pipefail
: "${GH_REPO:?}" "${AUTHORS:?}"
PRS="${PRS:-}"
DRY_RUN="${DRY_RUN:-0}"
SELF_WORKFLOW="iOS automerge"
SELF_JOB="ios-automerge"

# Paths that only the iPhone or the Mac build reads. Everything else can change another
# platform: Cargo.toml and Cargo.lock, and every workflow, this one included.
ONLY='^(apps/android/ios/'
ONLY+='|apps/android/src/(.+/)?(ios|metal)(\.rs$|/)'
ONLY+='|apps/android/scripts/[^/]*ios[^/]*$'
# A phone change file is one line of patch notes, which a PR has to carry.
ONLY+='|apps/android/changes/[^/]+\.md$'
ONLY+='|apps/linux/packaging/macos/'
ONLY+=')'

# The log goes to stderr: consider's stdout carries only the branches it merged.
say() { echo "#$1: $2" >&2; }

PASSED='IN("success", "skipped", "neutral")'

# Every check run and the newest run of every workflow on the commit finished as passed or
# skipped, and there is at least one run: a push whose CI has not started is not passed.
checks_pass() {
  local sha="$1" checks runs bad
  checks="$(gh api --paginate "repos/$GH_REPO/commits/$sha/check-runs" --jq '.check_runs[]')" ||
    return 1
  runs="$(gh api --paginate "repos/$GH_REPO/actions/runs?head_sha=$sha" --jq '.workflow_runs[]')" ||
    return 1
  runs="$(jq -sc --arg self "$SELF_WORKFLOW" \
    'map(select(.name != $self)) | group_by(.workflow_id) | map(max_by(.created_at))' <<<"$runs")"
  if [ "$(jq length <<<"$runs")" = 0 ]; then
    echo "    no workflow runs yet" >&2
    return 1
  fi
  bad="$(jq -rs --arg self "$SELF_JOB" "map(select(.name != \$self))[]
      | select(.status != \"completed\" or (.conclusion | $PASSED | not))
      | \"\(.name) \(.status) \(.conclusion)\"" <<<"$checks")"
  bad+="$(jq -r ".[] | select(.status != \"completed\" or (.conclusion | $PASSED | not))
      | \"\(.name) \(.status) \(.conclusion)\"" <<<"$runs")"
  [ -z "$bad" ] || { echo "$bad" | sed 's/^/    /' >&2; return 1; }
}

# Looks at one PR and merges it when it is ready. Prints the PRs stacked on one it merged.
consider() {
  local pr="$1" info sha files outside stacked
  info="$(gh pr view "$pr" --json state,isDraft,baseRefName,author,headRefOid,headRefName,mergeable)" ||
    { say "$pr" "could not read the PR"; return 0; }
  jq -e --argjson authors "$AUTHORS_JSON" '.state == "OPEN" and (.isDraft | not)
      and .baseRefName == "main" and (.author.login | IN($authors[]))' <<<"$info" >/dev/null ||
    return 0
  sha="$(jq -r .headRefOid <<<"$info")"
  # A renamed file counts at its old path too, so a shared file moved under ios/ still counts.
  files="$(gh api --paginate "repos/$GH_REPO/pulls/$pr/files" \
    --jq '.[] | .filename, (.previous_filename // empty)')" ||
    { say "$pr" "could not list the files"; return 0; }
  if [ -z "$files" ]; then
    say "$pr" "no files listed"
    return 0
  fi
  outside="$(grep -Ev "$ONLY" <<<"$files" | sort -u || true)"
  if [ -n "$outside" ]; then
    say "$pr" "changes $(wc -l <<<"$outside") file(s) outside iPhone and Mac, left to its author"
    return 0
  fi
  if ! checks_pass "$sha"; then
    say "$pr" "checks not passed yet on ${sha:0:8}"
    return 0
  fi
  if [ "$(jq -r .mergeable <<<"$info")" != MERGEABLE ]; then
    say "$pr" "not mergeable into main"
    return 0
  fi
  if [ "$DRY_RUN" = 1 ]; then
    say "$pr" "would merge ${sha:0:8}"
    return 0
  fi
  # Listed before the merge, since deleting the branch moves them onto main.
  stacked="$(gh pr list --state open --base "$(jq -r .headRefName <<<"$info")" \
    --json number --jq '.[].number' || true)"
  # --match-head-commit: a push after the checks above stops the merge.
  if gh pr merge "$pr" --squash --match-head-commit "$sha" >&2; then
    say "$pr" "merged ${sha:0:8}"
    echo "$stacked"
  else
    say "$pr" "merge refused"
  fi
}

AUTHORS_JSON="$(tr ', ' '\n\n' <<<"$AUTHORS" | sed '/^$/d' | jq -R . | jq -sc .)"
if [ -z "$PRS" ]; then
  PRS="$(gh pr list --state open --base main --limit 200 --json number --jq '.[].number')"
fi

# A merge deletes its branch, which moves a PR stacked on it onto main, so those PRs are
# looked at next, a few times over for a stack.
for _ in 1 2 3; do
  stacked=""
  for pr in $PRS; do
    stacked+="$(consider "$pr") "
  done
  [ -n "${stacked//[[:space:]]/}" ] || break
  sleep 10
  PRS="$stacked"
done
