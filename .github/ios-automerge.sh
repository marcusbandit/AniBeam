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
#   MERGE     1 to merge; anything else only says what it would do
#   DRY_RUN   1 to say what it would do without commenting, asking or merging
#
# It reads PRs through the API only and never runs their code.
set -euo pipefail
: "${GH_REPO:?}" "${AUTHORS:?}" "${OWNER:?}"
MERGE="${MERGE:-0}"
DRY_RUN="${DRY_RUN:-0}"
SELF_WORKFLOW="iOS automerge"
SELF_JOB="ios-automerge"
MARK="<!-- ios-automerge -->"

# Paths that only the iPhone or the Mac build reads. Everything else, Cargo.toml and
# Cargo.lock included, can change another platform.
ONLY='^(apps/android/ios/'
ONLY+='|apps/android/src/(.+/)?(ios|metal)(\.rs|/)'
ONLY+='|apps/android/scripts/[^/]*ios[^/]*'
ONLY+='|apps/android/changes/[^/]+\.md$'
ONLY+='|apps/linux/packaging/macos/'
ONLY+='|\.github/workflows/(ios-[^/]+|macos)\.yml$'
ONLY+=')'

say() { echo "#$1: $2"; }

# Every check run and workflow run on the commit finished as passed or skipped. A run that
# has not started yet keeps the PR waiting, so a push is never merged before its checks.
checks_pass() {
  local sha="$1" bad
  bad="$(gh api --paginate "repos/$GH_REPO/commits/$sha/check-runs" \
    --jq ".check_runs[] | select(.name != \"$SELF_JOB\")
          | select(.status != \"completed\" or (.conclusion | IN(\"success\", \"skipped\", \"neutral\") | not))
          | \"\(.name) \(.status) \(.conclusion)\"")"
  # The newest run of each workflow: an older one a later run cancelled does not count.
  bad+="$(gh api --paginate "repos/$GH_REPO/actions/runs?head_sha=$sha" --jq '.workflow_runs[]' |
    jq -rs --arg self "$SELF_WORKFLOW" '
      map(select(.name != $self)) | group_by(.workflow_id) | map(max_by(.created_at))[]
      | select(.status != "completed" or (.conclusion | IN("success", "skipped", "neutral") | not))
      | "\(.name) \(.status) \(.conclusion)"')"
  [ -z "$bad" ] || { echo "$bad" | sed 's/^/    /'; return 1; }
}

# OWNER's latest review that approves or turns down, on the head commit, is an approval.
approved() {
  local pr="$1" sha="$2"
  [ "$(gh api --paginate "repos/$GH_REPO/pulls/$pr/reviews" \
    --jq "[.[] | select(.user.login == \"$OWNER\" and .commit_id == \"$sha\")
           | select(.state | IN(\"APPROVED\", \"CHANGES_REQUESTED\"))] | last | .state // \"\"")" \
    = APPROVED ]
}

# One comment per PR listing what keeps it waiting, written once and kept current.
note() {
  local pr="$1" outside="$2" body old id
  body="$MARK
This PR changes more than the iPhone and Mac apps, so it merges itself only after $OWNER approves its latest commit. The files outside iPhone and Mac:

$(head -n 30 <<<"$outside" | sed 's/.*/- `&`/')"
  [ "$(wc -l <<<"$outside")" -le 30 ] || body+="
- and $(($(wc -l <<<"$outside") - 30)) more"
  old="$(gh api --paginate "repos/$GH_REPO/issues/$pr/comments" \
    --jq ".[] | select(.body | startswith(\"$MARK\")) | {id, body}" | jq -sc 'first // {}')"
  id="$(jq -r '.id // ""' <<<"$old")"
  [ "$(jq -r '.body // ""' <<<"$old")" != "$body" ] || return 0
  if [ "$DRY_RUN" = 1 ]; then
    say "$pr" "would ${id:+update the comment}${id:-comment and ask $OWNER for a review}"
    return
  fi
  if [ -n "$id" ]; then
    gh api -X PATCH "repos/$GH_REPO/issues/comments/$id" -f body="$body" >/dev/null
  else
    # Asked once, with the first comment, so a run never asks again after a review.
    gh api -X POST "repos/$GH_REPO/issues/$pr/comments" -f body="$body" >/dev/null
    gh pr edit "$pr" --add-reviewer "$OWNER" >/dev/null 2>&1 || true
  fi
}

sweep() {
  local merged=0 authors pr sha files outside
  authors="$(tr ', ' '\n\n' <<<"$AUTHORS" | sed '/^$/d' | jq -R . | jq -sc .)"
  for pr in $(gh pr list --state open --base main --limit 100 --json number,author,isDraft \
    --jq ".[] | select(.isDraft | not) | select(.author.login | IN(${authors}[])) | .number"); do
    sha="$(gh pr view "$pr" --json headRefOid --jq .headRefOid)"
    files="$(gh api --paginate "repos/$GH_REPO/pulls/$pr/files" --jq '.[].filename')"
    outside="$(grep -Ev "$ONLY" <<<"$files" || true)"
    if [ -n "$outside" ]; then
      note "$pr" "$outside"
      if ! approved "$pr" "$sha"; then
        say "$pr" "changes $(wc -l <<<"$outside") file(s) outside iPhone and Mac, waiting for $OWNER"
        continue
      fi
      say "$pr" "approved by $OWNER"
    fi
    if ! checks_pass "$sha"; then
      say "$pr" "checks not passed yet on ${sha:0:8}"
      continue
    fi
    if [ "$(gh pr view "$pr" --json mergeable --jq .mergeable)" != MERGEABLE ]; then
      say "$pr" "not mergeable into main"
      continue
    fi
    if [ "$MERGE" != 1 ] || [ "$DRY_RUN" = 1 ]; then
      say "$pr" "would merge ${sha:0:8}"
      continue
    fi
    # --match-head-commit: a push after the checks above stops the merge.
    if gh pr merge "$pr" --squash --match-head-commit "$sha"; then
      say "$pr" "merged ${sha:0:8}"
      merged=1
    else
      say "$pr" "merge refused"
    fi
  done
  return $((1 - merged))
}

# A merge deletes its branch, which moves a PR stacked on it onto main, so look again.
for _ in 1 2 3; do
  sweep || break
  sleep 5
done
