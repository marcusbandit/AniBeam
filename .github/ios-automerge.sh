#!/usr/bin/env bash
# Merges the open PRs of the iPhone and Mac contributors (#909). ios-automerge.yml runs it.
#
# A PR by one of AUTHORS, ready for review and aimed at main, is squash merged once every
# check and workflow run on its head commit has passed, when every file it changes is
# iPhone or Mac only (ONLY below). A PR that changes anything else, the core, the desktop,
# Android or the shared phone code among it, waits for OWNER: it asks them for a review,
# lists the files outside iPhone and Mac in one comment, and merges itself only once OWNER
# approves its head commit.
#
#   GH_REPO   owner/name
#   AUTHORS   GitHub logins, separated by spaces or commas
#   OWNER     the login whose approval lets any other PR through
#   PRS       the PR numbers to look at, from the event; empty looks at every open one
#   MERGE     1 to merge; anything else only comments
#   DRY_RUN   1 to say what it would do without commenting, asking or merging
#
# It reads PRs through the API only and never runs their code. Anything it cannot read
# leaves the PR waiting: an API error never counts as a pass.
set -euo pipefail
: "${GH_REPO:?}" "${AUTHORS:?}" "${OWNER:?}"
PRS="${PRS:-}"
MERGE="${MERGE:-0}"
DRY_RUN="${DRY_RUN:-0}"
SELF_WORKFLOW="iOS automerge"
SELF_JOB="ios-automerge"
MARK="<!-- ios-automerge -->"

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

# OWNER's latest review that approves or turns down, on the head commit, is an approval.
approved() {
  local pr="$1" sha="$2" state
  state="$(gh api --paginate "repos/$GH_REPO/pulls/$pr/reviews" --jq '.[]' |
    jq -rs --arg owner "$OWNER" --arg sha "$sha" '
      [.[] | select(.user.login == $owner and .commit_id == $sha)
       | select(.state | IN("APPROVED", "CHANGES_REQUESTED"))] | last | .state // ""')" ||
    return 1
  [ "$state" = APPROVED ]
}

# One comment per PR listing what keeps it waiting, written once and kept current.
note() {
  local pr="$1" outside="$2" body old id count
  count="$(wc -l <<<"$outside")"
  body="$MARK
This PR changes more than the iPhone and Mac apps, so it merges itself only after $OWNER approves its latest commit. The files outside iPhone and Mac:

$(head -n 30 <<<"$outside" | sed 's/.*/- `&`/')"
  [ "$count" -le 30 ] || body+="
- and $((count - 30)) more"
  old="$(gh api --paginate "repos/$GH_REPO/issues/$pr/comments" \
    --jq ".[] | select(.body | startswith(\"$MARK\")) | {id, body}" | jq -sc 'first // {}')" ||
    return 0
  id="$(jq -r '.id // ""' <<<"$old")"
  [ "$(jq -r '.body // ""' <<<"$old")" != "$body" ] || return 0
  if [ "$DRY_RUN" = 1 ]; then
    if [ -n "$id" ]; then
      say "$pr" "would update the comment"
    else
      say "$pr" "would comment and ask $OWNER for a review"
    fi
    return 0
  fi
  if [ -n "$id" ]; then
    gh api -X PATCH "repos/$GH_REPO/issues/comments/$id" -f body="$body" >/dev/null || true
  else
    # Asked once, with the first comment, so a run never asks again after a review.
    gh api -X POST "repos/$GH_REPO/issues/$pr/comments" -f body="$body" >/dev/null || true
    gh pr edit "$pr" --add-reviewer "$OWNER" >/dev/null ||
      say "$pr" "could not ask $OWNER for a review"
  fi
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
    note "$pr" "$outside"
    if ! approved "$pr" "$sha"; then
      say "$pr" "changes $(wc -l <<<"$outside") file(s) outside iPhone and Mac, waiting for $OWNER"
      return 0
    fi
    say "$pr" "approved by $OWNER"
  fi
  if ! checks_pass "$sha"; then
    say "$pr" "checks not passed yet on ${sha:0:8}"
    return 0
  fi
  if [ "$(jq -r .mergeable <<<"$info")" != MERGEABLE ]; then
    say "$pr" "not mergeable into main"
    return 0
  fi
  if [ "$MERGE" != 1 ] || [ "$DRY_RUN" = 1 ]; then
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
