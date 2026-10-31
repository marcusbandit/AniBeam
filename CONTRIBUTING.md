# Contributing

One naming scheme runs through the whole path from idea to main: the issue,
the branch, every commit, the PR title and the squash commit it becomes.

## The flow

1. **Issue first.** Anything bigger than a typo starts as a GitHub issue. The
   title states the outcome, the body says what done looks like. What happens
   to an issue once it is filed is in
   [`docs/agents/triage.md`](docs/agents/triage.md).
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
- `feat(downloads): a provider switch on the search bar`
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

## Patch notes

Each app keeps its own `CHANGELOG.md`: `apps/linux/CHANGELOG.md` for the desktop,
`apps/android/CHANGELOG.md` for the phone. The app builds it in and shows it as What's new
after an update, so it is written for the person using the app, the way Mojang writes
Minecraft's patch notes: a version, a few sections, one short line per change.

- A change someone can notice gets one line, in the same PR. One that reaches both apps
  gets a line in both files. Tests, CI, refactors and docs get none.
- The line goes under the newest heading, in New, Changed, Fixed or Removed. Use only the
  sections you need, in that order.
- Say what the person sees. No why, no how, no issue numbers, no file names, no full stop.
  80 characters at most; most lines fit in 50.
- The newest heading is the app's version, with the day: `## 2.1.1 · 2026-10-25`. Bumping
  the version adds a heading, and a heading for a new version bumps the version in the
  same PR. Never add a line under a version that has shipped.
- A version can have a title to remember it by, between the number and the day:
  `## 2.5.0 · Quality of life · 2026-11-02`, 40 characters at most. Most versions have
  none. What's new and the update notice show it, and the release goes out named
  "AniBeam 2.5.0 · Quality of life"; a nightly of that version carries it too.

Good:

    ### New
    - Delete episodes from a series page

    ### Fixed
    - The seek bar no longer jumps back after a drag

Not:

    - Fixed a bug where the seek bar would sometimes jump back after the user
      released it, caused by a stale position in the player (#160).

`cargo test -p anibeam-core --test changelog` checks both files, and CI runs it on every
pull request. The phone's `build.sh` refuses an APK whose newest heading is not its
`versionName`.

## Sources

AniBeam ships no download sources, and this repo takes no PR that adds one. Nothing in
the app points at a site: no address, no host, no default, no onboarding link, and no code,
test or fixture that names one. A PR that adds a site gets closed, however well it is built.

What the core does hold is endpoint kinds. A kind is a type of input the core knows how to
read and handle well: an RSS feed, a torznab API, a JSON search API, a feed that needs a key.
It is named for that shape, never for a site, and it knows nothing about which sites use it.
A new kind is welcome when no existing one can read a source; open an issue for it first.

The site itself comes from an endpoint repo the user attaches by URL
(`docs/superpowers/specs/2026-10-24-endpoint-repos.md` has the format). Its definition says
which kind to use and where to point it, and installing it is what tells the app how to handle
that site. A new source goes in one of two places:

- A PR to [`marcusbandit/anibeam-endpoints`](https://github.com/marcusbandit/anibeam-endpoints),
  the owner's own repo. Its README says how to write a definition.
- A repo of your own, signed with your own key, that anyone can add in AniBeam beside or
  instead of the owner's.

## Before you open it

- The test, clippy and fmt commands in the README's Development section pass.
- A visible change has a capture in the PR body, taken against a sandbox root.
- A change someone can notice has its line in the app's `CHANGELOG.md`.
- A new word for something in the domain has its entry in `CONTEXT.md`.
- Nothing in the diff names a download site. See Sources.

## Releases

Every build ships from main. What tells them apart is the version, and the app reads its
update track from that version:

| Build | Version | Comes from | Who gets it |
| --- | --- | --- | --- |
| Stable | `2.5.0` | Release workflow, promoting a nightly | everyone; the Latest release |
| Nightly | `2.5.0-nightly.20261102.40` | `nightly.yml`, every few hours | the Nightly track in Settings |
| PR test build | `2.5.0-pr.280.3` | `pr-build.yml`, the `test build` label | whoever the PR comment reaches |

The workspace version in `Cargo.toml` is always the next stable version, and every build
between two releases carries it as its base.

- **Nightly.** `nightly.yml` builds main every few hours when the app changed and the last
  nightly is six hours old, and publishes it as its own pre-release. The ten newest are
  kept. Run it from the Actions tab with `force` for one now.
- **PR test build.** Put the `test build` label on a PR to get installers for it on the
  `pr-builds` pre-release, linked from a comment on the PR. Every push rebuilds them;
  closing the PR or taking the label off removes them. Use it to have someone try a
  branch before it merges, a Mac change for Liam for example.
- **Stable.** When the newest nightly has been run for a while and is good, run the Release
  workflow on main from the Actions tab. It promotes that nightly's commit, so what ships is
  a build people have already used: nightly `2.5.0-nightly.20261102.40` ships as `v2.5.0`.
  It builds every platform, publishes the release once every file is on it, and moves main
  to `2.6.0` with a new `CHANGELOG.md` heading. Then update the AUR package and the brew
  cask: `apps/linux/packaging/aur/publish.sh 2.5.0` and
  `apps/linux/packaging/brew/bump.sh 2.5.0`.
- **Hotfix.** A shipped version that needs a fix gets a branch from its tag, only then:
  `git push origin v2.5.0^{commit}:refs/heads/release/2.5`. The fix lands on main first
  through the usual PR, then is cherry-picked onto the branch with `git cherry-pick -x` in
  a PR against `release/2.5`, with its patch note under a new `2.5.1` heading and the
  version bumped to match. Tag it to ship: `git tag v2.5.1 origin/release/2.5 && git push
  origin v2.5.1`. The Release workflow refuses a tag that is not on its own branch. The
  branch is never merged back, since everything on it came from main.

A phone build for testing goes on the newest nightly under one fixed name instead of a new
`android-v*` pre-release: `cp AniBeam-2.1.2.apk AniBeam-phone.apk && gh release upload
<nightly tag> AniBeam-phone.apk --clobber`. The phone ships under its own number,
`android-v2.1.2` for example, published by hand.

A build made outside CI (a checkout, the AUR) names itself after the newest stable tag in
its history plus commits, `2.4.0.r12.gabc1234`, and counts as the workspace version for
updates.
