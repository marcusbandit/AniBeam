# Contributing

One naming scheme runs through the whole path from idea to main: the issue,
the branch, every commit, the PR title and the squash commit it becomes.

## The flow

1. **Issue first.** Anything bigger than a typo starts as a GitHub issue. The
   title states the outcome, the body says what done looks like.
2. **Branch** off `main` as `<type>/<issue>-<slug>`.
3. **Commit** as you go, each message `<type>(<scope>): <outcome>`.
4. **Try it live** in a sandbox run of the branch's own build. See below.
5. **Open a PR** whose title follows the same pattern and whose body ends in
   `Closes #<issue>`.
6. **Review and checks.** Review comments answered, each fix as a new commit
   on the branch, and checks green once CI exists (#73). Never amend and force-push a pushed
   branch.
7. **Squash merge** with `gh pr merge --squash --delete-branch`. The PR title
   becomes the one commit on main, with `(#N)` appended by GitHub.

## Types

| Type       | Use it for                                          |
| ---------- | --------------------------------------------------- |
| `feat`     | something new the app can do                        |
| `fix`      | something broken now works                          |
| `perf`     | same behaviour, less CPU, memory or frame time      |
| `refactor` | code moves or changes shape, behaviour stays        |
| `docs`     | README, CONTEXT, CONTRIBUTING, `docs/`              |
| `test`     | tests only                                          |
| `chore`    | dependencies, tooling, cleanup                      |
| `ci`       | `.github/workflows`, once #73 adds them             |

## Scopes

The scope is the crate the change belongs to: `core`, `shell` for
`apps/linux`, `cli` for `apps/cli`. A shell change that lives on one page or
in one part of the window can name it instead: `home`, `library`, `series`,
`feed`, `watching`, `downloads`, `metadata`, `tokens`, `settings`, `player`.

A `docs` change names the document: `readme`, `contributing`, `context`,
`spec`. Outside the crates and the docs use `packaging` or `ci`. A change that
genuinely spans two scopes lists both, comma separated:
`fix(core,shell): ...`. More than two usually means the PR should split.

## Branch names

    <type>/<issue>-<slug>

- `feat/69-downloads-redesign`
- `perf/64-shared-cargo-target`
- `docs/contributing-naming` when no issue exists, which should be rare

The slug is a few lowercase hyphenated words naming the outcome.

## Trying a branch live

The launcher entry runs the main checkout's `target/release/anibeam`, so a
release build in the main checkout replaces what the launcher starts. Work on
a branch in a worktree under `~/Projects/_wt/` and run that worktree's own
build:

    cargo build --release -p anibeam
    ANIBEAM_THEMES_DIR=apps/linux/themes ANIBEAM_MPV_CONF=apps/linux/mpv.conf \
      target/release/anibeam --root /tmp/anibeam-dev

`--root` keeps the database, config, cache and secrets in that directory, and
gives the run its own bus names, so it opens beside the installed app instead
of handing off to it and never touches its data. For a picture without a
window, `apps/linux/scripts/shoot.sh` captures a page offscreen and
`apps/linux/scripts/drive.sh` clicks through a script first, and shoots a set
of pages in one run with a `pages` step; see `apps/linux/README.md`.

## Commit and PR titles

    <type>(<scope>): <what is different now>

Write the outcome a user would notice, in plain words, lowercase after the
colon, no trailing full stop. Say what changed for the person, not which file
moved.

Good:

- `fix(shell): the wheel scrolls like a browser`
- `feat(downloads): a provider switch with 1337x`
- `perf(shell): one drive run captures several pages`

Not:

- `Fix bug` says nothing
- `shell: update WheelStep.qml` names a file, not an outcome
- `Merge pull request #59 from marcusbandit/docs/readme-native-line` is a
  merge commit; squash instead

## The PR body

- What changed, in a few lines.
- Why, linking the issue.
- How it was verified: a screenshot or recording for anything visible,
  test output otherwise.
- `Closes #<issue>` on its own line.

## Before you open it

- The test, clippy and fmt commands in the README's Development section pass.
- A visible change has a capture in the PR body, taken against a sandbox root.
- A new word for something in the domain has its entry in `CONTEXT.md`.
