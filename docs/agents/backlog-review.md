# Backlog review

Once a week, Mondays at 09:00 from the "AniBeam backlog review" T3 schedule on banditbox, an agent walks the board's Backlog against `main` and refills Up next. Issues land in
Backlog faster than they leave it, and finished work often stays open because a PR landed a piece
of a tracker without ticking its box. The review is how nothing gets forgotten.

The board, its fields and the `gh project` commands are in [`issue-tracker.md`](issue-tracker.md).

## The pass

1. **Every open issue is on the board.** Compare `gh issue list --state open` with the board's items
   and add any that are missing.
2. **Check each Backlog issue against `main`.** Fetch first and read `origin/main`, never a stale
   checkout. Use the issue body, its comments, linked PRs, `git log` and `git grep`. Give each one
   a verdict:
   - **Done**: every part is on main, with a commit, PR or `file:line` to show for it.
   - **Partly**: some of it is on main.
   - **Not started**.
   - **Stale**: superseded by another issue, or its premise no longer holds.

   The verdicts are independent, so fan the issues out to parallel subagents in batches of about
   15, read-only.
3. **Act on the verdicts.**
   - Done: close it with a comment naming the evidence. When only a run on a device or a Mac can
     confirm it, comment instead and leave it open.
   - Partly: tick the checklist boxes that are on main, changing `[ ]` to `[x]` and nothing else,
     and comment with what is on main and what is still open. Skip the comment when the last
     review's comment still holds.
   - Stale: comment why and ask the owner in the summary. Never close a stale issue on your own.
   - Not started: leave it alone.
4. **Keep the other columns true.** A closed issue goes to Done. An In progress issue with no
   branch, PR or commit in two weeks goes back to Backlog with a comment.
5. **Refill Up next to at most 5.** Up next is the review's pick of what the owner should look at
   next. Prefer, in order: bugs a user can hit, small fixes (an hour or less) with a clear done,
   whatever unblocks other issues, and partly done issues close to finished. Leave out issues that
   need a device or a person the owner does not have to hand. Issues the owner moved into Up next
   themselves stay there.
6. **Report in a few lines**: what closed, what is Up next and why, and what needs a decision.

Comments start with `Backlog review, YYYY-MM-DD.` so the next pass can find the last one.
