# Contributing

One naming scheme runs through the whole path from idea to main: the issue,
the branch, every commit, the PR title and the squash commit it becomes.

## The flow

1. **Issue first.** Every change starts as a GitHub issue, except a typo-sized
   one (see "The PR body"). The
   title states the outcome, the body says what done looks like. What happens
   to an issue once it is filed is in
   [`docs/agents/triage.md`](docs/agents/triage.md). Every issue sits on the
   [Anibeam project board](https://github.com/users/marcusbandit/projects/1),
   and its card moves as the work does. A security problem never goes in an
   issue: [`SECURITY.md`](SECURITY.md) says where to report it privately.
   Before your first card, ask the owner for board access and run
   `gh auth refresh -s project`.
2. **Branch** off `main` as `<type>/<issue>-<slug>`.
3. **Commit** as you go, each message `<type>(<scope>): <outcome>`.
4. **Try it live** in a sandbox run of the branch's own build. See below.
5. **Open a PR** whose title follows the same pattern and whose body ends in
   `Closes #<issue>`.
6. **Review and checks.** Review comments answered, each fix as a new commit
   on the branch, and every check green. Never amend and force-push a pushed
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
| `ci`       | `.github/workflows`                                 |

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
- `docs/contributing-naming` for a `no issue` PR, the one kind without an issue

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
of handing off to it and never touches its data. anirss gets a home of its own
there too, `<root>/state/home`: the run reads anirss's config from
`<root>/state/home/.config/anirss/config.toml`, the anirss it starts keeps its
state there, and a fresh run sets up Anibeam's own qBittorrent rather than
taking one it finds on the machine. Point it at your own qBittorrent in
Settings when you want that. For a picture without a
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

Every PR closes an issue, and the `PR issue` check fails one that does not.
Only a closing keyword (`Closes`, `Fixes`, `Resolves`) in the body attaches it:
GitHub then lists the PR under the issue's Development section, and tools that
look work up by issue number can find it. These do not count:

- `Part of #<issue>`, `See #<issue>` or a bare mention
- a keyword in the title, a code span or a quote
- a keyword pointing at an issue that is already closed, such as the one a
  follow-up PR came from; a follow-up gets its own new issue

When a PR does only part of an issue, file a sub-issue for that part and close
the sub-issue. A typo-sized PR is the one exception: a maintainer gives it the
`no issue` label instead. Do not add that label to your own PR to get past the
check. When the check fails, its message says what to write; fix the body and
saving it reruns the check.

## Patch notes

Each app keeps its own `CHANGELOG.md`: `apps/linux/CHANGELOG.md` for the desktop,
`apps/android/CHANGELOG.md` for the phone. The app builds it in and shows it as What's new
after an update, so it is written for the person using the app, the way Mojang writes
Minecraft's patch notes: a version, a few sections, one short line per change.

- A change someone can notice gets one line, in the same PR. One that reaches both apps
  gets a line for each. Tests, CI, refactors and docs get none.
- The line goes in a change file of its own, never into `CHANGELOG.md`: a PR on
  `fix/160-seek-jump` adds `apps/linux/changes/160-seek-jump.md`, and
  `apps/android/changes/160-seek-jump.md` for the phone. It holds the line under New,
  Changed, Fixed or Removed, as a version does, with no heading. Use only the sections you
  need, in that order. Every PR writing into the same spot of `CHANGELOG.md` made each merge
  conflict with every other open PR (#504); separate files never do.
- The apps build the change files in and show them under the newest heading, so a nightly's
  What's new has them. When a version ships, `.github/changelog-fold.sh` folds them into
  `CHANGELOG.md` under its heading and deletes them: the Release workflow does it for both
  apps, each when its own version ships.
- Say what the person sees. No why, no how, no issue numbers, no file names, no full stop.
  80 characters at most; most lines fit in 50.
- The newest heading is the app's version, with the day: `## 2.1.1 · 2026-10-25`. Bumping
  the version adds a heading, and a heading for a new version bumps the version in the
  same PR. A line already in `CHANGELOG.md` can be fixed in place; a new one never goes
  there.
- A version can have a title to remember it by, between the number and the day:
  `## 2.5.0 · Quality of life · 2026-11-02`, 40 characters at most. Most versions have
  none. What's new and the update notice show it, and the release goes out named
  "Anibeam 2.5.0 · Quality of life"; a nightly of that version carries it too.

Good, as `apps/linux/changes/160-seek-jump.md`:

    ### New
    - Delete episodes from a series page

    ### Fixed
    - The seek bar no longer jumps back after a drag

Not:

    - Fixed a bug where the seek bar would sometimes jump back after the user
      released it, caused by a stale position in the player (#160).

`cargo test -p anibeam-core --test changelog` checks both files and every change file, and
that the fold writes them the way the apps show them; CI runs it on every pull request. CI
also runs `.github/changelog-newest.sh` against the PR's base, which fails a line the PR
writes into `CHANGELOG.md` under any heading but the newest, unless it was folded in from a
change file the same PR deletes. Run it by hand with `.github/changelog-newest.sh
origin/main`.
The phone's `build.sh` refuses an APK whose newest heading is not its version.

## Words

Every word a person reads lives in one catalog, the Fluent files under `i18n/` at the repo
root: `i18n/<lang>/core.ftl` for what the core says, `desktop.ftl` for the desktop and
`android.ftl` for the phone. English is the source; the other 12 languages translate it, and
an ID a language lacks shows in English. All of a language's files load as one, so a
`-term` or a shared message in `core.ftl` works from the others.

- A message has a stable ID naming where it shows and what it says, never the English:
  `player-skip-intro`, `downloads-empty`. Counts use Fluent's plurals, never a word picked
  in code.
- The code asks for the ID: `anibeam_core::i18n::tr("player-skip-intro", &[])` in Rust, and
  through the shell's own lookup in QML and Slint. No shell writes a word of its own.
- In QML, `words.t("library-empty", { count: n })` gives the words, and a binding that calls it
  follows a language switch. `Label`, `Button`, `Chip`, `Tooltip`, `Notice` and `ErrorLine`
  take the ID itself: `Button { msg: "action-cancel" }`, with `msgArgs: ({ count: n })`. An ID
  starts with its file's name: `library-` in `LibraryPage.qml`, `settings-playback-` in
  `SettingsPlaybackTab.qml`. Words every screen shares, such as `action-cancel` and
  `status-watching`, live in `core.ftl`.
- One message per sentence, its numbers and names as arguments. A sentence glued together in
  code cannot be translated, and a count picks its plural in the message, never in code.
- Never give an element `id: words`: it hides the global lookup from everything inside it.
- A message added or changed in English shows in English in the other languages until it is
  translated; add its translations in the same PR when you can, keeping the IDs, the
  attributes and the variables exactly as English has them.
- Labels must not grow. `apps/linux/scripts/layout-check.py --root <sandbox with a library>`
  shoots every page and Settings tab in every language at a wide and a narrow window and lists
  each piece of text that wraps onto more lines than in English or is cut off. Run it after a
  change to a label's words or to the layout around one.
- A new message goes in English in the same PR. A translation can follow later.

`.github/strings-check.py` runs on every pull request. It fails a raw string added to a
shell's UI files and an ID missing from `i18n/en/`. The files that held raw strings before
the catalog are listed in `.github/strings-baseline.txt`; a file may hold no more than its
count there, and a PR that moves strings out has to lower it in the same PR with
`.github/strings-check.py --update`. A line that holds no words for a person, such as a
format string, ends in `// i18n: none`.

## Sources

Anibeam ships no download sources, and this repo takes no PR that adds one. Nothing in
the app points at a site: no address, no host, no default, no onboarding link, and no code,
test or fixture that names one. A PR that adds a site gets closed, however well it is built.
CI keeps it that way: `.github/no-site-names.sh` greps the tree, file names included, for
the known site names and fails on any hit. It skips what is kept as history (the changelogs
and dated plans), applied SQL migrations, lockfiles, and `core/src/releases/kinds.rs`, the one
module that holds the kinds' old names to rename them.

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
- A repo of your own, signed with your own key, that anyone can add in Anibeam beside or
  instead of the owner's.

Endpoints are not part of this project, and that is final. No PR here decides which sources
exist, whether one is hidden or shown, or how a particular one is handled. A source that only
works with something else running, such as an indexer proxy or a challenge solver, is still
just a definition in an endpoint repo: it uses an existing kind, and its definition or that
repo's README says what has to be running and how to set it up. The app never bundles,
starts or special-cases a service for one source. If a source seems to need app code beyond a
kind, open an issue about the kind and leave the source out.

## Before you open it

- The test, clippy and fmt commands in `docs/build-from-source.md` under Working on it pass.
- A visible change has a capture in the PR body, taken against a sandbox root.
- A change someone can notice has its line in a change file under the app's `changes/`.
- A new word for something in the domain has its entry in `CONTEXT.md`.
- Nothing in the diff names a download site. See Sources.
- `.github/strings-check.py` passes: new words go in `i18n/en/` under an ID. See Words.

## Releases

Every build ships from main. What tells them apart is the version, and the app reads its
update track from that version:

| Build | Version | Comes from | Who gets it |
| --- | --- | --- | --- |
| Stable | `2.5.0` | Release workflow, promoting a nightly | everyone; the Latest release |
| Nightly | `2.5.0-nightly.20261102.40` | `nightly.yml`, run on demand | the Nightly track in Settings |
| PR test build | `2.5.0-pr.280.3` | `pr-build.yml`, the `test build` label | whoever the PR comment reaches |
| Phone stable | `android-v2.4.3` | Release workflow with app `phone`, promoting a phone nightly | everyone with the phone app |
| Phone nightly | `android-v2.4.3-nightly.20261113.41` | `nightly.yml`, the same run | whoever installs it |

The workspace version in `Cargo.toml` is always the next stable version, and every build
between two releases carries it as its base. The phone has a number of its own, the
`version` in `apps/android/Cargo.toml`, and the same rule holds for it. Its APK's
versionCode follows from the version (`apps/android/scripts/build.sh`), so a nightly
installs over the stable version before it, the stable version over its nightlies, and the
next version's nightlies over both. Nobody bumps a versionCode by hand.

Every nightly and stable release that brings something new also gets its page on the website:
[anibeam.app/whats-new](https://anibeam.app/whats-new/) lists every version, newest first, with
a page each, and shows what has merged since the last stable release as Coming next. The site
builds them from the patch notes below (`site/whats-new.py`), and `nightly.yml` and the Release
workflow deploy the site once their release is public, so writing the change files is all a
release needs. Nobody writes these pages by hand.

- **Nightly.** The less polished, frontier build, not a daily one. Run `nightly.yml` from
  the Actions tab when main holds something worth trying; it builds main and publishes it as
  its own pre-release, and skips the run when the app has not changed since the last
  nightly unless `force` is ticked. The ten newest are kept. The same run builds the phone's
  APK when the phone or what it builds on changed since the phone's last nightly, and
  publishes it as its own `android-v*-nightly.*` pre-release.
- **PR test build.** Put the `test build` label on a PR to get installers for it on the
  `pr-builds` pre-release, linked from a comment on the PR. Every push rebuilds them;
  closing the PR or taking the label off removes them. Use it to have someone try a
  branch before it merges, a Mac change for Liam for example.
- **Stable.** When the newest nightly has been run for a while and is good, run the Release
  workflow on main from the Actions tab. It promotes that nightly's commit, so what ships is
  a build people have already used: nightly `2.5.0-nightly.20261102.40` ships as `v2.5.0`.
  If a workflow file changed on main since that nightly, GitHub will not let the workflow
  tag it, so it stops and asks for a forced Nightly first.
  It builds every platform, publishes the release once every file is on it, folds the
  change files that shipped into `CHANGELOG.md` under `2.5.0`, and moves main to `2.6.0`
  with a new heading. Then update the AUR packages (`anibeam` and `anibeam-bin`, one
  command) and the brew cask: `apps/linux/packaging/aur/publish.sh 2.5.0` and
  `apps/linux/packaging/brew/bump.sh 2.5.0`.
- **Phone stable.** The same Release workflow with app set to `phone`: it promotes the
  newest phone nightly, so `android-v2.4.3-nightly.20261113.41` ships as `android-v2.4.3`,
  an APK built from that commit. It is never the Latest release, which stays the desktop's
  for the site and `install.sh`. Then it folds `apps/android/changes` into the phone's
  `CHANGELOG.md` and moves the phone to `2.5.0` with a new heading. A phone version that
  needs a fix gets it on main and ships in the next phone version.
- **Hotfix.** A shipped version that needs a fix gets a branch from its tag, only then:
  `git push origin v2.5.0^{commit}:refs/heads/release/2.5`. The fix lands on main first
  through the usual PR, then is cherry-picked onto the branch with `git cherry-pick -x` in
  a PR against `release/2.5`. The branch still holds 2.5.0's change files, since the fold
  happened on main, so that PR first runs `.github/changelog-fold.sh apps/linux` to put them
  under `2.5.0`, then adds a `2.5.1` heading with the version bumped to match. The change
  file the cherry-pick brings shows under it. Tag it to ship: `git tag v2.5.1 origin/release/2.5 && git push
  origin v2.5.1`. The Release workflow refuses a tag that is not on its own branch. The
  branch is never merged back, since everything on it came from main.

Every phone APK on a release is signed with one key, the `ANIBEAM_ANDROID_KEYSTORE`
secret (a base64 keystore), because Android installs an update only over an install signed
with the same key. A phone build is never published by hand: run the Nightly workflow.

A build made outside CI (a checkout, the AUR) names itself after the newest stable tag in
its history plus commits, `2.4.0.r12.gabc1234`, and counts as the workspace version for
updates.
