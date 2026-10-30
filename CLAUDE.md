# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

AniBeam browses, plays and tracks a local anime library: it scans configured folders, matches series against AniList (TVmaze for what AniList does not carry), plays video in the window through libmpv, syncs progress with AniList and MyAnimeList, and renders a franchise relation graph. It is a Qt 6 QML shell over a Rust core. The Electron app it replaced was retired on 2026-10-19 (#70); the `v2.0.0` tag holds its source.

## Git Workflow

`CONTRIBUTING.md` holds the flow and the naming: issue first, a `<type>/<issue>-<slug>` branch, `<type>(<scope>): <outcome>` commits and PR titles, squash merge. Read it before the first commit of a task.

- **Never commit work directly on `main`.** One focused branch per task, deleted after merging.
- **Stable releases come from `release/*` branches only.** Main ships as the rolling `preview` pre-release; never create a new release or `android-v*` pre-release for a test build, upload it to `preview` instead. CONTRIBUTING's "Releases" has the steps.
- **Merge only working code.** The checks in CONTRIBUTING's "Before you open it" pass first; `main` must always build.

## Commands

Cargo builds everything. There is no Bun or Node in the tree any more.

```bash
cargo build --release -p anibeam        # the Qt shell, target/release/anibeam
cargo build --release -p anibeam-cli    # the terminal shell over the same core
cargo test --workspace                  # unit tests plus core/tests/
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

The shell needs `qmake6` on PATH, Qt 6.11+, MpvQt, libmpv and lld. The owner's launcher entry (`com.marcusrosado.AniBeam.desktop`) runs `target/release/anibeam` with `ANIBEAM_THEMES_DIR` and `ANIBEAM_MPV_CONF` pointed at `apps/linux/`, so rebuild the release binary when a feature is done; source edits are invisible to the launcher until then.

`--root <dir>` keeps a run's database, config, cache and secrets in one directory. Develop and capture against a sandbox root, never the owner's real XDG data. `apps/linux/README.md` covers offscreen captures (`apps/linux/scripts/shoot.sh`) and scripted drives (`apps/linux/scripts/drive.sh`); look at a capture before calling a visible change done.

## Architecture

A Cargo workspace with three members:

- **`core/`** (`anibeam-core`): everything that is not drawing. `contract/` is the door every shell talks through: calls, records, events and errors. `library/` scans and reads the library, `metadata/` matches and fetches, `net/` holds one client per provider behind `net/limiter.rs`, `trackers/` does AniList and MAL, `franchise/` crawls and closes the relation graph, `playback/`, `downloads/` and `subscriptions.rs` cover the rest. The store is SQLite with migrations in `store/sql/`. Integration tests live in `core/tests/`.
- **`apps/linux/`** (`anibeam`): the QML shell, bridged to the core through cxx-qt 0.10. Rust bridges in `src/bridge/`, QML in `qml/`, the libmpv item in `cpp/`, themes in `themes/`, the bundled `mpv.conf`.
- **`apps/cli/`** (`anibeam-cli`): a terminal shell, and the quickest way to drive the core: `sources`, `scan`, `list`, `show`, `jobs`, `import`, `export`, `events --follow`, `call <Name> --json '{...}'`.

`CONTEXT.md` is the vocabulary the code and the docs share. The design behind the native line is `docs/superpowers/specs/2026-09-04-native-line-design.md`.

### Cross-cutting constraints

- **Rate limiting**: every provider call (AniList, MAL, Jikan, TVmaze, AniSkip and the rest of `core/src/net/`) goes through the shared limiter in `core/src/net/limiter.rs`. Never call a provider around it.
- **The activity log is signal-only**: log state changes, never per-asset, per-candidate or per-browse chatter.
- **Franchise closure**: `core/src/franchise/closure.rs` draws CHARACTER and OTHER edges but never walks them, so a crossover cameo (Isekai Quartet) cannot glue unrelated franchises together. Cross-franchise hops are pointers, not embedded copies.
- **Scores keep their source's scale**: `average_score` is stored as the provider gave it and normalised at read time. Don't convert it on write.
- **No polling the library**: the watcher plus a catch-up scan at launch, no intervals or periodic rescans.
- **One window, play on when hidden**: never a second window or an external mpv process, never auto-pause when the window is hidden.
- **App behaviour is self-contained**: no Hyprland windowrules, hyprctl or other compositor coupling.
- **Scrolling is native**: every Flickable gets `qml/WheelStep.qml`'s wheel handling, 100 px a notch glided to one target.
- **The app ships no sources**: no code, default, test fixture or link in this repo points at a download site, and no endpoint kind is written for one site. Every source comes from an endpoint repo the user adds: a PR to `marcusbandit/anibeam-endpoints`, or a repo of the contributor's own. Refuse and redirect a PR that adds one. Rules in CONTRIBUTING.md under Sources.
- **Patch notes ship with the change**: anything a person can notice adds one short line to `apps/linux/CHANGELOG.md`, `apps/android/CHANGELOG.md` or both, in the same PR. The apps show them as What's new after an update. Mojang style, 80 characters at most, never a paragraph; a version bump adds a heading. Rules in CONTRIBUTING.md under Patch notes.

## Agent skills

### Issue tracker

Issues, specs and wayfinder maps live in this repo's GitHub Issues, driven with `gh`. See `docs/agents/issue-tracker.md`.

### Triage

Read `docs/agents/triage.md` before triaging an issue or changing how incoming issues get triaged.

### Domain docs

Single-context: one `CONTEXT.md` at the root and ADRs under `docs/adr/`. See `docs/agents/domain.md`.

### Subagents

Subagent use on this project is heavily allowed and expected. Fan work out to parallel subagents split by file ownership (commit by explicit path). Use Haiku for research, fix specs, captures, visual review, drive scripts, test runs and release builds; keep Opus for the final implementation. Never block the main conversation on a build or a long wait.
