# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Anibeam browses, plays and tracks a local anime library: it scans configured folders, matches series against AniList (TVmaze for what AniList does not carry), plays video in the window through libmpv, syncs progress with AniList and MyAnimeList, and renders a franchise relation graph. It is a Qt 6 QML shell over a Rust core. The Electron app it replaced was retired on 2026-10-19 (#70); the `v2.0.0` tag holds its source.

## Git Workflow

`CONTRIBUTING.md` holds the flow and the naming: issue first, a `<type>/<issue>-<slug>` branch, `<type>(<scope>): <outcome>` commits and PR titles, squash merge. Read it before the first commit of a task.

- **Never commit work directly on `main`.** One focused branch per task, deleted after merging. The one exception is the Release workflow's own bot commit that moves main to the next version after a stable release; Actions here may not open PRs.
- **Every build ships from main.** Nightlies come from `nightly.yml`, a stable version is a promoted nightly (Release workflow), a PR gets installers from the `test build` label, and `release/X.Y` exists only for a hotfix. Never create a release or `android-v*` pre-release for a test build by hand. CONTRIBUTING's "Releases" has the steps.
- **Merge only working code.** The checks in CONTRIBUTING's "Before you open it" pass first; `main` must always build.

## Commands

Cargo builds everything. There is no Bun or Node in the tree any more.

