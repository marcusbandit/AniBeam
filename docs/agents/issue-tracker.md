# Issue tracker: GitHub

Issues and specs for this repo live as GitHub issues. Use the `gh` CLI for all operations.

## Conventions

- **Create an issue**: `gh issue create --title "..." --body "..."`. Use a heredoc for multi-line bodies.
- **Read an issue**: `gh issue view <number> --comments`, filtering comments by `jq` and also fetching labels.
- **List issues**: `gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'` with appropriate `--label` and `--state` filters.
- **Comment on an issue**: `gh issue comment <number> --body "..."`
- **Apply / remove labels**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **Close**: `gh issue close <number> --comment "..."`

Infer the repo from `git remote -v`; `gh` does this automatically when run inside a clone.

## Project board

Every open issue sits on the AniBeam project, https://github.com/users/marcusbandit/projects/1 (owner `marcusbandit`, number 1). Its Status column is what the owner reads to see where work stands, so keep it true:

| Status      | When                                                     |
| ----------- | -------------------------------------------------------- |
| Backlog     | Filed, nobody picked it                                  |
| Up next     | Picked for a coming session, see below                   |
| In progress | A thread started work on it, set before the first commit; label `in progress` |
| In review   | Its PR is open; label `in review`                        |
| Done        | Set by GitHub when the issue closes or the PR merges     |

Up next holds at most 5 issues. The weekly [backlog review](backlog-review.md) fills it with what the owner should look at next; the owner reorders it, and an issue they add themselves stays.

Platform is `Desktop`, `Phone`, `Both` (core work both apps use) or `Dev` (CI, build, tooling). Release names the version an issue is meant for, when one is set.

The board is private. A collaborator needs two things once: the owner adds them under the board's Settings, Manage access, and their `gh` login needs the `project` scope, `gh auth refresh -s project`. Without the scope every `gh project` command fails with a missing-scope error.

- **Add a new issue**: `gh project item-add 1 --owner marcusbandit --url <issue-url> --format json --jq .id` gives the item id. Every issue you file goes on the board.
- **Move a card**: look up the ids once with `gh project field-list 1 --owner marcusbandit --format json`, then `gh project item-edit --project-id <project-id> --id <item-id> --field-id <status-field-id> --single-select-option-id <option-id>`.
- **Find an issue's item**: `gh project item-list 1 --owner marcusbandit --limit 500 --format json --jq '.items[] | select(.content.number == <n>) | .id'`.

## Issue state

An issue says where its work stands through labels, so it reads true on GitHub and on a phone
without opening the board. `.github/workflows/issue-state.yml` sets them; nobody has to remember.

| Label | Goes on when | Comes off when |
| --- | --- | --- |
| `in progress` | a branch `<type>/<n>-<slug>` is pushed, or a draft PR closing #n opens | the PR is marked ready, the branch is deleted, or the issue closes |
| `in review` | a ready PR that closes #n opens | the PR closes; unmerged, the issue goes back to `in progress` |

A merged PR that says `Part of #n` leaves a comment on #n naming itself, and #n stays open. The
board's Status mirrors the labels through the daily sweep below.

What an agent does, on top of what the workflow does:

- **Starts work**: run `gh issue list` first and work under the matching issue, or file one. Push
  the branch, named for the issue, before or with the first commit, so `in progress` goes on, and
  move the card to In progress.
- **Notices something it was not asked to fix**: file it with one category label (`bug`,
  `enhancement`, `documentation`, `question`), add it to the board, and leave the code alone.
- **Finishes part of a tracker**: tick the boxes that are now on main, `[ ]` to `[x]` and nothing
  else, and put `Part of #n` in the PR body next to the `Closes` line for the sub-issue.
- **Stops before the work is done**: push a draft PR and comment on the issue with what is on the
  branch and what is left.
- **Finds an issue already solved**, by its own route or another: close it with a comment naming
  the commit, PR or `file:line`. When only a device, a Mac or a person can confirm it, comment
  and leave it open. An issue that looks stale or superseded gets a comment and a question to
  the owner, never a close on the agent's own judgment.

### Schedules

Two T3 scheduled tasks on banditbox keep the state true:

- **AniBeam issue sweep**, daily at 07:30, against `marcusbandit/AniBeam`. Every open issue is on the
  board; the board's Status matches the labels (`in progress` is In progress, `in review` is In
  review, neither is Backlog or Up next); every open issue has one category label; an
  `in progress` issue with no branch, PR or commit in 14 days loses the label and goes back to
  Backlog with a comment; an open issue whose closing PR merged is closed. It only reads
  GitHub, so it needs no worktree.
- **AniBeam backlog review**, Mondays at 09:00, a new thread per run: the full pass in
  [`backlog-review.md`](backlog-review.md).

## Triage

The triage labels and what happens to an issue once it is filed: [`triage.md`](triage.md).

## Pull requests as a triage surface

**PRs as a request surface: no.** _(Set to `yes` if this repo treats external PRs as feature requests; `/triage` reads this flag.)_

When set to `yes`, PRs run through the same labels and states as issues, using the `gh pr` equivalents:

- **Read a PR**: `gh pr view <number> --comments` and `gh pr diff <number>` for the diff.
- **List external PRs for triage**: `gh pr list --state open --json number,title,body,labels,author,authorAssociation,comments` then keep only `authorAssociation` of `CONTRIBUTOR`, `FIRST_TIME_CONTRIBUTOR`, or `NONE` (drop `OWNER`/`MEMBER`/`COLLABORATOR`).
- **Comment / label / close**: `gh pr comment`, `gh pr edit --add-label`/`--remove-label`, `gh pr close`.

GitHub shares one number space across issues and PRs, so a bare `#42` may be either: resolve with `gh pr view 42` and fall back to `gh issue view 42`.

## When a skill says "publish to the issue tracker"

Create a GitHub issue.

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --comments`.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a single issue with **child** issues as tickets.

- **Map**: a single issue labelled `wayfinder:map`, holding the Notes / Decisions-so-far / Fog body. `gh issue create --label wayfinder:map`.
- **Child ticket**: an issue linked to the map as a GitHub sub-issue (`gh api` on the sub-issues endpoint). Where sub-issues aren't enabled, add the child to a task list in the map body and put `Part of #<map>` at the top of the child body. Labels: `wayfinder:<type>` (`research`/`prototype`/`grilling`/`task`). Once claimed, the ticket is assigned to the driving dev.
- **Blocking**: GitHub's **native issue dependencies**, the canonical, UI-visible representation. Add an edge with `gh api --method POST repos/<owner>/<repo>/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-db-id>`, where `<blocker-db-id>` is the blocker's numeric **database id** (`gh api repos/<owner>/<repo>/issues/<n> --jq .id`, _not_ the `#number` or `node_id`). GitHub reports `issue_dependencies_summary.blocked_by` (open blockers only, the live gate). Where dependencies aren't available, fall back to a `Blocked by: #<n>, #<n>` line at the top of the child body. A ticket is unblocked when every blocker is closed.
- **Frontier query**: list the map's open children (`gh issue list --state open`, scoped to the map's sub-issues / task list), drop any with an open blocker (`issue_dependencies_summary.blocked_by > 0`, or an open issue in the `Blocked by` line) or an assignee; first in map order wins.
- **Claim**: `gh issue edit <n> --add-assignee @me`, the session's first write.
- **Resolve**: `gh issue comment <n> --body "<answer>"`, then `gh issue close <n>`, then append a context pointer (gist + link) to the map's Decisions-so-far.