```bash
cargo build --release -p anibeam        # the Qt shell for shipping; to try a change, see below
cargo build --release -p anibeam-cli    # the terminal shell over the same core
cargo test --workspace                  # unit tests plus core/tests/
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

**Build only the crate you changed.** The core is splitting into one crate per team (#854), and each of `contract/`, `foundation/`, `net/`, `library/`, `releases/`, `metadata/`, `trackers/`, `downloads/`, `playback/` and `host/` builds and tests on its own. Iterate with `cargo test -p anibeam-library` (or `check`/`clippy -p`), not `--workspace` and not the shell; run the whole workspace once before the PR.

The shell needs `qmake6` on PATH, Qt 6.11+, MpvQt, libmpv and lld.

**A QML-only change needs no cargo build.** Where a box has an Anibeam Dev install (`~/.local/opt/anibeam-dev`, launched by `anibeam-dev`), the branch is tried there. It is a `--profile tester` build with the `dev-qml` feature (#927) and reads `qml/`, `themes/` and `mpv.conf` from that folder through `ANIBEAM_QML_DIR`, `ANIBEAM_THEMES_DIR` and `ANIBEAM_MPV_CONF`. When a branch changes nothing outside `apps/linux/qml/`, `apps/linux/themes/` and `apps/linux/mpv.conf`, copy them in and restart it. That takes seconds:

```bash
dev=~/.local/opt/anibeam-dev
rsync -a --delete apps/linux/qml/ "$dev/qml/"
rsync -a --delete apps/linux/themes/ "$dev/themes/"
cp apps/linux/mpv.conf "$dev/mpv.conf"
pkill -f "$dev/anibeam"; pidwait -f "$dev/anibeam"
setsid -f anibeam-dev >"$XDG_RUNTIME_DIR/anibeam-dev.log" 2>&1
```

Only one Anibeam GUI runs at a time: a second start hands off to the one already running and exits. So wait for the old one to exit, as above, and if another build is open (an installed Nightly, say), ask before closing it. Read the log afterwards: QML that uses a bridge property the installed binary lacks fails at load there, and then the change needs Rust after all.

A branch that touches Rust needs the tester build, installed before the copy and restart steps above:

```bash
buildq --pull target/tester/anibeam -- cargo build --profile tester -p anibeam --features dev-qml
install -m755 target/tester/anibeam ~/.local/opt/anibeam-dev/anibeam
```

`--pull` brings the binary back when buildq builds on another box. `install` replaces the file, so it works while the old one runs; `cp` over a running binary fails with "Text file busy". When nothing outside `apps/linux/qml/`, `apps/linux/themes/` and `apps/linux/mpv.conf` changed since the checkout's last tester build, buildq answers at once and compiles nothing: the binary already in `target/tester/` is current, so install it anyway, since Dev may be running another branch's. The Dev install runs on the owner's real data, so a branch that migrates the store is tried against a `--root` sandbox instead, never there. Never run a release build just to look at a change.

`--root <dir>` keeps a run's database, config, cache and secrets in one directory. Develop and capture against a sandbox root, never the owner's real XDG data. `apps/linux/README.md` covers offscreen captures (`apps/linux/scripts/shoot.sh`) and scripted drives (`apps/linux/scripts/drive.sh`); look at a capture before calling a visible change done.

## Architecture

A Cargo workspace. The members most work touches:

- **`contract/`** (`anibeam-contract`): the door every shell talks through: calls, records, events and errors, plus the catalog lookup (`i18n`). It sits below the core, which re-exports all of it, so shells keep naming `anibeam_core::...`. Tracker #854 splits the rest of the core the same way.
- **`foundation/`** (`anibeam-foundation`): what every team shares: the store (SQLite, migrations in `src/store/sql/`), the event bus, jobs, paths, prefs, profiles, secrets, redact, `Services`, and small helpers more than one team reads. It depends on the contract and the two leaf crates `host-api` and `room`; the core hands each module on under its old path, so `anibeam_core::store` and `crate::foundation::services` still resolve.
- **`net/`** (`anibeam-net`): the net team: one client per provider behind `src/limiter.rs`, the fronted and DoH clients, the fake transport the tests use, the image cache, and the build's version, which its user agents send. It depends on foundation and the contract only; the core hands it on as `anibeam_core::net`, `anibeam_core::images` and `anibeam_core::VERSION`.
- **`library/`** (`anibeam-library`): the library team: the sources, the scans, the folder walk and the watcher, the classifier, and the reads that turn the tables into cards, details and the snapshot, plus history, delete and the trash. It depends on net, foundation, the contract and the leaf `host-api` (the host's wire records) only, and reaches the downloads above it through its own `Incoming` trait; the core hands it on as `anibeam_core::library`.
- **`releases/`** (`anibeam-releases`): the releases team: anirss's release logic (endpoints, quirks, the feed readers, title parsing, best fit, refinement, the grid, rules and stats), what a release title says (`facts`), the endpoint repos and their pacing (`repos`), and the Feed page's reads (`feed`). It depends on library, net, foundation and the contract only, and reaches the downloads above it through its own `Searcher` and `AnirssEndpoints` traits; the core hands it on as `anibeam_core::releases`, `anibeam_core::repos` and `anibeam_core::feed`.
- **`metadata/`** (`anibeam-metadata`): the metadata team: matching and fetching, the franchise crawl and closure (`src/franchise/`), Recommended, Airing now, the aired episodes and the calendar feed. It depends on library, net, foundation, the contract and `recs` only, and hears what the trackers change on the bus; the core hands it on as `anibeam_core::metadata`, with `franchise`, `recommended`, `airing_now`, `aired` and `calendar` at the core's root.
- **`trackers/`** (`anibeam-trackers`): the trackers team: AniList and MAL, their accounts and sign-in, the list cache and the writes, Watching, move up, On hold and Plan to watch. It depends on metadata, library, net, foundation and the contract only, and says a list was read again on the bus; its build script compiles in the bundled tracker apps from `.env.local` (`env_local.rs`, which playback/build.rs shares for Discord). The core hands it on as `anibeam_core::trackers`.
- **`downloads/`** (`anibeam-downloads`): the downloads team: the anirss sidecar and the torrent client, search, the feed reads, rules, rolling downloads, replace and rewire, the watch for new episodes and the subscriptions query, with the phone's own sources and torrent engine in `src/phone/` behind its `phone` feature. It depends on trackers, metadata, releases, library, net, foundation and the contract only; the core hands it on as `anibeam_core::downloads` and `anibeam_core::phone`, and its `phone` turns the crate's on.
- **`playback/`** (`anibeam-playback`): the playback team: the session rules, tracks, skip times and the mpv options, Discord presence (`src/presence/`), casting behind its `cast` feature (`src/cast/`) and the watch together room (`src/room/`), the top of the team, which nothing below it names. It depends on downloads, trackers, metadata, releases, library, net, foundation, the contract and the leaves `room` and `host-api` only; its build script compiles in the bundled Discord id. The core hands it on as `anibeam_core::playback`, `presence`, `cast` and `room`, and its `cast` turns the crate's on.
- **`host/`** (`anibeam-host`): the host team: the tailnet host the phone reads (its server, keys, peers, trust, history, conversions and streams) and the library's export and import (`src/transfer/`). It depends on downloads, trackers, metadata, releases, library, net, foundation, the contract and the leaf `host-api` only, and reaches the core through its own `Front` trait; the core hands it on as `anibeam_core::host` and `anibeam_core::transfer`.
- **`core/`** (`anibeam-core`): the app team, and everything a shell calls: `Core` and its call router, launch, update, the changelog, the agent, stats, the uniffi export and `Subscription`. Integration tests for every team live in `core/tests/`.
- **`apps/linux/`** (`anibeam`): the QML shell, bridged to the core through cxx-qt 0.10. Rust bridges in `src/bridge/`, QML in `qml/`, the libmpv item in `cpp/`, themes in `themes/`, the bundled `mpv.conf`.
- **`apps/cli/`** (`anibeam-cli`): a terminal shell, and the quickest way to drive the core: `sources`, `scan`, `list`, `show`, `jobs`, `import`, `export`, `events --follow`, `call <Name> --json '{...}'`.

`CONTEXT.md` is the vocabulary the code and the docs share. The design behind the native line is `docs/superpowers/specs/2026-09-04-native-line-design.md`.

### Cross-cutting constraints

- **Rate limiting**: every provider call (AniList, MAL, Jikan, TVmaze, AniSkip and the rest of `net/src/`) goes through the shared limiter in `net/src/limiter.rs`. Never call a provider around it.
- **The activity log is signal-only**: log state changes, never per-asset, per-candidate or per-browse chatter.
- **Franchise closure**: `metadata/src/franchise/closure.rs` draws CHARACTER and OTHER edges but never walks them, so a crossover cameo (Isekai Quartet) cannot glue unrelated franchises together. Cross-franchise hops are pointers, not embedded copies.
- **Scores keep their source's scale**: `average_score` is stored as the provider gave it and normalised at read time. Don't convert it on write.
- **No polling the library**: the watcher plus a catch-up scan at launch, no intervals or periodic rescans.
- **One window, play on when hidden**: never a second window or an external mpv process, never auto-pause when the window is hidden.
- **App behaviour is self-contained**: no Hyprland windowrules, hyprctl or other compositor coupling.
- **Scrolling is native**: every Flickable gets `qml/WheelStep.qml`'s wheel handling, 100 px a notch glided to one target.
- **The app ships no sources**: nothing in this repo names or points at a download site, not in code, defaults, tests, fixtures or links. The core holds endpoint kinds, each a type of input named for its shape (RSS, torznab, a JSON search API), never for a site; a new kind is fine when a source needs one. Sites come only from an endpoint repo the user attaches: a PR to `marcusbandit/anibeam-endpoints`, or a repo of the contributor's own. Endpoints are a separate project: no code here hides, shows, starts or special-cases one source, and a source that needs a service running says so in its own definition or repo README. Rules in CONTRIBUTING.md under Sources.
- **Patch notes ship with the change**: anything a person can notice adds one short line, in the same PR, as a change file named after the branch: `apps/linux/changes/<issue>-<slug>.md`, `apps/android/changes/...` or both, holding `### Fixed` (or New, Changed, Removed) and `- the line`. Never write into `CHANGELOG.md`; the release folds the change files in (`.github/changelog-fold.sh`), and the apps show them as What's new meanwhile. Mojang style, 80 characters at most, never a paragraph; a version bump adds a heading. Rules in CONTRIBUTING.md under Patch notes.
- **Every release has a page on anibeam.app**: a nightly or a stable release that brings something new shows on the website's What's new (`anibeam.app/whats-new/`, a page per version at `/whats-new/<version>/`), not only on GitHub. The site builds those pages from the same `CHANGELOG.md` files and change files (`site/whats-new.py`), and `nightly.yml` and `release.yml` deploy the site once a release is public, so the change files are the whole job. A release whose notes would be empty on the site is a bug: fix the change files, never write the page by hand.

## Agent skills

### Issue tracker

Issues, specs and wayfinder maps live in this repo's GitHub Issues, driven with `gh`. See `docs/agents/issue-tracker.md`, and its "Issue state" section before starting, pausing or finishing work on an issue: labels follow branches and PRs, and the agent files what it notices, ticks boxes and closes what is solved.

### Triage

Read `docs/agents/triage.md` before triaging an issue or changing how incoming issues get triaged.

### Backlog review

Read `docs/agents/backlog-review.md` before reviewing the backlog or filling the board's Up next column.

### Domain docs

Single-context: one `CONTEXT.md` at the root and ADRs under `docs/adr/`. See `docs/agents/domain.md`.

### Subagents

Subagent use on this project is heavily allowed and expected. Fan work out to parallel subagents split by file ownership (commit by explicit path). Use Haiku for research, fix specs, captures, visual review, drive scripts, test runs and release builds; keep Opus for the final implementation. Never block the main conversation on a build or a long wait.
