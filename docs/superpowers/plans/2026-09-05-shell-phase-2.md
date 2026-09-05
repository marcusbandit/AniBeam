# Linux shell phase 2 implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Linux shell of the native line, chapters 4 and 5 of the spec, as the Qt 6.11 QML app `anibeam` under `apps/linux/`, so that the parity checklist's switch line (units 1 to 4, 6 and 7) can go green on the real library on both machines, the launcher entry can point at the native binary, and `v2.0.0` can be cut.

**Architecture:** One Cargo package, `anibeam`, builds QML on top of a Rust half through cxx-qt 0.10.0 with no CMake. The Rust half links `anibeam-core` in-process, owns the window, the input, the theme code and the desktop integration, and holds no rule. One Rust QObject singleton, `Door`, is the shell's way into the core: one invokable per call, one Qt signal per event body, JSON objects for anything deep, and `CxxQtThread::queue` for every hop from a tokio thread to the Qt thread. A second singleton, `Theme`, derives the token set from the terminal palette, a theme pair or the portal and writes `theme.toml`. Lists sit on one Rust `QAbstractListModel`, `RecordModel`. The video surface is a C++ `MpvAbstractItem` subclass compiled into the same QML module. The look is the prototype's, carried in as QML and fed by the core instead of JSON files.

**Tech Stack:** Rust 2024 edition, cxx-qt, cxx-qt-lib and cxx-qt-build pinned `=0.10.0`, Qt 6.11 from pacman (qt6-base, qt6-declarative, qt6-svg), MpvQt 1.2.0 and mpv 0.41 (libmpv 2.5), tokio, zbus 5 for the portal and the application bus name, mpris-server 0.10 for MPRIS, notify 8.2 for file watching, material-colors 0.4.2 for the portal derivation, toml_edit 0.25 for the two settings files, serde_json for the bridge payloads, lld for the link, makepkg for the install.

**Spec:** `docs/superpowers/specs/2026-09-04-native-line-design.md`, chapter 4 (the shell: 4.1 the parity checklist, 4.2 the theme model, 4.3 the look, 4.4 the player, 4.5 frame and settings) and chapter 5 (the Linux shell: 5.1 the stack, 5.2 the video surface, 5.3 packaging, 5.4 the bundled mpv.conf), with section 1.5 for the phase's exit, 2.1 to 2.3 for the spike facts the shell leans on, and 3.1, 3.7 and 3.8 for what the core hands a shell. The vocabulary is `CONTEXT.md` at the repository root: core, shell, bridge, call, reply, event, job, session, tick, view, mark, resume point, completion, skip window, track choice, subtitle defaults, source, series, match, missing, forget, export, import, token, terminal palette, colour source, theme, mode, accent, density, frame, rail, status strip, activity log, unseen errors, inline confirm, switch line, app id, install. Use those words in code, comments, tests and commit messages, and none of the synonyms the glossary avoids.

The core's contract is the code under `core/src/contract/`, which phase 1 wrote from spec 3.1. Every record and enum this plan names is quoted from that code, so the field names here are the real ones; where this plan and `core/src/contract/` disagree, the code wins. The prototype's QML under `spikes/home-grid-qml/qml/` is the look; where this plan and a prototype file disagree on a number, the prototype wins unless the spec fixed the number in 4.3.

## Global constraints

Copied from the spec. Every task's requirements include this section.

- The shell is the package `anibeam` at `apps/linux/`, a member of the root workspace beside `core/` and `apps/cli/`. `main` stays green for both worlds: nothing here touches `src/`, `package.json` or the Electron build, and `bun run typecheck` must still pass before a merge. Work on a branch off `main`, `feat/shell-phase-2`, never on `main` directly; merge when the switch line's build is packaged and installed on the desktop.
- The workspace version stays `2.0.0-dev` and `rust-version` stays `1.95`, both set by phase 1. `anibeam --version` prints the same describe string as `anibeam-cli --version`, computed in `build.rs` from `git describe --tags --dirty` with `CARGO_PKG_VERSION` as the fallback (spec 5.3).
- The stack (spec 5.1): Qt 6.11 QML through cxx-qt 0.10.0, built with Cargo alone. `cxx-qt`, `cxx-qt-lib` and `cxx-qt-build` are pinned `=0.10.0`, never a caret. One `build.rs` calls `CxxQtBuilder::new_qml_module` once; there is no CMake anywhere. The `qt_minimal` feature stays off. The QML module is static. The link goes through lld, which is in the package's `makedepends`. Every Rust file that holds a `#[cxx_qt::bridge]` sits in one directory, `apps/linux/src/bridge/`, because cxx-qt panics on bridges spread across directories (QTBUG-93443). Integration tests under `apps/linux/tests/` do not link in a Cargo-only cxx-qt layout (cxx-qt issue 770), so every shell test is a unit test inside `src/`.
- Names (spec 5.3): package `anibeam`, binary `/usr/bin/anibeam`, app id `com.marcusrosado.AniBeam` for the desktop file, `StartupWMClass`, `QGuiApplication::setDesktopFileName`, the hicolor icon, the D-Bus name and MPRIS's `DesktopEntry`. XDG directories are named `anibeam`: `~/.config/anibeam/` holds `theme.toml`, `player.toml`, `mpv.conf` and `themes/`; `~/.local/share/anibeam/` the database; `~/.cache/anibeam/` the image cache; `~/.local/state/anibeam/` logs. Built-in themes ship at `/usr/share/anibeam/themes/` and the bundled `mpv.conf` at `/usr/share/anibeam/mpv.conf`. The shell never creates a file or directory under those three XDG directories with one of Electron's names (`config.json`, `metadata.json`, `image-cache/`, `images/`, `thumbs/`, `logs/` under config, and the rest of the list in spec 5.3).
- Process facts (spec 5.1, 4.5): the shell sets `QSG_RENDER_LOOP=threaded` and `QT_XCB_GL_INTEGRATION=xcb_egl` in its own environment before constructing `QGuiApplication`; calls `QQuickWindow::setGraphicsApi(QSGRendererInterface::OpenGL)` before the first window; takes a flock under `$XDG_RUNTIME_DIR` before anything else; owns `com.marcusrosado.AniBeam` on the session bus and serves `org.freedesktop.Application` there; serves MPRIS as `org.mpris.MediaPlayer2.anibeam` on the same connection. No session bus means one line on stderr and no MPRIS; a second launch with no bus prints one line and exits 1. The desktop file has `StartupNotify=true` and no `DBusActivatable`.
- One application, one window (spec 4.4). Nothing opens a second window and no mpv process is launched. Playback continues while the window is not presented; the shell does nothing on expose changes. Scripts never load; mpv's OSD stays off; the shell draws every overlay itself and handles every key itself.
- The bridge (spec 3.1 constraints 6 to 8, 5.1): enums with fields never cross. The `Door` singleton exposes one `#[qinvokable]` per call variant, with flat arguments or a `QJsonObject`, and one `#[qsignal]` per event body, and builds `Call` values on the Rust side. The core never sees a Qt type. Events cross from tokio threads through `CxxQtThread::queue`, coalesced: one queued closure per event, never one per file. A panic in any bridge function aborts the process, so bridge code catches and reports. A `rust_mut()` borrow ends before any signal is emitted. A QObject handed to QML has a parent or is created by QML. Every container instantiation that crosses gets its own alias line in the bridge.
- Look (spec 4.2, 4.3): every colour, size, radius, spacing and duration is a token on the `Theme` singleton; nothing is set inline. One `Corner` primitive draws every rounded shape, a QtQuick.Shapes squircle in reach semantics with the CurveRenderer; a bare `Rectangle.radius` in the shell is a bug. Smoothing 0.6, plain rounding is smoothing 0 on the same primitive. The radius ladder is base 14 times 1.4 to the power i (14, 19.6, 27.4, 38.4) times density. The space unit is 4 px times density, density 0.75, 1 or 1.25. Poster width is S 140, M 180 or L 240 and never scales with density. Type is three sizes from the system font: normal is the application font's point size, small 0.85 of it, large 1.4; weights DemiBold for titles and the active tab, Medium for chips, Bold for the page title alone, nothing above 700. The general face carries text and the fixed face numbers. Motion is 120, 200 and 320 ms; anything tracking a target uses exponential smoothing at rate 12 to 14. Icons are Lucide SVGs tinted from the text token through a `ColorImage`, never an icon theme lookup.
- The first frame (spec 4.5): the window's initial size is a hint that carries nothing; the first frame is the ground alone; the rail and the pages build once the compositor's size has arrived, or 200 ms after the first frame.
- Frame (spec 4.1 unit 1, 4.5): an undecorated window; a rail with Library, Feed, Watching, Metadata and Settings, the version and the JP / EN switch; a trail capped at 12 entries whose Back restores scroll; right-click outside the player always offers Back; Escape closes any open popover, menu or modal; every page reachable from the keyboard; the window title follows the page; the status strip under every page but the player and the drawer that rises from it on click or Ctrl+L; confirmations inline, never a modal dialog; no native `title` tooltips, hover text goes through the shell's own tooltip.
- Ticks (spec 3.8, 4.4): `Tick { session, position, paused }` once a second while playing, once on pause, once after a seek, once inside `ClosePlayback`, sampled from an observed `time-pos`, never from a getter. `ReportChapters` goes out on `fileLoaded` before the first tick. The shell seeks to `resume_from` before the first frame.
- No polling for anything the core owns: the shell never re-reads a list on a timer. The one-second timers that exist are UI timers (the tick sampler, the countdown chips while a card needs one, the controls hide timer) and the theme watchers are notify and D-Bus subscriptions.
- House style for everything written: no em dashes, no en dashes, sentence case headings, no decorative emoji, plain words. Commit messages follow `type(scope): summary`, for example `feat(shell): the door forwards every event as a Qt signal`. Never add a `Co-Authored-By` line.

## Decisions this plan makes

The spec's "Left open" lists for 4.3, 4.4, 4.5 and 5.5 and the shell questions on issue #26 are settled here so an implementer never has to guess; the task that implements each one repeats the decision.

| Section | Question | Decision |
|---|---|---|
| 5.5 | The QML module URI | `com.marcusrosado.AniBeam`, the app id, so there is one name. Resources sit at `qrc:/qt/qml/com/marcusrosado/AniBeam/<path>`; `Main.qml` is `qrc:/qt/qml/com/marcusrosado/AniBeam/qml/Main.qml`. |
| 5.5 | Where the entry, the icon, `mpv.conf` and `themes/` sit inside `apps/linux/` | `apps/linux/com.marcusrosado.AniBeam.desktop`, `apps/linux/assets/icon.png`, `apps/linux/assets/icons/<name>.svg` for Lucide, `apps/linux/mpv.conf`, `apps/linux/themes/<name>.yaml`, `apps/linux/packaging/PKGBUILD` and `package.sh`. The PKGBUILD in spec 5.3 already names these paths. |
| 5.5 | The Rust toolchain floor | 1.95, set by phase 1 on the workspace; unchanged. |
| 5.5 | How `PlaybackSession.artwork` reaches `mpris:artUrl` | A `file://` URL of the cache path, percent-encoded per path segment. |
| 4.3 | The hue slot for tv, music, novel, oneshot and vn | tv `cyan` (Electron `#6fd0e8`), music `green` (`#70d5a8`), novel `red` (`#f098bd`), oneshot `yellow` (`#e0cd70`), vn `purple` (`#da9aeb`): the nearest hue to Electron's value in each case. |
| 4.3 | `text.faint` on the portal path and `surface.sunken` on base16 | Both as the prototype does them: `text.faint` is `bg` mixed 0.45 toward `text` on every source, and `surface.sunken` is base00 pushed 0.03 away from base05. |
| 4.4 | The volume gain above 100 | Stays open as the spec records; `volume-max=100` and a 0 to 100 slider. |
| 4.5 | Whether Forget asks first | Yes, with the inline confirm, "Forget <title> and its history?", because it is the one irreversible action on the Metadata tab and costs one click. |
| 4.5 | Where the trail lives | In QML, a `Nav` object in `Frame.qml` holding an array of at most 12 entries `{ page, props, label, scrollY }`; pages are rebuilt from a `Loader` on Back and get their `scrollY` restored, and the search text rides in `props`. |
| 4.5 | The Escape order across the frame | The drawer, then an inline confirm, then the topmost popover or menu, then whatever the page claims (the player's own order in 4.4), then nothing. |
| 3.1 | Invokable and reply shape | Invokable names are the call variant in camelCase (`listSeries`, `openPlayback`). Every invokable returns a `QJsonObject`: `{ "kind": "<Reply variant>", "reply": { <variant fields> } }` on success, `{ "error": <CoreError as JSON> }` on failure, and `{ "kind": "Started", "reply": { "job": { "id": n, "kind": "Scan" } } }` for a job. Signals carry the event body's fields as arguments, deep ones as `QJsonObject` or `QJsonArray`, plus the envelope where a page needs it. |
| #26 | An episode 0 rendering as `S00E00` | The shell shows the core's `code` verbatim. Changing it is a core follow-up, not a shell rule. |
| #26 | `AuthUrlReady` is Debug and never persisted | The shell subscribes before `Core::start`, so every live event reaches it, Debug included. The Log in flow listens for the `authUrlReady` signal and opens the URL with `Qt.openUrlExternally`; `RecentEvents` only backfills the drawer. |
| 4.2 | `theme.toml` keys beyond the three the ticket named | `mode = "system" | "dark" | "light"`, `source = "system" | "theme"`, `accent = 1..6`, `density = "compact" | "normal" | "comfortable"`, `poster = "s" | "m" | "l"`, `corners = "smooth" | "plain"`, and `[theme]` with `dark` and `light` naming files by stem. Missing keys take the defaults: system, system, 4, normal, m, smooth, `anibeam-dark`, `anibeam-light`. |
| 4.4 | `player.toml` keys | `volume = 100`, `mute = false`, `use_my_mpv_conf = false`. |
| 4.2 | Which terminals ship a parser | Kitty, as the spec says. The probe order when `$TERMINAL` is unset is kitty, foot, alacritty, ghostty by config presence; a hit on one of the three without a parser is treated as no palette, so the portal fallback applies. Their parsers are follow-ups. |
| 4.2 | A YAML crate for base16 files | None. A base16 file is a flat document, so a sixty-line parser reads `system`, `name`, `author`, `variant`, `accent` and the `palette:` block; anything else is ignored. |
| 4.5 | MPRIS on the same connection as the application name | mpris-server 0.10 owns the connection (`Server::new("anibeam", player)`); `org.freedesktop.Application` is served on `server.connection()` and the app id requested there, so one connection carries both. |
| 4.5 | Window initial size hint | 1280 by 800. It carries nothing. |
| 4.5 | Seeing a page without the owner at the machine | `anibeam --shoot <png> [--page <name>] [--width w --height h]` renders the frame under the offscreen platform and writes one PNG through `grabToImage`; `apps/linux/scripts/shoot.sh` wraps it. The player is the one page that needs a real GL surface, and it is looked at on a monitor as the prototype was (workspace 2 on HDMI-A-1). |
| 4.4 | Where the track pick, the MPRIS lines and the time formats live | In the shell's Rust half as pure functions with unit tests, reached from QML through the `Fmt` and `Door` singletons, so the rules Electron carried have tests again. |
## File structure

```
apps/linux/
  Cargo.toml                          package anibeam, binary anibeam
  build.rs                            one CxxQtBuilder call: the QML module, the C++ files, the resources, the MpvQt lines
  com.marcusrosado.AniBeam.desktop    the entry from spec 5.3
  mpv.conf                            the bundled file from spec 5.4 (Task 10)
  assets/icon.png                     the current icon, copied from spikes/cxx-qt-pkgbuild/assets/icon.png
  assets/icons/<name>.svg             Lucide glyphs with a black stroke, plus LICENSE (ISC)
  themes/<name>.yaml                  the thirty built-ins (Task 4)
  packaging/PKGBUILD                  spec 5.3 (Task 25)
  packaging/package.sh                spec 5.3 (Task 25)
  scripts/shoot.sh                    offscreen capture of one page
  scripts/bench.sh                    launch on a monitor's workspace and capture the window (the prototype's shoot-main.sh)
  cpp/helpers.h, cpp/helpers.cpp      free functions cxx-qt-lib does not wrap
  cpp/videoitem.h, cpp/videoitem.cpp  VideoItem : MpvAbstractItem (Task 10)
  src/main.rs                         arguments, environment, the lock, the runtime, the core, the application
  src/args.rs                         the command line
  src/runtime.rs                      the tokio runtime and the core behind OnceLocks
  src/paths.rs                        the shell's own files: theme.toml, player.toml, the theme directories, the lock
  src/format.rs                       relative time, countdown, clock readouts, bytes (Task 2)
  src/tracks.rs                       the track pick rule (Task 11)
  src/nowplaying.rs                   the MPRIS title and artist lines (Task 13)
  src/json.rs                         Call from a name and a JSON object; Reply and Event to JSON (Task 6)
  src/theme/mod.rs                    Palette, the token set and ThemeSettings (Task 3)
  src/theme/colour.rs                 Rgb, mixing, lightness, HSL, contrast (Task 3)
  src/theme/tokens.rs                 terminal, base16 and portal inputs to a Palette; forced modes (Task 3)
  src/theme/kitty.rs                  the kitty config chain (Task 4)
  src/theme/base16.rs                 the base16 file and the kitty theme file (Task 4)
  src/theme/config.rs                 theme.toml through toml_edit (Task 4)
  src/theme/portal.rs                 org.freedesktop.portal.Settings over zbus (Task 5)
  src/theme/engine.rs                 resolve, watch, push (Task 5)
  src/player_config.rs                player.toml (Task 10)
  src/dbus/instance.rs                the flock, org.freedesktop.Application, the activation token (Task 13)
  src/dbus/mpris.rs                   the mpris-server Player (Task 13)
  src/bridge/mod.rs                   the one directory every bridge lives in
  src/bridge/helpers.rs               the extern "C++" declarations for cpp/helpers.h
  src/bridge/door.rs                  the Door singleton (Task 6)
  src/bridge/model.rs                 RecordModel (Task 6)
  src/bridge/fmt.rs                   the Fmt singleton (Task 2)
  src/bridge/theme.rs                 the Theme singleton (Task 5)
  qml/Main.qml                        the window: the ground first, the Frame once settled, --shoot
  qml/Frame.qml                       the rail, the page loader, the strip, the drawer, the overlay, Nav (Task 7)
  qml/Nav.qml                         the trail (Task 7)
  qml/Tokens.qml                      the derived tokens every component reads as `theme` (Task 5)
  qml/Corner.qml, Icon.qml, Chip.qml, Seg.qml, Switch.qml, Button.qml, Field.qml, Dropdown.qml,
  qml/Swatches.qml, SliderRow.qml, SettingRow.qml, Panel.qml, Card.qml, StatusStrip.qml,
  qml/ActivityDrawer.qml              carried from spikes/home-grid-qml/qml/ (Tasks 5, 7, 8, 14)
  qml/Tooltip.qml, Menu.qml, InlineConfirm.qml, EmptyState.qml, SectionHeader.qml, Note.qml,
  qml/Tiles.qml, UsageBar.qml, ScorePicker.qml, EpisodeRow.qml, Modal.qml   new primitives (Tasks 7, 9, 15, 20)
  qml/LibraryPage.qml (8), SeriesPage.qml (9), PlayerPage.qml (10 to 12), FeedPage.qml (21),
  qml/WatchingPage.qml (22), MetadataPage.qml (19), MatchModal.qml (20), SubscriptionsPage.qml (23),
  qml/FranchiseGraph.qml (24), SettingsPage.qml (15), SettingsLibraryTab.qml (15),
  qml/SettingsAppearanceTab.qml (16), SettingsPlaybackTab.qml (17), SettingsDataTab.qml (18),
  qml/LookPreview.qml, LookPane.qml (16), SourceRow.qml, TrackerRow.qml (15)
```

QML files stay flat under `qml/`, as the prototype's do, because a type in a cxx-qt QML module is named after its file stem. The Rust bridges all live in `src/bridge/`.

Test conventions used throughout:

- Every Rust rule in the shell has a unit test beside it in `#[cfg(test)] mod tests`, run with `cargo test -p anibeam`. Nothing under `apps/linux/tests/` (cxx-qt issue 770).
- A test that needs a core opens one on `tempfile::tempdir()` through `anibeam_core::CorePaths::under` and `Core::open_with_secrets(paths, Secrets::file_only(...))`, the way `apps/cli/src/main.rs` does under `--root`, so no test touches the real XDG directories or the keyring.
- A page is verified by a capture: `apps/linux/scripts/shoot.sh <name> --page <page> [--root <dir>]` writes `apps/linux/captures/<name>.png` (gitignored) and the task lists what the picture must show. The player is verified on a monitor with `scripts/bench.sh`.
- Every task ends with `cargo build -p anibeam`, `cargo test -p anibeam` green, `cargo clippy -p anibeam --all-targets -- -D warnings` clean and `cargo fmt --all --check` clean.

---

### Task 1: The crate, the build, the first frame and the shoot harness

**Files:**
- Modify: `Cargo.toml` (workspace members)
- Modify: `.gitignore` (add `apps/linux/captures/`)
- Create: `apps/linux/Cargo.toml`
- Create: `apps/linux/build.rs`
- Create: `apps/linux/com.marcusrosado.AniBeam.desktop`
- Create: `apps/linux/assets/icon.png` (copy of `spikes/cxx-qt-pkgbuild/assets/icon.png`)
- Create: `apps/linux/cpp/helpers.h`, `apps/linux/cpp/helpers.cpp`
- Create: `apps/linux/src/main.rs`, `apps/linux/src/args.rs`, `apps/linux/src/runtime.rs`, `apps/linux/src/paths.rs`
- Create: `apps/linux/src/bridge/mod.rs`, `apps/linux/src/bridge/helpers.rs`, `apps/linux/src/bridge/shell.rs`
- Create: `apps/linux/qml/Main.qml`
- Create: `apps/linux/scripts/shoot.sh`, `apps/linux/scripts/bench.sh`
- Create: `apps/linux/README.md`

**Interfaces:**
- Consumes: `anibeam_core::VERSION`, `anibeam_core::CorePaths`.
- Produces: `args::Args { version: bool, shoot: Option<String>, page: Option<String>, width: u32, height: u32, root: Option<PathBuf>, action: Option<String> }` and `args::parse(argv: &[String]) -> Result<Args, String>`; `runtime::runtime() -> &'static tokio::runtime::Runtime`; `runtime::core() -> &'static Arc<Core>` and `runtime::install_core(Arc<Core>)`; `runtime::args() -> &'static Args` and `runtime::install_args(Args)`; `paths::ShellPaths { config_dir, runtime_dir, data_dir, cache_dir, state_dir }` with `ShellPaths::resolve(root: Option<&Path>) -> Result<ShellPaths, String>`, `theme_toml()`, `player_toml()`, `user_themes_dir()`, `builtin_themes_dir()` (`/usr/share/anibeam/themes`, or `apps/linux/themes` beside the binary's source in a dev run: `ANIBEAM_THEMES_DIR` overrides), `bundled_mpv_conf()`, `user_mpv_conf()`, `anibeam_mpv_conf()`, `lock_path()`; the C++ helpers `use_opengl_scene_graph()`, `set_desktop_file_name(&QString)`, `set_render_loop_env()`; `Main.qml` with `settled`, `Shell.shoot`, and the `--shoot` grab.

- [ ] **Step 1: Write the failing tests for the arguments and the paths**

`apps/linux/src/args.rs`:

```rust
//! The command line. Nothing here opens a window: `--version` prints and leaves, `--shoot`
//! renders one page offscreen and leaves, `--root` sandboxes every path, `--action` is what
//! a second launch forwards as ActivateAction (Task 13).

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Args {
    pub version: bool,
    pub shoot: Option<String>,
    pub page: Option<String>,
    pub width: u32,
    pub height: u32,
    pub root: Option<PathBuf>,
    pub action: Option<String>,
}

impl Default for Args {
    fn default() -> Self {
        Args { version: false, shoot: None, page: None, width: 1280, height: 800, root: None, action: None }
    }
}

pub fn parse(argv: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = argv.iter().skip(1);
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or_else(|| format!("{name} needs a value"));
        match arg.as_str() {
            "--version" | "-V" => a.version = true,
            "--shoot" => a.shoot = Some(value("--shoot")?),
            "--page" => a.page = Some(value("--page")?),
            "--width" => a.width = value("--width")?.parse().map_err(|_| "--width needs a number".to_string())?,
            "--height" => a.height = value("--height")?.parse().map_err(|_| "--height needs a number".to_string())?,
            "--root" => a.root = Some(PathBuf::from(value("--root")?)),
            "--action" => a.action = Some(value("--action")?),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &str) -> Vec<String> {
        std::iter::once("anibeam".to_string()).chain(s.split_whitespace().map(String::from)).collect()
    }

    #[test]
    fn defaults_and_every_flag() {
        assert_eq!(parse(&argv("")).unwrap(), Args::default());
        let a = parse(&argv("--shoot out.png --page library --width 1600 --height 1000 --root /tmp/x --action open")).unwrap();
        assert_eq!(a.shoot.as_deref(), Some("out.png"));
        assert_eq!(a.page.as_deref(), Some("library"));
        assert_eq!((a.width, a.height), (1600, 1000));
        assert_eq!(a.root.as_deref(), Some(std::path::Path::new("/tmp/x")));
        assert_eq!(a.action.as_deref(), Some("open"));
        assert!(parse(&argv("--version")).unwrap().version);
    }

    #[test]
    fn a_missing_value_and_an_unknown_flag_are_errors() {
        assert_eq!(parse(&argv("--shoot")).unwrap_err(), "--shoot needs a value");
        assert_eq!(parse(&argv("--bogus")).unwrap_err(), "unknown argument --bogus");
        assert_eq!(parse(&argv("--width x")).unwrap_err(), "--width needs a number");
    }
}
```

`apps/linux/src/paths.rs`:

```rust
//! The shell's own files. The core has its four XDG directories; the shell adds theme.toml,
//! player.toml, the two theme directories, the three mpv.conf layers and the lock file.

use std::path::{Path, PathBuf};

use anibeam_core::CorePaths;

#[derive(Clone, Debug, PartialEq)]
pub struct ShellPaths {
    pub core: CorePaths,
    pub runtime_dir: PathBuf,
    pub builtin_themes: PathBuf,
}

impl ShellPaths {
    /// Under `root` everything sits inside it, the runtime directory included, so a dev run
    /// or a test never touches the real files. Without a root the core's XDG paths apply
    /// and the lock sits under $XDG_RUNTIME_DIR.
    pub fn resolve(root: Option<&Path>) -> Result<ShellPaths, String> {
        let builtin_themes = std::env::var_os("ANIBEAM_THEMES_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/usr/share/anibeam/themes"));
        match root {
            Some(r) => Ok(ShellPaths { core: CorePaths::under(r), runtime_dir: r.join("runtime"), builtin_themes }),
            None => {
                let core = CorePaths::xdg().map_err(|e| e.to_string())?;
                let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
                    .map(PathBuf::from)
                    .ok_or_else(|| "XDG_RUNTIME_DIR is not set".to_string())?;
                Ok(ShellPaths { core, runtime_dir, builtin_themes })
            }
        }
    }

    pub fn config_dir(&self) -> PathBuf { PathBuf::from(&self.core.config_dir) }
    pub fn theme_toml(&self) -> PathBuf { self.config_dir().join("theme.toml") }
    pub fn player_toml(&self) -> PathBuf { self.config_dir().join("player.toml") }
    pub fn user_themes_dir(&self) -> PathBuf { self.config_dir().join("themes") }
    pub fn builtin_themes_dir(&self) -> PathBuf { self.builtin_themes.clone() }
    pub fn anibeam_mpv_conf(&self) -> PathBuf { self.config_dir().join("mpv.conf") }
    /// $XDG_CONFIG_HOME/mpv/mpv.conf: the user's own, behind the Use my mpv.conf setting.
    pub fn user_mpv_conf(&self) -> PathBuf {
        self.config_dir().parent().map(|p| p.join("mpv").join("mpv.conf")).unwrap_or_default()
    }
    pub fn bundled_mpv_conf(&self) -> PathBuf {
        std::env::var_os("ANIBEAM_MPV_CONF").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/usr/share/anibeam/mpv.conf"))
    }
    pub fn lock_path(&self) -> PathBuf { self.runtime_dir.join("anibeam.lock") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_root_keeps_every_path_inside_it() {
        let p = ShellPaths::resolve(Some(Path::new("/tmp/sandbox"))).unwrap();
        assert_eq!(p.theme_toml(), PathBuf::from("/tmp/sandbox/config/theme.toml"));
        assert_eq!(p.player_toml(), PathBuf::from("/tmp/sandbox/config/player.toml"));
        assert_eq!(p.user_themes_dir(), PathBuf::from("/tmp/sandbox/config/themes"));
        assert_eq!(p.anibeam_mpv_conf(), PathBuf::from("/tmp/sandbox/config/mpv.conf"));
        assert_eq!(p.user_mpv_conf(), PathBuf::from("/tmp/sandbox/mpv/mpv.conf"));
        assert_eq!(p.lock_path(), PathBuf::from("/tmp/sandbox/runtime/anibeam.lock"));
        assert_eq!(p.bundled_mpv_conf(), PathBuf::from("/usr/share/anibeam/mpv.conf"));
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p anibeam`
Expected: the package does not exist yet; cargo reports `package ID specification anibeam did not match any packages`.

- [ ] **Step 3: Write the manifests, the build script, the helpers and the entry**

Add `"apps/linux"` to `members` in the root `Cargo.toml`. Append `apps/linux/captures/` to `.gitignore`.

`apps/linux/Cargo.toml`:

```toml
[package]
name = "anibeam"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "AniBeam's Linux shell: Qt 6 QML over the core, one window, libmpv inside it"
build = "build.rs"

[[bin]]
name = "anibeam"
path = "src/main.rs"

[dependencies]
anibeam-core.workspace = true
cxx = "1"
cxx-qt = "=0.10.0"
cxx-qt-lib = { version = "=0.10.0", features = ["qt_gui", "qt_qml"] }
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
zbus = { version = "5", default-features = false, features = ["tokio"] }
mpris-server = { version = "=0.10.0", features = ["tokio"] }
notify = "8.2"
notify-debouncer-full = "0.7"
material-colors = { version = "=0.4.2", default-features = false, features = ["std"] }
toml_edit = "0.25"
rustix = { version = "1", features = ["fs"] }
percent-encoding = "2"

[build-dependencies]
cxx-qt-build = "=0.10.0"

[dev-dependencies]
tempfile = "3"
```

`apps/linux/build.rs` (the QML and C++ file lists grow task by task; every later task that adds a file names the line to add):

```rust
// One build script, no CMake. cxx-qt-build finds Qt through qmake6, runs moc, rcc,
// qmlcachegen and qmltyperegistrar, compiles the C++ beside the generated bridge and
// links the pacman Qt. MpvQt has no .pc file, so its two libraries are named here.
use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("com.marcusrosado.AniBeam")
            .version(1, 0)
            .qml_file("qml/Main.qml"),
    )
    .qt_module("Quick")
    .files(["src/bridge/helpers.rs", "src/bridge/shell.rs"])
    .include_dir("cpp")
    // mpvqt_export.h includes mpvqt_version.h bare; CMake's target used to supply this.
    .include_dir("/usr/include/MpvQt")
    .cpp_files(["cpp/helpers.cpp"])
    // qrc:/qt/qml/com/marcusrosado/AniBeam/assets/icon.png
    .qrc_resources(["assets/icon.png"])
    .build();

    println!("cargo:rustc-link-lib=MpvQt");
    println!("cargo:rustc-link-lib=mpv");
}
```

`apps/linux/cpp/helpers.h`:

```cpp
#pragma once
// Free functions cxx-qt-lib 0.10 does not wrap. Each is declared to Rust in
// src/bridge/helpers.rs. Later tasks add to this file; the list at the end of Task 13 is
// the whole set.
#include <QtCore/QString>

void use_opengl_scene_graph();
void set_desktop_file_name(const QString &name);
// QSG_RENDER_LOOP=threaded on both GPUs and QT_XCB_GL_INTEGRATION=xcb_egl for the X11
// fallback, set into this process's environment before QGuiApplication reads it. A value
// the user set in the environment wins.
void set_render_loop_env();
```

`apps/linux/cpp/helpers.cpp`:

```cpp
#include "helpers.h"
#include <QtCore/QByteArray>
#include <QtCore/qglobal.h>
#include <QtGui/QGuiApplication>
#include <QtQuick/QQuickWindow>

void use_opengl_scene_graph()
{
    QQuickWindow::setGraphicsApi(QSGRendererInterface::OpenGL);
}

void set_desktop_file_name(const QString &name)
{
    QGuiApplication::setDesktopFileName(name);
}

void set_render_loop_env()
{
    if (qgetenv("QSG_RENDER_LOOP").isEmpty())
        qputenv("QSG_RENDER_LOOP", QByteArrayLiteral("threaded"));
    if (qgetenv("QT_XCB_GL_INTEGRATION").isEmpty())
        qputenv("QT_XCB_GL_INTEGRATION", QByteArrayLiteral("xcb_egl"));
}
```

`apps/linux/src/bridge/mod.rs`:

```rust
//! Every `#[cxx_qt::bridge]` lives in this one directory: cxx-qt panics on bridges spread
//! across directories of one QML module (QTBUG-93443). Nothing outside `bridge/` mentions
//! a Qt type except `main.rs`, which constructs the application.

pub mod helpers;
pub mod shell;
```

`apps/linux/src/bridge/helpers.rs`:

```rust
//! The C++ helpers, declared once here and reached as `bridge::helpers::ffi::*`. A cxx-qt
//! bridge with only `extern "C++"` blocks is what cxx-qt-build expects in `.files()`.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "C++" {
        include!("helpers.h");
        /// QQuickWindow::setGraphicsApi(OpenGL); must run before the first window exists.
        fn use_opengl_scene_graph();
        /// QGuiApplication::setDesktopFileName, so the Wayland app id is the desktop entry's.
        fn set_desktop_file_name(name: &QString);
        /// The two environment variables the spikes settled, before QGuiApplication reads them.
        fn set_render_loop_env();
    }
}
```

`apps/linux/src/runtime.rs`:

```rust
//! The process-wide singletons the QML engine cannot be handed through a constructor: the
//! tokio runtime, the core and the parsed arguments. `main` installs each once; the bridge
//! objects, which the QML engine constructs, find them here.

use std::sync::{Arc, OnceLock};

use anibeam_core::Core;

use crate::args::Args;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static CORE: OnceLock<Arc<Core>> = OnceLock::new();
static ARGS: OnceLock<Args> = OnceLock::new();

pub fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("anibeam-shell")
            .enable_all()
            .build()
            .expect("tokio runtime")
    })
}

pub fn install_core(core: Arc<Core>) {
    CORE.set(core).ok();
}

pub fn core() -> &'static Arc<Core> {
    CORE.get().expect("the core is installed before the QML engine loads")
}

pub fn install_args(args: Args) {
    ARGS.set(args).ok();
}

pub fn args() -> &'static Args {
    ARGS.get().expect("the arguments are installed before the QML engine loads")
}
```

`apps/linux/src/main.rs` (Tasks 6 and 13 add the core's open and the lock between the argument parse and the application; the order of the calls below is the spec's: environment, graphics API, application, desktop file name, engine):

```rust
mod args;
mod bridge;
mod paths;
mod runtime;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

pub const APP_ID: &str = "com.marcusrosado.AniBeam";
pub const MAIN_QML: &str = "qrc:/qt/qml/com/marcusrosado/AniBeam/qml/Main.qml";

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let args = match args::parse(&argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("anibeam: {e}");
            std::process::exit(2);
        }
    };
    if args.version {
        println!("anibeam {}", anibeam_core::VERSION);
        return;
    }
    let paths = match paths::ShellPaths::resolve(args.root.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("anibeam: {e}");
            std::process::exit(2);
        }
    };
    let _ = &paths; // Task 6 opens the core on these; Task 13 takes the lock first.
    runtime::install_args(args);

    bridge::helpers::ffi::set_render_loop_env();
    bridge::helpers::ffi::use_opengl_scene_graph();
    let mut app = QGuiApplication::new();
    bridge::helpers::ffi::set_desktop_file_name(&QString::from(APP_ID));

    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(MAIN_QML));
    }
    if let Some(app) = app.as_mut() {
        std::process::exit(app.exec());
    }
}
```

`apps/linux/qml/Main.qml`. The first frame is the ground alone; `settled` is the first resize after the first frame, or 200 ms after it. The `--shoot` path sizes the window from the arguments, waits for the frame after settle, grabs and quits. Task 7 replaces the placeholder `Text` with `Frame`:

```qml
// The window. The first frame is the ground alone: Hyprland answers the first configure
// with 0x0 and sends the tile's size only after the window has mapped, so everything else
// is built once the window is settled and laid out once, at the compositor's size.
import QtQuick
import QtQuick.Window
import com.marcusrosado.AniBeam

Window {
    id: window
    width: Shell.shootWidth > 0 ? Shell.shootWidth : 1280   // a hint; the compositor sizes the window
    height: Shell.shootHeight > 0 ? Shell.shootHeight : 800
    visible: true
    title: "AniBeam"
    color: "#101216"                                        // Task 5 binds this to theme.bg

    property bool firstFrame: false
    property bool settled: false
    onAfterAnimating: if (!firstFrame) { firstFrame = true; settle.start() }
    onWidthChanged: if (firstFrame && !settled) settled = true
    onHeightChanged: if (firstFrame && !settled) settled = true
    Timer { id: settle; interval: 200; onTriggered: window.settled = true }

    // Task 7 replaces this with Frame { anchors.fill: parent; visible: window.settled }
    Text {
        visible: window.settled
        anchors.centerIn: parent
        text: "AniBeam " + Shell.version
        color: "#e4e7ee"
    }

    // --shoot <png>: one capture of the frame after settle, then quit. grabToImage renders
    // the scene into an image, so it works under QT_QPA_PLATFORM=offscreen.
    onSettledChanged: if (settled && Shell.shoot !== "") shootTimer.start()
    Timer {
        id: shootTimer
        interval: 400
        onTriggered: window.contentItem.grabToImage(function(result) {
            result.saveToFile(Shell.shoot)
            Qt.quit()
        })
    }
}
```

That needs one small bridge object, `Shell`, holding the arguments for QML, in `src/bridge/shell.rs`:

```rust
//! Shell: what QML needs to know about this run before anything else exists. The version,
//! the --shoot arguments and, from Task 7 on, the page a shoot opens.

use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, version)]
        #[qproperty(QString, shoot)]
        #[qproperty(QString, page)]
        #[qproperty(i32, shoot_width)]
        #[qproperty(i32, shoot_height)]
        type Shell = super::ShellRust;
    }
}

pub struct ShellRust {
    version: QString,
    shoot: QString,
    page: QString,
    shoot_width: i32,
    shoot_height: i32,
}

impl Default for ShellRust {
    fn default() -> Self {
        let a = crate::runtime::args();
        let shooting = a.shoot.is_some();
        ShellRust {
            version: QString::from(anibeam_core::VERSION),
            shoot: QString::from(a.shoot.as_deref().unwrap_or("")),
            page: QString::from(a.page.as_deref().unwrap_or("library")),
            shoot_width: if shooting { a.width as i32 } else { 0 },
            shoot_height: if shooting { a.height as i32 } else { 0 },
        }
    }
}
```

`apps/linux/com.marcusrosado.AniBeam.desktop`, verbatim from spec 5.3:

```ini
[Desktop Entry]
Type=Application
Name=AniBeam
Comment=Browse, play, and track your local anime library
Exec=anibeam
Icon=com.marcusrosado.AniBeam
Terminal=false
Categories=AudioVideo;Video;Player;
Keywords=anibeam;anime;media;video;
StartupWMClass=com.marcusrosado.AniBeam
StartupNotify=true
```

Copy the icon: `cp spikes/cxx-qt-pkgbuild/assets/icon.png apps/linux/assets/icon.png`.

`apps/linux/scripts/shoot.sh`:

```bash
#!/usr/bin/env bash
# usage: shoot.sh <name> [anibeam args...]     writes apps/linux/captures/<name>.png
# Renders one page under the offscreen platform and grabs it; no window lands anywhere.
# ANIBEAM_ROOT sandboxes the run (default: a copy-free empty root under captures/root).
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
name="$1"; shift
out="$here/captures"; mkdir -p "$out"
root="${ANIBEAM_ROOT:-$out/root}"; mkdir -p "$root"
cargo build -p anibeam --quiet
QT_QPA_PLATFORM=offscreen QT_FORCE_STDERR_LOGGING=1 ANIBEAM_THEMES_DIR="$here/themes" ANIBEAM_MPV_CONF="$here/mpv.conf" \
  "$here/../../target/debug/anibeam" --root "$root" --shoot "$out/$name.png" --width "${W:-1600}" --height "${H:-1000}" "$@" \
  2> "$out/$name.log" || { echo "anibeam exited $?; see $out/$name.log"; exit 1; }
file "$out/$name.png" | grep -q PNG && echo "$name ok" || { echo "no PNG written; see $out/$name.log"; exit 1; }
```

`apps/linux/scripts/bench.sh`, the prototype's `shoot-main.sh` with the binary and class renamed (Hyprland 0.56 Lua dispatch; `pkill -x`, never `-f`):

```bash
#!/usr/bin/env bash
# usage: bench.sh <name> <workspace> [keep] [anibeam args...]
# Launches the shell on the main monitor's workspace, captures the window's own rectangle
# with grim into apps/linux/captures/<name>.png, and closes it unless keep is given.
set -uo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
name="$1"; ws="$2"; keep="${3:-}"; shift 3 2>/dev/null || shift $#
out="$here/captures"; mkdir -p "$out"
nap() { python3 -c "import time; time.sleep($1)"; }
pkill -x anibeam 2>/dev/null; nap 0.4
hyprctl dispatch "hl.dsp.focus({ workspace = $ws })" >/dev/null; nap 0.3
QT_FORCE_STDERR_LOGGING=1 ANIBEAM_THEMES_DIR="$here/themes" ANIBEAM_MPV_CONF="$here/mpv.conf" \
  setsid nohup "$here/../../target/release/anibeam" "$@" > "$out/$name.log" 2>&1 &
for i in $(seq 1 60); do
  hyprctl clients -j | jq -e '.[] | select(.class=="com.marcusrosado.AniBeam" and .mapped==true)' >/dev/null 2>&1 && break
  nap 0.2
done
nap 2.5
geom=$(hyprctl clients -j | jq -r '[.[] | select(.class=="com.marcusrosado.AniBeam")] | .[0] | "\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"')
grim -g "$geom" "$out/$name.png" && echo "$name ok ($geom)"
[ "$keep" = keep ] || pkill -x anibeam
true
```

`chmod +x apps/linux/scripts/*.sh`.

`apps/linux/README.md`, the whole file:

```markdown
# AniBeam Linux shell

Qt 6.11 QML over the Rust core through cxx-qt 0.10, built with Cargo alone. Spec: chapters 4 and 5 of
`docs/superpowers/specs/2026-09-04-native-line-design.md`; plan: `docs/superpowers/plans/2026-09-05-shell-phase-2.md`.

    cargo build -p anibeam                          # needs qmake6 on PATH, lld, mpvqt, qt6-svg
    target/debug/anibeam --root /tmp/sandbox        # a sandboxed run; without --root the real XDG dirs
    target/debug/anibeam --version
    scripts/shoot.sh library --page library         # one offscreen capture into captures/
    scripts/bench.sh player 2 keep                  # the real window on the main monitor's workspace 2
    packaging/package.sh                            # build, package, install (Task 25)

Environment the shell sets for itself: QSG_RENDER_LOOP=threaded, QT_XCB_GL_INTEGRATION=xcb_egl.
ANIBEAM_THEMES_DIR and ANIBEAM_MPV_CONF point a dev run at the checkout's themes/ and mpv.conf.
```

- [ ] **Step 4: Build, run the tests, run the binary three ways**

Run: `cargo build -p anibeam && cargo test -p anibeam`
Expected: the build links (Qt6Quick, MpvQt and mpv in the link line), 4 tests pass.

Run: `target/debug/anibeam --version`
Expected: `anibeam 1.0.0.r<n>.g<hash>` (the nearest tag is `v1.0.0` until the switch).

Run: `apps/linux/scripts/shoot.sh first`
Expected: `first ok`; `apps/linux/captures/first.png` is a 1600 by 1000 image of the ground colour with the version centred.

Run: `cargo build --release -p anibeam && apps/linux/scripts/bench.sh first-window 2`
Expected: the window maps on workspace 2 with class `com.marcusrosado.AniBeam`, and `captures/first-window.png` shows it tiled at the compositor's size; the log has no QML errors. The portal line `Could not register app ID` is expected until Task 25 installs the entry.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml .gitignore apps/linux
git commit -m "feat(shell): the anibeam crate, a Cargo-only cxx-qt build and the first frame"
```

---
### Task 2: The format helpers and the Fmt singleton

Every readout Electron formatted in the renderer is a pure function here with a test, reached from QML as `Fmt.<name>(...)`. The rules are carried from `src/renderer/utils/relativeTime.ts`, `src/renderer/utils/airingUtils.ts` and `src/renderer/pages/VideoPlayer.tsx`.

**Files:**
- Create: `apps/linux/src/format.rs`
- Create: `apps/linux/src/bridge/fmt.rs`
- Modify: `apps/linux/src/main.rs` (add `mod format;`), `apps/linux/src/bridge/mod.rs` (add `pub mod fmt;`), `apps/linux/build.rs` (add `"src/bridge/fmt.rs"` to `.files`)

**Interfaces:**
- Consumes: nothing.
- Produces: `format::relative(ts_secs: f64, now_secs: f64) -> String`, `format::countdown(secs_left: f64) -> String`, `format::countdown_seconds(secs_left: f64) -> String`, `format::clock(secs: f64) -> String`, `format::clock_ms(secs: f64) -> String`, `format::bytes(n: u64) -> String`, `format::plural(n: u64, one: &str, many: &str) -> String`, `format::watched_chip(watched: Option<u32>, total: Option<u32>, estimate: bool) -> String`, `format::score(x: f64) -> String`; the QML singleton `Fmt` with invokables of the same names (`relative(ts, now)`, `countdown(secs)`, `countdownSeconds(secs)`, `clock(secs)`, `clockMs(secs)`, `bytes(n)`, `plural(n, one, many)`, `watchedChip(watched, total, estimate)` where `watched` and `total` are `-1` for none, `score(x)`).

- [ ] **Step 1: Write the failing tests**

`apps/linux/src/format.rs`, the tests first (the module body follows in Step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_time_has_no_weeks_bucket_and_says_in_or_ago() {
        let now = 1_000_000.0;
        assert_eq!(relative(now - 30.0, now), "just now");
        assert_eq!(relative(now - 5.0 * 60.0, now), "5m ago");
        assert_eq!(relative(now - 3.0 * 3600.0, now), "3h ago");
        assert_eq!(relative(now - 29.0 * 86400.0, now), "29d ago");
        assert_eq!(relative(now - 45.0 * 86400.0, now), "1mo ago");
        assert_eq!(relative(now - 400.0 * 86400.0, now), "1y 1mo ago");
        assert_eq!(relative(now - 730.0 * 86400.0, now), "2y ago");
        assert_eq!(relative(now + 2.0 * 3600.0, now), "in 2h");
    }

    #[test]
    fn countdowns_pad_the_lower_units() {
        assert_eq!(countdown(0.0), "now");
        assert_eq!(countdown(12.0 * 60.0), "12m");
        assert_eq!(countdown(4.0 * 3600.0 + 12.0 * 60.0), "4h 12m");
        assert_eq!(countdown(2.0 * 86400.0 + 4.0 * 3600.0 + 5.0 * 60.0), "2d 04h 05m");
        assert_eq!(countdown_seconds(59.0), "59s");
        assert_eq!(countdown_seconds(61.0), "1m 01s");
        assert_eq!(countdown_seconds(3661.0), "1h 01m 01s");
        assert_eq!(countdown_seconds(90061.0), "1d 01h 01m 01s");
    }

    #[test]
    fn clocks_switch_to_hours_and_keep_milliseconds() {
        assert_eq!(clock(-1.0), "0:00");
        assert_eq!(clock(65.0), "1:05");
        assert_eq!(clock(3665.0), "1:01:05");
        assert_eq!(clock_ms(95.9705), "1:35.971");
        assert_eq!(clock_ms(3600.5), "1:00:00.500");
        assert_eq!(clock_ms(f64::NAN), "0:00.000");
    }

    #[test]
    fn bytes_use_base_1024_with_one_decimal_above_bytes() {
        assert_eq!(bytes(512), "512 B");
        assert_eq!(bytes(1536), "1.5 KB");
        assert_eq!(bytes(312 * 1024 * 1024), "312.0 MB");
        assert_eq!(bytes(5 * 1024 * 1024 * 1024), "5.0 GB");
    }

    #[test]
    fn the_watched_chip_pads_to_the_total() {
        assert_eq!(watched_chip(None, Some(12), false), "");
        assert_eq!(watched_chip(Some(4), Some(12), false), "04/12");
        assert_eq!(watched_chip(Some(4), Some(5), true), "04/05+");
        assert_eq!(watched_chip(Some(4), None, false), "04/?");
        assert_eq!(watched_chip(Some(120), Some(1100), false), "0120/1100");
        assert_eq!(plural(1, "file", "files"), "1 file");
        assert_eq!(plural(3, "file", "files"), "3 files");
        assert_eq!(score(7.25), "7.3");
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p anibeam format`
Expected: compile errors, the functions do not exist.

- [ ] **Step 3: Write the module and the singleton**

The body of `apps/linux/src/format.rs`, above the tests:

```rust
//! Readouts. Every rule here is Electron's, from `relativeTime.ts`, `airingUtils.ts` and
//! `VideoPlayer.tsx`: no weeks bucket, zero-padded lower units on a countdown, `m:ss` under
//! an hour, base 1024 bytes with one decimal past bytes.

const MIN: f64 = 60.0;
const HOUR: f64 = 3600.0;
const DAY: f64 = 86400.0;
const MONTH: f64 = 30.0 * DAY;

/// "just now", "5m ago", "3h ago", "29d ago", "1mo ago", "1y 1mo ago", or "in 2h".
pub fn relative(ts_secs: f64, now_secs: f64) -> String {
    let diff = now_secs - ts_secs;
    let abs = diff.abs();
    if abs < MIN {
        return "just now".to_string();
    }
    let label = if abs < HOUR {
        format!("{}m", (abs / MIN).floor())
    } else if abs < DAY {
        format!("{}h", (abs / HOUR).floor())
    } else if abs < MONTH {
        format!("{}d", (abs / DAY).floor())
    } else {
        let total_mo = (abs / MONTH).floor() as u64;
        let y = total_mo / 12;
        let mo = total_mo % 12;
        if y > 0 {
            if mo > 0 { format!("{y}y {mo}mo") } else { format!("{y}y") }
        } else {
            format!("{total_mo}mo")
        }
    };
    if diff < 0.0 { format!("in {label}") } else { format!("{label} ago") }
}

/// "2d 04h 05m", "4h 12m", "12m", or "now".
pub fn countdown(secs_left: f64) -> String {
    if !(secs_left > 0.0) {
        return "now".to_string();
    }
    let total = secs_left.floor() as u64;
    let (d, h, m) = (total / 86400, (total % 86400) / 3600, (total % 3600) / 60);
    if d > 0 {
        format!("{d}d {h:02}h {m:02}m")
    } else if h > 0 {
        format!("{h}h {m:02}m")
    } else {
        format!("{m}m")
    }
}

/// The series hero's countdown, with seconds: "1d 01h 01m 01s", "1h 01m 01s", "1m 01s", "59s".
pub fn countdown_seconds(secs_left: f64) -> String {
    if !(secs_left > 0.0) {
        return "now".to_string();
    }
    let total = secs_left.floor() as u64;
    let (d, h, m, s) = (total / 86400, (total % 86400) / 3600, (total % 3600) / 60, total % 60);
    if d > 0 {
        format!("{d}d {h:02}h {m:02}m {s:02}s")
    } else if h > 0 {
        format!("{h}h {m:02}m {s:02}s")
    } else if m > 0 {
        format!("{m}m {s:02}s")
    } else {
        format!("{s}s")
    }
}

/// "m:ss", or "h:mm:ss" once there is an hour.
pub fn clock(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "0:00".to_string();
    }
    let total = secs.floor() as u64;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
}

/// "m:ss.mmm", or "h:mm:ss.mmm" once there is an hour: the frame step HUD.
pub fn clock_ms(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "0:00.000".to_string();
    }
    let total_ms = (secs * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total = total_ms / 1000;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}.{ms:03}") } else { format!("{m}:{s:02}.{ms:03}") }
}

pub fn bytes(n: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 { format!("{n} B") } else { format!("{v:.1} {}", UNITS[i]) }
}

pub fn plural(n: u64, one: &str, many: &str) -> String {
    if n == 1 { format!("{n} {one}") } else { format!("{n} {many}") }
}

/// The card's top right chip: "04/12", "04/05+" when the total is an airing estimate,
/// "04/?" when no total is known, "" when nothing is tracked.
pub fn watched_chip(watched: Option<u32>, total: Option<u32>, estimate: bool) -> String {
    let Some(w) = watched else { return String::new() };
    match total {
        Some(t) if t > 0 => {
            let width = t.to_string().len().max(2);
            format!("{w:0width$}/{t}{}", if estimate { "+" } else { "" })
        }
        _ => format!("{w:02}/?"),
    }
}

pub fn score(x: f64) -> String {
    format!("{x:.1}")
}
```

`apps/linux/src/bridge/fmt.rs`:

```rust
//! Fmt: the format helpers as a QML singleton. Every method is a pure call into `format`.

use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type Fmt = super::FmtRust;

        #[qinvokable]
        fn relative(self: &Fmt, ts: f64, now: f64) -> QString;
        #[qinvokable]
        fn countdown(self: &Fmt, secs: f64) -> QString;
        #[qinvokable]
        fn countdown_seconds(self: &Fmt, secs: f64) -> QString;
        #[qinvokable]
        fn clock(self: &Fmt, secs: f64) -> QString;
        #[qinvokable]
        fn clock_ms(self: &Fmt, secs: f64) -> QString;
        #[qinvokable]
        fn bytes(self: &Fmt, n: f64) -> QString;
        #[qinvokable]
        fn plural(self: &Fmt, n: f64, one: &QString, many: &QString) -> QString;
        /// `watched` and `total` are -1 for none.
        #[qinvokable]
        fn watched_chip(self: &Fmt, watched: i32, total: i32, estimate: bool) -> QString;
        #[qinvokable]
        fn score(self: &Fmt, x: f64) -> QString;
    }
}

#[derive(Default)]
pub struct FmtRust;

impl qobject::Fmt {
    pub fn relative(&self, ts: f64, now: f64) -> QString { QString::from(&crate::format::relative(ts, now)) }
    pub fn countdown(&self, secs: f64) -> QString { QString::from(&crate::format::countdown(secs)) }
    pub fn countdown_seconds(&self, secs: f64) -> QString { QString::from(&crate::format::countdown_seconds(secs)) }
    pub fn clock(&self, secs: f64) -> QString { QString::from(&crate::format::clock(secs)) }
    pub fn clock_ms(&self, secs: f64) -> QString { QString::from(&crate::format::clock_ms(secs)) }
    pub fn bytes(&self, n: f64) -> QString { QString::from(&crate::format::bytes(n.max(0.0) as u64)) }
    pub fn plural(&self, n: f64, one: &QString, many: &QString) -> QString {
        QString::from(&crate::format::plural(n.max(0.0) as u64, &one.to_string(), &many.to_string()))
    }
    pub fn watched_chip(&self, watched: i32, total: i32, estimate: bool) -> QString {
        let opt = |v: i32| if v < 0 { None } else { Some(v as u32) };
        QString::from(&crate::format::watched_chip(opt(watched), opt(total), estimate))
    }
    pub fn score(&self, x: f64) -> QString { QString::from(&crate::format::score(x)) }
}
```

Add `mod format;` to `main.rs`, `pub mod fmt;` to `bridge/mod.rs`, and `"src/bridge/fmt.rs"` to `.files([...])` in `build.rs`.

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p anibeam format && cargo build -p anibeam`
Expected: 5 tests pass; the build links with `Fmt` registered in the module.

- [ ] **Step 5: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the readout formats as pure functions and the Fmt singleton"
```

---

### Task 3: Colour maths and the token set

The theme model's derivations from spec 4.2 and 4.3 as pure functions: a terminal palette, a base16 file or the portal's scheme and accent become one `Palette` of tokens, for either mode. The numbers are the prototype's (`spikes/home-grid-qml/qml/Theme.qml`): the mix steps, the forced-mode grounds, the re-tone rule, the orange and brown, the accent slots, the focus ring, the scrim.

**Files:**
- Create: `apps/linux/src/theme/mod.rs`, `apps/linux/src/theme/colour.rs`, `apps/linux/src/theme/tokens.rs`
- Modify: `apps/linux/src/main.rs` (add `mod theme;`)

**Interfaces:**
- Consumes: `material_colors` 0.4.2.
- Produces:
  - `colour::Rgb { r: f64, g: f64, b: f64 }` (0 to 1) with `Rgb::hex(&str) -> Option<Rgb>` (accepts `#rrggbb` and `#rgb`), `to_hex() -> String` (`#rrggbb`), `mix(self, other, t)`, `lightness()`, `luminance()`, `contrast(self, other)`, `to_hsl() -> (h, s, l)`, `Rgb::from_hsl(h, s, l)`, `hue_between(a, b)`, `browned(self)`, `retone(self, mode)`.
  - `theme::Mode { Dark, Light }` with `Mode::of_ground(bg: Rgb)` (dark below lightness 0.5).
  - `theme::Palette` with the fields `bg, surface, surface_raised, surface_sunken, surface_pressed, line, line_strong, text, text_dim, text_faint, accent, accent_text, accent_soft, red_soft, focus, red, orange, yellow, green, cyan, blue, purple, brown: Rgb`, `scrim_alpha: f64` (0.8), `mode: Mode`, `source_label: String`, and `Palette::get(&self, name: &str) -> Option<Rgb>` for the token names `bg`, `surface`, `surface.raised`, `surface.sunken`, `surface.pressed`, `line`, `line.strong`, `text`, `text.dim`, `text.faint`, `accent`, `accent.text`, `accent.soft`, `red.soft`, `focus`, `red`, `orange`, `yellow`, `green`, `cyan`, `blue`, `purple`, `brown`; `Palette::NAMES: [&str; 23]` in that order.
  - `theme::Steps { sunken: 0.03, surface: 0.05, raised: 0.10, line: 0.16, line_strong: 0.26, faint: 0.45, dim: 0.70 }` with `Default`.
  - `theme::TerminalPalette { foreground: Rgb, background: Rgb, colors: [Rgb; 16], source: String }`, `theme::Base16Theme { stem: String, name: String, variant: Option<Mode>, accent: String, palette: [Rgb; 16] }` with `Base16Theme::mode() -> Mode` (variant, else base00's lightness), `theme::Portal { scheme: Option<Mode>, contrast: bool, accent: Option<Rgb> }`.
  - `tokens::from_terminal(term: &TerminalPalette, mode: Mode, slot: u8, contrast: bool, steps: &Steps) -> Palette`, `tokens::from_base16(theme: &Base16Theme, contrast: bool, steps: &Steps) -> Palette`, `tokens::from_portal(portal: &Portal, mode: Mode, contrast: bool, steps: &Steps) -> Palette`.
  - `theme::format_hue(anilist_format: &str) -> &'static str` (the token name: `cyan` for TV and TV_SHORT, `yellow` MOVIE, `purple` OVA, `green` ONA, `red` SPECIAL, `green` MUSIC, `orange` MANGA, `red` NOVEL and LIGHT_NOVEL, `yellow` ONE_SHOT, `purple` VISUAL_NOVEL, else `text.dim`) and `theme::status_hue(list_status: &str) -> &'static str` (`accent` watching, `blue` completed, `yellow` paused, `red` dropped, `text.faint` planning, `purple` repeating).

- [ ] **Step 1: Write the failing tests**

`apps/linux/src/theme/colour.rs`, tests only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Mode;

    #[test]
    fn hex_round_trips_and_short_form_expands() {
        assert_eq!(Rgb::hex("#191724").unwrap().to_hex(), "#191724");
        assert_eq!(Rgb::hex("#fff").unwrap().to_hex(), "#ffffff");
        assert_eq!(Rgb::hex("191724").unwrap().to_hex(), "#191724");
        assert!(Rgb::hex("#12345").is_none());
        assert!(Rgb::hex("blue").is_none());
    }

    #[test]
    fn mix_lightness_and_contrast() {
        let black = Rgb::hex("#000000").unwrap();
        let white = Rgb::hex("#ffffff").unwrap();
        assert_eq!(black.mix(white, 0.5).to_hex(), "#808080");
        assert!(black.lightness() < 0.01 && white.lightness() > 0.99);
        assert!((black.contrast(white) - 21.0).abs() < 0.01);
        assert_eq!(Mode::of_ground(Rgb::hex("#0f1114").unwrap()), Mode::Dark);
        assert_eq!(Mode::of_ground(Rgb::hex("#f6f7fa").unwrap()), Mode::Light);
    }

    #[test]
    fn retone_caps_lightness_on_light_and_floors_it_on_dark() {
        let pale = Rgb::hex("#eb6f92").unwrap();
        let (h, s, _) = pale.to_hsl();
        let on_light = pale.retone(Mode::Light);
        let (h2, s2, l2) = on_light.to_hsl();
        assert!((h - h2).abs() < 0.01 && (s - s2).abs() < 0.01);
        assert!(l2 <= 0.42 + 0.001);
        let dark = Rgb::hex("#402030").unwrap();
        assert!(dark.retone(Mode::Dark).to_hsl().2 >= 0.62 - 0.001);
    }

    #[test]
    fn orange_sits_between_red_and_yellow() {
        let red = Rgb::hex("#eb6f92").unwrap();
        let yellow = Rgb::hex("#f6c177").unwrap();
        let (hr, _, _) = red.to_hsl();
        let (hy, _, _) = yellow.to_hsl();
        let (ho, _, _) = hue_between(red, yellow).to_hsl();
        let lo = hr.min(hy);
        let hi = hr.max(hy);
        assert!(ho > lo && ho < hi, "{ho} not between {lo} and {hi}");
        let (_, so, lo2) = browned(hue_between(red, yellow)).to_hsl();
        assert!(so < 0.6 && lo2 < 0.5);
    }
}
```

`apps/linux/src/theme/tokens.rs`, tests only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::colour::Rgb;
    use crate::theme::{Base16Theme, Mode, Portal, Steps, TerminalPalette};

    fn rose_pine() -> TerminalPalette {
        let hex = |s: &str| Rgb::hex(s).unwrap();
        TerminalPalette {
            foreground: hex("#e0def4"),
            background: hex("#191724"),
            colors: [
                hex("#26233a"), hex("#eb6f92"), hex("#31748f"), hex("#f6c177"), hex("#9ccfd8"), hex("#c4a7e7"), hex("#ebbcba"), hex("#e0def4"),
                hex("#6e6a86"), hex("#eb6f92"), hex("#31748f"), hex("#f6c177"), hex("#9ccfd8"), hex("#c4a7e7"), hex("#ebbcba"), hex("#e0def4"),
            ],
            source: "kitty".into(),
        }
    }

    #[test]
    fn a_terminal_palette_in_its_own_mode_keeps_bg_text_and_slots() {
        let p = from_terminal(&rose_pine(), Mode::Dark, 4, false, &Steps::default());
        assert_eq!(p.bg.to_hex(), "#191724");
        assert_eq!(p.text.to_hex(), "#e0def4");
        assert_eq!(p.red.to_hex(), "#eb6f92");
        assert_eq!(p.blue.to_hex(), "#9ccfd8");
        assert_eq!(p.accent.to_hex(), "#9ccfd8");
        assert_eq!(p.surface.to_hex(), p.bg.mix(p.text, 0.05).to_hex());
        assert_eq!(p.line.to_hex(), p.bg.mix(p.text, 0.16).to_hex());
        assert_eq!(p.text_faint.to_hex(), p.bg.mix(p.text, 0.45).to_hex());
        // the bright pair equals the normal slot, so the focus ring is the accent itself
        assert_eq!(p.focus.to_hex(), p.accent.to_hex());
        assert_eq!(p.accent_soft.to_hex(), p.bg.mix(p.accent, 0.2).to_hex());
        assert_eq!(p.mode, Mode::Dark);
        assert_eq!(p.source_label, "terminal kitty");
    }

    #[test]
    fn contrast_widens_the_line_and_faint_steps_by_half() {
        let p = from_terminal(&rose_pine(), Mode::Dark, 4, true, &Steps::default());
        assert_eq!(p.line.to_hex(), p.bg.mix(p.text, 0.24).to_hex());
        assert_eq!(p.line_strong.to_hex(), p.bg.mix(p.text, 0.39).to_hex());
        assert_eq!(p.text_faint.to_hex(), p.bg.mix(p.text, 0.675).to_hex());
        assert_eq!(p.text_dim.to_hex(), p.bg.mix(p.text, 0.70).to_hex());
    }

    #[test]
    fn a_forced_mode_derives_a_neutral_ground_and_retones_the_hues() {
        let p = from_terminal(&rose_pine(), Mode::Light, 4, false, &Steps::default());
        let ground = Rgb::hex("#f6f7fa").unwrap().mix(Rgb::hex("#9ccfd8").unwrap(), 0.03);
        assert_eq!(p.bg.to_hex(), ground.to_hex());
        assert_eq!(p.text.to_hex(), "#1b1e26");
        assert!(p.red.to_hsl().2 <= 0.42 + 0.001);
        assert_eq!(p.accent.to_hsl().0, Rgb::hex("#9ccfd8").unwrap().to_hsl().0);
        assert_eq!(p.source_label, "terminal kitty (forced light)");
        assert_eq!(p.mode, Mode::Light);
    }

    #[test]
    fn slot_seven_is_the_derived_orange() {
        let p = from_terminal(&rose_pine(), Mode::Dark, 7, false, &Steps::default());
        assert_eq!(p.accent.to_hex(), p.orange.to_hex());
    }

    fn mocha() -> Base16Theme {
        let hex = |s: &str| Rgb::hex(s).unwrap();
        Base16Theme {
            stem: "catppuccin-mocha".into(),
            name: "Catppuccin Mocha".into(),
            variant: Some(Mode::Dark),
            accent: "base0D".into(),
            palette: [
                hex("#1e1e2e"), hex("#181825"), hex("#313244"), hex("#45475a"), hex("#585b70"), hex("#cdd6f4"), hex("#f5e0dc"), hex("#b4befe"),
                hex("#f38ba8"), hex("#fab387"), hex("#f9e2af"), hex("#a6e3a1"), hex("#94e2d5"), hex("#89b4fa"), hex("#cba6f7"), hex("#f2cdcd"),
            ],
        }
    }

    #[test]
    fn base16_slots_land_on_their_roles() {
        let p = from_base16(&mocha(), false, &Steps::default());
        assert_eq!(p.bg.to_hex(), "#1e1e2e");
        assert_eq!(p.surface.to_hex(), "#181825");
        assert_eq!(p.surface_raised.to_hex(), "#313244");
        assert_eq!(p.line.to_hex(), "#313244");
        assert_eq!(p.line_strong.to_hex(), "#45475a");
        assert_eq!(p.text_faint.to_hex(), "#45475a");
        assert_eq!(p.text_dim.to_hex(), "#585b70");
        assert_eq!(p.text.to_hex(), "#cdd6f4");
        assert_eq!(p.red.to_hex(), "#f38ba8");
        assert_eq!(p.brown.to_hex(), "#f2cdcd");
        assert_eq!(p.accent.to_hex(), "#89b4fa");
        assert_eq!(p.focus.to_hex(), p.accent.to_hex());
        // sunken: base00 pushed 0.03 away from base05
        let away = Rgb { r: 0.0, g: 0.0, b: 0.0 };
        assert_eq!(p.surface_sunken.to_hex(), p.bg.mix(away, 0.03).to_hex());
        assert_eq!(p.source_label, "theme Catppuccin Mocha");
        let mut purple = mocha();
        purple.accent = "base0E".into();
        assert_eq!(from_base16(&purple, false, &Steps::default()).accent.to_hex(), "#cba6f7");
    }

    #[test]
    fn the_portal_path_derives_from_the_seed_and_generates_the_hues() {
        let portal = Portal { scheme: Some(Mode::Dark), contrast: false, accent: Some(Rgb::hex("#3584e4").unwrap()) };
        let p = from_portal(&portal, Mode::Dark, false, &Steps::default());
        assert_eq!(p.mode, Mode::Dark);
        assert!(p.bg.lightness() < 0.2, "a dark scheme has a dark ground");
        assert!(p.text.lightness() > 0.8);
        assert!(p.accent.to_hsl().2 > 0.5, "the primary reads on a dark ground");
        let (hue, _, _) = p.yellow.to_hsl();
        assert!((0.10..=0.20).contains(&hue), "yellow hue {hue}");
        let (hue, _, _) = p.green.to_hsl();
        assert!((0.25..=0.45).contains(&hue), "green hue {hue}");
        assert_eq!(p.text_faint.to_hex(), p.bg.mix(p.text, 0.45).to_hex());
        assert_eq!(p.source_label, "portal, derived (dark)");
        let none = Portal { scheme: None, contrast: false, accent: None };
        let q = from_portal(&none, Mode::Light, false, &Steps::default());
        assert!(q.bg.lightness() > 0.8);
        assert_eq!(q.source_label, "portal, derived (unset)");
    }

    #[test]
    fn format_and_status_hues() {
        use crate::theme::{format_hue, status_hue};
        assert_eq!(format_hue("TV"), "cyan");
        assert_eq!(format_hue("TV_SHORT"), "cyan");
        assert_eq!(format_hue("MOVIE"), "yellow");
        assert_eq!(format_hue("OVA"), "purple");
        assert_eq!(format_hue("ONA"), "green");
        assert_eq!(format_hue("SPECIAL"), "red");
        assert_eq!(format_hue("MUSIC"), "green");
        assert_eq!(format_hue("MANGA"), "orange");
        assert_eq!(format_hue("LIGHT_NOVEL"), "red");
        assert_eq!(format_hue("ONE_SHOT"), "yellow");
        assert_eq!(format_hue("VISUAL_NOVEL"), "purple");
        assert_eq!(format_hue("weird"), "text.dim");
        assert_eq!(status_hue("Watching"), "accent");
        assert_eq!(status_hue("Repeating"), "purple");
        assert_eq!(status_hue("Planning"), "text.faint");
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p anibeam theme`
Expected: compile errors.

- [ ] **Step 3: Write the three modules**

`apps/linux/src/theme/mod.rs`:

```rust
//! The theme model, spec 4.2: colour sources become one Palette of tokens per mode.
//! Nothing here touches Qt; the bridge in `bridge/theme.rs` hands the result to QML.

pub mod base16;
pub mod colour;
pub mod config;
pub mod engine;
pub mod kitty;
pub mod portal;
pub mod tokens;

use colour::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    /// Dark below the midpoint of the ground's lightness, light above it.
    pub fn of_ground(bg: Rgb) -> Mode {
        if bg.lightness() < 0.5 { Mode::Dark } else { Mode::Light }
    }
    pub fn as_str(self) -> &'static str {
        match self { Mode::Dark => "dark", Mode::Light => "light" }
    }
}

/// The mix steps of `bg` toward `text`; sunken is away from text. Prototype-tunable
/// defaults the prototype kept unchanged.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Steps {
    pub sunken: f64,
    pub surface: f64,
    pub raised: f64,
    pub line: f64,
    pub line_strong: f64,
    pub faint: f64,
    pub dim: f64,
}

impl Default for Steps {
    fn default() -> Self {
        Steps { sunken: 0.03, surface: 0.05, raised: 0.10, line: 0.16, line_strong: 0.26, faint: 0.45, dim: 0.70 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TerminalPalette {
    pub foreground: Rgb,
    pub background: Rgb,
    pub colors: [Rgb; 16],
    /// "kitty" today; what the settings page shows as the source.
    pub source: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Base16Theme {
    pub stem: String,
    pub name: String,
    pub variant: Option<Mode>,
    /// "base0D" unless the file says otherwise; the one non-standard key.
    pub accent: String,
    pub palette: [Rgb; 16],
}

impl Base16Theme {
    pub fn mode(&self) -> Mode {
        self.variant.unwrap_or_else(|| Mode::of_ground(self.palette[0]))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Portal {
    pub scheme: Option<Mode>,
    pub contrast: bool,
    pub accent: Option<Rgb>,
}

/// The token set. `scrim` is `bg` at `scrim_alpha`, so it is not a separate colour.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    pub bg: Rgb,
    pub surface: Rgb,
    pub surface_raised: Rgb,
    pub surface_sunken: Rgb,
    pub surface_pressed: Rgb,
    pub line: Rgb,
    pub line_strong: Rgb,
    pub text: Rgb,
    pub text_dim: Rgb,
    pub text_faint: Rgb,
    pub accent: Rgb,
    pub accent_text: Rgb,
    pub accent_soft: Rgb,
    pub red_soft: Rgb,
    pub focus: Rgb,
    pub red: Rgb,
    pub orange: Rgb,
    pub yellow: Rgb,
    pub green: Rgb,
    pub cyan: Rgb,
    pub blue: Rgb,
    pub purple: Rgb,
    pub brown: Rgb,
    pub scrim_alpha: f64,
    pub mode: Mode,
    pub source_label: String,
}

impl Palette {
    pub const NAMES: [&'static str; 23] = [
        "bg", "surface", "surface.raised", "surface.sunken", "surface.pressed", "line", "line.strong", "text", "text.dim",
        "text.faint", "accent", "accent.text", "accent.soft", "red.soft", "focus", "red", "orange", "yellow", "green", "cyan",
        "blue", "purple", "brown",
    ];

    pub fn get(&self, name: &str) -> Option<Rgb> {
        Some(match name {
            "bg" => self.bg,
            "surface" => self.surface,
            "surface.raised" => self.surface_raised,
            "surface.sunken" => self.surface_sunken,
            "surface.pressed" => self.surface_pressed,
            "line" => self.line,
            "line.strong" => self.line_strong,
            "text" => self.text,
            "text.dim" => self.text_dim,
            "text.faint" => self.text_faint,
            "accent" => self.accent,
            "accent.text" => self.accent_text,
            "accent.soft" => self.accent_soft,
            "red.soft" => self.red_soft,
            "focus" => self.focus,
            "red" => self.red,
            "orange" => self.orange,
            "yellow" => self.yellow,
            "green" => self.green,
            "cyan" => self.cyan,
            "blue" => self.blue,
            "purple" => self.purple,
            "brown" => self.brown,
            _ => return None,
        })
    }
}

/// The ten format colours as fixed mappings onto the hues (spec 4.2, the five open slots
/// settled in this plan's decisions).
pub fn format_hue(anilist_format: &str) -> &'static str {
    match anilist_format {
        "TV" | "TV_SHORT" => "cyan",
        "MOVIE" => "yellow",
        "OVA" => "purple",
        "ONA" => "green",
        "SPECIAL" => "red",
        "MUSIC" => "green",
        "MANGA" => "orange",
        "NOVEL" | "LIGHT_NOVEL" => "red",
        "ONE_SHOT" => "yellow",
        "VISUAL_NOVEL" => "purple",
        _ => "text.dim",
    }
}

/// The list status colours, spec 4.2. The argument is the contract's `ListStatus` name.
pub fn status_hue(list_status: &str) -> &'static str {
    match list_status {
        "Watching" => "accent",
        "Completed" => "blue",
        "Paused" => "yellow",
        "Dropped" => "red",
        "Planning" => "text.faint",
        "Repeating" => "purple",
        _ => "text.faint",
    }
}
```

`apps/linux/src/theme/colour.rs` (above its tests):

```rust
//! sRGB colour maths for the token derivation: mixing, lightness, WCAG contrast, HSL for
//! the re-tone rule, and the two derived hues.

use crate::theme::Mode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Rgb {
    /// `#rrggbb` or `#rgb`, with or without the hash. Anything else is None.
    pub fn hex(s: &str) -> Option<Rgb> {
        let s = s.trim().trim_start_matches('#');
        let expanded: String = match s.len() {
            6 => s.to_string(),
            3 => s.chars().flat_map(|c| [c, c]).collect(),
            _ => return None,
        };
        let v = u32::from_str_radix(&expanded, 16).ok()?;
        Some(Rgb {
            r: ((v >> 16) & 0xff) as f64 / 255.0,
            g: ((v >> 8) & 0xff) as f64 / 255.0,
            b: (v & 0xff) as f64 / 255.0,
        })
    }

    pub fn to_hex(self) -> String {
        let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", c(self.r), c(self.g), c(self.b))
    }

    pub fn bytes(self) -> (u8, u8, u8) {
        let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        (c(self.r), c(self.g), c(self.b))
    }

    /// The colour `t` of the way from self toward `other`.
    pub fn mix(self, other: Rgb, t: f64) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        Rgb { r: self.r + (other.r - self.r) * t, g: self.g + (other.g - self.g) * t, b: self.b + (other.b - self.b) * t }
    }

    pub fn lightness(self) -> f64 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    fn lin(v: f64) -> f64 {
        if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    }

    pub fn luminance(self) -> f64 {
        0.2126 * Self::lin(self.r) + 0.7152 * Self::lin(self.g) + 0.0722 * Self::lin(self.b)
    }

    /// WCAG contrast ratio, 1 to 21.
    pub fn contrast(self, other: Rgb) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// Hue, saturation and lightness, each 0 to 1.
    pub fn to_hsl(self) -> (f64, f64, f64) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < f64::EPSILON {
            return (0.0, 0.0, l);
        }
        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        let mut h = if max == self.r {
            (self.g - self.b) / d + if self.g < self.b { 6.0 } else { 0.0 }
        } else if max == self.g {
            (self.b - self.r) / d + 2.0
        } else {
            (self.r - self.g) / d + 4.0
        };
        h /= 6.0;
        (h, s, l)
    }

    pub fn from_hsl(h: f64, s: f64, l: f64) -> Rgb {
        fn hue(p: f64, q: f64, mut t: f64) -> f64 {
            if t < 0.0 { t += 1.0 }
            if t > 1.0 { t -= 1.0 }
            if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t }
            if t < 0.5 { return q }
            if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0 }
            p
        }
        if s <= 0.0 {
            return Rgb { r: l, g: l, b: l };
        }
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        Rgb { r: hue(p, q, h + 1.0 / 3.0), g: hue(p, q, h), b: hue(p, q, h - 1.0 / 3.0) }
    }

    /// Same hue and saturation, lightness capped at 0.42 on a light ground or floored at
    /// 0.62 on a dark one: the forced-mode rule that keeps a dark terminal's pastels
    /// visible on white.
    pub fn retone(self, mode: Mode) -> Rgb {
        let (h, s, l) = self.to_hsl();
        let l = match mode { Mode::Light => l.min(0.42), Mode::Dark => l.max(0.62) };
        Rgb::from_hsl(h, s, l)
    }
}

/// The hue halfway from `a` to `b` the short way round, at their mean saturation and
/// lightness: the terminal palette's orange.
pub fn hue_between(a: Rgb, b: Rgb) -> Rgb {
    let (ha, sa, la) = a.to_hsl();
    let (hb, sb, lb) = b.to_hsl();
    let mut dh = hb - ha;
    if dh > 0.5 { dh -= 1.0 }
    if dh < -0.5 { dh += 1.0 }
    let mut h = ha + dh / 2.0;
    if h < 0.0 { h += 1.0 }
    if h > 1.0 { h -= 1.0 }
    Rgb::from_hsl(h, (sa + sb) / 2.0, (la + lb) / 2.0)
}

/// The terminal palette's brown: the orange with its saturation at 0.55 and lightness at 0.72.
pub fn browned(c: Rgb) -> Rgb {
    let (h, s, l) = c.to_hsl();
    Rgb::from_hsl(h, s * 0.55, l * 0.72)
}
```

`apps/linux/src/theme/tokens.rs` (above its tests):

```rust
//! Three ways to fill the token set: the terminal palette, a base16 file, the portal's
//! scheme and accent. The ratios and grounds are the prototype's (`qml/Theme.qml`).

use material_colors::color::Argb;
use material_colors::hct::Hct;
use material_colors::palette::TonalPalette;
use material_colors::scheme::variant::SchemeTonalSpot;

use crate::theme::colour::{browned, hue_between, Rgb};
use crate::theme::{Base16Theme, Mode, Palette, Portal, Steps, TerminalPalette};

const DARK_GROUND: &str = "#101216";
const LIGHT_GROUND: &str = "#f6f7fa";
const DARK_TEXT: &str = "#e4e7ee";
const LIGHT_TEXT: &str = "#1b1e26";
/// The AniBeam teal, the seed when the portal has no accent.
const ANIBEAM_TEAL: &str = "#46e0c4";
pub const SCRIM_ALPHA: f64 = 0.8;

fn hex(s: &str) -> Rgb {
    Rgb::hex(s).expect("a literal colour")
}

/// The mixes every source shares, given the ground and the text.
fn finish(bg: Rgb, text: Rgb, accent: Rgb, red: Rgb, mode: Mode, focus: Rgb, hues: [Rgb; 8], sunken: Rgb, mixed: Mixed, label: String) -> Palette {
    let [red_h, orange, yellow, green, cyan, blue, purple, brown] = hues;
    let _ = red_h;
    let accent_text = if accent.contrast(bg) > accent.contrast(text) { bg } else { text };
    Palette {
        bg,
        surface: mixed.surface,
        surface_raised: mixed.raised,
        surface_sunken: sunken,
        surface_pressed: mixed.pressed,
        line: mixed.line,
        line_strong: mixed.line_strong,
        text,
        text_dim: mixed.dim,
        text_faint: mixed.faint,
        accent,
        accent_text,
        accent_soft: bg.mix(accent, 0.2),
        red_soft: bg.mix(red, 0.2),
        focus,
        red,
        orange,
        yellow,
        green,
        cyan,
        blue,
        purple,
        brown,
        scrim_alpha: SCRIM_ALPHA,
        mode,
        source_label: label,
    }
}

struct Mixed {
    surface: Rgb,
    raised: Rgb,
    pressed: Rgb,
    line: Rgb,
    line_strong: Rgb,
    faint: Rgb,
    dim: Rgb,
}

fn mixes(bg: Rgb, text: Rgb, contrast: bool, s: &Steps) -> Mixed {
    let m = if contrast { 1.5 } else { 1.0 };
    Mixed {
        surface: bg.mix(text, s.surface),
        raised: bg.mix(text, s.raised),
        pressed: bg.mix(text, s.raised * 1.5),
        line: bg.mix(text, s.line * m),
        line_strong: bg.mix(text, s.line_strong * m),
        faint: bg.mix(text, s.faint * m),
        dim: bg.mix(text, s.dim),
    }
}

fn away(mode: Mode) -> Rgb {
    match mode { Mode::Dark => Rgb { r: 0.0, g: 0.0, b: 0.0 }, Mode::Light => Rgb { r: 1.0, g: 1.0, b: 1.0 } }
}

/// `slot` is 1 to 6 for a terminal colour, 7 for the derived orange.
pub fn from_terminal(term: &TerminalPalette, mode: Mode, slot: u8, contrast: bool, steps: &Steps) -> Palette {
    let native = Mode::of_ground(term.background);
    let slot = slot.clamp(1, 7) as usize;
    let mut c = term.colors;
    let (bg, text, forced) = if native == mode {
        (term.background, term.foreground, false)
    } else {
        let tint = c[slot.min(6)];
        let ground = match mode { Mode::Dark => hex(DARK_GROUND), Mode::Light => hex(LIGHT_GROUND) };
        let text = match mode { Mode::Dark => hex(DARK_TEXT), Mode::Light => hex(LIGHT_TEXT) };
        for (i, colour) in c.iter_mut().enumerate() {
            if !matches!(i, 0 | 7 | 8 | 15) {
                *colour = colour.retone(mode);
            }
        }
        (ground.mix(tint, 0.03), text, true)
    };
    let (red, green, yellow, blue, purple, cyan) = (c[1], c[2], c[3], c[4], c[5], c[6]);
    let orange = hue_between(red, yellow);
    let brown = browned(orange);
    let accent = if slot == 7 { orange } else { c[slot] };
    let focus = if slot == 7 {
        orange
    } else if term.colors[slot].to_hex() == term.colors[slot + 8].to_hex() {
        accent
    } else {
        c[slot + 8]
    };
    let mixed = mixes(bg, text, contrast, steps);
    let sunken = bg.mix(away(mode), steps.sunken);
    let label = format!("terminal {}{}", term.source, if forced { format!(" (forced {})", mode.as_str()) } else { String::new() });
    finish(bg, text, accent, red, mode, focus, [red, orange, yellow, green, cyan, blue, purple, brown], sunken, mixed, label)
}

/// The tinted-theming slot roles: base00 bg, base01 surface, base02 raised and line,
/// base03 line.strong and text.faint, base04 text.dim, base05 text, base08 to base0F the hues.
pub fn from_base16(theme: &Base16Theme, contrast: bool, steps: &Steps) -> Palette {
    let p = theme.palette;
    let mode = theme.mode();
    let (bg, text) = (p[0], p[5]);
    let m = if contrast { 1.5 } else { 1.0 };
    let mixed = Mixed {
        surface: p[1],
        raised: p[2],
        pressed: p[3],
        line: if contrast { bg.mix(p[2], m) } else { p[2] },
        line_strong: if contrast { bg.mix(p[3], m.min(1.0)) } else { p[3] },
        faint: p[3],
        dim: p[4],
    };
    let hues = [p[8], p[9], p[10], p[11], p[12], p[13], p[14], p[15]];
    let accent = match theme.accent.as_str() {
        "base08" => p[8], "base09" => p[9], "base0A" => p[10], "base0B" => p[11],
        "base0C" => p[12], "base0E" => p[14], "base0F" => p[15], _ => p[13],
    };
    let sunken = bg.mix(away(mode), steps.sunken);
    let label = format!("theme {}", theme.name);
    finish(bg, text, accent, p[8], mode, accent, hues, sunken, mixed, label)
}

fn argb_to_rgb(a: Argb) -> Rgb {
    Rgb { r: a.red as f64 / 255.0, g: a.green as f64 / 255.0, b: a.blue as f64 / 255.0 }
}

fn rgb_to_argb(c: Rgb) -> Argb {
    let (r, g, b) = c.bytes();
    Argb::new(255, r, g, b)
}

/// Material's tonal spot scheme from the seed; the roles land on the tokens as spec 4.2's
/// table says, `text.faint` is the mix step, and the hues are generated at the seed's
/// chroma from fixed HCT hue angles.
pub fn from_portal(portal: &Portal, mode: Mode, contrast: bool, steps: &Steps) -> Palette {
    let seed = portal.accent.unwrap_or_else(|| hex(ANIBEAM_TEAL));
    let seed_hct = Hct::new(rgb_to_argb(seed));
    let level = if contrast || portal.contrast { Some(1.0) } else { Some(0.0) };
    let scheme = SchemeTonalSpot::new(seed_hct, mode == Mode::Dark, level).scheme;
    let bg = argb_to_rgb(scheme.background());
    let text = argb_to_rgb(scheme.on_surface());
    let chroma = seed_hct.get_chroma().max(24.0);
    let tone = match mode { Mode::Dark => 75, Mode::Light => 45 };
    let hue_at = |h: f64, t: i32| argb_to_rgb(TonalPalette::of(h, chroma).tone(t));
    let hues = [
        argb_to_rgb(scheme.error()),
        hue_at(55.0, tone),
        hue_at(90.0, tone),
        hue_at(145.0, tone),
        hue_at(200.0, tone),
        hue_at(260.0, tone),
        hue_at(310.0, tone),
        argb_to_rgb(TonalPalette::of(55.0, chroma * 0.5).tone(match mode { Mode::Dark => 55, Mode::Light => 35 })),
    ];
    let mixed = Mixed {
        surface: argb_to_rgb(scheme.surface_container()),
        raised: argb_to_rgb(scheme.surface_container_high()),
        pressed: argb_to_rgb(scheme.surface_container_highest()),
        line: argb_to_rgb(scheme.outline_variant()),
        line_strong: argb_to_rgb(scheme.outline()),
        faint: bg.mix(text, steps.faint * if contrast { 1.5 } else { 1.0 }),
        dim: argb_to_rgb(scheme.on_surface_variant()),
    };
    let accent = argb_to_rgb(scheme.primary());
    let sunken = argb_to_rgb(scheme.surface_container_lowest());
    let label = format!("portal, derived ({})", portal.scheme.map(Mode::as_str).unwrap_or("unset"));
    let mut p = finish(bg, text, accent, hues[0], mode, accent, hues, sunken, mixed, label);
    p.accent_text = argb_to_rgb(scheme.on_primary());
    p.accent_soft = argb_to_rgb(scheme.primary_container());
    p
}
```

Add `mod theme;` to `main.rs`. The four modules `base16`, `config`, `engine`, `kitty`, `portal` are declared in `theme/mod.rs` already; create them as empty files now (`//! Task 4` / `//! Task 5`) so the crate compiles.

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p anibeam theme`
Expected: 10 tests pass. If `from_portal`'s hue assertions miss by a few hundredths, adjust the hue angles, never the assertions' intent: yellow must read yellow and green must read green on both grounds.

- [ ] **Step 5: Commit**

```bash
git add apps/linux/src
git commit -m "feat(shell): colour maths and the token set from the terminal, base16 and the portal"
```

---

### Task 4: Theme sources: the kitty chain, base16 files, theme.toml and the thirty built-ins

**Files:**
- Create: `apps/linux/src/theme/kitty.rs`, `apps/linux/src/theme/base16.rs`, `apps/linux/src/theme/config.rs`
- Create: `apps/linux/themes/anibeam-dark.yaml`, `apps/linux/themes/anibeam-light.yaml`, and 28 files fetched from tinted-theming

**Interfaces:**
- Consumes: `theme::{TerminalPalette, Base16Theme, Mode}`, `colour::Rgb`.
- Produces:
  - `kitty::Chain { files: Vec<PathBuf>, palette: Option<TerminalPalette> }`, `kitty::read_chain(root: &Path, env: &dyn Fn(&str) -> Option<String>) -> Chain` (follows `include`, `globinclude` and `envinclude`, last write wins, depth cap 16, `~` and relative paths resolved), `kitty::root_config(env) -> PathBuf` (`$KITTY_CONFIG_DIRECTORY/kitty.conf`, else `$XDG_CONFIG_HOME/kitty/kitty.conf`, else `~/.config/kitty/kitty.conf`), `kitty::parse_conf(text: &str) -> HashMap<String, String>` (the last value per key of `foreground`, `background`, `color0` to `color15`, plus the include directives in order as `("include"|"globinclude"|"envinclude", arg)`), `kitty::Terminal { Kitty, Foot, Alacritty, Ghostty }`, `kitty::probe(env) -> Option<Terminal>` (`$TERMINAL` first, then config presence in the order kitty, foot, alacritty, ghostty).
  - `base16::parse(stem: &str, text: &str) -> Option<Base16Theme>`, `base16::Theme { Base16(Base16Theme), Kitty(TerminalPalette) }` with `Theme::stem()`, `Theme::name()`, `Theme::mode()`, `base16::load_dir(dir: &Path) -> Vec<Theme>` (`*.yaml` through `parse`, `*.conf` through `kitty::parse_conf` into a `TerminalPalette` with `source` = the stem; sorted by stem), `base16::load_all(builtin: &Path, user: &Path) -> Vec<Theme>` (user overrides builtin by stem).
  - `config::{ModeSetting { Dark, Light, System }, Source { System, Theme }, Density { Compact, Normal, Comfortable }, Poster { S, M, L }, Corners { Smooth, Plain }}` each with `as_str()` and `parse(&str) -> Option<Self>`; `config::ThemeSettings { mode, source, accent: u8, density, poster, corners, theme_dark: String, theme_light: String }` with `Default` (system, system, 4, normal, m, smooth, `anibeam-dark`, `anibeam-light`); `config::load(path: &Path) -> ThemeSettings` (missing file or bad key: the default for that key), `config::save(path: &Path, s: &ThemeSettings) -> std::io::Result<()>` (through `toml_edit::DocumentMut` so comments and unknown keys survive; creates the directory).

- [ ] **Step 1: Write the failing tests**

`apps/linux/src/theme/kitty.rs`, tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |k: &str| map.get(k).cloned()
    }

    #[test]
    fn parse_conf_keeps_the_last_value_and_the_directives_in_order() {
        let text = "# comment\nforeground #e0def4\nbackground   #191724\ncolor0 #26233a\ncolor0 #000\ninclude theme.conf\nglobinclude parts/*.conf\nenvinclude KITTY_CONF_*\nfont_size 12\n";
        let c = parse_conf(text);
        assert_eq!(c.values.get("foreground").map(String::as_str), Some("#e0def4"));
        assert_eq!(c.values.get("color0").map(String::as_str), Some("#000"));
        assert!(!c.values.contains_key("font_size"));
        assert_eq!(c.directives, vec![
            ("include".to_string(), "theme.conf".to_string()),
            ("globinclude".to_string(), "parts/*.conf".to_string()),
            ("envinclude".to_string(), "KITTY_CONF_*".to_string()),
        ]);
    }

    #[test]
    fn the_chain_follows_includes_and_the_last_write_wins() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("kitty.conf");
        std::fs::create_dir_all(dir.path().join("parts")).unwrap();
        std::fs::write(&root, "background #111111\nforeground #eeeeee\ninclude parts/a.conf\nglobinclude parts/b*.conf\nenvinclude KITTY_TEST_*\ncolor15 #ffffff\n").unwrap();
        std::fs::write(dir.path().join("parts/a.conf"), "background #222222\ncolor0 #000000\ncolor1 #ff0000\ncolor2 #00ff00\ncolor3 #ffff00\ncolor4 #0000ff\ncolor5 #ff00ff\ncolor6 #00ffff\ncolor7 #cccccc\ncolor8 #444444\ncolor9 #ff8888\ncolor10 #88ff88\ncolor11 #ffff88\ncolor12 #8888ff\ncolor13 #ff88ff\ncolor14 #88ffff\n").unwrap();
        std::fs::write(dir.path().join("parts/b1.conf"), "color4 #123456\n").unwrap();
        let env = env_of(&[("KITTY_TEST_ONE", "color5 #654321")]);
        let chain = read_chain(&root, &env);
        let p = chain.palette.expect("a full palette");
        assert_eq!(p.background.to_hex(), "#222222", "the include wrote after the root");
        assert_eq!(p.colors[4].to_hex(), "#123456", "the glob wrote after the include");
        assert_eq!(p.colors[5].to_hex(), "#654321", "the env include wrote last");
        assert_eq!(p.colors[15].to_hex(), "#ffffff");
        assert_eq!(chain.files.len(), 3);
        assert_eq!(chain.files[0], root);
    }

    #[test]
    fn a_missing_slot_means_no_palette_but_the_files_are_still_listed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("kitty.conf");
        std::fs::write(&root, "background #111111\nforeground #eeeeee\ninclude gone.conf\n").unwrap();
        let chain = read_chain(&root, &|_| None);
        assert!(chain.palette.is_none());
        assert_eq!(chain.files, vec![root.clone(), dir.path().join("gone.conf")]);
    }

    #[test]
    fn tilde_and_relative_paths_resolve() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(home.join(".config/theme")).unwrap();
        std::fs::write(home.join(".config/theme/k.conf"), "color9 #ff9999\n").unwrap();
        let root = dir.path().join("kitty.conf");
        std::fs::write(&root, "include ~/.config/theme/k.conf\n").unwrap();
        let env = env_of(&[("HOME", home.to_str().unwrap())]);
        let chain = read_chain(&root, &env);
        assert_eq!(chain.files[1], home.join(".config/theme/k.conf"));
    }

    #[test]
    fn probe_prefers_terminal_then_config_presence() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("config");
        std::fs::create_dir_all(cfg.join("foot")).unwrap();
        std::fs::write(cfg.join("foot/foot.ini"), "").unwrap();
        let env = env_of(&[("XDG_CONFIG_HOME", cfg.to_str().unwrap()), ("TERMINAL", "kitty")]);
        assert_eq!(probe(&env), Some(Terminal::Kitty));
        let env = env_of(&[("XDG_CONFIG_HOME", cfg.to_str().unwrap())]);
        assert_eq!(probe(&env), Some(Terminal::Foot));
        assert_eq!(root_config(&env), cfg.join("kitty/kitty.conf"));
    }
}
```

`apps/linux/src/theme/base16.rs`, tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Mode;

    const MOCHA: &str = r##"system: "base16"
name: "Catppuccin Mocha"
author: "https://github.com/catppuccin/catppuccin"
variant: "dark"
palette:
  base00: "#1e1e2e" # base
  base01: "#181825" # mantle
  base02: "#313244" # surface0
  base03: "#45475a" # surface1
  base04: "#585b70" # surface2
  base05: "#cdd6f4" # text
  base06: "#f5e0dc" # rosewater
  base07: "#b4befe" # lavender
  base08: "#f38ba8" # red
  base09: "#fab387" # peach
  base0A: "#f9e2af" # yellow
  base0B: "#a6e3a1" # green
  base0C: "#94e2d5" # teal
  base0D: "#89b4fa" # blue
  base0E: "#cba6f7" # mauve
  base0F: "#f2cdcd" # flamingo
"##;

    #[test]
    fn parses_a_tinted_theming_file_and_the_accent_key() {
        let t = parse("catppuccin-mocha", MOCHA).unwrap();
        assert_eq!(t.name, "Catppuccin Mocha");
        assert_eq!(t.variant, Some(Mode::Dark));
        assert_eq!(t.accent, "base0D");
        assert_eq!(t.palette[0].to_hex(), "#1e1e2e");
        assert_eq!(t.palette[15].to_hex(), "#f2cdcd");
        let with_accent = format!("{MOCHA}accent: \"base0E\"\n");
        assert_eq!(parse("x", &with_accent).unwrap().accent, "base0E");
        let no_variant = MOCHA.replace("variant: \"dark\"\n", "");
        assert_eq!(parse("x", &no_variant).unwrap().mode(), Mode::Dark);
        assert!(parse("x", "name: nothing\n").is_none(), "sixteen slots are required");
    }

    #[test]
    fn a_directory_loads_yaml_and_kitty_files_and_the_user_dir_overrides_by_stem() {
        let dir = tempfile::tempdir().unwrap();
        let builtin = dir.path().join("builtin");
        let user = dir.path().join("user");
        std::fs::create_dir_all(&builtin).unwrap();
        std::fs::create_dir_all(&user).unwrap();
        std::fs::write(builtin.join("catppuccin-mocha.yaml"), MOCHA).unwrap();
        std::fs::write(user.join("catppuccin-mocha.yaml"), MOCHA.replace("Catppuccin Mocha", "My Mocha")).unwrap();
        let mut kitty = String::from("foreground #e0def4\nbackground #191724\n");
        for i in 0..16 { kitty.push_str(&format!("color{i} #{:02x}{:02x}{:02x}\n", i * 10, i * 10, i * 10)); }
        std::fs::write(user.join("rose.conf"), kitty).unwrap();
        std::fs::write(user.join("notes.txt"), "ignored").unwrap();
        let all = load_all(&builtin, &user);
        let stems: Vec<&str> = all.iter().map(|t| t.stem()).collect();
        assert_eq!(stems, vec!["catppuccin-mocha", "rose"]);
        assert_eq!(all[0].name(), "My Mocha");
        assert_eq!(all[1].mode(), Mode::Dark);
        assert!(matches!(all[1], Theme::Kitty(_)));
    }

    #[test]
    fn every_shipped_theme_parses() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("themes");
        let all = load_dir(&dir);
        assert_eq!(all.len(), 30, "thirty built-ins");
        assert!(all.iter().any(|t| t.stem() == "anibeam-dark" && t.mode() == Mode::Dark));
        assert!(all.iter().any(|t| t.stem() == "anibeam-light" && t.mode() == Mode::Light));
        assert!(all.iter().any(|t| t.stem() == "kanagawa-dragon"));
    }
}
```

`apps/linux/src/theme/config.rs`, tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_missing_and_per_key_when_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("theme.toml");
        assert_eq!(load(&path), ThemeSettings::default());
        std::fs::write(&path, "mode = \"purple\"\naccent = 9\ndensity = \"compact\"\n[theme]\ndark = \"gruvbox-dark-medium\"\n").unwrap();
        let s = load(&path);
        assert_eq!(s.mode, ModeSetting::System, "an unknown mode falls back");
        assert_eq!(s.accent, 4, "an out of range accent falls back");
        assert_eq!(s.density, Density::Compact);
        assert_eq!(s.theme_dark, "gruvbox-dark-medium");
        assert_eq!(s.theme_light, "anibeam-light");
    }

    #[test]
    fn save_keeps_comments_and_unknown_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/theme.toml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "# my notes\nmode = \"dark\"\nfuture_key = 1\n\n[theme]\n# pair\ndark = \"anibeam-dark\"\n").unwrap();
        let mut s = load(&path);
        s.mode = ModeSetting::Light;
        s.poster = Poster::L;
        s.theme_light = "catppuccin-latte".into();
        save(&path, &s).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# my notes"));
        assert!(text.contains("future_key = 1"));
        assert!(text.contains("# pair"));
        assert!(text.contains("mode = \"light\""));
        assert!(text.contains("poster = \"l\""));
        assert!(text.contains("light = \"catppuccin-latte\""));
        assert_eq!(load(&path), s);
    }

    #[test]
    fn save_creates_the_file_and_the_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new/theme.toml");
        save(&path, &ThemeSettings::default()).unwrap();
        assert_eq!(load(&path), ThemeSettings::default());
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p anibeam theme::`
Expected: compile errors for the three modules.

- [ ] **Step 3: Write the modules and fetch the themes**

`apps/linux/src/theme/kitty.rs`:

```rust
//! The kitty config chain, spec 4.2: the root file, then `include`, `globinclude` and
//! `envinclude` in order, last write wins, so the palette is what kitty itself would
//! paint. The files the chain touched are what the engine watches.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::theme::colour::Rgb;
use crate::theme::TerminalPalette;

const MAX_DEPTH: usize = 16;
const KEYS: [&str; 18] = [
    "foreground", "background", "color0", "color1", "color2", "color3", "color4", "color5", "color6", "color7", "color8",
    "color9", "color10", "color11", "color12", "color13", "color14", "color15",
];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Conf {
    pub values: HashMap<String, String>,
    pub directives: Vec<(String, String)>,
}

pub fn parse_conf(text: &str) -> Conf {
    let mut c = Conf::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, rest)) = line.split_once(char::is_whitespace) else { continue };
        let value = rest.trim();
        match key {
            "include" | "globinclude" | "envinclude" => c.directives.push((key.to_string(), value.to_string())),
            k if KEYS.contains(&k) => {
                c.values.insert(k.to_string(), value.to_string());
            }
            _ => {}
        }
    }
    c
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chain {
    pub files: Vec<PathBuf>,
    pub palette: Option<TerminalPalette>,
}

fn expand(path: &str, base: &Path, env: &dyn Fn(&str) -> Option<String>) -> PathBuf {
    let p = if let Some(rest) = path.strip_prefix("~/") {
        PathBuf::from(env("HOME").unwrap_or_default()).join(rest)
    } else {
        PathBuf::from(path)
    };
    if p.is_absolute() { p } else { base.join(p) }
}

fn glob_paths(pattern: &Path) -> Vec<PathBuf> {
    // One directory level of `*` is what kitty configs use; anything deeper is a plain path.
    let Some(parent) = pattern.parent() else { return vec![pattern.to_path_buf()] };
    let Some(name) = pattern.file_name().and_then(|n| n.to_str()) else { return vec![] };
    let Some((prefix, suffix)) = name.split_once('*') else { return vec![pattern.to_path_buf()] };
    let Ok(rd) = std::fs::read_dir(parent) else { return vec![] };
    let mut out: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with(prefix) && n.ends_with(suffix)))
        .collect();
    out.sort();
    out
}

fn walk(path: &Path, depth: usize, values: &mut HashMap<String, String>, files: &mut Vec<PathBuf>, env: &dyn Fn(&str) -> Option<String>) {
    files.push(path.to_path_buf());
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(text) = std::fs::read_to_string(path) else { return };
    let conf = parse_conf(&text);
    for (k, v) in conf.values {
        values.insert(k, v);
    }
    let base = path.parent().unwrap_or(Path::new("/"));
    for (kind, arg) in conf.directives {
        match kind.as_str() {
            "include" => walk(&expand(&arg, base, env), depth + 1, values, files, env),
            "globinclude" => {
                for p in glob_paths(&expand(&arg, base, env)) {
                    walk(&p, depth + 1, values, files, env);
                }
            }
            "envinclude" => {
                let prefix = arg.trim_end_matches('*');
                let mut names: Vec<String> = std::env::vars_os()
                    .filter_map(|(k, _)| k.into_string().ok())
                    .filter(|k| k.starts_with(prefix))
                    .collect();
                names.sort();
                for name in names {
                    if let Some(text) = env(&name) {
                        for (k, v) in parse_conf(&text).values {
                            values.insert(k, v);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn read_chain(root: &Path, env: &dyn Fn(&str) -> Option<String>) -> Chain {
    let mut values = HashMap::new();
    let mut files = Vec::new();
    walk(root, 0, &mut values, &mut files, env);
    let colour = |k: &str| values.get(k).and_then(|v| Rgb::hex(v));
    let mut colors = [Rgb { r: 0.0, g: 0.0, b: 0.0 }; 16];
    let mut complete = true;
    for (i, slot) in colors.iter_mut().enumerate() {
        match colour(&format!("color{i}")) {
            Some(c) => *slot = c,
            None => complete = false,
        }
    }
    let palette = match (colour("foreground"), colour("background"), complete) {
        (Some(foreground), Some(background), true) => Some(TerminalPalette { foreground, background, colors, source: "kitty".into() }),
        _ => None,
    };
    Chain { files, palette }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terminal {
    Kitty,
    Foot,
    Alacritty,
    Ghostty,
}

fn config_home(env: &dyn Fn(&str) -> Option<String>) -> PathBuf {
    env("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env("HOME").unwrap_or_default()).join(".config"))
}

pub fn root_config(env: &dyn Fn(&str) -> Option<String>) -> PathBuf {
    match env("KITTY_CONFIG_DIRECTORY") {
        Some(dir) => PathBuf::from(dir).join("kitty.conf"),
        None => config_home(env).join("kitty").join("kitty.conf"),
    }
}

/// `$TERMINAL` first, then config presence in the order kitty, foot, alacritty, ghostty.
/// Only kitty has a parser today; the other three make the engine fall back to the portal.
pub fn probe(env: &dyn Fn(&str) -> Option<String>) -> Option<Terminal> {
    if let Some(t) = env("TERMINAL") {
        let name = Path::new(&t).file_name().and_then(|n| n.to_str()).unwrap_or(&t);
        return match name {
            "kitty" => Some(Terminal::Kitty),
            "foot" | "footclient" => Some(Terminal::Foot),
            "alacritty" => Some(Terminal::Alacritty),
            "ghostty" => Some(Terminal::Ghostty),
            _ => None,
        };
    }
    let home = config_home(env);
    let present = |rel: &str| home.join(rel).is_file();
    if root_config(env).is_file() { Some(Terminal::Kitty) }
    else if present("foot/foot.ini") { Some(Terminal::Foot) }
    else if present("alacritty/alacritty.toml") { Some(Terminal::Alacritty) }
    else if present("ghostty/config") { Some(Terminal::Ghostty) }
    else { None }
}
```

`apps/linux/src/theme/base16.rs`:

```rust
//! Theme files: tinted-theming base16 YAML (verbatim, plus the optional `accent` key) and
//! a kitty `.conf` as the second format. No YAML crate: a base16 file is flat.

use std::path::Path;

use crate::theme::colour::Rgb;
use crate::theme::kitty;
use crate::theme::{Base16Theme, Mode, TerminalPalette};

fn unquote(v: &str) -> String {
    let v = match v.find(" #") { Some(i) => &v[..i], None => v };
    v.trim().trim_matches('"').trim_matches('\'').to_string()
}

pub fn parse(stem: &str, text: &str) -> Option<Base16Theme> {
    let mut name = stem.to_string();
    let mut variant = None;
    let mut accent = "base0D".to_string();
    let mut slots: [Option<Rgb>; 16] = [None; 16];
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else { continue };
        let key = key.trim();
        let value = unquote(value);
        match key {
            "name" => name = value,
            "variant" => variant = match value.as_str() { "dark" => Some(Mode::Dark), "light" => Some(Mode::Light), _ => None },
            "accent" => accent = value,
            k if k.starts_with("base") && k.len() == 6 => {
                if let (Ok(i), Some(c)) = (usize::from_str_radix(&k[4..], 16), Rgb::hex(&value)) {
                    if i < 16 {
                        slots[i] = Some(c);
                    }
                }
            }
            _ => {}
        }
    }
    let mut palette = [Rgb { r: 0.0, g: 0.0, b: 0.0 }; 16];
    for (i, s) in slots.iter().enumerate() {
        palette[i] = (*s)?;
    }
    Some(Base16Theme { stem: stem.to_string(), name, variant, accent, palette })
}

#[derive(Clone, Debug, PartialEq)]
pub enum Theme {
    Base16(Base16Theme),
    Kitty(TerminalPalette),
}

impl Theme {
    pub fn stem(&self) -> &str {
        match self { Theme::Base16(t) => &t.stem, Theme::Kitty(k) => &k.source }
    }
    pub fn name(&self) -> &str {
        match self { Theme::Base16(t) => &t.name, Theme::Kitty(k) => &k.source }
    }
    pub fn mode(&self) -> Mode {
        match self { Theme::Base16(t) => t.mode(), Theme::Kitty(k) => Mode::of_ground(k.background) }
    }
}

fn kitty_theme(stem: &str, text: &str) -> Option<TerminalPalette> {
    let conf = kitty::parse_conf(text);
    let colour = |k: &str| conf.values.get(k).and_then(|v| Rgb::hex(v));
    let mut colors = [Rgb { r: 0.0, g: 0.0, b: 0.0 }; 16];
    for (i, slot) in colors.iter_mut().enumerate() {
        *slot = colour(&format!("color{i}"))?;
    }
    Some(TerminalPalette { foreground: colour("foreground")?, background: colour("background")?, colors, source: stem.to_string() })
}

pub fn load_dir(dir: &Path) -> Vec<Theme> {
    let Ok(rd) = std::fs::read_dir(dir) else { return vec![] };
    let mut out = Vec::new();
    for entry in rd.flatten() {
        let path = entry.path();
        let (Some(stem), Some(ext)) = (path.file_stem().and_then(|s| s.to_str()), path.extension().and_then(|e| e.to_str())) else { continue };
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let theme = match ext {
            "yaml" | "yml" => parse(stem, &text).map(Theme::Base16),
            "conf" => kitty_theme(stem, &text).map(Theme::Kitty),
            _ => None,
        };
        if let Some(t) = theme {
            out.push(t);
        }
    }
    out.sort_by(|a, b| a.stem().cmp(b.stem()));
    out
}

/// Built-ins first, then the user's directory, which replaces a built-in of the same stem.
pub fn load_all(builtin: &Path, user: &Path) -> Vec<Theme> {
    let mut all = load_dir(builtin);
    for t in load_dir(user) {
        match all.iter_mut().find(|b| b.stem() == t.stem()) {
            Some(slot) => *slot = t,
            None => all.push(t),
        }
    }
    all.sort_by(|a, b| a.stem().cmp(b.stem()));
    all
}
```

`apps/linux/src/theme/config.rs`:

```rust
//! theme.toml: the theme settings, written through toml_edit so hand edits and comments
//! survive. Every key has a default; a missing or invalid key takes it.

use std::path::Path;

use toml_edit::{value, DocumentMut, Item, Table};

macro_rules! choice {
    ($name:ident { $($variant:ident = $s:literal),+ $(,)? }, default $default:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub fn as_str(self) -> &'static str { match self { $($name::$variant => $s),+ } }
            pub fn parse(s: &str) -> Option<Self> { match s { $($s => Some($name::$variant),)+ _ => None } }
        }
        impl Default for $name { fn default() -> Self { $name::$default } }
    };
}

choice!(ModeSetting { Dark = "dark", Light = "light", System = "system" }, default System);
choice!(Source { System = "system", Theme = "theme" }, default System);
choice!(Density { Compact = "compact", Normal = "normal", Comfortable = "comfortable" }, default Normal);
choice!(Poster { S = "s", M = "m", L = "l" }, default M);
choice!(Corners { Smooth = "smooth", Plain = "plain" }, default Smooth);

impl Density {
    pub fn factor(self) -> f64 {
        match self { Density::Compact => 0.75, Density::Normal => 1.0, Density::Comfortable => 1.25 }
    }
}

impl Poster {
    pub fn width(self) -> i32 {
        match self { Poster::S => 140, Poster::M => 180, Poster::L => 240 }
    }
}

impl Corners {
    pub fn smoothing(self) -> f64 {
        match self { Corners::Smooth => 0.6, Corners::Plain => 0.0 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeSettings {
    pub mode: ModeSetting,
    pub source: Source,
    pub accent: u8,
    pub density: Density,
    pub poster: Poster,
    pub corners: Corners,
    pub theme_dark: String,
    pub theme_light: String,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        ThemeSettings {
            mode: ModeSetting::System,
            source: Source::System,
            accent: 4,
            density: Density::Normal,
            poster: Poster::M,
            corners: Corners::Smooth,
            theme_dark: "anibeam-dark".into(),
            theme_light: "anibeam-light".into(),
        }
    }
}

fn str_of(doc: &DocumentMut, key: &str) -> Option<String> {
    doc.get(key).and_then(Item::as_str).map(String::from)
}

pub fn load(path: &Path) -> ThemeSettings {
    let d = ThemeSettings::default();
    let Ok(text) = std::fs::read_to_string(path) else { return d };
    let Ok(doc) = text.parse::<DocumentMut>() else { return d };
    let theme = doc.get("theme").and_then(Item::as_table);
    let pair = |k: &str, default: &str| theme.and_then(|t| t.get(k)).and_then(Item::as_str).map(String::from).unwrap_or_else(|| default.to_string());
    ThemeSettings {
        mode: str_of(&doc, "mode").and_then(|s| ModeSetting::parse(&s)).unwrap_or(d.mode),
        source: str_of(&doc, "source").and_then(|s| Source::parse(&s)).unwrap_or(d.source),
        accent: doc.get("accent").and_then(Item::as_integer).filter(|a| (1..=7).contains(a)).map(|a| a as u8).unwrap_or(d.accent),
        density: str_of(&doc, "density").and_then(|s| Density::parse(&s)).unwrap_or(d.density),
        poster: str_of(&doc, "poster").and_then(|s| Poster::parse(&s)).unwrap_or(d.poster),
        corners: str_of(&doc, "corners").and_then(|s| Corners::parse(&s)).unwrap_or(d.corners),
        theme_dark: pair("dark", &d.theme_dark),
        theme_light: pair("light", &d.theme_light),
    }
}

pub fn save(path: &Path, s: &ThemeSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut doc = std::fs::read_to_string(path).ok().and_then(|t| t.parse::<DocumentMut>().ok()).unwrap_or_default();
    doc["mode"] = value(s.mode.as_str());
    doc["source"] = value(s.source.as_str());
    doc["accent"] = value(i64::from(s.accent));
    doc["density"] = value(s.density.as_str());
    doc["poster"] = value(s.poster.as_str());
    doc["corners"] = value(s.corners.as_str());
    if !doc.contains_table("theme") {
        doc["theme"] = Item::Table(Table::new());
    }
    doc["theme"]["dark"] = value(s.theme_dark.as_str());
    doc["theme"]["light"] = value(s.theme_light.as_str());
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, doc.to_string())?;
    std::fs::rename(tmp, path)
}
```

The thirty built-ins. Write the two AniBeam files from the prototype and fetch the rest at a pinned commit of tinted-theming/schemes, verbatim:

```bash
mkdir -p apps/linux/themes
cp spikes/home-grid-qml/themes/anibeam-dark.yaml spikes/home-grid-qml/themes/anibeam-light.yaml apps/linux/themes/
commit=$(gh api repos/tinted-theming/schemes/commits/spec-0.11 --jq .sha)
for n in catppuccin-latte catppuccin-frappe catppuccin-macchiato catppuccin-mocha \
         gruvbox-dark-hard gruvbox-dark-medium gruvbox-dark-soft gruvbox-light-hard gruvbox-light-medium gruvbox-light-soft \
         tokyo-night-dark tokyo-night-storm tokyo-night-moon tokyo-night-light nord nord-light dracula onedark \
         solarized-dark solarized-light everforest-dark-hard everforest-dark-medium everforest-dark-soft \
         everforest-light-hard everforest-light-medium everforest-light-soft kanagawa kanagawa-dragon; do
  curl -fsSL "https://raw.githubusercontent.com/tinted-theming/schemes/$commit/base16/$n.yaml" -o "apps/linux/themes/$n.yaml"
done
ls apps/linux/themes | wc -l   # 30
echo "$commit" > apps/linux/themes/UPSTREAM   # the commit the 28 files came from
```

(`apps/linux/themes/UPSTREAM` is one line, the commit hash; `load_dir` ignores it since it has no extension. The `every_shipped_theme_parses` test counts themes, not files.)

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p anibeam theme::`
Expected: all theme tests pass, including `every_shipped_theme_parses` with 30.

- [ ] **Step 5: Commit**

```bash
git add apps/linux/src/theme apps/linux/themes
git commit -m "feat(shell): the kitty chain, base16 files, theme.toml and the thirty built-in themes"
```

---
### Task 5: The theme engine, the portal, the Theme singleton, Tokens.qml and the primitives

The Rust half now owns colour end to end: it reads the inputs, resolves both modes' palettes, watches every file and the portal, and hands QML a `Theme` singleton. `Tokens.qml` derives the sizes from it and is what every component reads as `theme`, so the prototype's QML carries in unchanged, and `LookPane` can still instantiate its own `Tokens` forced to the other mode.

**Files:**
- Create: `apps/linux/src/theme/portal.rs`, `apps/linux/src/theme/engine.rs`
- Create: `apps/linux/src/bridge/theme.rs`
- Modify: `apps/linux/cpp/helpers.h`, `apps/linux/cpp/helpers.cpp` (add `set_app_palette`), `apps/linux/src/bridge/helpers.rs`, `apps/linux/src/bridge/mod.rs`, `apps/linux/build.rs`, `apps/linux/src/main.rs`
- Create: `apps/linux/qml/Tokens.qml`, `apps/linux/qml/TokensPage.qml`
- Create (copied from `spikes/home-grid-qml/qml/`): `Corner.qml`, `Icon.qml`, `Chip.qml`, `Seg.qml`, `Switch.qml`, `Button.qml`, `Field.qml`, `Dropdown.qml`, `Swatches.qml`, `SliderRow.qml`, `SettingRow.qml`, `Panel.qml`
- Create: `apps/linux/assets/icons/*.svg` and `apps/linux/assets/icons/LICENSE`, `apps/linux/scripts/icons.sh`
- Modify: `apps/linux/qml/Main.qml` (the ground from the tokens, `Tokens { id: theme }`, the page switch for `--page tokens`)

**Interfaces:**
- Consumes: `theme::{config, kitty, base16, tokens, Palette, Mode, Portal, TerminalPalette, Steps}`, `paths::ShellPaths`, `runtime::runtime()`.
- Produces:
  - `portal::read(conn: &zbus::Connection) -> Portal` (async; any error or absent key is unset), `portal::parse_scheme(&Value) -> Option<Mode>`, `portal::parse_contrast(&Value) -> bool`, `portal::parse_accent(&Value) -> Option<Rgb>`, `portal::watch(conn, on_change: impl Fn() + Send + 'static)` (async, runs until the connection drops).
  - `engine::Inputs { settings: ThemeSettings, terminal: Option<TerminalPalette>, terminal_files: Vec<PathBuf>, portal: Portal, themes: Vec<base16::Theme> }`, `engine::Resolved { inputs: Inputs, dark: Palette, light: Palette, mode: Mode }`, `engine::resolve(inputs: Inputs) -> Resolved` (pure), `engine::read_inputs(paths: &ShellPaths, portal: Portal) -> Inputs`, `engine::run(paths: ShellPaths, push: impl Fn(Resolved) + Send + Sync + 'static, commands: tokio::sync::mpsc::UnboundedReceiver<ThemeSettings>)` (async: the initial read, the watchers, the portal stream, and settings written on request).
  - The QML singleton `Theme` with read-only-by-convention properties `mode`, `source`, `accent` (int), `density`, `poster`, `corners`, `themeDark`, `themeLight` (the settings as strings), `resolvedMode` (`"dark"` or `"light"`), `sourceLabel`, `contrast` (bool), `dark` and `light` (maps of token name to colour: `bg`, `surface`, `surface_raised`, `surface_sunken`, `surface_pressed`, `line`, `line_strong`, `text`, `text_dim`, `text_faint`, `accent`, `accent_text`, `accent_soft`, `red_soft`, `focus`, `red`, `orange`, `yellow`, `green`, `cyan`, `blue`, `purple`, `brown`), `themes` (array of `{ stem, name, mode }`), `densityFactor`, `posterWidth`, `smoothing`, `ready`; invokables `pickMode(s)`, `pickSource(s)`, `pickAccent(n)`, `pickDensity(s)`, `pickPoster(s)`, `pickCorners(s)`, `pickTheme(mode, stem)`, `formatHue(anilistFormat) -> token name`, `statusHue(listStatus) -> token name`.
  - `Tokens.qml`: an `Item` with `property string mode` (default `Theme.resolvedMode`) and every derived token: the colours above as `color` properties in camelCase (`surfaceRaised`, `textDim`, ...), `scrim`, `focusRing`, `space(n)`, `densityFactor`, `radiusSm/Md/Lg/Xl`, `cornerSmoothing`, `cornerBase` (14), `typeSmall/Normal/Large`, `fontSans`, `fontMono`, `motionFast/Normal/Slow`, `controlHeight`, `disabledOpacity`, `posterWidth`, the status colours `statusWatching`, `statusCompleted`, `statusPaused`, `statusDropped`, `statusPlanning`, `statusRewatching`, `behind`, `caughtUp`, `token(name)` (a token by its Rust name), `hue(name)` (a hue by `formatHue`/`statusHue` output), `tone(a, b, t)`.
  - The C++ helper `set_app_palette(window, text, base, highlight, highlighted_text, button, button_text: &QColor)`.

- [ ] **Step 1: Write the failing tests**

`apps/linux/src/theme/portal.rs`, tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::{Structure, Value};

    #[test]
    fn scheme_contrast_and_accent_parse_and_reject() {
        assert_eq!(parse_scheme(&Value::U32(1)), Some(Mode::Dark));
        assert_eq!(parse_scheme(&Value::U32(2)), Some(Mode::Light));
        assert_eq!(parse_scheme(&Value::U32(0)), None);
        assert_eq!(parse_scheme(&Value::U32(9)), None);
        assert_eq!(parse_scheme(&Value::Str("dark".into())), None);
        assert!(parse_contrast(&Value::U32(1)));
        assert!(!parse_contrast(&Value::U32(0)));
        let accent = Value::Structure(Structure::from((0.2078_f64, 0.5176_f64, 0.8941_f64)));
        assert_eq!(parse_accent(&accent).unwrap().to_hex(), "#3584e4");
        let out_of_range = Value::Structure(Structure::from((-1.0_f64, 0.5_f64, 0.5_f64)));
        assert_eq!(parse_accent(&out_of_range), None);
        // a value wrapped once more, the deprecated Read's shape, still parses
        let wrapped = Value::Value(Box::new(Value::U32(2)));
        assert_eq!(parse_scheme(&wrapped), Some(Mode::Light));
    }
}
```

`apps/linux/src/theme/engine.rs`, tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::base16::Theme as ThemeFile;
    use crate::theme::colour::Rgb;
    use crate::theme::config::{ModeSetting, Source, ThemeSettings};
    use crate::theme::{Base16Theme, Mode, Portal, TerminalPalette};

    fn term(bg: &str) -> TerminalPalette {
        let c = Rgb::hex("#888888").unwrap();
        TerminalPalette { foreground: Rgb::hex("#e0def4").unwrap(), background: Rgb::hex(bg).unwrap(), colors: [c; 16], source: "kitty".into() }
    }

    fn theme(stem: &str, mode: Mode) -> ThemeFile {
        let (bg, fg) = match mode { Mode::Dark => ("#101010", "#f0f0f0"), Mode::Light => ("#f0f0f0", "#101010") };
        let mut palette = [Rgb::hex("#777777").unwrap(); 16];
        palette[0] = Rgb::hex(bg).unwrap();
        palette[5] = Rgb::hex(fg).unwrap();
        ThemeFile::Base16(Base16Theme { stem: stem.into(), name: stem.into(), variant: Some(mode), accent: "base0D".into(), palette })
    }

    fn inputs() -> Inputs {
        Inputs {
            settings: ThemeSettings::default(),
            terminal: Some(term("#191724")),
            terminal_files: vec![],
            portal: Portal { scheme: Some(Mode::Light), contrast: false, accent: None },
            themes: vec![theme("anibeam-dark", Mode::Dark), theme("anibeam-light", Mode::Light), theme("mocha", Mode::Dark)],
        }
    }

    #[test]
    fn system_mode_follows_the_terminal_before_the_portal() {
        let r = resolve(inputs());
        assert_eq!(r.mode, Mode::Dark, "the terminal is dark, the portal says light");
        assert_eq!(r.dark.bg.to_hex(), "#191724");
        assert_eq!(r.light.mode, Mode::Light, "the other mode is still resolved for the preview");
        let mut no_terminal = inputs();
        no_terminal.terminal = None;
        let r = resolve(no_terminal);
        assert_eq!(r.mode, Mode::Light, "with no terminal the portal decides");
        assert!(r.light.source_label.starts_with("portal"));
    }

    #[test]
    fn a_forced_mode_and_a_theme_pair() {
        let mut i = inputs();
        i.settings.mode = ModeSetting::Light;
        assert_eq!(resolve(i.clone()).mode, Mode::Light);
        i.settings.source = Source::Theme;
        i.settings.theme_dark = "mocha".into();
        let r = resolve(i.clone());
        assert_eq!(r.dark.source_label, "theme mocha");
        assert_eq!(r.light.source_label, "theme anibeam-light");
        i.settings.theme_light = "missing".into();
        let r = resolve(i);
        assert_eq!(r.light.source_label, "theme anibeam-light", "a missing stem falls back to the built-in");
    }

    #[test]
    fn a_kitty_theme_file_is_a_terminal_palette() {
        let mut i = inputs();
        i.settings.source = Source::Theme;
        i.settings.theme_dark = "rose".into();
        i.themes.push(ThemeFile::Kitty(TerminalPalette { source: "rose".into(), ..term("#191724") }));
        let r = resolve(i);
        assert_eq!(r.dark.source_label, "terminal rose");
        assert_eq!(r.dark.bg.to_hex(), "#191724");
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p anibeam theme::`
Expected: compile errors for `portal` and `engine`.

- [ ] **Step 3: Write the portal reader, the engine, the singleton, the QML**

`apps/linux/src/theme/portal.rs`:

```rust
//! org.freedesktop.portal.Settings over zbus: `ReadOne` for the three appearance keys and
//! the `SettingChanged` stream. Every failure is "unset", because on Hyprland with the gtk
//! backend a missing accent-color is the normal answer.

use futures_util::StreamExt;
use zbus::zvariant::{OwnedValue, Value};

use crate::theme::colour::Rgb;
use crate::theme::{Mode, Portal};

const NAMESPACE: &str = "org.freedesktop.appearance";

#[zbus::proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    fn read_one(&self, namespace: &str, key: &str) -> zbus::Result<OwnedValue>;

    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: Value<'_>) -> zbus::Result<()>;
}

fn unwrap<'a>(v: &'a Value<'a>) -> &'a Value<'a> {
    match v { Value::Value(inner) => inner, other => other }
}

pub fn parse_scheme(v: &Value) -> Option<Mode> {
    match unwrap(v) { Value::U32(1) => Some(Mode::Dark), Value::U32(2) => Some(Mode::Light), _ => None }
}

pub fn parse_contrast(v: &Value) -> bool {
    matches!(unwrap(v), Value::U32(1))
}

pub fn parse_accent(v: &Value) -> Option<Rgb> {
    let Value::Structure(s) = unwrap(v) else { return None };
    let fields = s.fields();
    if fields.len() != 3 {
        return None;
    }
    let mut out = [0.0; 3];
    for (i, f) in fields.iter().enumerate() {
        let Value::F64(x) = f else { return None };
        if !(0.0..=1.0).contains(x) {
            return None;
        }
        out[i] = *x;
    }
    Some(Rgb { r: out[0], g: out[1], b: out[2] })
}

pub async fn read(conn: &zbus::Connection) -> Portal {
    let Ok(proxy) = SettingsProxy::new(conn).await else { return Portal::default() };
    let get = |key: &'static str| {
        let proxy = proxy.clone();
        async move { proxy.read_one(NAMESPACE, key).await.ok().map(|v| Value::from(v)) }
    };
    let scheme = get("color-scheme").await.and_then(|v| parse_scheme(&v));
    let contrast = get("contrast").await.is_some_and(|v| parse_contrast(&v));
    let accent = get("accent-color").await.and_then(|v| parse_accent(&v));
    Portal { scheme, contrast, accent }
}

/// Calls `on_change` for every SettingChanged in the appearance namespace, until the bus
/// goes away. The caller re-reads with `read`; the signal's value is not trusted alone.
pub async fn watch(conn: zbus::Connection, on_change: impl Fn() + Send + 'static) {
    let Ok(proxy) = SettingsProxy::new(&conn).await else { return };
    let Ok(mut stream) = proxy.receive_setting_changed().await else { return };
    while let Some(signal) = stream.next().await {
        if let Ok(args) = signal.args() {
            if args.namespace() == &NAMESPACE {
                on_change();
            }
        }
    }
}
```

Add `futures-util = "0.3"` to `[dependencies]` in `apps/linux/Cargo.toml`.

`apps/linux/src/theme/engine.rs`:

```rust
//! Resolve the inputs into both modes' palettes, and keep them current: notify on the
//! kitty chain, the theme directories and theme.toml; the portal's SettingChanged; and the
//! settings the Theme singleton writes. Nothing here is a rule about the library.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;
use tokio::sync::mpsc;

use crate::paths::ShellPaths;
use crate::theme::base16::{self, Theme as ThemeFile};
use crate::theme::config::{self, ModeSetting, Source, ThemeSettings};
use crate::theme::{kitty, portal, tokens, Mode, Palette, Portal, Steps, TerminalPalette};

#[derive(Clone, Debug, PartialEq)]
pub struct Inputs {
    pub settings: ThemeSettings,
    pub terminal: Option<TerminalPalette>,
    pub terminal_files: Vec<PathBuf>,
    pub portal: Portal,
    pub themes: Vec<ThemeFile>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    pub inputs: Inputs,
    pub dark: Palette,
    pub light: Palette,
    pub mode: Mode,
}

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

fn palette_for(i: &Inputs, mode: Mode) -> Palette {
    let steps = Steps::default();
    let contrast = i.portal.contrast;
    let accent = i.settings.accent;
    let from_theme = |stem: &str| -> Option<Palette> {
        i.themes.iter().find(|t| t.stem() == stem).map(|t| match t {
            ThemeFile::Base16(b) => tokens::from_base16(b, contrast, &steps),
            ThemeFile::Kitty(k) => tokens::from_terminal(k, mode, accent, contrast, &steps),
        })
    };
    match i.settings.source {
        Source::System => match &i.terminal {
            Some(term) => tokens::from_terminal(term, mode, accent, contrast, &steps),
            None => tokens::from_portal(&i.portal, mode, contrast, &steps),
        },
        Source::Theme => {
            let (stem, builtin) = match mode {
                Mode::Dark => (i.settings.theme_dark.as_str(), "anibeam-dark"),
                Mode::Light => (i.settings.theme_light.as_str(), "anibeam-light"),
            };
            from_theme(stem)
                .or_else(|| from_theme(builtin))
                .unwrap_or_else(|| tokens::from_portal(&i.portal, mode, contrast, &steps))
        }
    }
}

pub fn resolve(inputs: Inputs) -> Resolved {
    let system = match &inputs.terminal {
        Some(t) => Mode::of_ground(t.background),
        None => inputs.portal.scheme.unwrap_or(Mode::Dark),
    };
    let mode = match inputs.settings.mode {
        ModeSetting::System => system,
        ModeSetting::Dark => Mode::Dark,
        ModeSetting::Light => Mode::Light,
    };
    let dark = palette_for(&inputs, Mode::Dark);
    let light = palette_for(&inputs, Mode::Light);
    Resolved { inputs, dark, light, mode }
}

pub fn read_inputs(paths: &ShellPaths, portal: Portal) -> Inputs {
    let settings = config::load(&paths.theme_toml());
    let (terminal, terminal_files) = match kitty::probe(&env) {
        Some(kitty::Terminal::Kitty) => {
            let chain = kitty::read_chain(&kitty::root_config(&env), &env);
            (chain.palette, chain.files)
        }
        _ => (None, vec![]),
    };
    let themes = base16::load_all(&paths.builtin_themes_dir(), &paths.user_themes_dir());
    Inputs { settings, terminal, terminal_files, portal, themes }
}

/// The engine's loop. `push` receives every new resolution on whatever thread produced it;
/// the bridge hops it to the Qt thread. `commands` carries settings the singleton wrote.
pub async fn run(paths: ShellPaths, push: impl Fn(Resolved) + Send + Sync + 'static, mut commands: mpsc::UnboundedReceiver<ThemeSettings>) {
    let push = Arc::new(push);
    let conn = zbus::Connection::session().await.ok();
    let portal_state = match &conn {
        Some(c) => portal::read(c).await,
        None => Portal::default(),
    };
    let (wake_tx, mut wake_rx) = mpsc::unbounded_channel::<()>();

    // Files: the chain, both theme directories, theme.toml's directory (the file is replaced
    // by rename on save). The debouncer's own thread sends a wake; the loop re-reads.
    let mut watched: Vec<PathBuf> = Vec::new();
    let wake = wake_tx.clone();
    let mut debouncer = new_debouncer(Duration::from_millis(200), None, move |_res| {
        let _ = wake.send(());
    })
    .ok();
    let mut inputs = read_inputs(&paths, portal_state);
    let rewatch = |debouncer: &mut Option<notify_debouncer_full::Debouncer<_, _>>, watched: &mut Vec<PathBuf>, inputs: &Inputs| {
        let Some(d) = debouncer else { return };
        for p in watched.drain(..) {
            let _ = d.unwatch(&p);
        }
        let mut wanted = vec![paths.config_dir(), paths.user_themes_dir(), paths.builtin_themes_dir()];
        wanted.extend(inputs.terminal_files.iter().filter_map(|f| f.parent().map(PathBuf::from)));
        wanted.sort();
        wanted.dedup();
        for p in wanted {
            if p.is_dir() && d.watch(&p, RecursiveMode::NonRecursive).is_ok() {
                watched.push(p);
            }
        }
    };
    rewatch(&mut debouncer, &mut watched, &inputs);
    push(resolve(inputs.clone()));

    if let Some(c) = conn.clone() {
        let wake = wake_tx.clone();
        tokio::spawn(portal::watch(c, move || {
            let _ = wake.send(());
        }));
    }

    loop {
        tokio::select! {
            Some(settings) = commands.recv() => {
                if let Err(e) = config::save(&paths.theme_toml(), &settings) {
                    eprintln!("anibeam: theme.toml: {e}");
                }
                inputs.settings = settings;
                push(resolve(inputs.clone()));
            }
            Some(()) = wake_rx.recv() => {
                let portal_state = match &conn { Some(c) => portal::read(c).await, None => Portal::default() };
                let fresh = read_inputs(&paths, portal_state);
                if fresh != inputs {
                    inputs = fresh;
                    rewatch(&mut debouncer, &mut watched, &inputs);
                    push(resolve(inputs.clone()));
                }
            }
            else => break,
        }
    }
}
```

`apps/linux/src/bridge/theme.rs`:

```rust
//! Theme: the singleton QML reads colours and the theme settings from. The engine pushes
//! resolutions through the Qt thread; the pick* invokables write settings through the
//! engine and apply the result at once, so a switch never waits on a file watcher.

use core::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QColor, QJsonArray, QJsonObject, QJsonValue, QMap, QMapPair_QString_QVariant, QString, QVariant};
use tokio::sync::mpsc;

use crate::theme::config::{Corners, Density, ModeSetting, Poster, Source, ThemeSettings};
use crate::theme::engine::{resolve, Resolved};
use crate::theme::{format_hue, status_hue, Mode, Palette};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant = cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
        include!("cxx-qt-lib/qjsonarray.h");
        type QJsonArray = cxx_qt_lib::QJsonArray;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, mode)]
        #[qproperty(QString, source)]
        #[qproperty(i32, accent)]
        #[qproperty(QString, density)]
        #[qproperty(QString, poster)]
        #[qproperty(QString, corners)]
        #[qproperty(QString, theme_dark)]
        #[qproperty(QString, theme_light)]
        #[qproperty(QString, resolved_mode)]
        #[qproperty(QString, source_label)]
        #[qproperty(bool, contrast)]
        #[qproperty(QMap_QString_QVariant, dark)]
        #[qproperty(QMap_QString_QVariant, light)]
        #[qproperty(QJsonArray, themes)]
        #[qproperty(f64, density_factor)]
        #[qproperty(i32, poster_width)]
        #[qproperty(f64, smoothing)]
        #[qproperty(bool, ready)]
        type Theme = super::ThemeRust;

        #[qinvokable]
        fn pick_mode(self: Pin<&mut Self>, mode: &QString);
        #[qinvokable]
        fn pick_source(self: Pin<&mut Self>, source: &QString);
        #[qinvokable]
        fn pick_accent(self: Pin<&mut Self>, slot: i32);
        #[qinvokable]
        fn pick_density(self: Pin<&mut Self>, density: &QString);
        #[qinvokable]
        fn pick_poster(self: Pin<&mut Self>, poster: &QString);
        #[qinvokable]
        fn pick_corners(self: Pin<&mut Self>, corners: &QString);
        #[qinvokable]
        fn pick_theme(self: Pin<&mut Self>, mode: &QString, stem: &QString);
        #[qinvokable]
        fn format_hue(self: &Self, format: &QString) -> QString;
        #[qinvokable]
        fn status_hue(self: &Self, status: &QString) -> QString;
    }

    impl cxx_qt::Threading for Theme {}
    impl cxx_qt::Initialize for Theme {}
}

pub struct ThemeRust {
    mode: QString,
    source: QString,
    accent: i32,
    density: QString,
    poster: QString,
    corners: QString,
    theme_dark: QString,
    theme_light: QString,
    resolved_mode: QString,
    source_label: QString,
    contrast: bool,
    dark: QMap<QMapPair_QString_QVariant>,
    light: QMap<QMapPair_QString_QVariant>,
    themes: QJsonArray,
    density_factor: f64,
    poster_width: i32,
    smoothing: f64,
    ready: bool,
    resolved: Option<Resolved>,
    commands: Option<mpsc::UnboundedSender<ThemeSettings>>,
}

impl Default for ThemeRust {
    fn default() -> Self {
        ThemeRust {
            mode: QString::from("system"),
            source: QString::from("system"),
            accent: 4,
            density: QString::from("normal"),
            poster: QString::from("m"),
            corners: QString::from("smooth"),
            theme_dark: QString::from("anibeam-dark"),
            theme_light: QString::from("anibeam-light"),
            resolved_mode: QString::from("dark"),
            source_label: QString::default(),
            contrast: false,
            dark: QMap::default(),
            light: QMap::default(),
            themes: QJsonArray::default(),
            density_factor: 1.0,
            poster_width: 180,
            smoothing: 0.6,
            ready: false,
            resolved: None,
            commands: None,
        }
    }
}

fn colour_map(p: &Palette) -> QMap<QMapPair_QString_QVariant> {
    let mut m = QMap::<QMapPair_QString_QVariant>::default();
    for name in Palette::NAMES {
        let c = p.get(name).expect("every name resolves");
        let (r, g, b) = c.bytes();
        let key = name.replace('.', "_");
        m.insert(QString::from(&key), QVariant::from(&QColor::from_rgb(i32::from(r), i32::from(g), i32::from(b))));
    }
    m
}

impl cxx_qt::Initialize for qobject::Theme {
    fn initialize(mut self: Pin<&mut Self>) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.as_mut().rust_mut().commands = Some(tx);
        let qt = self.qt_thread();
        let paths = crate::runtime::paths().clone();
        crate::runtime::runtime().spawn(crate::theme::engine::run(
            paths,
            move |resolved: Resolved| {
                qt.queue(move |theme: Pin<&mut qobject::Theme>| theme.apply(resolved)).ok();
            },
            rx,
        ));
    }
}

impl qobject::Theme {
    /// Runs on the Qt thread: every property from one resolution.
    pub fn apply(mut self: Pin<&mut Self>, r: Resolved) {
        let s = &r.inputs.settings;
        self.as_mut().set_mode(QString::from(s.mode.as_str()));
        self.as_mut().set_source(QString::from(s.source.as_str()));
        self.as_mut().set_accent(i32::from(s.accent));
        self.as_mut().set_density(QString::from(s.density.as_str()));
        self.as_mut().set_poster(QString::from(s.poster.as_str()));
        self.as_mut().set_corners(QString::from(s.corners.as_str()));
        self.as_mut().set_theme_dark(QString::from(&s.theme_dark));
        self.as_mut().set_theme_light(QString::from(&s.theme_light));
        self.as_mut().set_density_factor(s.density.factor());
        self.as_mut().set_poster_width(s.poster.width());
        self.as_mut().set_smoothing(s.corners.smoothing());
        self.as_mut().set_contrast(r.inputs.portal.contrast);
        let mut themes = QJsonArray::default();
        for t in &r.inputs.themes {
            let mut o = QJsonObject::default();
            o.insert(&QString::from("stem"), &QJsonValue::from(&QString::from(t.stem())));
            o.insert(&QString::from("name"), &QJsonValue::from(&QString::from(t.name())));
            o.insert(&QString::from("mode"), &QJsonValue::from(&QString::from(t.mode().as_str())));
            themes.append(&QJsonValue::from(&o));
        }
        self.as_mut().set_themes(themes);
        self.as_mut().set_dark(colour_map(&r.dark));
        self.as_mut().set_light(colour_map(&r.light));
        let current = match r.mode { Mode::Dark => &r.dark, Mode::Light => &r.light };
        self.as_mut().set_source_label(QString::from(&current.source_label));
        self.as_mut().set_resolved_mode(QString::from(r.mode.as_str()));
        let (window, text, base, highlight, on_accent, button) = (
            current.bg, current.text, current.surface_sunken, current.accent, current.accent_text, current.surface_raised,
        );
        let q = |c: crate::theme::colour::Rgb| { let (r, g, b) = c.bytes(); QColor::from_rgb(i32::from(r), i32::from(g), i32::from(b)) };
        crate::bridge::helpers::ffi::set_app_palette(&q(window), &q(text), &q(base), &q(highlight), &q(on_accent), &q(button), &q(text));
        self.as_mut().rust_mut().resolved = Some(r);
        self.as_mut().set_ready(true);
    }

    fn change(mut self: Pin<&mut Self>, edit: impl FnOnce(&mut ThemeSettings)) {
        let Some(mut r) = self.as_ref().resolved.clone() else { return };
        edit(&mut r.inputs.settings);
        let settings = r.inputs.settings.clone();
        let fresh = resolve(r.inputs);
        self.as_mut().apply(fresh);
        if let Some(tx) = &self.as_ref().commands {
            tx.send(settings).ok();
        }
    }

    pub fn pick_mode(self: Pin<&mut Self>, mode: &QString) {
        if let Some(m) = ModeSetting::parse(&mode.to_string()) { self.change(|s| s.mode = m) }
    }
    pub fn pick_source(self: Pin<&mut Self>, source: &QString) {
        if let Some(v) = Source::parse(&source.to_string()) { self.change(|s| s.source = v) }
    }
    pub fn pick_accent(self: Pin<&mut Self>, slot: i32) {
        if (1..=7).contains(&slot) { self.change(|s| s.accent = slot as u8) }
    }
    pub fn pick_density(self: Pin<&mut Self>, density: &QString) {
        if let Some(v) = Density::parse(&density.to_string()) { self.change(|s| s.density = v) }
    }
    pub fn pick_poster(self: Pin<&mut Self>, poster: &QString) {
        if let Some(v) = Poster::parse(&poster.to_string()) { self.change(|s| s.poster = v) }
    }
    pub fn pick_corners(self: Pin<&mut Self>, corners: &QString) {
        if let Some(v) = Corners::parse(&corners.to_string()) { self.change(|s| s.corners = v) }
    }
    pub fn pick_theme(self: Pin<&mut Self>, mode: &QString, stem: &QString) {
        let stem = stem.to_string();
        match mode.to_string().as_str() {
            "dark" => self.change(|s| s.theme_dark = stem),
            "light" => self.change(|s| s.theme_light = stem),
            _ => {}
        }
    }
    pub fn format_hue(&self, format: &QString) -> QString {
        QString::from(format_hue(&format.to_string()))
    }
    pub fn status_hue(&self, status: &QString) -> QString {
        QString::from(status_hue(&status.to_string()))
    }
}
```

`runtime.rs` gains the paths:

```rust
static PATHS: OnceLock<crate::paths::ShellPaths> = OnceLock::new();
pub fn install_paths(p: crate::paths::ShellPaths) { PATHS.set(p).ok(); }
pub fn paths() -> &'static crate::paths::ShellPaths { PATHS.get().expect("paths are installed before the QML engine loads") }
```

and `main.rs` calls `runtime::install_paths(paths.clone())` where Task 1 left `let _ = &paths;`.

The palette helper, appended to `cpp/helpers.h`:

```cpp
#include <QtGui/QColor>
// The tokens into the application palette, so a stock control (the file dialog, a scroll
// bar) matches the shell.
void set_app_palette(const QColor &window, const QColor &text, const QColor &base, const QColor &highlight,
                     const QColor &highlightedText, const QColor &button, const QColor &buttonText);
```

and to `cpp/helpers.cpp`:

```cpp
#include <QtGui/QPalette>

void set_app_palette(const QColor &window, const QColor &text, const QColor &base, const QColor &highlight,
                     const QColor &highlightedText, const QColor &button, const QColor &buttonText)
{
    QPalette p = QGuiApplication::palette();
    p.setColor(QPalette::Window, window);
    p.setColor(QPalette::WindowText, text);
    p.setColor(QPalette::Base, base);
    p.setColor(QPalette::Text, text);
    p.setColor(QPalette::Highlight, highlight);
    p.setColor(QPalette::HighlightedText, highlightedText);
    p.setColor(QPalette::Button, button);
    p.setColor(QPalette::ButtonText, buttonText);
    p.setColor(QPalette::Accent, highlight);
    QGuiApplication::setPalette(p);
}
```

and its declaration in `src/bridge/helpers.rs` inside the second `unsafe extern "C++"` block, with `include!("cxx-qt-lib/qcolor.h"); type QColor = cxx_qt_lib::QColor;` added to the first:

```rust
        fn set_app_palette(window: &QColor, text: &QColor, base: &QColor, highlight: &QColor, highlighted_text: &QColor, button: &QColor, button_text: &QColor);
```

`apps/linux/qml/Tokens.qml`:

```qml
// The derived tokens. Instantiated once at the root of Main.qml as `theme`, which every
// component reaches through the context chain; LookPane instantiates its own with `mode`
// forced, so a preview renders the other mode with the same components. Colours come from
// the Rust Theme singleton; the sizes are arithmetic on its settings.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: root
    property string mode: Theme.resolvedMode
    readonly property bool dark: mode !== "light"
    readonly property var set: dark ? Theme.dark : Theme.light
    function token(name) { var v = set[name.replace(".", "_")]; return v === undefined ? "#ff00ff" : v }

    // Colours
    readonly property color bg: token("bg")
    readonly property color surface: token("surface")
    readonly property color surfaceRaised: token("surface_raised")
    readonly property color surfaceSunken: token("surface_sunken")
    readonly property color surfacePressed: token("surface_pressed")
    readonly property color line: token("line")
    readonly property color lineStrong: token("line_strong")
    readonly property color text: token("text")
    readonly property color textDim: token("text_dim")
    readonly property color textFaint: token("text_faint")
    readonly property color accent: token("accent")
    readonly property color accentText: token("accent_text")
    readonly property color accentSoft: token("accent_soft")
    readonly property color redSoft: token("red_soft")
    readonly property color focusRing: token("focus")
    readonly property color red: token("red")
    readonly property color orange: token("orange")
    readonly property color yellow: token("yellow")
    readonly property color green: token("green")
    readonly property color cyan: token("cyan")
    readonly property color blue: token("blue")
    readonly property color purple: token("purple")
    readonly property color brown: token("brown")
    readonly property color scrim: Qt.rgba(bg.r, bg.g, bg.b, 0.8)
    readonly property string sourceLabel: Theme.sourceLabel

    // Status and fraction colours: fixed mappings onto the hues
    readonly property color statusWatching: accent
    readonly property color statusCompleted: blue
    readonly property color statusPaused: yellow
    readonly property color statusDropped: red
    readonly property color statusPlanning: textFaint
    readonly property color statusRewatching: purple
    readonly property color behind: yellow
    readonly property color caughtUp: accent
    // A hue by the name Theme.formatHue or Theme.statusHue returns
    function hue(name) { return token(name) }

    // Spacing, radii, type, motion
    readonly property real densityFactor: Theme.densityFactor
    function space(n) { return Math.round(4 * densityFactor * n) }
    readonly property real cornerBase: 14
    readonly property real cornerSmoothing: Theme.smoothing
    readonly property real radiusSm: cornerBase * densityFactor
    readonly property real radiusMd: cornerBase * 1.4 * densityFactor
    readonly property real radiusLg: cornerBase * 1.4 * 1.4 * densityFactor
    readonly property real radiusXl: cornerBase * 1.4 * 1.4 * 1.4 * densityFactor
    readonly property int posterWidth: Theme.posterWidth
    readonly property real systemPointSize: Qt.application.font.pointSize > 0 ? Qt.application.font.pointSize : 10
    readonly property real typeNormal: systemPointSize
    readonly property real typeSmall: systemPointSize * 0.85
    readonly property real typeLarge: systemPointSize * 1.4
    readonly property string fontSans: Qt.application.font.family
    readonly property string fontMono: "monospace"
    readonly property int motionFast: 120
    readonly property int motionNormal: 200
    readonly property int motionSlow: 320
    readonly property real controlHeight: space(8)
    readonly property real disabledOpacity: 0.45

    // A colour t of the way from a to b, alpha included; takes Qt colours
    function tone(a, b, t) {
        t = Math.max(0, Math.min(1, t))
        return Qt.rgba(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
    }
}
```

The primitives. Copy these twelve files from `spikes/home-grid-qml/qml/` unchanged: `Corner.qml`, `Chip.qml`, `Seg.qml`, `Switch.qml`, `Button.qml`, `Field.qml`, `Dropdown.qml`, `Swatches.qml`, `SliderRow.qml`, `SettingRow.qml`, `Panel.qml`. Copy `Icon.qml` and change its one path line to:

```qml
    source: glyph !== "" ? "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/icons/" + glyph + ".svg" : ""
```

Every one of them refers to `theme` through the context chain, which `Main.qml` now provides. Nothing else in them changes.

The icons. `apps/linux/scripts/icons.sh` copies the prototype's set and fetches the rest from Lucide at a pinned tag, with the stroke set to black as the prototype's are (QtSvg does not read `currentColor`):

```bash
#!/usr/bin/env bash
# Fetch every Lucide glyph the shell names into assets/icons/, stroke set to black so QtSvg
# reads it and ColorImage tints it. Re-run to add a name; then list it in build.rs.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
tag="1.41.0"
out="$here/assets/icons"; mkdir -p "$out"
cp -n "$here/../../spikes/home-grid-qml/assets/icons/"*.svg "$out"/ 2>/dev/null || true
cp "$here/../../spikes/home-grid-qml/assets/icons/LICENSE" "$out/LICENSE"
names=(search arrow-left arrow-right arrow-down arrow-up chevron-down chevron-up chevron-left chevron-right chevrons-right
  external-link link star eye-off plus pause skip-back skip-forward volume-2 volume-x maximize minimize rotate-ccw rotate-cw
  audio-lines languages clock film tv layers list-filter keyboard scan ban circle-check circle-x clapperboard image
  sliders-horizontal triangle-alert square-check square calendar-clock book-open users case-sensitive chart-pie check-check
  badge-check bell circle-question-mark step-back step-forward file-down file-up circle-play list-video bookmark sparkles
  radio folder-search check pencil trash)
for n in "${names[@]}"; do
  [ -f "$out/$n.svg" ] && continue
  curl -fsSL "https://raw.githubusercontent.com/lucide-icons/lucide/$tag/icons/$n.svg" \
    | sed 's/stroke="currentColor"/stroke="#000"/' > "$out/$n.svg"
done
ls "$out"/*.svg | wc -l
```

Run it, then list every SVG in `build.rs`'s `.qrc_resources([...])` (the prototype's 36 plus the names above; a resource left out of the list is a blank icon at runtime, not a build error, so the list is generated: `ls apps/linux/assets/icons/*.svg | sed 's|apps/linux/||;s|.*|        "&",|'` and paste).

`build.rs` also gains, in the QML module, one `.qml_file("qml/<Name>.qml")` line per new file: `Tokens`, `TokensPage`, `Corner`, `Icon`, `Chip`, `Seg`, `Switch`, `Button`, `Field`, `Dropdown`, `Swatches`, `SliderRow`, `SettingRow`, `Panel`; and `"src/bridge/theme.rs"` in `.files`. `bridge/mod.rs` gains `pub mod theme;`.

`apps/linux/qml/TokensPage.qml`, a dev page behind `--page tokens` that lays the whole token set and every primitive out for a capture:

```qml
// Every token and every primitive on one page, for a capture. Reached with --page tokens.
import QtQuick

Flickable {
    id: root
    contentHeight: column.implicitHeight + theme.space(8)
    clip: true
    Column {
        id: column
        x: theme.space(8); y: theme.space(7)
        width: parent.width - theme.space(16)
        spacing: theme.space(4)
        Text { text: "Tokens"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold }
        Text { text: theme.sourceLabel + ", " + theme.mode; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
        Flow {
            width: parent.width
            spacing: theme.space(2)
            Repeater {
                model: ["bg", "surface", "surface_raised", "surface_sunken", "surface_pressed", "line", "line_strong", "text", "text_dim", "text_faint",
                        "accent", "accent_text", "accent_soft", "red_soft", "focus", "red", "orange", "yellow", "green", "cyan", "blue", "purple", "brown"]
                Column {
                    required property string modelData
                    spacing: theme.space(1)
                    Corner { width: theme.space(16); height: theme.space(10); radius: theme.radiusSm; smoothing: theme.cornerSmoothing; color: theme.token(modelData); borderColor: theme.line; borderWidth: 1 }
                    Text { text: modelData; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
                }
            }
        }
        Row {
            spacing: theme.space(3)
            Chip { text: "EP 12" }
            Chip { text: "12/24"; textColor: theme.behind }
            Chip { text: "Selected"; selected: true; mono: false }
            Chip { text: "2 errors"; icon: "circle-alert"; small: true; color: theme.redSoft; textColor: theme.red }
            Seg { options: ["All", "Series", "Movies"]; index: 1 }
            Switch { checked: true }
            Button { text: "Button"; icon: "check" }
            Button { text: "Remove"; icon: "trash-2"; danger: true }
            Button { text: "Flat"; flat: true }
        }
        Row {
            spacing: theme.space(3)
            Field { placeholder: "A field" }
            Dropdown { options: ["AniBeam Dark", "Catppuccin Mocha"]; index: 0 }
            Swatches { slot: 4 }
            SliderRow { from: 0; to: 150; value: 100 }
        }
        Row {
            spacing: theme.space(3)
            Repeater {
                model: [theme.radiusSm, theme.radiusMd, theme.radiusLg, theme.radiusXl]
                Corner { required property real modelData; width: theme.space(24); height: theme.space(16); radius: modelData; smoothing: theme.cornerSmoothing; color: theme.surface; borderColor: theme.lineStrong; borderWidth: 1 }
            }
        }
        Column {
            spacing: theme.space(1)
            Text { text: "Large " + theme.typeLarge.toFixed(1); color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold }
            Text { text: "Normal " + theme.typeNormal.toFixed(1); color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
            Text { text: "Small, dim " + theme.typeSmall.toFixed(1); color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
            Text { text: "0123456789 in the fixed face"; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeNormal }
        }
    }
}
```

`Main.qml` changes: `import com.marcusrosado.AniBeam` already present; add `Tokens { id: theme }` as the first child; bind `color: theme.bg`; replace the placeholder `Text` with a `Loader` that shows `TokensPage` when `Shell.page === "tokens"` (Task 7 turns this into the `Frame`):

```qml
    Tokens { id: theme }
    color: theme.bg
    font.family: theme.fontSans
    font.pointSize: theme.typeNormal

    Loader {
        anchors.fill: parent
        active: window.settled && Theme.ready
        sourceComponent: Shell.page === "tokens" ? tokensPage : placeholder
    }
    Component { id: tokensPage; TokensPage {} }
    Component { id: placeholder; Text { anchors.centerIn: parent; text: "AniBeam " + Shell.version; color: theme.text } }
```

and the shoot timer waits for `Theme.ready` as well: `onSettledChanged` becomes a function `maybeShoot()` called from both `onSettledChanged` and a `Connections { target: Theme; function onReadyChanged() { window.maybeShoot() } }`, starting the timer when `settled && Theme.ready && Shell.shoot !== ""`.

- [ ] **Step 4: Run the tests, build, capture the token sheet three ways**

Run: `cargo test -p anibeam theme::`
Expected: every theme test passes, the engine and portal ones included.

Run: `apps/linux/scripts/shoot.sh tokens-terminal --page tokens`
Expected: the sheet under the terminal palette (the owner's kitty chain), `sourceLabel` reading `terminal kitty`, the swatches showing the kitty colours, every primitive drawn with G2 corners.

Run: `printf 'mode = "light"\n' > apps/linux/captures/root/config/theme.toml && apps/linux/scripts/shoot.sh tokens-light --page tokens`
Expected: the forced light ground `#f6f7fa` tinted toward the accent, hues re-toned darker.

Run: `printf 'source = "theme"\n[theme]\ndark = "catppuccin-mocha"\n' > apps/linux/captures/root/config/theme.toml && apps/linux/scripts/shoot.sh tokens-mocha --page tokens`
Expected: Mocha's `#1e1e2e` ground and `#89b4fa` accent; the source label `theme Catppuccin Mocha`.

Run: `apps/linux/scripts/bench.sh tokens-live 2 keep --page tokens`, then edit `~/.config/kitty/kitty.conf`'s included theme file (or `~/.config/anibeam/theme.toml`) and watch the window
Expected: the window recolours within a second with no restart. `pkill -x anibeam` afterwards.

- [ ] **Step 5: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the theme engine, the portal, the Theme singleton and the primitives"
```

---
### Task 6: The door: calls in, events out, and the list model

The `Door` singleton is the shell's only way into the core: one invokable per call variant, one Qt signal per event body, JSON objects for anything deep, and the hop from the core's tokio threads to the Qt thread through `CxxQtThread::queue`. `RecordModel` is the one Rust `QAbstractListModel` every list sits on. The JSON layer is pure and tested against a real core on a temp directory.

**Files:**
- Create: `apps/linux/src/json.rs`
- Create: `apps/linux/src/bridge/door.rs`, `apps/linux/src/bridge/model.rs`
- Modify: `apps/linux/src/main.rs` (open the core, shut it down), `apps/linux/src/bridge/mod.rs`, `apps/linux/build.rs`

**Interfaces:**
- Consumes: `anibeam_core::{Core, CorePaths, Call, Reply, Event, EventBody, CoreError, EventListener, Subscription, JobPhase, Level, ...}`, `runtime::core()`.
- Produces:
  - `json::call_from(name: &str, args: Value) -> Result<Call, String>`; `json::reply_json(r: Result<Reply, CoreError>) -> Value` (`{ "kind", "reply" }` or `{ "error": { "kind", ...fields, "message" } }`); `json::event_json(e: &Event) -> Value` (`{ seq, at, level, stage, message, job: { id, kind, phase } | null, kind, body }`); `json::flatten_times(&mut Value)` (every `{ secs_since_epoch, nanos_since_epoch }` becomes a number of seconds); `json::to_qjson(&Value) -> QJsonValue`, `json::to_qjson_object(&Value) -> QJsonObject`, `json::to_qjson_array(&Value) -> QJsonArray`, `json::from_qjson(&QJsonValue) -> Value`, `json::from_qjson_object(&QJsonObject) -> Value`; `json::dispatch(core: &Core, call: Call) -> Value`.
  - The QML singleton `Door`: the invokables in the table below, the signals in the table after it, and the properties `ready`, `revealHidden` (read and write, session-only), `preferences`, `settings`, `trackers`, `about` (JSON objects, kept current from events), `runningJobs` (JSON array of `{ id, kind, done, total, label }`), `latestLine` (the newest Info-or-above event's envelope), `unseenErrors` (int), with `markLogSeen()` zeroing it.
  - The QML element `RecordModel` with `roles` (a string list, dotted paths allowed), `idKey` (default `id`, dotted paths allowed), `count`, and `reset(array)`, `upsert(object)`, `upsertAll(array)`, `remove(id)`, `removeAll(ids)`, `patch(id, fields)`, `at(row) -> object`, `indexOf(id) -> row`.

Every invokable returns a JSON object: `{ kind, reply }` on success, `{ error }` on failure. Ids and counts are numbers; a `-1` stands for none where an argument is optional. Enum arguments are the contract's variant names (`"All"`, `"LastViewed"`, `"Asc"`, `"Anilist"`, `"Ended"`).

| Invokable | Arguments | Call |
|---|---|---|
| `listSources()` | | `ListSources` |
| `addSource(path)` | string | `AddSource { path }` |
| `removeSource(source)` | number | `RemoveSource { source }` |
| `forgetSeries(series)` | number | `ForgetSeries { series }` |
| `scan(source)` | number, -1 for all | `Scan { source }` |
| `rescanSeries(series)` | number | `RescanSeries { series }` |
| `lookup(path)` | string | `Lookup { path }` |
| `listSeries(tab, query, sort, direction, revealHidden)` | strings, string, strings, bool | `ListSeries { .. }` |
| `listAiring(offset, limit)` | numbers | `ListAiring { offset, limit }` |
| `getSeries(series)` | number | `GetSeries { series }` |
| `setHidden(series, hidden)` | number, bool | `SetHidden { series, hidden }` |
| `listFeed(sort)` | `"Recent"` or `"Upcoming"` | `ListFeed { sort }` |
| `listMetadata(filter, query, revealHidden)` | string, string, bool | `ListMetadata { .. }` |
| `listSubscriptions()` | | `ListSubscriptions` |
| `searchProvider(query, limit)` | string, number | `SearchProvider { provider: Anilist, query, limit }` |
| `resolveLink(url)` | string | `ResolveLink { url }` |
| `applyMatch(series, target)` | number, object `{"Anilist":{"id":n,"season":null}}` or `{"Mal":{"id":n}}` | `ApplyMatch { series, target }` |
| `clearMatch(series)`, `refreshSeries(series)`, `refreshAiring(series)` | number | the call of the same name |
| `refreshAll()`, `autoMatch()`, `getStorage()`, `clearImages()`, `getTrackers()`, `listWatching()`, `about()`, `getPreferences()`, `getSettings()`, `clearEvents()`, `listJobs()` | | the call of the same name |
| `setTrackerCredentials(tracker, clientId, clientSecret)` | string, string, string (empty for none) | `SetTrackerCredentials { .. }` |
| `connectTracker(tracker)`, `disconnectTracker(tracker)`, `setMainTracker(tracker)` | `"Anilist"` or `"Mal"` | the call of the same name |
| `markEpisode(series, episode)` | number, number | `MarkEpisode { series, episode }` |
| `setProgress(series, progress)` | number, number | `SetProgress { series, progress }` |
| `setScore(series, score)` | number, number (-1 clears) | `SetScore { series, score }` |
| `refreshProgress(tracker)` | string, empty for both | `RefreshProgress { tracker }` |
| `getFranchiseGraph(series)` | number | `GetFranchiseGraph { series }` |
| `openPlayback(file)` | number | `OpenPlayback { file }` |
| `reportChapters(session, chapters, duration)` | number, array of `{ title, start }`, number | `ReportChapters { .. }` |
| `tick(session, position, paused)` | number, number, bool | `Tick { .. }` |
| `closePlayback(session, position, reason)` | number, number, `"Ended"` / `"Stopped"` / `"Switched"` | `ClosePlayback { .. }` |
| `setTrackChoice(series, audio, subtitle)` | number, object or null, object or null | `SetTrackChoice { .. }` |
| `setPreferences(preferences)` | object | `SetPreferences { preferences }` |
| `setSubtitleDefaults(defaults)` | object | `SetSubtitleDefaults { defaults }` |
| `setAutoSkip(intro, outro)` | bools | `SetAutoSkip { intro, outro }` |
| `exportLibrary(path, privateData)` | string, bool | `Export { path, private }` |
| `importLibrary(path)` | string | `Import { path }` |
| `recentEvents(limit)` | number | `RecentEvents { limit }` |
| `cancelJob(job)` | number | `CancelJob { job }` |
| `call(name, args)` | string, object | any call, the CLI's door, for the odd case |

Signals, one per event body plus the envelope and one derived signal. Deep payloads are JSON.

| Signal | Arguments |
|---|---|
| `event(envelope)` | object: every event, Debug included |
| `jobFinished(job, kind, ok)` | number, string, bool: derived from `job.phase == Finished`, `ok` false for `JobFailed` and `JobCancelled` |
| `ready()`, `notice()`, `settingsChanged()`, `jobCancelled(job, kind)` | |
| `jobStarted(job, kind)` | number, string |
| `jobProgress(job, kind, done, total, label)` | number, string, number, number (-1 unknown), string |
| `jobFailed(job, kind, error)` | number, string, object |
| `sourceChanged(source)` | object |
| `sourceRemoved(source)` | number |
| `seriesChanged(cards)` | array of cards |
| `seriesRemoved(ids)` | array of numbers |
| `scanFinished(source, added, changed, removed)` | number (-1 for all), numbers |
| `subscriptionsListed(result)` | object: `{ kind: "Ok", feeds: [...] }` or `{ kind: "Missing" \| "NeedsAuth" \| "Timeout" }` |
| `searchFinished(job, results)` | number, array |
| `linkResolved(job, target)` | number, object |
| `matchApplied(series)` | number |
| `refreshFinished(refreshed, failed)` | numbers |
| `autoMatchFinished(backfilled, matched, unmatched)` | numbers |
| `airingRefreshed(series, updated)` | number, bool |
| `imagesCleared(removed)` | number |
| `trackersChanged(state)` | object |
| `authUrlReady(tracker, openUrl, redirectUrl)` | strings |
| `trackerConnected(tracker, username)` | strings |
| `marked(series, episode, outcomes)` | number, number, array |
| `progressSet(series, progress, outcomes)` | number, number, array |
| `scored(series, score, outcomes)` | number, number (-1 none), array |
| `progressRefreshed(tracker)` | string |
| `watchingRefreshed(list)` | object |
| `graphChanged(root)` | number |
| `crawlFinished(fetched, deferred)` | numbers |
| `skipWindowsReady(session, windows)` | number, array |
| `resumePointChanged(file, position)` | number, number (-1 cleared) |
| `viewed(series, episode)` | number, string |
| `preferencesChanged(preferences)` | object |
| `exportFinished(path)` | string |
| `importFinished(summary)` | object |

- [ ] **Step 1: Write the failing tests**

`apps/linux/src/json.rs`, tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_core::*;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn calls_come_from_a_name_and_a_json_object() {
        assert_eq!(call_from("ListSources", Value::Null).unwrap(), Call::ListSources);
        assert_eq!(call_from("ListSources", json!({})).unwrap(), Call::ListSources);
        let c = call_from("ListSeries", json!({"tab": "Movies", "query": "gup", "sort": "LastViewed", "direction": "Desc", "reveal_hidden": true})).unwrap();
        assert_eq!(c, Call::ListSeries { tab: Tab::Movies, query: "gup".into(), sort: Sort::LastViewed, direction: Direction::Desc, reveal_hidden: true });
        assert!(call_from("ListSeries", json!({})).is_err(), "missing fields are an error");
        assert!(call_from("Nope", Value::Null).is_err());
        let c = call_from("ApplyMatch", json!({"series": 3, "target": {"Anilist": {"id": 21, "season": null}}})).unwrap();
        assert_eq!(c, Call::ApplyMatch { series: 3, target: MatchTarget::Anilist { id: 21, season: None } });
    }

    #[test]
    fn replies_and_errors_take_one_shape() {
        let ok = reply_json(Ok(Reply::Started { job: 7 }));
        assert_eq!(ok, json!({"kind": "Started", "reply": {"job": 7}}));
        assert_eq!(reply_json(Ok(Reply::Ok)), json!({"kind": "Ok", "reply": {}}));
        let err = reply_json(Err(CoreError::NotFound { what: Entity::Series, id: 9 }));
        assert_eq!(err["error"]["kind"], "NotFound");
        assert_eq!(err["error"]["what"], "Series");
        assert_eq!(err["error"]["id"], 9);
        assert_eq!(err["error"]["message"], "Series 9 not found");
        let refused = reply_json(Err(CoreError::Refused { reason: Refusal::Hidden }));
        assert_eq!(refused["error"]["reason"], "Hidden");
    }

    #[test]
    fn times_flatten_to_seconds_everywhere() {
        let at = UNIX_EPOCH + Duration::from_secs(1_700_000_000) + Duration::from_millis(250);
        let mut v = serde_json::to_value(vec![Some(at), None]).unwrap();
        flatten_times(&mut v);
        assert_eq!(v, json!([1700000000.25, null]));
        let mut nested = json!({"a": {"secs_since_epoch": 5, "nanos_since_epoch": 0, "extra": 1}});
        flatten_times(&mut nested);
        assert_eq!(nested["a"]["secs_since_epoch"], 5, "an object with more keys is not a time");
    }

    #[test]
    fn an_event_carries_its_kind_flat() {
        let e = Event {
            seq: 4,
            at: UNIX_EPOCH + Duration::from_secs(10),
            level: Level::Info,
            stage: Stage::Library,
            message: "scan finished: 1 added".into(),
            job: Some(JobRef { id: 2, kind: JobKind::Scan, phase: JobPhase::Finished }),
            body: EventBody::ScanFinished { source: None, added: 1, changed: 0, removed: 0 },
        };
        let v = event_json(&e);
        assert_eq!(v["kind"], "ScanFinished");
        assert_eq!(v["body"]["added"], 1);
        assert_eq!(v["job"]["kind"], "Scan");
        assert_eq!(v["job"]["phase"], "Finished");
        assert_eq!(v["at"], 10.0);
        assert_eq!(v["level"], "Info");
        let unit = Event { body: EventBody::Ready, job: None, ..e };
        let v = event_json(&unit);
        assert_eq!(v["kind"], "Ready");
        assert_eq!(v["body"], json!({}));
        assert_eq!(v["job"], Value::Null);
    }

    #[test]
    fn dispatch_answers_from_a_real_core() {
        let dir = tempfile::tempdir().unwrap();
        let paths = CorePaths::under(dir.path());
        let secrets = anibeam_core::trackers::Secrets::file_only(paths.secrets_path());
        let core: Arc<Core> = Core::open_with_secrets(paths, secrets).unwrap();
        let about = dispatch(&core, call_from("About", Value::Null).unwrap());
        assert_eq!(about["kind"], "About");
        assert_eq!(about["reply"]["about"]["version"], anibeam_core::VERSION);
        let sources = dispatch(&core, Call::ListSources);
        assert_eq!(sources, json!({"kind": "Sources", "reply": {"sources": []}}));
        let missing = dispatch(&core, Call::GetSeries { series: 1 });
        assert_eq!(missing["error"]["kind"], "NotFound");
        core.shutdown();
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p anibeam json`
Expected: compile errors.

- [ ] **Step 3: Write the JSON layer, the door and the model**

`apps/linux/src/json.rs` (above its tests):

```rust
//! The shapes that cross the bridge as JSON. The core's enums are externally tagged
//! (`{"ListSeries": {...}}`, `"ListSources"`); QML sees one flat shape per direction.

use anibeam_core::{Call, Core, CoreError, Event, Reply};
use cxx_qt_lib::{QJsonArray, QJsonObject, QJsonValue, QString};
use serde_json::{json, Map, Value};

pub fn call_from(name: &str, args: Value) -> Result<Call, String> {
    let empty = args.is_null() || args.as_object().is_some_and(Map::is_empty);
    let wrapped = if empty { Value::String(name.to_string()) } else { json!({ name: args }) };
    serde_json::from_value(wrapped).map_err(|e| e.to_string())
}

/// `{secs_since_epoch, nanos_since_epoch}`, serde's SystemTime, becomes seconds.
pub fn flatten_times(v: &mut Value) {
    match v {
        Value::Object(o) => {
            if o.len() == 2 {
                if let (Some(s), Some(n)) = (o.get("secs_since_epoch").and_then(Value::as_f64), o.get("nanos_since_epoch").and_then(Value::as_f64)) {
                    *v = Value::from(s + n / 1e9);
                    return;
                }
            }
            for (_, child) in o.iter_mut() {
                flatten_times(child);
            }
        }
        Value::Array(a) => a.iter_mut().for_each(flatten_times),
        _ => {}
    }
}

/// An externally tagged enum value as (variant name, fields object).
fn split_tag(v: Value) -> (String, Value) {
    match v {
        Value::String(kind) => (kind, json!({})),
        Value::Object(o) if o.len() == 1 => {
            let (kind, inner) = o.into_iter().next().expect("one entry");
            (kind, if inner.is_null() { json!({}) } else { inner })
        }
        other => ("Unknown".to_string(), other),
    }
}

pub fn reply_json(r: Result<Reply, CoreError>) -> Value {
    match r {
        Ok(reply) => {
            let mut v = serde_json::to_value(&reply).unwrap_or(Value::Null);
            flatten_times(&mut v);
            let (kind, inner) = split_tag(v);
            json!({ "kind": kind, "reply": inner })
        }
        Err(e) => json!({ "error": error_json(&e) }),
    }
}

pub fn error_json(e: &CoreError) -> Value {
    let (kind, fields) = split_tag(serde_json::to_value(e).unwrap_or(Value::Null));
    let mut o = match fields { Value::Object(o) => o, _ => Map::new() };
    o.insert("kind".into(), Value::String(kind));
    o.insert("message".into(), Value::String(e.to_string()));
    Value::Object(o)
}

pub fn event_json(e: &Event) -> Value {
    let mut body = serde_json::to_value(&e.body).unwrap_or(Value::Null);
    flatten_times(&mut body);
    let (kind, fields) = split_tag(body);
    let mut at = serde_json::to_value(e.at).unwrap_or(Value::Null);
    flatten_times(&mut at);
    json!({
        "seq": e.seq,
        "at": at,
        "level": serde_json::to_value(e.level).unwrap_or(Value::Null),
        "stage": serde_json::to_value(e.stage).unwrap_or(Value::Null),
        "message": e.message,
        "job": e.job.as_ref().map(|j| json!({ "id": j.id, "kind": serde_json::to_value(j.kind).unwrap_or(Value::Null), "phase": serde_json::to_value(j.phase).unwrap_or(Value::Null) })),
        "kind": kind,
        "body": fields,
    })
}

pub fn dispatch(core: &Core, call: Call) -> Value {
    reply_json(core.call(call))
}

pub fn to_qjson(v: &Value) -> QJsonValue {
    match v {
        Value::Null => QJsonValue::default(),
        Value::Bool(b) => QJsonValue::from(*b),
        Value::Number(n) => match n.as_i64() {
            Some(i) => QJsonValue::from(i),
            None => QJsonValue::from(n.as_f64().unwrap_or(0.0)),
        },
        Value::String(s) => QJsonValue::from(&QString::from(s)),
        Value::Array(_) => QJsonValue::from(&to_qjson_array(v)),
        Value::Object(_) => QJsonValue::from(&to_qjson_object(v)),
    }
}

pub fn to_qjson_array(v: &Value) -> QJsonArray {
    let mut arr = QJsonArray::default();
    if let Value::Array(items) = v {
        for x in items {
            arr.append(&to_qjson(x));
        }
    }
    arr
}

/// A non-object is wrapped as `{ "value": v }`.
pub fn to_qjson_object(v: &Value) -> QJsonObject {
    let mut o = QJsonObject::default();
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                o.insert(&QString::from(k), &to_qjson(x));
            }
        }
        other => o.insert(&QString::from("value"), &to_qjson(other)),
    }
    o
}

pub fn from_qjson(v: &QJsonValue) -> Value {
    if v.is_bool() {
        Value::Bool(v.to_bool())
    } else if v.is_double() {
        let d = v.to_double();
        if d.fract() == 0.0 && d.abs() < 9.0e15 { Value::from(d as i64) } else { Value::from(d) }
    } else if v.is_string() {
        Value::String(v.to_string().to_string())
    } else if v.is_array() {
        Value::Array(v.to_array().iter().map(|x| from_qjson(&x)).collect())
    } else if v.is_object() {
        from_qjson_object(&v.to_object())
    } else {
        Value::Null
    }
}

pub fn from_qjson_object(o: &QJsonObject) -> Value {
    let mut m = Map::new();
    for key in o.keys().iter() {
        m.insert(key.to_string(), from_qjson(&o.value(key)));
    }
    Value::Object(m)
}
```

`apps/linux/src/bridge/door.rs`. The bridge declares every invokable and signal; the Rust half builds `Call` values, dispatches, and forwards events. The listener runs on the core's threads and queues one closure per event.

```rust
//! Door: the shell's one way into the core. One invokable per call, one signal per event
//! body, JSON for anything deep. The listener hops every event to the Qt thread through
//! CxxQtThread::queue, and the state QML shares (preferences, settings, trackers, running
//! jobs, the latest line, unseen errors) is kept current here from those events.

use core::pin::Pin;
use std::sync::Arc;

use anibeam_core::{
    Call, CloseReason, CoreError, Direction, Event, EventBody, EventListener, FeedSort, JobPhase, Level, MatchTarget,
    MetadataFilter, Preferences, Provider, Sort, SubscriptionsResult, SubtitleChoice, SubtitleDefaults, Tab,
    TrackRef, Tracker, Chapter,
};
use anibeam_core::events::Subscription;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QJsonArray, QJsonObject, QString};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::json::{self, from_qjson_object, to_qjson_array, to_qjson_object};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qjsonobject.h");
        type QJsonObject = cxx_qt_lib::QJsonObject;
        include!("cxx-qt-lib/qjsonarray.h");
        type QJsonArray = cxx_qt_lib::QJsonArray;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(bool, ready)]
        #[qproperty(bool, reveal_hidden)]
        #[qproperty(QJsonObject, preferences)]
        #[qproperty(QJsonObject, settings)]
        #[qproperty(QJsonObject, trackers)]
        #[qproperty(QJsonObject, about)]
        #[qproperty(QJsonArray, running_jobs)]
        #[qproperty(QJsonObject, latest_line)]
        #[qproperty(i32, unseen_errors)]
        type Door = super::DoorRust;

        // Library
        #[qinvokable] fn list_sources(self: &Door) -> QJsonObject;
        #[qinvokable] fn add_source(self: &Door, path: &QString) -> QJsonObject;
        #[qinvokable] fn remove_source(self: &Door, source: f64) -> QJsonObject;
        #[qinvokable] fn forget_series(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable] fn scan(self: &Door, source: f64) -> QJsonObject;
        #[qinvokable] fn rescan_series(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable] fn lookup(self: &Door, path: &QString) -> QJsonObject;
        #[qinvokable] fn list_series(self: &Door, tab: &QString, query: &QString, sort: &QString, direction: &QString, reveal_hidden: bool) -> QJsonObject;
        #[qinvokable] fn list_airing(self: &Door, offset: f64, limit: f64) -> QJsonObject;
        #[qinvokable] fn get_series(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable] fn set_hidden(self: &Door, series: f64, hidden: bool) -> QJsonObject;
        #[qinvokable] fn list_feed(self: &Door, sort: &QString) -> QJsonObject;
        #[qinvokable] fn list_metadata(self: &Door, filter: &QString, query: &QString, reveal_hidden: bool) -> QJsonObject;
        #[qinvokable] fn list_subscriptions(self: &Door) -> QJsonObject;
        // Metadata
        #[qinvokable] fn search_provider(self: &Door, query: &QString, limit: i32) -> QJsonObject;
        #[qinvokable] fn resolve_link(self: &Door, url: &QString) -> QJsonObject;
        #[qinvokable] fn apply_match(self: &Door, series: f64, target: &QJsonObject) -> QJsonObject;
        #[qinvokable] fn clear_match(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable] fn refresh_series(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable] fn refresh_all(self: &Door) -> QJsonObject;
        #[qinvokable] fn auto_match(self: &Door) -> QJsonObject;
        #[qinvokable] fn refresh_airing(self: &Door, series: f64) -> QJsonObject;
        #[qinvokable] fn get_storage(self: &Door) -> QJsonObject;
        #[qinvokable] fn clear_images(self: &Door) -> QJsonObject;
        // Trackers
        #[qinvokable] fn get_trackers(self: &Door) -> QJsonObject;
        #[qinvokable] fn set_tracker_credentials(self: &Door, tracker: &QString, client_id: &QString, client_secret: &QString) -> QJsonObject;
        #[qinvokable] fn connect_tracker(self: &Door, tracker: &QString) -> QJsonObject;
        #[qinvokable] fn disconnect_tracker(self: &Door, tracker: &QString) -> QJsonObject;
        #[qinvokable] fn set_main_tracker(self: &Door, tracker: &QString) -> QJsonObject;
        #[qinvokable] fn mark_episode(self: &Door, series: f64, episode: f64) -> QJsonObject;
        #[qinvokable] fn set_progress(self: &Door, series: f64, progress: i32) -> QJsonObject;
        #[qinvokable] fn set_score(self: &Door, series: f64, score: f64) -> QJsonObject;
        #[qinvokable] fn refresh_progress(self: &Door, tracker: &QString) -> QJsonObject;
        #[qinvokable] fn list_watching(self: &Door) -> QJsonObject;
        // Franchise
        #[qinvokable] fn get_franchise_graph(self: &Door, series: f64) -> QJsonObject;
        // Playback
        #[qinvokable] fn open_playback(self: &Door, file: f64) -> QJsonObject;
        #[qinvokable] fn report_chapters(self: &Door, session: f64, chapters: &QJsonArray, duration: f64) -> QJsonObject;
        #[qinvokable] fn tick(self: &Door, session: f64, position: f64, paused: bool) -> QJsonObject;
        #[qinvokable] fn close_playback(self: &Door, session: f64, position: f64, reason: &QString) -> QJsonObject;
        #[qinvokable] fn set_track_choice(self: &Door, series: f64, audio: &QJsonObject, subtitle: &QJsonObject) -> QJsonObject;
        // Store
        #[qinvokable] fn about(self: &Door) -> QJsonObject;
        #[qinvokable] fn get_preferences(self: &Door) -> QJsonObject;
        #[qinvokable] fn set_preferences(self: &Door, preferences: &QJsonObject) -> QJsonObject;
        #[qinvokable] fn get_settings(self: &Door) -> QJsonObject;
        #[qinvokable] fn set_subtitle_defaults(self: &Door, defaults: &QJsonObject) -> QJsonObject;
        #[qinvokable] fn set_auto_skip(self: &Door, intro: bool, outro: bool) -> QJsonObject;
        #[qinvokable] fn export_library(self: &Door, path: &QString, private_data: bool) -> QJsonObject;
        #[qinvokable] fn import_library(self: &Door, path: &QString) -> QJsonObject;
        #[qinvokable] fn recent_events(self: &Door, limit: f64) -> QJsonObject;
        #[qinvokable] fn clear_events(self: &Door) -> QJsonObject;
        #[qinvokable] fn list_jobs(self: &Door) -> QJsonObject;
        #[qinvokable] fn cancel_job(self: &Door, job: f64) -> QJsonObject;
        /// Any call by name, the CLI's door.
        #[qinvokable] fn call(self: &Door, name: &QString, args: &QJsonObject) -> QJsonObject;
        #[qinvokable] fn mark_log_seen(self: Pin<&mut Door>);

        // The envelope and the derived job signal
        #[qsignal] fn event(self: Pin<&mut Door>, envelope: QJsonObject);
        #[qsignal] fn job_finished(self: Pin<&mut Door>, job: f64, kind: QString, ok: bool);
        // One per event body
        #[qsignal] fn ready(self: Pin<&mut Door>);
        #[qsignal] fn notice(self: Pin<&mut Door>);
        #[qsignal] fn job_started(self: Pin<&mut Door>, job: f64, kind: QString);
        #[qsignal] fn job_progress(self: Pin<&mut Door>, job: f64, kind: QString, done: f64, total: f64, label: QString);
        #[qsignal] fn job_failed(self: Pin<&mut Door>, job: f64, kind: QString, error: QJsonObject);
        #[qsignal] fn job_cancelled(self: Pin<&mut Door>, job: f64, kind: QString);
        #[qsignal] fn source_changed(self: Pin<&mut Door>, source: QJsonObject);
        #[qsignal] fn source_removed(self: Pin<&mut Door>, source: f64);
        #[qsignal] fn series_changed(self: Pin<&mut Door>, cards: QJsonArray);
        #[qsignal] fn series_removed(self: Pin<&mut Door>, ids: QJsonArray);
        #[qsignal] fn scan_finished(self: Pin<&mut Door>, source: f64, added: f64, changed: f64, removed: f64);
        #[qsignal] fn subscriptions_listed(self: Pin<&mut Door>, result: QJsonObject);
        #[qsignal] fn search_finished(self: Pin<&mut Door>, job: f64, results: QJsonArray);
        #[qsignal] fn link_resolved(self: Pin<&mut Door>, job: f64, target: QJsonObject);
        #[qsignal] fn match_applied(self: Pin<&mut Door>, series: f64);
        #[qsignal] fn refresh_finished(self: Pin<&mut Door>, refreshed: f64, failed: f64);
        #[qsignal] fn auto_match_finished(self: Pin<&mut Door>, backfilled: f64, matched: f64, unmatched: f64);
        #[qsignal] fn airing_refreshed(self: Pin<&mut Door>, series: f64, updated: bool);
        #[qsignal] fn images_cleared(self: Pin<&mut Door>, removed: f64);
        #[qsignal] fn trackers_changed(self: Pin<&mut Door>, state: QJsonObject);
        #[qsignal] fn auth_url_ready(self: Pin<&mut Door>, tracker: QString, open_url: QString, redirect_url: QString);
        #[qsignal] fn tracker_connected(self: Pin<&mut Door>, tracker: QString, username: QString);
        #[qsignal] fn marked(self: Pin<&mut Door>, series: f64, episode: i32, outcomes: QJsonArray);
        #[qsignal] fn progress_set(self: Pin<&mut Door>, series: f64, progress: i32, outcomes: QJsonArray);
        #[qsignal] fn scored(self: Pin<&mut Door>, series: f64, score: f64, outcomes: QJsonArray);
        #[qsignal] fn progress_refreshed(self: Pin<&mut Door>, tracker: QString);
        #[qsignal] fn watching_refreshed(self: Pin<&mut Door>, list: QJsonObject);
        #[qsignal] fn graph_changed(self: Pin<&mut Door>, root: f64);
        #[qsignal] fn crawl_finished(self: Pin<&mut Door>, fetched: f64, deferred: f64);
        #[qsignal] fn skip_windows_ready(self: Pin<&mut Door>, session: f64, windows: QJsonArray);
        #[qsignal] fn resume_point_changed(self: Pin<&mut Door>, file: f64, position: f64);
        #[qsignal] fn viewed(self: Pin<&mut Door>, series: f64, episode: QString);
        #[qsignal] fn preferences_changed(self: Pin<&mut Door>, preferences: QJsonObject);
        #[qsignal] fn settings_changed(self: Pin<&mut Door>);
        #[qsignal] fn export_finished(self: Pin<&mut Door>, path: QString);
        #[qsignal] fn import_finished(self: Pin<&mut Door>, summary: QJsonObject);
    }

    impl cxx_qt::Threading for Door {}
    impl cxx_qt::Initialize for Door {}
}

#[derive(Default)]
pub struct DoorRust {
    ready: bool,
    reveal_hidden: bool,
    preferences: QJsonObject,
    settings: QJsonObject,
    trackers: QJsonObject,
    about: QJsonObject,
    running_jobs: QJsonArray,
    latest_line: QJsonObject,
    unseen_errors: i32,
    subscription: Option<Arc<Subscription>>,
    jobs: Vec<Value>,
}

/// Runs on the core's threads; every event becomes one queued closure on the Qt thread.
struct Forwarder {
    qt: cxx_qt::CxxQtThread<qobject::Door>,
}

impl EventListener for Forwarder {
    fn on_event(&self, event: Event) {
        let envelope = json::event_json(&event);
        self.qt.queue(move |door: Pin<&mut qobject::Door>| door.receive(event, envelope)).ok();
    }
}

impl cxx_qt::Initialize for qobject::Door {
    fn initialize(mut self: Pin<&mut Self>) {
        let core = crate::runtime::core();
        // Subscribed before start, so the Ready line and a fast job's first events are seen.
        let sub = core.subscribe(Arc::new(Forwarder { qt: self.qt_thread() }));
        self.as_mut().rust_mut().subscription = Some(sub);
        if let Err(e) = core.start() {
            eprintln!("anibeam: core start: {e}");
        }
        self.as_mut().refresh_shared();
        self.as_mut().set_ready(true);
    }
}

fn id(v: f64) -> u64 {
    if v.is_finite() && v >= 0.0 { v as u64 } else { 0 }
}

fn opt_id(v: f64) -> Option<u64> {
    if v.is_finite() && v >= 0.0 { Some(v as u64) } else { None }
}

fn parse_enum<T: DeserializeOwned>(field: &str, s: &QString) -> Result<T, CoreError> {
    serde_json::from_value(Value::String(s.to_string())).map_err(|_| CoreError::invalid(field, format!("unknown value {s}")))
}

fn parse_object<T: DeserializeOwned>(field: &str, o: &QJsonObject) -> Result<T, CoreError> {
    serde_json::from_value(from_qjson_object(o)).map_err(|e| CoreError::invalid(field, e.to_string()))
}

fn parse_option<T: DeserializeOwned>(field: &str, o: &QJsonObject) -> Result<Option<T>, CoreError> {
    if o.is_empty() { Ok(None) } else { parse_object(field, o).map(Some) }
}

impl qobject::Door {
    fn dispatch(&self, call: Result<Call, CoreError>) -> QJsonObject {
        let v = match call {
            Ok(c) => json::dispatch(crate::runtime::core(), c),
            Err(e) => json!({ "error": json::error_json(&e) }),
        };
        to_qjson_object(&v)
    }

    fn reply_of(&self, call: Call) -> Option<Value> {
        let v = json::dispatch(crate::runtime::core(), call);
        v.get("reply").cloned()
    }

    /// Preferences, settings, trackers, about and the running jobs, read once after start
    /// and again whenever an event says one of them changed.
    fn refresh_shared(mut self: Pin<&mut Self>) {
        if let Some(p) = self.as_ref().reply_of(Call::GetPreferences) { self.as_mut().set_preferences(to_qjson_object(&p["preferences"])); }
        if let Some(s) = self.as_ref().reply_of(Call::GetSettings) { self.as_mut().set_settings(to_qjson_object(&s["settings"])); }
        if let Some(t) = self.as_ref().reply_of(Call::GetTrackers) { self.as_mut().set_trackers(to_qjson_object(&t["state"])); }
        if let Some(a) = self.as_ref().reply_of(Call::About) { self.as_mut().set_about(to_qjson_object(&a["about"])); }
        if let Some(j) = self.as_ref().reply_of(Call::ListJobs) {
            let jobs: Vec<Value> = j["jobs"].as_array().cloned().unwrap_or_default().into_iter().map(|job| {
                json!({ "id": job["id"], "kind": job["kind"], "done": job["progress"]["done"], "total": job["progress"]["total"], "label": job["progress"]["label"] })
            }).collect();
            self.as_mut().rust_mut().jobs = jobs.clone();
            self.as_mut().set_running_jobs(to_qjson_array(&Value::Array(jobs)));
        }
    }

    /// On the Qt thread: the shared state, then the envelope, then the body's own signal.
    pub fn receive(mut self: Pin<&mut Self>, event: Event, envelope: Value) {
        let job = event.job.clone();
        let finished = job.as_ref().is_some_and(|j| j.phase == JobPhase::Finished);
        {
            let mut rust = self.as_mut().rust_mut();
            if let Some(j) = &job {
                match (&event.body, finished) {
                    (EventBody::JobStarted { kind }, _) => {
                        rust.jobs.push(json!({ "id": j.id, "kind": serde_json::to_value(kind).unwrap_or(Value::Null), "done": 0, "total": null, "label": "" }));
                    }
                    (EventBody::JobProgress { done, total, label }, _) => {
                        if let Some(entry) = rust.jobs.iter_mut().find(|e| e["id"] == j.id) {
                            entry["done"] = json!(done);
                            entry["total"] = json!(total);
                            entry["label"] = json!(label);
                        }
                    }
                    (_, true) => rust.jobs.retain(|e| e["id"] != j.id),
                    _ => {}
                }
            }
        }
        let jobs = self.as_ref().jobs.clone();
        self.as_mut().set_running_jobs(to_qjson_array(&Value::Array(jobs)));
        if event.level >= Level::Info {
            self.as_mut().set_latest_line(to_qjson_object(&envelope));
        }
        if event.level == Level::Error {
            let n = *self.as_ref().unseen_errors() + 1;
            self.as_mut().set_unseen_errors(n);
        }
        self.as_mut().event(to_qjson_object(&envelope));
        if let Some(j) = &job {
            if finished {
                let ok = !matches!(event.body, EventBody::JobFailed { .. } | EventBody::JobCancelled);
                let kind = QString::from(&serde_json::to_value(j.kind).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default());
                self.as_mut().job_finished(j.id as f64, kind, ok);
            }
        }
        let job_id = job.as_ref().map(|j| j.id as f64).unwrap_or(-1.0);
        let job_kind = || QString::from(&job.as_ref().and_then(|j| serde_json::to_value(j.kind).ok()).and_then(|v| v.as_str().map(String::from)).unwrap_or_default());
        let body = &envelope["body"];
        let s = |k: &str| QString::from(body[k].as_str().unwrap_or(""));
        let n = |k: &str| body[k].as_f64().unwrap_or(-1.0);
        let arr = |k: &str| to_qjson_array(&body[k]);
        let obj = |k: &str| to_qjson_object(&body[k]);
        match &event.body {
            EventBody::Ready => self.as_mut().ready(),
            EventBody::Notice => self.as_mut().notice(),
            EventBody::JobStarted { .. } => self.as_mut().job_started(job_id, job_kind()),
            EventBody::JobProgress { .. } => self.as_mut().job_progress(job_id, job_kind(), n("done"), n("total"), s("label")),
            EventBody::JobFailed { error } => self.as_mut().job_failed(job_id, job_kind(), to_qjson_object(&json::error_json(error))),
            EventBody::JobCancelled => self.as_mut().job_cancelled(job_id, job_kind()),
            EventBody::SourceChanged { .. } => self.as_mut().source_changed(obj("source")),
            EventBody::SourceRemoved { .. } => self.as_mut().source_removed(n("source")),
            EventBody::SeriesChanged { .. } => self.as_mut().series_changed(arr("series")),
            EventBody::SeriesRemoved { .. } => self.as_mut().series_removed(arr("ids")),
            EventBody::ScanFinished { .. } => self.as_mut().scan_finished(n("source"), n("added"), n("changed"), n("removed")),
            EventBody::SubscriptionsListed { result } => {
                let flat = match result {
                    SubscriptionsResult::Ok { .. } => json!({ "kind": "Ok", "feeds": body["result"]["Ok"]["feeds"] }),
                    other => json!({ "kind": serde_json::to_value(other).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default() }),
                };
                self.as_mut().subscriptions_listed(to_qjson_object(&flat))
            }
            EventBody::SearchFinished { .. } => self.as_mut().search_finished(job_id, arr("results")),
            EventBody::LinkResolved { .. } => self.as_mut().link_resolved(job_id, obj("target")),
            EventBody::MatchApplied { .. } => self.as_mut().match_applied(n("series")),
            EventBody::RefreshFinished { .. } => self.as_mut().refresh_finished(n("refreshed"), n("failed")),
            EventBody::AutoMatchFinished { .. } => self.as_mut().auto_match_finished(n("backfilled"), n("matched"), n("unmatched")),
            EventBody::AiringRefreshed { updated, .. } => self.as_mut().airing_refreshed(n("series"), *updated),
            EventBody::ImagesCleared { .. } => self.as_mut().images_cleared(n("removed")),
            EventBody::TrackersChanged { .. } => {
                self.as_mut().set_trackers(obj("state"));
                self.as_mut().trackers_changed(obj("state"))
            }
            EventBody::AuthUrlReady { .. } => self.as_mut().auth_url_ready(s("tracker"), s("open_url"), s("redirect_url")),
            EventBody::TrackerConnected { .. } => self.as_mut().tracker_connected(s("tracker"), s("username")),
            EventBody::Marked { episode, .. } => self.as_mut().marked(n("series"), *episode as i32, arr("outcomes")),
            EventBody::ProgressSet { progress, .. } => self.as_mut().progress_set(n("series"), *progress as i32, arr("outcomes")),
            EventBody::Scored { .. } => self.as_mut().scored(n("series"), n("score"), arr("outcomes")),
            EventBody::ProgressRefreshed { .. } => self.as_mut().progress_refreshed(s("tracker")),
            EventBody::WatchingRefreshed { .. } => self.as_mut().watching_refreshed(obj("list")),
            EventBody::GraphChanged { .. } => self.as_mut().graph_changed(n("root")),
            EventBody::CrawlFinished { .. } => self.as_mut().crawl_finished(n("fetched"), n("deferred")),
            EventBody::SkipWindowsReady { .. } => self.as_mut().skip_windows_ready(n("session"), arr("windows")),
            EventBody::ResumePointChanged { .. } => self.as_mut().resume_point_changed(n("file"), n("position")),
            EventBody::Viewed { .. } => self.as_mut().viewed(n("series"), s("episode")),
            EventBody::PreferencesChanged { .. } => {
                self.as_mut().set_preferences(obj("preferences"));
                self.as_mut().preferences_changed(obj("preferences"))
            }
            EventBody::SettingsChanged => {
                if let Some(v) = self.as_ref().reply_of(Call::GetSettings) { self.as_mut().set_settings(to_qjson_object(&v["settings"])); }
                self.as_mut().settings_changed()
            }
            EventBody::ExportFinished { .. } => self.as_mut().export_finished(s("path")),
            EventBody::ImportFinished { .. } => self.as_mut().import_finished(obj("summary")),
        }
    }

    pub fn mark_log_seen(self: Pin<&mut Self>) {
        self.set_unseen_errors(0);
    }

    // Library
    pub fn list_sources(&self) -> QJsonObject { self.dispatch(Ok(Call::ListSources)) }
    pub fn add_source(&self, path: &QString) -> QJsonObject { self.dispatch(Ok(Call::AddSource { path: path.to_string() })) }
    pub fn remove_source(&self, source: f64) -> QJsonObject { self.dispatch(Ok(Call::RemoveSource { source: id(source) })) }
    pub fn forget_series(&self, series: f64) -> QJsonObject { self.dispatch(Ok(Call::ForgetSeries { series: id(series) })) }
    pub fn scan(&self, source: f64) -> QJsonObject { self.dispatch(Ok(Call::Scan { source: opt_id(source) })) }
    pub fn rescan_series(&self, series: f64) -> QJsonObject { self.dispatch(Ok(Call::RescanSeries { series: id(series) })) }
    pub fn lookup(&self, path: &QString) -> QJsonObject { self.dispatch(Ok(Call::Lookup { path: path.to_string() })) }
    pub fn list_series(&self, tab: &QString, query: &QString, sort: &QString, direction: &QString, reveal_hidden: bool) -> QJsonObject {
        self.dispatch((|| -> Result<Call, CoreError> { Ok(Call::ListSeries {
            tab: parse_enum::<Tab>("tab", tab)?, query: query.to_string(), sort: parse_enum::<Sort>("sort", sort)?,
            direction: parse_enum::<Direction>("direction", direction)?, reveal_hidden,
        }) })())
    }
    pub fn list_airing(&self, offset: f64, limit: f64) -> QJsonObject { self.dispatch(Ok(Call::ListAiring { offset: id(offset), limit: id(limit) })) }
    pub fn get_series(&self, series: f64) -> QJsonObject { self.dispatch(Ok(Call::GetSeries { series: id(series) })) }
    pub fn set_hidden(&self, series: f64, hidden: bool) -> QJsonObject { self.dispatch(Ok(Call::SetHidden { series: id(series), hidden })) }
    pub fn list_feed(&self, sort: &QString) -> QJsonObject { self.dispatch(parse_enum::<FeedSort>("sort", sort).map(|sort| Call::ListFeed { sort })) }
    pub fn list_metadata(&self, filter: &QString, query: &QString, reveal_hidden: bool) -> QJsonObject {
        self.dispatch(parse_enum::<MetadataFilter>("filter", filter).map(|filter| Call::ListMetadata { filter, query: query.to_string(), reveal_hidden }))
    }
    pub fn list_subscriptions(&self) -> QJsonObject { self.dispatch(Ok(Call::ListSubscriptions)) }
    // Metadata
    pub fn search_provider(&self, query: &QString, limit: i32) -> QJsonObject {
        self.dispatch(Ok(Call::SearchProvider { provider: Provider::Anilist, query: query.to_string(), limit: limit.max(1) as u32 }))
    }
    pub fn resolve_link(&self, url: &QString) -> QJsonObject { self.dispatch(Ok(Call::ResolveLink { url: url.to_string() })) }
    pub fn apply_match(&self, series: f64, target: &QJsonObject) -> QJsonObject {
        self.dispatch(parse_object::<MatchTarget>("target", target).map(|target| Call::ApplyMatch { series: id(series), target }))
    }
    pub fn clear_match(&self, series: f64) -> QJsonObject { self.dispatch(Ok(Call::ClearMatch { series: id(series) })) }
    pub fn refresh_series(&self, series: f64) -> QJsonObject { self.dispatch(Ok(Call::RefreshSeries { series: id(series) })) }
    pub fn refresh_all(&self) -> QJsonObject { self.dispatch(Ok(Call::RefreshAll)) }
    pub fn auto_match(&self) -> QJsonObject { self.dispatch(Ok(Call::AutoMatch)) }
    pub fn refresh_airing(&self, series: f64) -> QJsonObject { self.dispatch(Ok(Call::RefreshAiring { series: id(series) })) }
    pub fn get_storage(&self) -> QJsonObject { self.dispatch(Ok(Call::GetStorage)) }
    pub fn clear_images(&self) -> QJsonObject { self.dispatch(Ok(Call::ClearImages)) }
    // Trackers
    pub fn get_trackers(&self) -> QJsonObject { self.dispatch(Ok(Call::GetTrackers)) }
    pub fn set_tracker_credentials(&self, tracker: &QString, client_id: &QString, client_secret: &QString) -> QJsonObject {
        let secret = client_secret.to_string();
        self.dispatch(parse_enum::<Tracker>("tracker", tracker).map(|tracker| Call::SetTrackerCredentials {
            tracker, client_id: client_id.to_string(), client_secret: if secret.is_empty() { None } else { Some(secret) },
        }))
    }
    pub fn connect_tracker(&self, tracker: &QString) -> QJsonObject { self.dispatch(parse_enum::<Tracker>("tracker", tracker).map(|tracker| Call::ConnectTracker { tracker })) }
    pub fn disconnect_tracker(&self, tracker: &QString) -> QJsonObject { self.dispatch(parse_enum::<Tracker>("tracker", tracker).map(|tracker| Call::DisconnectTracker { tracker })) }
    pub fn set_main_tracker(&self, tracker: &QString) -> QJsonObject { self.dispatch(parse_enum::<Tracker>("tracker", tracker).map(|tracker| Call::SetMainTracker { tracker })) }
    pub fn mark_episode(&self, series: f64, episode: f64) -> QJsonObject { self.dispatch(Ok(Call::MarkEpisode { series: id(series), episode })) }
    pub fn set_progress(&self, series: f64, progress: i32) -> QJsonObject { self.dispatch(Ok(Call::SetProgress { series: id(series), progress: progress.max(0) as u32 })) }
    pub fn set_score(&self, series: f64, score: f64) -> QJsonObject { self.dispatch(Ok(Call::SetScore { series: id(series), score: if score < 0.0 { None } else { Some(score) } })) }
    pub fn refresh_progress(&self, tracker: &QString) -> QJsonObject {
        let call = if tracker.to_string().is_empty() { Ok(Call::RefreshProgress { tracker: None }) } else { parse_enum::<Tracker>("tracker", tracker).map(|t| Call::RefreshProgress { tracker: Some(t) }) };
        self.dispatch(call)
    }
    pub fn list_watching(&self) -> QJsonObject { self.dispatch(Ok(Call::ListWatching)) }
    // Franchise
    pub fn get_franchise_graph(&self, series: f64) -> QJsonObject { self.dispatch(Ok(Call::GetFranchiseGraph { series: id(series) })) }
    // Playback
    pub fn open_playback(&self, file: f64) -> QJsonObject { self.dispatch(Ok(Call::OpenPlayback { file: id(file) })) }
    pub fn report_chapters(&self, session: f64, chapters: &QJsonArray, duration: f64) -> QJsonObject {
        let list: Result<Vec<Chapter>, CoreError> = serde_json::from_value(Value::Array(chapters.iter().map(|c| json::from_qjson(&c)).collect()))
            .map_err(|e| CoreError::invalid("chapters", e.to_string()));
        self.dispatch(list.map(|chapters| Call::ReportChapters { session: id(session), chapters, duration }))
    }
    pub fn tick(&self, session: f64, position: f64, paused: bool) -> QJsonObject { self.dispatch(Ok(Call::Tick { session: id(session), position, paused })) }
    pub fn close_playback(&self, session: f64, position: f64, reason: &QString) -> QJsonObject {
        self.dispatch(parse_enum::<CloseReason>("reason", reason).map(|reason| Call::ClosePlayback { session: id(session), position, reason }))
    }
    pub fn set_track_choice(&self, series: f64, audio: &QJsonObject, subtitle: &QJsonObject) -> QJsonObject {
        // `{ off: true }` from QML is SubtitleChoice::Off; an empty object is none; anything
        // else is `{ Track: { track: TrackRef } }`.
        let subtitle = if subtitle.contains(&QString::from("off")) { Ok(Some(SubtitleChoice::Off)) } else { parse_option::<SubtitleChoice>("subtitle", subtitle) };
        self.dispatch((|| -> Result<Call, CoreError> { Ok(Call::SetTrackChoice {
            series: id(series), audio: parse_option::<TrackRef>("audio", audio)?, subtitle: subtitle?,
        }) })())
    }
    // Store
    pub fn about(&self) -> QJsonObject { self.dispatch(Ok(Call::About)) }
    pub fn get_preferences(&self) -> QJsonObject { self.dispatch(Ok(Call::GetPreferences)) }
    pub fn set_preferences(&self, preferences: &QJsonObject) -> QJsonObject {
        self.dispatch(parse_object::<Preferences>("preferences", preferences).map(|preferences| Call::SetPreferences { preferences }))
    }
    pub fn get_settings(&self) -> QJsonObject { self.dispatch(Ok(Call::GetSettings)) }
    pub fn set_subtitle_defaults(&self, defaults: &QJsonObject) -> QJsonObject {
        self.dispatch(parse_object::<SubtitleDefaults>("defaults", defaults).map(|defaults| Call::SetSubtitleDefaults { defaults }))
    }
    pub fn set_auto_skip(&self, intro: bool, outro: bool) -> QJsonObject { self.dispatch(Ok(Call::SetAutoSkip { intro, outro })) }
    pub fn export_library(&self, path: &QString, private_data: bool) -> QJsonObject { self.dispatch(Ok(Call::Export { path: path.to_string(), private: private_data })) }
    pub fn import_library(&self, path: &QString) -> QJsonObject { self.dispatch(Ok(Call::Import { path: path.to_string() })) }
    pub fn recent_events(&self, limit: f64) -> QJsonObject { self.dispatch(Ok(Call::RecentEvents { limit: id(limit) })) }
    pub fn clear_events(&self) -> QJsonObject { self.dispatch(Ok(Call::ClearEvents)) }
    pub fn list_jobs(&self) -> QJsonObject { self.dispatch(Ok(Call::ListJobs)) }
    pub fn cancel_job(&self, job: f64) -> QJsonObject { self.dispatch(Ok(Call::CancelJob { job: id(job) })) }
    pub fn call(&self, name: &QString, args: &QJsonObject) -> QJsonObject {
        self.dispatch(json::call_from(&name.to_string(), from_qjson_object(args)).map_err(|e| CoreError::invalid("call", e)))
    }
}
```

Names checked against the crate: `Chapter` and every contract type come through `anibeam_core::*`; `Subscription` lives at `anibeam_core::events::Subscription`; `Secrets` at `anibeam_core::trackers::Secrets`; `CoreError::invalid(field: &str, message: impl Into<String>)`. If the compiler disagrees, follow the crate, not this text.

`apps/linux/src/bridge/model.rs`:

```rust
//! RecordModel: the one QAbstractListModel every list sits on. Rows are JSON objects keyed
//! by `idKey`; `roles` names the keys a delegate reads (dotted paths reach into nested
//! objects, so a feed card's `series.title` is a role). Every begin/end pair is written
//! out in the same function with no early return between them; cxx-qt ships no guard.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QJsonArray, QJsonObject, QJsonValue, QList, QModelIndex, QString, QStringList, QVariant, QVector};

const USER_ROLE: i32 = 256;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
        include!("cxx-qt-lib/qjsonobject.h");
        type QJsonObject = cxx_qt_lib::QJsonObject;
        include!("cxx-qt-lib/qjsonarray.h");
        type QJsonArray = cxx_qt_lib::QJsonArray;
    }

    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(QStringList, roles)]
        #[qproperty(QString, id_key)]
        #[qproperty(i32, count)]
        type RecordModel = super::RecordModelRust;

        #[qinvokable]
        #[cxx_override]
        fn data(self: &RecordModel, index: &QModelIndex, role: i32) -> QVariant;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &RecordModel, parent: &QModelIndex) -> i32;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &RecordModel) -> QHash_i32_QByteArray;

        #[qinvokable] fn reset(self: Pin<&mut RecordModel>, records: &QJsonArray);
        #[qinvokable] fn upsert(self: Pin<&mut RecordModel>, record: &QJsonObject);
        #[qinvokable] fn upsert_all(self: Pin<&mut RecordModel>, records: &QJsonArray);
        #[qinvokable] fn remove(self: Pin<&mut RecordModel>, id: f64);
        #[qinvokable] fn remove_all(self: Pin<&mut RecordModel>, ids: &QJsonArray);
        #[qinvokable] fn patch(self: Pin<&mut RecordModel>, id: f64, fields: &QJsonObject);
        #[qinvokable] fn at(self: &RecordModel, row: i32) -> QJsonObject;
        #[qinvokable] fn index_of(self: &RecordModel, id: f64) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut RecordModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
    }

    extern "RustQt" {
        #[inherit] #[cxx_name = "beginInsertRows"] unsafe fn begin_insert_rows(self: Pin<&mut RecordModel>, parent: &QModelIndex, first: i32, last: i32);
        #[inherit] #[cxx_name = "endInsertRows"] unsafe fn end_insert_rows(self: Pin<&mut RecordModel>);
        #[inherit] #[cxx_name = "beginRemoveRows"] unsafe fn begin_remove_rows(self: Pin<&mut RecordModel>, parent: &QModelIndex, first: i32, last: i32);
        #[inherit] #[cxx_name = "endRemoveRows"] unsafe fn end_remove_rows(self: Pin<&mut RecordModel>);
        #[inherit] #[cxx_name = "beginResetModel"] unsafe fn begin_reset_model(self: Pin<&mut RecordModel>);
        #[inherit] #[cxx_name = "endResetModel"] unsafe fn end_reset_model(self: Pin<&mut RecordModel>);
        #[inherit] fn index(self: &RecordModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
    }

    impl cxx_qt::Initialize for RecordModel {}
}

pub struct RecordModelRust {
    roles: QStringList,
    id_key: QString,
    count: i32,
    keys: Vec<String>,
    rows: Vec<(f64, QJsonObject)>,
}

impl Default for RecordModelRust {
    fn default() -> Self {
        RecordModelRust { roles: QStringList::default(), id_key: QString::from("id"), count: 0, keys: vec![], rows: vec![] }
    }
}

/// `a.b.c` into a JSON object.
fn lookup(o: &QJsonObject, path: &str) -> QJsonValue {
    let mut current = QJsonValue::from(o);
    for part in path.split('.') {
        if !current.is_object() {
            return QJsonValue::default();
        }
        current = current.to_object().value(&QString::from(part));
    }
    current
}

fn id_of(o: &QJsonObject, key: &str) -> f64 {
    let v = lookup(o, key);
    if v.is_double() { v.to_double() } else { -1.0 }
}

fn to_variant(v: &QJsonValue) -> QVariant {
    if v.is_bool() {
        QVariant::from(&v.to_bool())
    } else if v.is_double() {
        QVariant::from(&v.to_double())
    } else if v.is_string() {
        QVariant::from(&v.to_string())
    } else if v.is_null() || v.is_undefined() {
        QVariant::default()
    } else {
        QVariant::from(v)
    }
}

impl cxx_qt::Initialize for qobject::RecordModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_roles_changed(|mut model| {
            let keys: Vec<String> = QList::<QString>::from(model.as_ref().roles()).iter().map(|s| s.to_string()).collect();
            model.as_mut().rust_mut().keys = keys;
        })
        .release();
    }
}

impl qobject::RecordModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let (Some((_, row)), Some(key)) = (self.rows.get(index.row() as usize), self.keys.get((role - USER_ROLE) as usize)) else {
            return QVariant::default();
        };
        to_variant(&lookup(row, key))
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut h = QHash::<QHashPair_i32_QByteArray>::default();
        for (i, key) in self.keys.iter().enumerate() {
            h.insert(USER_ROLE + i as i32, QByteArray::from(key.as_str()));
        }
        h
    }

    fn sync_count(mut self: Pin<&mut Self>) {
        let n = self.rows.len() as i32;
        self.as_mut().set_count(n);
    }

    pub fn reset(mut self: Pin<&mut Self>, records: &QJsonArray) {
        let key = self.id_key().to_string();
        let rows: Vec<(f64, QJsonObject)> = records.iter().filter(|v| v.is_object()).map(|v| { let o = v.to_object(); (id_of(&o, &key), o) }).collect();
        unsafe {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().rows = rows;
            self.as_mut().end_reset_model();
        }
        self.sync_count();
    }

    pub fn index_of(&self, id: f64) -> i32 {
        self.rows.iter().position(|(rid, _)| *rid == id).map(|i| i as i32).unwrap_or(-1)
    }

    pub fn at(&self, row: i32) -> QJsonObject {
        self.rows.get(row as usize).map(|(_, o)| o.clone()).unwrap_or_default()
    }

    fn touch(mut self: Pin<&mut Self>, row: i32) {
        let index = self.as_ref().index(row, 0, &QModelIndex::default());
        let roles = QVector::<i32>::default();
        self.as_mut().data_changed(&index, &index, &roles);
    }

    pub fn upsert(mut self: Pin<&mut Self>, record: &QJsonObject) {
        let key = self.id_key().to_string();
        let id = id_of(record, &key);
        let row = self.as_ref().index_of(id);
        if row >= 0 {
            self.as_mut().rust_mut().rows[row as usize].1 = record.clone();
            self.touch(row);
        } else {
            let end = self.rows.len() as i32;
            unsafe {
                self.as_mut().begin_insert_rows(&QModelIndex::default(), end, end);
                self.as_mut().rust_mut().rows.push((id, record.clone()));
                self.as_mut().end_insert_rows();
            }
            self.sync_count();
        }
    }

    pub fn upsert_all(mut self: Pin<&mut Self>, records: &QJsonArray) {
        for v in records.iter() {
            if v.is_object() {
                self.as_mut().upsert(&v.to_object());
            }
        }
    }

    pub fn remove(mut self: Pin<&mut Self>, id: f64) {
        let row = self.as_ref().index_of(id);
        if row < 0 {
            return;
        }
        unsafe {
            self.as_mut().begin_remove_rows(&QModelIndex::default(), row, row);
            self.as_mut().rust_mut().rows.remove(row as usize);
            self.as_mut().end_remove_rows();
        }
        self.sync_count();
    }

    pub fn remove_all(mut self: Pin<&mut Self>, ids: &QJsonArray) {
        for v in ids.iter() {
            if v.is_double() {
                self.as_mut().remove(v.to_double());
            }
        }
    }

    pub fn patch(mut self: Pin<&mut Self>, id: f64, fields: &QJsonObject) {
        let row = self.as_ref().index_of(id);
        if row < 0 {
            return;
        }
        {
            let mut rust = self.as_mut().rust_mut();
            let (_, record) = &mut rust.rows[row as usize];
            for key in fields.keys().iter() {
                record.insert(key, &fields.value(key));
            }
        }
        self.touch(row);
    }
}
```

`main.rs`: open the core before the application, install it, shut it down after `exec`. Between `install_paths` and the environment calls:

```rust
    let core = {
        let opened = match args.root.as_deref() {
            Some(_) => {
                let secrets = anibeam_core::trackers::Secrets::file_only(paths.core.secrets_path());
                anibeam_core::Core::open_with_secrets(paths.core.clone(), secrets)
            }
            None => anibeam_core::Core::open(paths.core.clone()),
        };
        match opened {
            Ok(c) => c,
            Err(e) => {
                eprintln!("anibeam: {e}");
                std::process::exit(2);
            }
        }
    };
    runtime::install_core(core);
```

and the tail becomes:

```rust
    let code = match app.as_mut() {
        Some(app) => app.exec(),
        None => 1,
    };
    drop(engine);
    runtime::core().shutdown();
    std::process::exit(code);
```

`mod json;` in `main.rs`; `pub mod door; pub mod model;` in `bridge/mod.rs`; `"src/bridge/door.rs", "src/bridge/model.rs"` in `build.rs`.

- [ ] **Step 4: Run the tests, then a smoke run against a sandbox with one series**

Run: `cargo test -p anibeam json && cargo build -p anibeam`
Expected: 5 tests pass, the door and the model register.

Run a smoke test of the door from QML by adding, to `TokensPage.qml`'s column for now (Task 7 removes it):

```qml
        Text { text: "core " + (Door.ready ? "ready, " + Door.about.version + ", " + Door.runningJobs.length + " jobs" : "starting"); color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
```

then `apps/linux/scripts/shoot.sh door --page tokens`
Expected: the line reads `core ready, 1.0.0.r<n>.g<hash>, 0 jobs`; `captures/door.log` shows no QML errors; the sandbox root now holds `data/anibeam.db`.

Run: `mkdir -p /tmp/lib/Anime/Frieren && touch "/tmp/lib/Anime/Frieren/Frieren - 01.mkv" && target/debug/anibeam-cli --root apps/linux/captures/root call AddSource --json '{"path":"/tmp/lib/Anime"}' --wait` (build the CLI first with `cargo build -p anibeam-cli`), then `apps/linux/scripts/shoot.sh door2 --page tokens`
Expected: the sandbox scans one series; the capture still renders; `door2.log` is clean. (The CLI writes while no shell runs, which the contract allows.)

- [ ] **Step 5: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the door into the core, one signal per event, and the record model"
```

---
### Task 7: The frame: rail, trail, pages, menu, tooltip, Escape, keys, window title, status strip

Spec 4.1 unit 1 and 4.5. The frame is what every page shares. After this task the app navigates between placeholder pages, Back works with its label and scroll, right-click offers Back everywhere outside the player, Escape closes the top thing in the spec's order, the window title follows the page, the JP / EN switch writes the core's preference, and the status strip shows the latest line and the running job.

**Files:**
- Create: `apps/linux/qml/Frame.qml`, `apps/linux/qml/Nav.qml`, `apps/linux/qml/Menu.qml`, `apps/linux/qml/Tooltip.qml`, `apps/linux/qml/InlineConfirm.qml`, `apps/linux/qml/EmptyState.qml`, `apps/linux/qml/SectionHeader.qml`, `apps/linux/qml/PagePlaceholder.qml`, `apps/linux/qml/PageScroll.qml`
- Create (copied from the prototype and edited): `apps/linux/qml/Rail.qml`, `apps/linux/qml/StatusStrip.qml`
- Modify: `apps/linux/qml/Main.qml`, `apps/linux/build.rs`, `apps/linux/qml/TokensPage.qml` (drop the smoke line)

**Interfaces:**
- Consumes: `Door`, `Theme`, `Shell`, `Fmt`, `Tokens` as `theme`.
- Produces, reachable by every page through the context chain as `frame`:
  - `frame.nav`: `open(page, props, label)`, `replace(page, props, label)`, `back()`, `current` (`{ page, props, label }`), `backLabel`, `railIndex`, `trail` (array, at most 12).
  - `frame.openMenu(x, y, items)` with `items` as `[{ text, icon, action }]` in frame coordinates, `frame.closeMenu()`.
  - `frame.showTip(item, text)`, `frame.hideTip()`.
  - `frame.escapeStack.push(kind, closer)` and `pop(closer)`, kinds `"drawer"`, `"confirm"`, `"popover"`; `frame.escape()` closes the highest kind, most recent first, then asks the page (`page.item.escape()` when defined).
  - `frame.overlay`: the `Item` menus and tips are parented to.
  - `frame.toast(text, seconds)`: a passing notice at the foot of the content (the player has its own).
  - The page contract: a page is a QML file `<Name>Page.qml` (`LibraryPage`, `SeriesPage`, `PlayerPage`, `FeedPage`, `WatchingPage`, `MetadataPage`, `SettingsPage`, `SubscriptionsPage`) with `property var props`, `property real scrollY` (read when leaving, set on return), optional `function escape() -> bool`, optional `function contextItems() -> array`, optional `property bool fullWindow` (the player), optional `property string title` (for the window and the trail; defaults to the page's label).
  - `PageScroll.qml`: a `Flickable` with the shell's scroll bar and `scrollY` alias, the base most pages sit on.

- [ ] **Step 1: Write the frame**

`apps/linux/qml/Nav.qml`:

```qml
// The navigation trail. Back goes to the page you came from, labelled with its name, and
// restores its scroll; sideways moves keep the trail; twelve entries at most. The frame
// reads the leaving page's scrollY and restores it on the way back.
import QtQuick

QtObject {
    id: nav
    property var trail: []
    property var current: ({ page: "library", props: {}, label: "Library" })
    property real pendingScroll: -1
    signal changed()

    readonly property var labels: ({ library: "Library", feed: "Feed", watching: "Watching", metadata: "Metadata", settings: "Settings",
                                     subscriptions: "Subscriptions", series: "Series", player: "Player" })
    readonly property var railPages: ["library", "feed", "watching", "metadata", "settings"]
    readonly property string backLabel: trail.length ? trail[trail.length - 1].label : "Library"
    readonly property int railIndex: {
        var i = railPages.indexOf(current.page)
        if (i >= 0) return i
        if (current.page === "subscriptions") return 4
        for (var k = 0; k < trail.length; k++) { var j = railPages.indexOf(trail[k].page); if (j >= 0) return j }
        return 0
    }

    function key(e) { return e.page + ":" + JSON.stringify(e.props || {}) }
    function labelOf(page, label) { return label || labels[page] || "Back" }

    // Descend: the page we leave joins the trail with its scroll, de-duplicated and capped.
    function open(page, props, label, leavingScroll) {
        var here = { page: current.page, props: current.props, label: current.label, scrollY: leavingScroll || 0 }
        var t = trail.filter(function(e) { return key(e) !== key(here) })
        t.push(here)
        trail = t.slice(-12)
        current = { page: page, props: props || {}, label: labelOf(page, label) }
        pendingScroll = 0
        changed()
    }
    // Sideways: the trail stays, so Back leaves the level rather than the episode.
    function replace(page, props, label) {
        current = { page: page, props: props || {}, label: labelOf(page, label) }
        pendingScroll = 0
        changed()
    }
    function back() {
        if (!trail.length) { if (current.page !== "library") replace("library", {}, "Library"); return }
        var t = trail.slice()
        var target = t.pop()
        trail = t
        current = { page: target.page, props: target.props, label: target.label }
        pendingScroll = target.scrollY || 0
        changed()
    }
    // A page changed its own label (a series title arrived)
    function relabel(label) { current = { page: current.page, props: current.props, label: label } }
}
```

`apps/linux/qml/PageScroll.qml`:

```qml
// The scrolling base most pages sit on: a Flickable with the shell's thin scroll bar, a
// focus sink so a click on empty space releases a field, and scrollY for the trail.
import QtQuick
import QtQuick.Controls.Basic as QC

Flickable {
    id: root
    property alias scrollY: root.contentY
    default property alias content: inner.data
    property real footInset: theme.space(10)
    contentWidth: width
    contentHeight: inner.implicitHeight + footInset
    clip: true
    boundsBehavior: Flickable.StopAtBounds
    QC.ScrollBar.vertical: QC.ScrollBar {
        policy: QC.ScrollBar.AsNeeded
        visible: size < 1
        contentItem: Rectangle { implicitWidth: 4; radius: 2; color: theme.lineStrong; opacity: parent.active ? 1 : 0.4 }
    }
    MouseArea { anchors.fill: parent; onPressed: function(m) { root.forceActiveFocus(); m.accepted = false } }
    Column {
        id: inner
        x: theme.space(8)
        y: theme.space(7)
        width: root.width - theme.space(16)
        spacing: theme.space(4)
    }
}
```

`apps/linux/qml/PagePlaceholder.qml` (a page that is not built yet; each page task replaces its use):

```qml
import QtQuick

PageScroll {
    id: page
    property var props: ({})
    property string title: frame.nav.current.label
    Text { text: page.title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold }
    Text { text: "Not built yet"; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
}
```

`apps/linux/qml/Menu.qml` (the frame's one menu, drawn in the overlay):

```qml
// The frame's menu: rows of text and a glyph on a raised surface. Opened by the frame at a
// point; closes on a pick, a click outside, or Escape through the escape stack.
import QtQuick

Item {
    id: root
    property var items: []
    property bool open: false
    property real originX: 0
    property real originY: 0
    visible: open
    anchors.fill: parent
    z: 1000

    function openAt(x, y, list) {
        items = list; originX = x; originY = y; open = true
        frame.escapeStack.push("popover", root)
    }
    function close() { if (!open) return; open = false; frame.escapeStack.pop(root) }

    MouseArea { anchors.fill: parent; acceptedButtons: Qt.LeftButton | Qt.RightButton; onPressed: root.close() }
    Corner {
        readonly property real margin: theme.space(2)
        x: Math.min(root.originX, root.width - width - margin)
        y: Math.min(root.originY, root.height - height - margin)
        width: column.implicitWidth + theme.space(2) * 2
        height: column.implicitHeight + theme.space(2) * 2
        radius: theme.radiusMd
        smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised
        borderColor: theme.lineStrong
        borderWidth: 1
        Column {
            id: column
            x: theme.space(2); y: theme.space(2)
            Repeater {
                model: root.items
                Corner {
                    required property var modelData
                    width: Math.max(theme.space(40), row.implicitWidth + theme.space(6))
                    height: theme.controlHeight
                    radius: theme.radiusSm
                    smoothing: theme.cornerSmoothing
                    color: m.containsMouse ? theme.surfacePressed : "transparent"
                    Row {
                        id: row
                        anchors.left: parent.left; anchors.leftMargin: theme.space(3)
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: theme.space(2)
                        Icon { visible: !!modelData.icon; glyph: modelData.icon || ""; anchors.verticalCenter: parent.verticalCenter; size: theme.space(4) }
                        Text { text: modelData.text; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; anchors.verticalCenter: parent.verticalCenter }
                    }
                    MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                        onClicked: { root.close(); if (modelData.action) modelData.action() } }
                }
            }
        }
    }
}
```

`apps/linux/qml/Tooltip.qml` (a hover affordance any item attaches; never a native `title`):

```qml
// Hover text for one item. The tip itself is drawn by the frame in its overlay after a
// 600 ms hover intent, so it is never clipped by the item's own parent.
import QtQuick

MouseArea {
    id: root
    property string text: ""
    anchors.fill: parent
    hoverEnabled: true
    acceptedButtons: Qt.NoButton
    propagateComposedEvents: true
    onEntered: if (text !== "") intent.start()
    onExited: { intent.stop(); frame.hideTip() }
    onTextChanged: if (containsMouse && text === "") frame.hideTip()
    Timer { id: intent; interval: 600; onTriggered: frame.showTip(root, root.text) }
}
```

`apps/linux/qml/InlineConfirm.qml` (the guard on a destructive action, spec 4.5):

```qml
// The inline confirm: the row's controls give way to a line naming the consequence, a red
// confirm button and Keep. Escape or Keep restores the row. No modal dialog exists.
import QtQuick

Row {
    id: root
    property string question: ""
    property string confirmText: "Remove"
    property string confirmIcon: "trash-2"
    signal accepted()
    signal kept()
    spacing: theme.space(3)
    function close() { kept() }
    // Registered only while shown: a row toggles its confirm with `visible`
    function sync() { if (visible) frame.escapeStack.push("confirm", root); else frame.escapeStack.pop(root) }
    Component.onCompleted: sync()
    onVisibleChanged: sync()
    Component.onDestruction: frame.escapeStack.pop(root)
    Text { anchors.verticalCenter: parent.verticalCenter; text: root.question; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
    Button { text: root.confirmText; icon: root.confirmIcon; danger: true; onClicked: root.accepted() }
    Button { text: "Keep"; flat: true; onClicked: root.kept() }
}
```

`apps/linux/qml/EmptyState.qml` and `apps/linux/qml/SectionHeader.qml`:

```qml
// EmptyState.qml: a glyph, a title and one line, centred in the page.
import QtQuick
Column {
    property string icon: "info"
    property string title: ""
    property string body: ""
    default property alias actions: actionRow.data
    anchors.centerIn: parent
    spacing: theme.space(3)
    Icon { glyph: icon; size: theme.space(12); color: theme.textFaint; anchors.horizontalCenter: parent.horizontalCenter }
    Text { text: title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; anchors.horizontalCenter: parent.horizontalCenter }
    Text { text: body; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal; anchors.horizontalCenter: parent.horizontalCenter; horizontalAlignment: Text.AlignHCenter; width: theme.space(100); wrapMode: Text.Wrap }
    Row { id: actionRow; spacing: theme.space(2); anchors.horizontalCenter: parent.horizontalCenter }
}
```

```qml
// SectionHeader.qml: a bold title, a count chip and an action slot on the right.
import QtQuick
Item {
    property string title: ""
    property int count: -1
    default property alias actions: right.data
    width: parent ? parent.width : implicitWidth
    implicitHeight: theme.controlHeight
    Row {
        anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(3)
        Text { text: title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
        Chip { visible: count >= 0; text: String(count); small: true; color: theme.surface; textColor: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
    }
    Row { id: right; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; spacing: theme.space(2) }
}
```

`Rail.qml`: copy the prototype's and make these edits: `property int active: 0` stays, add `signal picked(int index)`, change the entry's `onClicked: root.active = entry.index` to `onClicked: root.picked(entry.index)`, make each entry `activeFocusOnTab: true` with `Keys.onReturnPressed: root.picked(entry.index)` and `Keys.onSpacePressed: root.picked(entry.index)`, and a focus ring `borderColor: entry.activeFocus ? theme.focusRing : "transparent"; borderWidth: entry.activeFocus ? theme.space(0.5) : 0`; the icon path in the brand tile becomes `qrc:/qt/qml/com/marcusrosado/AniBeam/assets/icon.png`; the version text becomes `text: Shell.version`; the JP / EN switch reads and writes the core: `index: Door.preferences.title_language === "English" ? 1 : 0` and `onPicked: function(i) { var p = JSON.parse(JSON.stringify(Door.preferences)); p.title_language = i === 1 ? "English" : "Romaji"; Door.setPreferences(p) }`. Add `import com.marcusrosado.AniBeam` at the top. Remove `titleLang` and `langPicked`.

`StatusStrip.qml`: copy the prototype's unchanged. The frame binds it.

`apps/linux/qml/Frame.qml`:

```qml
// The frame: the rail, the page area with the status strip at its foot and the drawer
// rising from it, and an overlay for menus and tips. Everything in spec 4.1 unit 1 that
// is not a page lives here. The player takes the whole window and hides the rest.
import QtQuick
import com.marcusrosado.AniBeam

FocusScope {
    id: frame
    property Window window
    readonly property alias nav: nav
    readonly property alias overlay: overlay
    readonly property alias escapeStack: escapeStack
    readonly property bool fullWindow: page.item ? (page.item.fullWindow === true) : false
    readonly property string windowTitle: (page.item && page.item.title ? page.item.title : nav.current.label) + " - AniBeam"

    Nav { id: nav }
    QtObject {
        id: escapeStack
        property var entries: []
        readonly property var rank: ({ drawer: 3, confirm: 2, popover: 1 })
        function push(kind, closer) { pop(closer); entries = entries.concat([{ kind: kind, closer: closer }]) }
        function pop(closer) { entries = entries.filter(function(e) { return e.closer !== closer }) }
        function top() {
            var best = null
            entries.forEach(function(e) { if (!best || rank[e.kind] >= rank[best.kind]) best = e })
            return best
        }
    }
    function escape() {
        var top = escapeStack.top()
        if (top) { top.closer.close(); return }
        if (page.item && page.item.escape && page.item.escape()) return
    }
    Keys.onEscapePressed: escape()

    // Pages by name; a page task swaps its placeholder for the real file
    readonly property var pages: ({
        library: libraryPage, feed: placeholder, watching: placeholder, metadata: placeholder, settings: placeholder,
        subscriptions: placeholder, series: placeholder, player: placeholder
    })
    Component { id: placeholder; PagePlaceholder {} }
    Component { id: libraryPage; PagePlaceholder {} }

    function leavingScroll() { return page.item && page.item.scrollY !== undefined ? page.item.scrollY : 0 }
    function go(name, props, label) { nav.open(name, props, label, leavingScroll()) }

    Rail {
        id: rail
        visible: !frame.fullWindow
        anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom
        active: nav.railIndex
        onPicked: function(i) { if (i !== nav.railIndex || nav.current.page !== nav.railPages[i]) frame.go(nav.railPages[i]) }
    }

    Item {
        id: content
        anchors.left: frame.fullWindow ? parent.left : rail.right
        anchors.right: parent.right; anchors.top: parent.top; anchors.bottom: parent.bottom

        Loader {
            id: page
            anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
            anchors.bottom: frame.fullWindow ? parent.bottom : strip.top
            focus: true
            sourceComponent: frame.pages[nav.current.page] || placeholder
            onLoaded: {
                item.props = nav.current.props
                if (nav.pendingScroll > 0 && item.scrollY !== undefined) Qt.callLater(function() { if (page.item) page.item.scrollY = nav.pendingScroll })
                item.forceActiveFocus()
            }
        }
        Connections {
            target: nav
            function onChanged() { page.active = false; page.active = true }
        }

        // Right-click anywhere outside the player: a menu that always offers Back
        MouseArea {
            anchors.fill: page
            enabled: !frame.fullWindow
            acceptedButtons: Qt.RightButton
            propagateComposedEvents: true
            onPressed: function(m) {
                var p = mapToItem(frame, m.x, m.y)
                var items = [{ text: "Back", icon: "arrow-left", action: nav.back }]
                if (page.item && page.item.contextItems) items = items.concat(page.item.contextItems())
                frame.openMenu(p.x, p.y, items)
            }
        }

        // Task 14 fills the drawer; the strip is wired now
        Item { id: drawerSlot; anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: strip.top }
        StatusStrip {
            id: strip
            visible: !frame.fullWindow
            height: visible ? theme.space(7) : 0
            anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
            readonly property var job: Door.runningJobs.length ? Door.runningJobs[0] : null
            readonly property var line: Door.latestLine
            stage: job ? String(job.kind).toLowerCase() : (line && line.stage ? String(line.stage).toLowerCase() : "system")
            message: job ? (job.label || job.kind) + (job.total > 0 ? " · " + job.done + " of " + job.total : "") : (line && line.message ? line.message : "")
            time: job || !line || !line.at ? "" : Qt.formatTime(new Date(line.at * 1000), "hh:mm")
            running: job !== null
            fraction: job && job.total > 0 ? job.done / job.total : 0
            unseenErrors: Door.unseenErrors
            onClicked: frame.toggleDrawer()
        }
    }
    function toggleDrawer() {}   // Task 14

    // Overlay: menus, tips, toasts
    Item {
        id: overlay
        anchors.fill: parent
        Menu { id: menu }
        Corner {
            id: tip
            visible: false
            radius: theme.radiusSm; smoothing: theme.cornerSmoothing
            color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
            width: tipText.implicitWidth + theme.space(4); height: tipText.implicitHeight + theme.space(2)
            Text { id: tipText; anchors.centerIn: parent; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
        }
        Corner {
            id: toastBox
            visible: false
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(10)
            radius: height / 2; smoothing: theme.cornerSmoothing
            color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
            width: toastText.implicitWidth + theme.space(6); height: theme.controlHeight
            Text { id: toastText; anchors.centerIn: parent; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
            Timer { id: toastTimer; onTriggered: toastBox.visible = false }
        }
    }
    function openMenu(x, y, items) { menu.openAt(x, y, items) }
    function closeMenu() { menu.close() }
    function showTip(item, text) {
        var p = item.mapToItem(overlay, item.width / 2, item.height)
        tipText.text = text
        tip.x = Math.max(theme.space(2), Math.min(p.x - tip.width / 2, overlay.width - tip.width - theme.space(2)))
        tip.y = Math.min(p.y + theme.space(1), overlay.height - tip.height - theme.space(2))
        tip.visible = true
    }
    function hideTip() { tip.visible = false }
    function toast(text, seconds) { toastText.text = text; toastBox.visible = true; toastTimer.interval = (seconds || 4) * 1000; toastTimer.restart() }

    Shortcut { sequence: "Ctrl+K"; onActivated: { if (nav.current.page !== "library") frame.go("library"); Qt.callLater(function() { if (page.item && page.item.focusSearch) page.item.focusSearch() }) } }
    Shortcut { sequence: "/"; enabled: nav.current.page === "library"; onActivated: if (page.item && page.item.focusSearch) page.item.focusSearch() }
    Shortcut { sequence: "Ctrl+,"; onActivated: if (nav.current.page !== "settings") frame.go("settings") }
    Shortcut { sequence: "Ctrl+L"; enabled: !frame.fullWindow; onActivated: frame.toggleDrawer() }
    Shortcut { sequence: "Ctrl+Q"; onActivated: Qt.quit() }
    Shortcut { sequence: "Alt+Left"; onActivated: nav.back() }
}
```

`Main.qml`: replace the Loader from Task 5 with

```qml
    Loader {
        id: frame
        anchors.fill: parent
        active: window.settled && Theme.ready && Door.ready
        sourceComponent: Shell.page === "tokens" ? tokensPage : frameComponent
        onLoaded: if (Shell.page !== "tokens" && Shell.page !== "library" && item.nav) item.nav.replace(Shell.page, {}, undefined)
    }
    Component { id: frameComponent; Frame { window: window } }
    Component { id: tokensPage; TokensPage {} }
    title: frame.item && frame.item.windowTitle ? frame.item.windowTitle : "AniBeam"
```

and the shoot timer waits for `frame.status === Loader.Ready` too (`maybeShoot()` checks `frame.item`).

`build.rs`: add `.qml_file(...)` for `Frame`, `Nav`, `Menu`, `Tooltip`, `InlineConfirm`, `EmptyState`, `SectionHeader`, `PagePlaceholder`, `PageScroll`, `Rail`, `StatusStrip`.

- [ ] **Step 2: Build and capture**

Run: `cargo build -p anibeam && apps/linux/scripts/shoot.sh frame --page library`
Expected: the rail with five entries and the JP / EN switch and the version at the foot, the placeholder "Library / Not built yet", the strip at the foot showing the core's Ready line ("AniBeam core ... ready") with the `system` chip and a time.

Run: `apps/linux/scripts/shoot.sh frame-settings --page settings`
Expected: Settings lit on the rail, the title "Settings".

Run on a monitor: `cargo build --release -p anibeam && apps/linux/scripts/bench.sh frame-live 2 keep`, then click through the rail, right-click and pick Back, press Escape with the menu open, Alt+Left, Tab through the rail entries and press Return, Ctrl+, and Ctrl+K
Expected: the window title reads `Feed - AniBeam` after picking Feed; Back returns to the previous page; the menu closes on Escape; Tab moves a focus ring over the rail entries. `pkill -x anibeam` afterwards.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the frame with the rail, the trail, the menu, tooltips, Escape and the strip"
```

---

### Task 8: The library page

Spec 4.1 unit 2 and 4.3. The prototype's grid, fed by the core: the search, the tabs, the sort and its direction, the Airing section, the count chip, the empty home with Import, and the live countdown. The core sorts and searches (`ListSeries`); the page keeps its list in a `RecordModel` and reloads on `seriesChanged` and `seriesRemoved` with a 250 ms debounce, keeping its scroll.

**Files:**
- Create: `apps/linux/qml/LibraryPage.qml`, `apps/linux/qml/Card.qml` (from the prototype, edited), `apps/linux/qml/SearchField.qml`, `apps/linux/qml/Pager.qml`
- Modify: `apps/linux/qml/Frame.qml` (`libraryPage` becomes `LibraryPage {}`), `apps/linux/build.rs`

**Interfaces:**
- Consumes: `Door.listSeries`, `Door.listAiring`, `Door.preferences`, `Door.setPreferences`, `Door.revealHidden`, `Door.importLibrary`, the `seriesChanged`, `seriesRemoved`, `preferencesChanged` signals; `Fmt`; `frame.go`.
- Produces: `LibraryPage` with `props.q` (the search text riding the trail), `focusSearch()`, `scrollY`; `Card` with `property var item` (a `SeriesCard` as JSON), `property real posterWidth`, `property real nowMs`, `signal opened()`; `SearchField` (`text`, `placeholder`, `hint`, `focusInput()`, `cleared()`); `Pager` (`page`, `hasMore`, `prev()`, `next()`).

Card facts, from the contract: `code` is the core's chip text ("EP 12" or "Movie"); `watched`, `total_episodes`, `total_is_estimate` and `watched_state` (`Behind`, `CaughtUp`, `Unknown`) feed the fraction chip; `strip` carries three fractions `watched`, `aired_unwatched`, `unknown`; `community_score` is already on the 0 to 10 scale; `my_score` is 0 to 10; `hidden`; `next_airing.at` and `last_viewed_at` are seconds; `episodes_on_disk`; `titles.folder` is the hover text; `title` is resolved by the core for the title language, so a language switch reloads the list.

- [ ] **Step 1: Write the page**

`apps/linux/qml/SearchField.qml`:

```qml
// The pill search field: placeholder in faint text, a "/  Ctrl K" hint at rest, an X once
// there is text. Escape clears and leaves it.
import QtQuick

Corner {
    id: root
    property alias text: input.text
    property string placeholder: "Search romaji, english or folder"
    property string hint: "/  Ctrl K"
    signal cleared()
    width: Math.min(parent ? parent.width : theme.space(120), theme.space(120))
    height: theme.controlHeight
    radius: height / 2
    smoothing: theme.cornerSmoothing
    color: theme.surfaceSunken
    borderColor: input.activeFocus ? theme.focusRing : theme.line
    borderWidth: 1
    function focusInput() { input.forceActiveFocus(); input.selectAll() }
    TextInput {
        id: input
        anchors.fill: parent
        anchors.leftMargin: theme.space(4); anchors.rightMargin: theme.space(10)
        verticalAlignment: TextInput.AlignVCenter
        color: theme.text
        font.family: theme.fontSans; font.pointSize: theme.typeNormal
        selectionColor: theme.accentSoft; selectedTextColor: theme.text
        clip: true
        Keys.onEscapePressed: { text = ""; focus = false; root.cleared() }
        Text { anchors.fill: parent; verticalAlignment: Text.AlignVCenter; visible: !input.text; text: root.placeholder; color: theme.textFaint; font: input.font }
    }
    Text {
        anchors.right: parent.right; anchors.rightMargin: theme.space(4); anchors.verticalCenter: parent.verticalCenter
        visible: !input.activeFocus && !input.text
        text: root.hint; color: theme.textFaint; font.family: theme.fontMono; font.pointSize: theme.typeSmall
    }
    Icon {
        visible: input.text !== ""
        anchors.right: parent.right; anchors.rightMargin: theme.space(3); anchors.verticalCenter: parent.verticalCenter
        glyph: "x"; size: theme.space(4); color: theme.textDim
        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { input.text = ""; root.cleared() } }
    }
}
```

`apps/linux/qml/Pager.qml`:

```qml
// ‹ page ›: the Airing section's pager. Disabled at the ends.
import QtQuick

Row {
    id: root
    property int page: 0
    property bool hasMore: false
    signal prev()
    signal next()
    spacing: theme.space(1)
    Button { text: "‹"; small: true; flat: true; enabled: root.page > 0; opacity: enabled ? 1 : theme.disabledOpacity; onClicked: root.prev() }
    Chip { text: String(root.page + 1); small: true; color: theme.surface; textColor: theme.textDim; anchors.verticalCenter: parent.verticalCenter }
    Button { text: "›"; small: true; flat: true; enabled: root.hasMore; opacity: enabled ? 1 : theme.disabledOpacity; onClicked: root.next() }
}
```

`apps/linux/qml/Card.qml`: copy the prototype's and replace the derived properties and the two `ago`/`until` functions so the card reads the contract's fields:

```qml
    readonly property string displayTitle: item.title || ""
    readonly property string folderName: item.titles ? item.titles.folder || "" : ""
    readonly property bool hasWatched: item.watched !== null && item.watched !== undefined
    readonly property string watchedLabel: Fmt.watchedChip(hasWatched ? item.watched : -1, item.total_episodes === null || item.total_episodes === undefined ? -1 : item.total_episodes, !!item.total_is_estimate)
    readonly property color watchedColor: item.watched_state === "Behind" ? theme.behind : item.watched_state === "Unknown" ? theme.textDim : theme.caughtUp
    readonly property bool totalKnown: item.total_episodes !== null && item.total_episodes !== undefined && item.total_episodes > 0
    readonly property real watchedPct: item.strip ? item.strip.watched : 0
    readonly property real airedPct: item.strip ? item.strip.watched + item.strip.aired_unwatched : 0
    readonly property real unknownPct: item.strip ? item.strip.unknown : 0
    readonly property string epBadge: item.code || ""
    readonly property string metaLeft: item.last_viewed_at ? Fmt.relative(item.last_viewed_at, nowMs / 1000) : Fmt.plural(item.episodes_on_disk || 0, "file", "files")
    readonly property string countdown: item.next_airing && item.next_airing.at * 1000 > nowMs ? Fmt.countdown(item.next_airing.at - nowMs / 1000) : ""
    signal opened()
```

The chips bind to these: the top left `Chip { text: root.epBadge }`, the top right `Chip { text: root.watchedLabel; textColor: root.watchedColor }`, the scores `Number(item.community_score).toFixed(1)` and `Number(item.my_score).toFixed(1)` as before, and a new bottom right `Chip { visible: !!item.hidden; text: "Hidden"; small: true; textColor: theme.textDim; anchors.right: parent.right; anchors.rightMargin: theme.space(2); anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2) + (strip.visible ? strip.height + theme.space(1.5) : 0) }`. The strip's three `Corner`s draw, back to front, the base in `root.totalKnown ? theme.scrim : theme.line` at full width, then `theme.line` over `unknownPct` from the right when `unknownPct > 0`, then `theme.behind` over `airedPct`, then `theme.caughtUp` over `watchedPct`. The title gets `Tooltip { text: root.folderName }`. The `MouseArea` gains `onClicked: root.opened()`. The `titleLang` property and the two functions go.

`apps/linux/qml/LibraryPage.qml`:

```qml
// Spec 4.1 unit 2: the grid of every series, the search, the tabs, the sort, the Airing
// section and the count chip. The core searches and sorts; the page keeps the list in a
// RecordModel and reloads it, debounced, when the core says a series changed.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

Item {
    id: page
    property var props: ({})
    property real scrollY: grid.contentY
    onScrollYChanged: if (Math.abs(grid.contentY - scrollY) > 1) grid.contentY = scrollY
    function focusSearch() { search.focusInput() }
    function escape() { return false }

    readonly property var tabs: ["All", "Series", "Movies"]
    readonly property var sorts: [["Alpha", "A to Z"], ["LastViewed", "Last viewed"], ["Progress", "Progress"], ["CommunityScore", "Score"], ["MyScore", "My score"]]
    readonly property var prefs: Door.preferences
    property string tab: prefs.library_tab || "All"
    property string sort: prefs.library_sort || "Alpha"
    property string direction: prefs.library_direction || "Asc"
    property string query: props.q || ""
    property bool hiddenExist: false
    property real nowMs: Date.now()
    property int airingPage: 0
    property bool airingMore: false
    property bool libraryEmpty: false

    RecordModel { id: cards; roles: ["id", "title", "titles", "poster", "code", "watched", "watched_state", "total_episodes", "total_is_estimate", "strip", "community_score", "my_score", "hidden", "next_airing", "last_viewed_at", "episodes_on_disk", "kind"] }
    RecordModel { id: airing; roles: cards.roles }

    function persist() {
        var p = JSON.parse(JSON.stringify(Door.preferences))
        p.library_tab = tab === "Hidden" ? p.library_tab : tab
        p.library_sort = sort; p.library_direction = direction
        Door.setPreferences(p)
    }
    function pickTab(i) { var names = tabs.concat(showHidden ? ["Hidden"] : []); tab = names[i]; persist(); reload() }
    function pickSort(key) { sort = key; direction = key === "Alpha" ? "Asc" : "Desc"; persist(); reload() }
    function flipDirection() { direction = direction === "Asc" ? "Desc" : "Asc"; persist(); reload() }
    readonly property bool showHidden: Door.revealHidden && hiddenExist
    readonly property var tabNames: tabs.concat(showHidden ? ["Hidden"] : [])

    function reload() {
        var keep = grid.contentY
        var r = Door.listSeries(tab, query, sort, direction, Door.revealHidden)
        if (r.error) { frame.toast(r.error.message); return }
        cards.reset(r.reply.series)
        grid.contentY = Math.min(keep, Math.max(0, grid.contentHeight - grid.height))
        var all = query === "" && tab === "All" ? r.reply.series.length : Door.listSeries("All", "", "Alpha", "Asc", false).reply.series.length
        libraryEmpty = all === 0 && !Door.revealHidden
        hiddenExist = Door.revealHidden ? Door.listSeries("Hidden", "", "Alpha", "Asc", true).reply.series.length > 0 : false
        reloadAiring()
    }
    function reloadAiring() {
        var r = Door.listAiring(airingPage * 10, 11)
        if (r.error) return
        var rows = r.reply.series
        airingMore = rows.length > 10
        airing.reset(rows.slice(0, 10))
        if (rows.length === 0 && airingPage > 0) { airingPage = 0; reloadAiring() }
    }
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Timer { id: queryDebounce; interval: 150; onTriggered: { page.query = search.text; frame.nav.current.props = { q: page.query }; page.reload() } }
    Timer { interval: 30000; running: true; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections {
        target: Door
        function onSeriesChanged(cards) { debounce.restart() }
        function onSeriesRemoved(ids) { debounce.restart() }
        function onPreferencesChanged(p) { if (page.tab !== "Hidden") page.tab = p.library_tab; page.sort = p.library_sort; page.direction = p.library_direction; debounce.restart() }
        function onRevealHiddenChanged() { if (page.tab === "Hidden" && !Door.revealHidden) page.tab = "All"; debounce.restart() }
    }
    Component.onCompleted: { search.text = query; reload() }

    Column {
        id: header
        anchors.top: parent.top; anchors.topMargin: theme.space(7)
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        anchors.right: parent.right; anchors.rightMargin: theme.space(8)
        spacing: theme.space(4)
        visible: !page.libraryEmpty
        Row {
            spacing: theme.space(3)
            Text { text: "Library"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
            Chip { anchors.verticalCenter: parent.verticalCenter; text: cards.count + (page.tab === "Movies" ? " films" : " series"); small: true; color: theme.surface; textColor: theme.textDim }
        }
        SearchField { id: search; onTextChanged: queryDebounce.restart(); onCleared: { queryDebounce.stop(); page.query = ""; frame.nav.current.props = {}; page.reload() } }
        Row {
            width: parent.width
            spacing: theme.space(3)
            Seg { options: page.tabNames; index: Math.max(0, page.tabNames.indexOf(page.tab)); onPicked: function(i) { page.pickTab(i) } }
            Item { width: theme.space(2); height: 1 }
            Repeater {
                model: page.sorts
                Chip {
                    required property var modelData
                    anchors.verticalCenter: parent.verticalCenter
                    text: modelData[1]; mono: false; clickable: true
                    selected: page.sort === modelData[0]
                    color: selected ? theme.accentSoft : theme.surface; textColor: theme.textDim
                    onClicked: page.pickSort(modelData[0])
                }
            }
            Chip { anchors.verticalCenter: parent.verticalCenter; text: page.direction === "Desc" ? "Desc" : "Asc"; clickable: true; color: theme.surface; textColor: theme.textDim; onClicked: page.flipDirection() }
        }
    }

    GridView {
        id: grid
        anchors.top: header.visible ? header.bottom : parent.top
        anchors.topMargin: theme.space(6)
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        anchors.right: parent.right; anchors.rightMargin: theme.space(8)
        anchors.bottom: parent.bottom
        visible: !page.libraryEmpty
        clip: true
        readonly property real gapX: theme.space(5)
        readonly property real gapY: theme.space(6)
        readonly property int columns: Math.max(1, Math.floor((width + gapX) / (theme.posterWidth + gapX)))
        cellWidth: Math.floor((width + gapX) / columns)
        readonly property real cardWidth: cellWidth - gapX
        cellHeight: Math.ceil(cardWidth * 1.5 + theme.space(2) + theme.typeNormal * 2 * 1.5 + theme.typeSmall * 1.5 + theme.space(1)) + gapY
        model: cards
        cacheBuffer: 1200
        header: Column {
            width: grid.width
            spacing: theme.space(4)
            visible: airing.count > 0 && page.query === ""
            height: visible ? implicitHeight + theme.space(6) : 0
            SectionHeader { title: "Airing"; count: airing.count; Pager { page: page.airingPage; hasMore: page.airingMore; onPrev: { page.airingPage--; page.reloadAiring() }; onNext: { page.airingPage++; page.reloadAiring() } } }
            Flow {
                width: parent.width
                spacing: grid.gapX
                Repeater {
                    model: airing
                    Card { required property int index; item: airing.at(index); posterWidth: grid.cardWidth; nowMs: page.nowMs; onOpened: frame.go("series", { id: item.id }, item.title) }
                }
            }
            Rectangle { width: parent.width; height: 1; color: theme.line }
        }
        delegate: Item {
            required property int index
            width: grid.cellWidth; height: grid.cellHeight
            Card { item: cards.at(index); posterWidth: grid.cardWidth; nowMs: page.nowMs; onOpened: frame.go("series", { id: item.id }, item.title) }
        }
        footer: Item { width: 1; height: theme.space(10) }
        EmptyState {
            visible: cards.count === 0 && !page.libraryEmpty
            icon: "search"
            title: page.query !== "" ? "No matches for \"" + page.query + "\"." : "Nothing here"
            body: page.query !== "" ? "" : "No " + (page.tab === "Series" ? "series" : page.tab === "Movies" ? "films" : "items") + " in your library yet."
        }
    }

    // The empty home: Import, and a pointer at Settings
    EmptyState {
        visible: page.libraryEmpty
        icon: "tv"
        title: "Your library is empty"
        body: "Add a folder in Settings, or import an AniBeam export."
        Button { text: "Import"; icon: "download"; onClicked: importDialog.open() }
        Button { text: "Settings"; icon: "settings"; flat: true; onClicked: frame.go("settings") }
    }
    FileDialog {
        id: importDialog
        title: "Import an AniBeam export"
        nameFilters: ["AniBeam export (*.json)", "All files (*)"]
        onAccepted: { var r = Door.importLibrary(decodeURIComponent(String(selectedFile).replace("file://", ""))); if (r.error) frame.toast(r.error.message); else frame.toast("Import started") }
    }
}
```

The `RecordModel` delegate reads `cards.at(index)` for the whole card rather than one role per field, because `Card` takes the record as one object; the roles list still names `id` so `upsert` and `remove` key correctly. `Frame.qml`'s `libraryPage` component becomes `LibraryPage {}`. `build.rs` gains `LibraryPage`, `Card`, `SearchField`, `Pager`.

- [ ] **Step 2: Build and capture against the real library**

Run against a sandbox seeded with the real posters: `ANIBEAM_ROOT=/tmp/sandbox` where the sandbox was made by `mkdir -p /tmp/sandbox && anibeam-cli --root /tmp/sandbox call AddSource --json '{"path":"/mnt/wd_general/media/Anime"}' --wait && anibeam-cli --root /tmp/sandbox call RefreshAll --wait` (the second call fetches posters into the sandbox's own cache; it takes a while and needs the network; the sandbox is reused by every later task).

Run: `ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh library --page library`
Expected: the grid at medium posters, the count chip reading `45 series`, every card with its `EP NN` or `Movie` chip, the fraction chip coloured by state, the strip, the scores, the meta line; the Airing section above the grid when any series is releasing.

Run: `ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh library-empty` with `ANIBEAM_ROOT=/tmp/empty`
Expected: the empty home with Import and Settings.

Run on a monitor: `ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/bench.sh library-live 2 keep --root /tmp/sandbox`; type in the search, switch tabs, pick sorts, flip the direction, press Escape in the search, open a card and press Alt+Left
Expected: the search filters as you type and Escape clears it; the sort chips reset the direction to the key's natural default; Back returns to the grid with the search text and the scroll position restored; relaunching shows the persisted tab, sort and direction.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the library page on the core's list, search, tabs, sort and airing"
```

---
### Task 9: The series page

Spec 4.1 unit 3. One `GetSeries` reply draws the whole page; `seriesChanged` naming this series re-reads it. The rules Electron kept in the renderer are listed where each lands.

**Files:**
- Create: `apps/linux/qml/SeriesPage.qml`, `apps/linux/qml/EpisodeRow.qml`, `apps/linux/qml/ScorePicker.qml`, `apps/linux/qml/PersonCard.qml`, `apps/linux/qml/RecommendationCard.qml`, `apps/linux/qml/StatusDot.qml`
- Modify: `apps/linux/qml/Frame.qml` (`series: seriesPage` with `Component { id: seriesPage; SeriesPage {} }`), `apps/linux/build.rs`

**Interfaces:**
- Consumes: `Door.getSeries`, `Door.setHidden`, `Door.setScore`, `Door.setProgress`, `Door.markEpisode`, `Door.refreshAiring`, `Door.rescanSeries`, `Door.openPlayback` (through `frame.go("player", { file })`), the `seriesChanged`, `airingRefreshed`, `scored`, `progressSet`, `marked` signals; `Theme.formatHue`, `Theme.statusHue`; `Fmt`.
- Produces: `SeriesPage` with `props.id`, `title`, `scrollY`, `contextItems()`; `EpisodeRow` (`episode` as JSON, `extra: bool`, `hasTracker: bool`, `signal play()`, `signal marker()`); `ScorePicker` (`open(anchorItem, current)`, `signal saved(real)`, `signal cleared()`), reused by the player's rating prompt; `StatusDot` (`status`); `PersonCard`; `RecommendationCard`.

- [ ] **Step 1: Write the page**

`apps/linux/qml/StatusDot.qml`:

```qml
// A list-status dot: the status hue, pulsing for Watching.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    property string status: ""
    width: theme.space(2.5); height: width
    radius: width / 2; smoothing: theme.cornerSmoothing
    color: theme.hue(Theme.statusHue(status))
    visible: status !== ""
    SequentialAnimation on opacity { running: status === "Watching"; loops: Animation.Infinite; NumberAnimation { to: 0.35; duration: 900 } NumberAnimation { to: 1; duration: 900 } }
    Tooltip { text: "On your list: " + (status === "Repeating" ? "Rewatching" : status) }
}
```

`apps/linux/qml/ScorePicker.qml` (0.0 to 10.0 in 0.1 steps, Save, Clear; registers as a popover on the escape stack):

```qml
// The score picker: 101 values from 0.0 to 10.0, Save, and Clear when a score exists.
// Opens under an anchor in the frame's overlay; Escape or a click outside closes it.
import QtQuick
import QtQuick.Controls.Basic as QC

Item {
    id: root
    property bool open: false
    property real current: -1
    property real draft: 8.0
    signal saved(real value)
    signal cleared()
    anchors.fill: parent
    visible: open
    z: 900

    readonly property var values: { var v = []; for (var i = 0; i <= 100; i++) v.push((i / 10).toFixed(1)); return v }
    function openAt(anchor, currentScore) {
        current = currentScore
        draft = currentScore >= 0 ? currentScore : 8.0
        var p = anchor.mapToItem(root, 0, anchor.height)
        panel.x = Math.min(p.x, root.width - panel.width - theme.space(2))
        panel.y = Math.min(p.y + theme.space(1), root.height - panel.height - theme.space(2))
        open = true
        frame.escapeStack.push("popover", root)
        list.positionViewAtIndex(Math.round(draft * 10), ListView.Center)
    }
    function close() { if (!open) return; open = false; frame.escapeStack.pop(root) }

    MouseArea { anchors.fill: parent; onPressed: root.close() }
    Corner {
        id: panel
        width: theme.space(48); height: theme.space(60)
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
        MouseArea { anchors.fill: parent }
        ListView {
            id: list
            anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
            anchors.bottom: buttons.top; anchors.margins: theme.space(2)
            clip: true
            model: root.values
            delegate: Corner {
                required property string modelData
                width: list.width; height: theme.space(6)
                radius: theme.radiusSm; smoothing: theme.cornerSmoothing
                color: Number(modelData) === Number(root.draft.toFixed(1)) ? theme.accentSoft : (m.containsMouse ? theme.surfacePressed : "transparent")
                Text { anchors.centerIn: parent; text: modelData; color: Number(modelData) === Number(root.draft.toFixed(1)) ? theme.accent : theme.text; font.family: theme.fontMono; font.pointSize: theme.typeNormal }
                MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; onClicked: root.draft = Number(modelData) }
            }
            QC.ScrollBar.vertical: QC.ScrollBar { contentItem: Rectangle { implicitWidth: 4; radius: 2; color: theme.lineStrong } }
        }
        Row {
            id: buttons
            anchors.bottom: parent.bottom; anchors.right: parent.right; anchors.margins: theme.space(2)
            spacing: theme.space(2)
            Button { visible: root.current >= 0; text: "Clear"; danger: true; small: true; onClicked: { root.close(); root.cleared() } }
            Button { text: "Save"; small: true; onClicked: { root.close(); root.saved(root.draft) } }
        }
    }
}
```

`apps/linux/qml/EpisodeRow.qml`:

```qml
// One episode: the marker (track to here / untrack to here), the code, the title, the
// pills, and a resume bar under it. Clicking the row opens the player.
import QtQuick

Corner {
    id: root
    property var episode: ({})
    property bool extra: false
    property bool hasTracker: false
    property string title: episode.title || ""
    signal play()
    signal marker()
    width: parent ? parent.width : implicitWidth
    height: theme.space(11)
    radius: theme.radiusSm; smoothing: theme.cornerSmoothing
    color: hover.containsMouse ? theme.surface : "transparent"

    readonly property bool watched: !!episode.watched
    readonly property bool nextUp: !!episode.next_up
    readonly property real resumeFraction: episode.resume && episode.resume.duration > 0 ? episode.resume.position / episode.resume.duration : 0

    Row {
        anchors.left: parent.left; anchors.leftMargin: theme.space(2)
        anchors.right: pills.left; anchors.rightMargin: theme.space(3)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(3)
        Corner {
            id: mark
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(6); height: width
            radius: width / 2; smoothing: theme.cornerSmoothing
            color: root.watched ? theme.accentSoft : theme.surfaceSunken
            borderColor: markHover.containsMouse ? theme.accent : theme.line; borderWidth: 1
            Icon { anchors.centerIn: parent; glyph: root.watched ? "check" : "play"; size: theme.space(3.5); color: root.watched ? theme.accent : theme.textDim }
            MouseArea { id: markHover; anchors.fill: parent; hoverEnabled: true; enabled: root.hasTracker && !root.extra; cursorShape: Qt.PointingHandCursor; onClicked: root.marker() }
            Tooltip { text: root.hasTracker && !root.extra ? (root.watched ? "untrack to here" : "track to here") : "" }
        }
        Text { anchors.verticalCenter: parent.verticalCenter; text: episode.code || ""; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall; width: theme.space(16) }
        Text { anchors.verticalCenter: parent.verticalCenter; text: root.title; color: root.nextUp ? theme.text : (root.watched ? theme.textDim : theme.text); font.family: theme.fontSans; font.pointSize: theme.typeNormal; elide: Text.ElideRight; width: parent.width - mark.width - theme.space(16) - theme.space(6) }
    }
    Row {
        id: pills
        anchors.right: parent.right; anchors.rightMargin: theme.space(3)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(1.5)
        Chip { visible: root.extra; text: "Extra"; small: true; mono: false; color: theme.tone(theme.bg, theme.yellow, 0.2); textColor: theme.yellow }
        Chip { visible: root.nextUp; text: "Next up"; small: true; mono: false; color: theme.accentSoft; textColor: theme.accent }
        Chip { visible: root.watched && !root.nextUp; text: "Watched"; small: true; mono: false; color: theme.surface; textColor: theme.textDim }
    }
    Corner {
        visible: root.resumeFraction > 0 && hover.containsMouse
        x: theme.space(11); width: (parent.width - theme.space(14)) * root.resumeFraction; height: theme.space(0.5)
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1)
        radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.accent
    }
    MouseArea { id: hover; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; z: -1; onClicked: root.play() }
}
```

`apps/linux/qml/PersonCard.qml` and `apps/linux/qml/RecommendationCard.qml`:

```qml
// PersonCard.qml: a character, portrait or a users glyph, name, role in lower case.
import QtQuick
Column {
    property var person: ({})
    width: theme.space(28)
    spacing: theme.space(1)
    Corner {
        width: parent.width; height: width * 1.4
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.surface; borderColor: theme.line; borderWidth: 1
        fillItem: portrait.status === Image.Ready ? portrait : null
        Image { id: portrait; visible: false; width: parent.width; height: parent.height; source: person.image ? "file://" + person.image : ""; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize.width: 240 }
        Icon { visible: !person.image; anchors.centerIn: parent; glyph: "users"; size: theme.space(6); color: theme.textFaint }
    }
    Text { width: parent.width; text: person.name || "Unknown"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold; elide: Text.ElideRight }
    Text { width: parent.width; text: (person.role || "").toLowerCase(); color: theme.textFaint; font.family: theme.fontSans; font.pointSize: theme.typeSmall; elide: Text.ElideRight }
}
```

```qml
// RecommendationCard.qml: poster, Available or AniList pill, list-status dot, title.
import QtQuick
Column {
    id: root
    property var rec: ({})
    signal opened()
    width: theme.space(32)
    spacing: theme.space(1)
    Corner {
        width: parent.width; height: width * 1.5
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.surface; borderColor: m.containsMouse ? theme.lineStrong : theme.line; borderWidth: 1
        fillItem: art.status === Image.Ready ? art : null
        Image { id: art; visible: false; width: parent.width; height: parent.height; source: rec.poster ? "file://" + rec.poster : ""; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize.width: 320 }
        Chip { x: theme.space(1.5); y: theme.space(1.5); small: true; mono: false; text: rec.owned ? "Available" : "AniList"; textColor: rec.owned ? theme.accent : theme.textDim }
        StatusDot { anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.margins: theme.space(2); status: rec.list_status || "" }
        MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.opened() }
        Tooltip { text: rec.owned ? "Open " + rec.title + " in your library" : "Open " + rec.title + " on AniList" }
    }
    Text { width: parent.width; text: rec.title || ""; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold; elide: Text.ElideRight; maximumLineCount: 2; wrapMode: Text.Wrap }
}
```

`apps/linux/qml/SeriesPage.qml`:

```qml
// Spec 4.1 unit 3: the hero, the chips, the synopsis and tags, Continue, the episodes,
// the extras, characters, recommendations and Related. One GetSeries draws it all.
import QtQuick
import QtQuick.Effects
import com.marcusrosado.AniBeam

PageScroll {
    id: page
    property var props: ({})
    property var detail: null
    readonly property var card: detail ? detail.card : ({})
    property string title: card.title || frame.nav.current.label
    onTitleChanged: frame.nav.relabel(title)
    property bool spoilers: false
    property bool synopsisOpen: false
    property bool tagsOpen: false
    property real nowMs: Date.now()
    property real optimisticProgress: -1
    readonly property bool isMovie: card.kind === "Movie"
    readonly property bool hasTracker: !!(card.match_info && (card.match_info.anilist_id || card.match_info.mal_id))
    readonly property string altTitle: {
        var t = card.titles || {}
        if (t.romaji && t.english && t.romaji !== t.english) return card.title === t.english ? t.romaji : t.english
        return ""
    }
    function contextItems() {
        return [
            { text: "Rescan show", icon: "refresh-cw", action: function() { var r = Door.rescanSeries(props.id); frame.toast(r.error ? r.error.message : "Rescan started") } },
            { text: "To Metadata", icon: "database", action: function() { frame.go("metadata", { q: page.title }) } }
        ]
    }
    function load() {
        var r = Door.getSeries(props.id)
        if (r.error) { frame.toast(r.error.message); return }
        detail = r.reply.detail
        optimisticProgress = -1
    }
    Component.onCompleted: { load(); Door.refreshAiring(props.id) }
    Timer { interval: 1000; running: !!(card.next_airing); repeat: true; onTriggered: page.nowMs = Date.now() }
    Timer { id: reloadDebounce; interval: 200; onTriggered: page.load() }
    Connections {
        target: Door
        function onSeriesChanged(cards) { for (var i = 0; i < cards.length; i++) if (cards[i].id === page.props.id) { reloadDebounce.restart(); return } }
        function onAiringRefreshed(series, updated) { if (series === page.props.id && updated) reloadDebounce.restart() }
        function onProgressSet(series, progress, outcomes) { if (series === page.props.id) reloadDebounce.restart() }
        function onScored(series, score, outcomes) { if (series === page.props.id) { reloadDebounce.restart(); frame.toast(outcomes.every(function(o) { return o.ok }) ? "Rated " + (score < 0 ? "cleared" : Fmt.score(score)) : "Score failed") } }
        function onMarked(series, episode, outcomes) { if (series === page.props.id) reloadDebounce.restart() }
    }
    function openFile(file) { frame.go("player", { file: file }, page.title) }
    function status(s) { return { Releasing: "Airing", Finished: "Finished", NotYetReleased: "Upcoming", Cancelled: "Cancelled", Hiatus: "Hiatus" }[s] || s }
    function formatLabel(f) { return isMovie ? "Movie" : !f ? "Series" : ({ TV: "TV", TV_SHORT: "TV Short", OVA: "OVA", ONA: "ONA", SPECIAL: "Special" })[f] || f.replace(/_/g, " ") }
    function progressText() {
        var p = detail.progress
        if (p.watched === null || p.watched === undefined) return p.on_disk + " on disk"
        var denom = p.total ? p.total : "?"
        var width = p.total ? String(p.total).length : 2
        return String(p.watched).padStart(width, "0") + " / " + denom + (p.estimate ? "+" : "")
    }
    // Track to here / untrack to here, optimistic; the core confirms through progressSet
    function marker(ep) {
        var watched = ep.watched
        var target = watched ? Math.max(0, Math.floor(ep.number) - 1) : Math.floor(ep.number)
        optimisticProgress = target
        var r = Door.setProgress(props.id, target)
        if (r.error) { optimisticProgress = -1; frame.toast(r.error.message) }
    }
    function watchedWithOptimism(ep) { return optimisticProgress >= 0 ? ep.number <= optimisticProgress : ep.watched }

    // Hero
    Item {
        width: parent.width; height: theme.space(60)
        Corner {
            anchors.fill: parent
            radius: theme.radiusXl; smoothing: theme.cornerSmoothing; color: theme.surface
            fillItem: art.status === Image.Ready ? art : null
            Image { id: art; visible: false; width: parent.width; height: parent.height; fillMode: Image.PreserveAspectCrop; asynchronous: true
                source: detail && detail.banner ? "file://" + detail.banner : (card.poster ? "file://" + card.poster : "") }
        }
        // No banner: the poster blown up and blurred behind a scrim
        MultiEffect { visible: detail && !detail.banner && !!card.poster; anchors.fill: parent; source: art; blurEnabled: true; blur: 1.0; blurMax: 64; opacity: 0.6 }
        Rectangle { anchors.fill: parent; color: theme.scrim; opacity: 0.55 }
        Chip { x: theme.space(4); y: theme.space(4); text: frame.nav.backLabel; icon: "arrow-left"; mono: false; clickable: true; onClicked: frame.nav.back() }
        Row {
            anchors.left: parent.left; anchors.bottom: parent.bottom; anchors.margins: theme.space(6)
            spacing: theme.space(5)
            Corner {
                width: theme.space(36); height: width * 1.5
                radius: theme.radiusLg; smoothing: theme.cornerSmoothing; color: theme.surfaceRaised; borderColor: theme.line; borderWidth: 1
                fillItem: poster.status === Image.Ready ? poster : null
                Image { id: poster; visible: false; width: parent.width; height: parent.height; source: card.poster ? "file://" + card.poster : ""; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize.width: 480 }
            }
            Column {
                anchors.bottom: parent.bottom
                spacing: theme.space(2)
                Text { text: page.title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; width: page.width - theme.space(64); elide: Text.ElideRight; wrapMode: Text.Wrap; maximumLineCount: 2 }
                Text { visible: page.altTitle !== ""; text: page.altTitle; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
                Row {
                    spacing: theme.space(2)
                    Chip { visible: card.community_score !== null && card.community_score !== undefined; icon: "star"; text: Fmt.score(card.community_score || 0); textColor: theme.yellow; Tooltip { text: "Average rating" } }
                    Chip { id: myScore; icon: "star"; mono: false; text: card.my_score !== null && card.my_score !== undefined ? Fmt.score(card.my_score) + "  You" : "Rate"; clickable: true; selected: card.my_score !== null && card.my_score !== undefined
                        onClicked: scorePicker.openAt(myScore, card.my_score === null || card.my_score === undefined ? -1 : card.my_score) }
                    Chip { visible: !!(detail && detail.site_url); text: "AniList"; icon: "external-link"; mono: false; clickable: true; onClicked: Qt.openUrlExternally(detail.site_url) }
                    Chip { text: card.hidden ? "Unhide" : "Hide"; icon: card.hidden ? "eye" : "eye-off"; mono: false; clickable: true; onClicked: { var r = Door.setHidden(props.id, !card.hidden); if (r.error) frame.toast(r.error.message) }
                        Tooltip { text: "Incognito: stops tracker sync and hides from all lists" } }
                }
            }
        }
    }

    // Info chips
    Flow {
        width: parent.width
        spacing: theme.space(2)
        Chip { text: page.formatLabel(card.format); mono: false; textColor: theme.hue(Theme.formatHue(page.isMovie ? "MOVIE" : (card.format || ""))) }
        Chip { visible: !!(detail && detail.year); text: String(detail ? detail.year : ""); }
        Chip { visible: !page.isMovie && card.total_episodes; text: card.total_episodes + " ep" }
        Chip { visible: !!(detail && detail.studio); text: detail ? detail.studio : ""; mono: false; Tooltip { text: "Animation studio" } }
        Chip { visible: !!card.status; text: page.status(card.status); mono: false }
        Chip { visible: !!card.next_airing && card.next_airing.at * 1000 > page.nowMs; icon: "clock"; textColor: theme.accent
            text: card.next_airing ? "EP " + String(card.next_airing.episode).padStart(2, "0") + " in " + Fmt.countdownSeconds(card.next_airing.at - page.nowMs / 1000) : "" }
        Chip { visible: !!card.list_status; mono: false; text: card.list_status === "Repeating" ? "Rewatching" : (card.list_status || "")
            Row { anchors.left: parent.left; anchors.leftMargin: theme.space(1); anchors.verticalCenter: parent.verticalCenter; StatusDot { status: card.list_status || "" } } }
        Chip { visible: !!(detail && detail.rewatch_count > 0); icon: "rotate-cw"; text: detail ? detail.rewatch_count + "x rewatched" : ""; textColor: theme.purple }
    }

    // Synopsis, five lines, More and Less only when it overflows
    Column {
        width: parent.width
        spacing: theme.space(1)
        Text {
            id: synopsis
            width: parent.width
            text: detail ? detail.synopsis.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim() : ""
            color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal
            wrapMode: Text.Wrap; elide: Text.ElideRight
            maximumLineCount: page.synopsisOpen ? 1000 : 5
        }
        Chip { visible: synopsis.truncated || page.synopsisOpen; text: page.synopsisOpen ? "Less" : "More"; mono: false; clickable: true; color: theme.surface; textColor: theme.textDim; onClicked: page.synopsisOpen = !page.synopsisOpen }
    }

    // Tags by rank; spoiler and adult tags behind the toggle
    Column {
        width: parent.width
        spacing: theme.space(2)
        visible: detail && detail.tags.length > 0
        readonly property var tags: detail ? detail.tags.filter(function(t) { return page.spoilers || (!t.spoiler && !t.adult) }).sort(function(a, b) { return b.rank - a.rank }) : []
        readonly property bool anySpoiler: detail ? detail.tags.some(function(t) { return t.spoiler || t.adult }) : false
        Row {
            spacing: theme.space(2)
            Text { text: "Tags"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
            Chip { visible: parent.parent.anySpoiler; text: "Spoilers"; icon: page.spoilers ? "eye-off" : "eye"; mono: false; clickable: true; selected: page.spoilers; color: selected ? theme.accentSoft : theme.surface; textColor: theme.textDim; onClicked: page.spoilers = !page.spoilers }
            Chip { visible: tagFlow.implicitHeight > tagClip.height || page.tagsOpen; text: page.tagsOpen ? "Less" : "Show all"; mono: false; clickable: true; color: theme.surface; textColor: theme.textDim; onClicked: page.tagsOpen = !page.tagsOpen }
        }
        Item {
            id: tagClip
            width: parent.width
            height: page.tagsOpen ? tagFlow.implicitHeight : Math.min(tagFlow.implicitHeight, theme.space(30))
            clip: true
            Flow {
                id: tagFlow
                width: parent.width; spacing: theme.space(1.5)
                Repeater {
                    model: parent.parent.parent.tags
                    Chip { required property var modelData; text: modelData.name + "  " + modelData.rank; mono: false; color: theme.surface; textColor: modelData.spoiler || modelData.adult ? theme.yellow : theme.textDim }
                }
            }
        }
    }

    // Continue or Play, and the progress line
    Row {
        spacing: theme.space(4)
        readonly property var nextFile: detail ? (page.isMovie ? (detail.episodes.length && !(card.list_status === "Completed" || (card.watched > 0)) ? detail.episodes[0].file : null) : detail.next_up) : null
        readonly property var nextEpisode: detail && !page.isMovie && detail.next_up ? detail.episodes.find(function(e) { return e.file === detail.next_up }) : null
        Button { visible: parent.nextFile !== null && parent.nextFile !== undefined; icon: "play"; text: page.isMovie ? "Play" : "Continue" + (parent.nextEpisode ? "  " + parent.nextEpisode.code : ""); onClicked: page.openFile(parent.nextFile) }
        Column {
            visible: !page.isMovie && !!detail
            anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(1)
            width: theme.space(60)
            Row { width: parent.width
                Text { text: detail && detail.progress.watched !== null ? "Tracked" : "Not tracked"; color: theme.textFaint; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
                Item { width: parent.width - trackedText.width - untrackedLabel.width; height: 1; Text { id: untrackedLabel; visible: false } }
                Text { id: trackedText; text: detail ? page.progressText() : ""; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall } }
            Corner { width: parent.width; height: theme.space(1.5); radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken; borderColor: theme.line; borderWidth: 1
                Corner { readonly property real f: detail && detail.progress.watched !== null && detail.progress.total ? Math.min(1, detail.progress.watched / detail.progress.total) : 0
                    width: parent.width * f; height: parent.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.accent } }
        }
    }

    // Episodes
    Column {
        width: parent.width
        spacing: theme.space(0.5)
        visible: detail && detail.episodes.length > 0
        SectionHeader { title: page.isMovie ? "Film" : "Episodes"; count: detail ? detail.episodes.length : 0 }
        Repeater {
            model: detail ? detail.episodes : []
            EpisodeRow { required property var modelData; episode: modelData; hasTracker: page.hasTracker; title: page.isMovie && detail.episodes.length === 1 ? page.title : (modelData.title || modelData.path.split("/").pop())
                watched: page.watchedWithOptimism(modelData)
                onPlay: page.openFile(modelData.file); onMarker: page.marker(modelData) }
        }
    }

    // Extras, grouped
    Column {
        width: parent.width
        spacing: theme.space(2)
        visible: detail && detail.extras.length > 0
        SectionHeader { title: "Openings, Endings & More"; count: detail ? detail.extras.length : 0 }
        Repeater {
            model: [["Op", "Openings"], ["Ed", "Endings"], ["Pv", "Previews & Trailers"], ["Sp", "Specials"], ["Other", "Other"]]
            Column {
                required property var modelData
                readonly property var group: detail ? detail.extras.filter(function(x) { return x.kind === modelData[0] }) : []
                visible: group.length > 0
                width: parent.width
                spacing: theme.space(0.5)
                Row { spacing: theme.space(2); Text { text: modelData[1]; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter } Chip { text: String(group.length); small: true; color: theme.surface; textColor: theme.textFaint; anchors.verticalCenter: parent.verticalCenter } }
                Repeater { model: group; EpisodeRow { required property var modelData; episode: ({ code: modelData.code, title: modelData.label, resume: modelData.resume, watched: false, next_up: false }); title: modelData.label; onPlay: page.openFile(modelData.file) } }
            }
        }
    }

    // Files numbered past the matched count
    Column {
        width: parent.width
        spacing: theme.space(0.5)
        visible: detail && detail.unmatched_files.length > 0
        SectionHeader { title: "Extra files"; count: detail ? detail.unmatched_files.length : 0 }
        Row { spacing: theme.space(2); Icon { glyph: "triangle-alert"; size: theme.space(4); color: theme.yellow; anchors.verticalCenter: parent.verticalCenter }
            Text { width: page.width - theme.space(24); wrapMode: Text.Wrap; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall
                text: detail ? (detail.unmatched_files.length === 1 ? "1 file goes" : detail.unmatched_files.length + " files go") + " beyond the expected " + card.total_episodes + " episode" + (card.total_episodes === 1 ? "" : "s") + " for this title, likely misnamed, duplicates, or specials. Review them and rename or remove what doesn't belong." : "" } }
        Repeater { model: detail ? detail.unmatched_files : []; EpisodeRow { required property var modelData; episode: modelData; extra: true; title: modelData.title || modelData.path.split("/").pop(); onPlay: page.openFile(modelData.file) } }
    }

    // Characters, then Recommendations
    Column {
        width: parent.width; spacing: theme.space(2)
        visible: detail && detail.characters.length > 0
        SectionHeader { title: "Characters"; count: detail ? detail.characters.length : 0 }
        Flow { width: parent.width; spacing: theme.space(3); Repeater { model: detail ? detail.characters : []; PersonCard { required property var modelData; person: modelData } } }
    }
    Column {
        width: parent.width; spacing: theme.space(2)
        visible: detail && detail.recommendations.length > 0
        SectionHeader { title: "Recommendations"; count: detail ? detail.recommendations.length : 0 }
        Flow { width: parent.width; spacing: theme.space(3)
            Repeater { model: detail ? detail.recommendations : []
                RecommendationCard { required property var modelData; rec: modelData
                    onOpened: modelData.owned ? frame.go("series", { id: modelData.owned }, modelData.title) : Qt.openUrlExternally("https://anilist.co/anime/" + modelData.anilist_id) } } }
    }

    // Related: the franchise graph (Task 24 fills the component)
    Column {
        width: parent.width; spacing: theme.space(2)
        visible: detail && detail.has_graph
        SectionHeader { title: "Related" }
        Loader { id: related; width: parent.width; height: theme.space(120); active: detail && detail.has_graph; sourceComponent: relatedPlaceholder }
        Component { id: relatedPlaceholder; Rectangle { color: theme.surfaceSunken; Text { anchors.centerIn: parent; text: "Franchise graph, Task 24"; color: theme.textFaint } } }
    }

    ScorePicker { id: scorePicker; parent: frame.overlay
        onSaved: function(v) { var r = Door.setScore(page.props.id, v); if (r.error) frame.toast(r.error.message) }
        onCleared: { var r = Door.setScore(page.props.id, -1); if (r.error) frame.toast(r.error.message) } }
}
```

`EpisodeRow` gains `property bool watched: !!episode.watched` as an overridable property (the series page binds the optimistic value), replacing the `readonly` line. `Frame.qml`: `series: seriesPage` and `Component { id: seriesPage; SeriesPage {} }`. `build.rs`: `SeriesPage`, `EpisodeRow`, `ScorePicker`, `PersonCard`, `RecommendationCard`, `StatusDot`. `MultiEffect` needs `import QtQuick.Effects`, shipped in qt6-declarative.

- [ ] **Step 2: Build and capture**

Run: `cargo build -p anibeam && ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh series --page series --width 1600 --height 1400`, with `Main.qml`'s shoot path extended so `--page series` opens the first series in the library (`Shell.page` of `series` with no id resolves to `Door.listSeries("All","","Alpha","Asc",false).reply.series[0].id`; put that resolution in `Main.qml`'s `onLoaded`)
Expected: the hero with the banner or the blurred poster, the Back chip reading `Library`, the title and the alternate title, the score and Hide chips, the info chips with the format coloured by its hue, the synopsis clamped with More, the tags, Continue with the next code, the progress line `NN / MM`, the episode rows with markers and pills, the extras groups, characters and recommendations.

Run on a monitor with `bench.sh` and a matched series open: click a marker (the progress flips at once, then the core's `progressSet` confirms), open the score picker and Save, press Escape with it open, right-click and pick To Metadata, press Alt+Left
Expected: the marker's optimism holds until the core answers; Escape closes the picker first; To Metadata opens the Metadata placeholder with `props.q` set; Back returns to the series page at its scroll.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the series page from one GetSeries reply"
```

---
### Task 10: The video item, the config layers, the session and the ticks

Spec 4.4 (the surface, mpv configuration, completion and the tick, volume) and 5.2. After this task any file plays on first click, hardware decoded, from its resume point; the core receives ticks once a second plus pause, seek and close; the header, Previous and Next, play/pause, the time readout, mute and the volume slider, fullscreen and the 2.5 s hide are there. Nothing reads mpv through a getter on a timer: every value the page shows is an observed property.

**Files:**
- Create: `apps/linux/cpp/videoitem.h`, `apps/linux/cpp/videoitem.cpp`
- Create: `apps/linux/mpv.conf` (spec 5.4, verbatim)
- Create: `apps/linux/src/player_config.rs`, `apps/linux/src/bridge/player.rs`
- Create: `apps/linux/qml/PlayerPage.qml`, `apps/linux/qml/PlayerButton.qml`
- Modify: `apps/linux/build.rs` (the C++ files, the QML files, the bridge), `apps/linux/src/main.rs` (`mod player_config;`), `apps/linux/src/bridge/mod.rs`, `apps/linux/qml/Frame.qml` (`player: playerPage`)

**Interfaces:**
- Consumes: MpvQt 1.2.0 (`MpvAbstractItem`, `MpvController`), `Door.openPlayback`, `Door.reportChapters`, `Door.tick`, `Door.closePlayback`, `paths::ShellPaths`, `Fmt`.
- Produces:
  - The QML element `VideoItem` (C++): `enum Format { Flag, Int64, Double, String, Node }`; signals `loaded()`, `ended(string reason)`, `changed(string name, var value)`, `reconfigured()`, and the inherited `ready()`; invokables `observe(name, format)`, `include(path)`, and the inherited `setProperty`, `setPropertyAsync`, `getProperty`, `command`, `commandAsync`.
  - `player_config::PlayerSettings { volume: f64 (100), mute: bool, use_my_mpv_conf: bool }` with `load(path)`, `save(path, &s)` through `toml_edit`; `player_config::owned_options() -> Vec<(&'static str, String)>` (the spec's list, minus the subtitle and volume ones, which the callers append); `player_config::preview_options() -> Vec<(&'static str, &'static str)>`; `player_config::config_layers(paths: &ShellPaths, use_my_conf: bool) -> Vec<PathBuf>` (bundled, then the user's when the toggle is on, then AniBeam's, existing files only).
  - The QML singleton `Player`: properties `volume`, `mute`, `useMyMpvConf` (each with a `set*` invokable that persists: `setVolume(v)`, `setMute(b)`, `setUseMyMpvConf(b)`), `configLayers` (string list), `ownedOptions` (JSON array of `[name, value]`), `previewOptions` (JSON array).
  - `PlayerPage` with `props.file`, `fullWindow: true`, `title`, `escape()`, and internal state the next two tasks extend: `session` (the `PlaybackSession` JSON), `video` (the main `VideoItem`), `timePos`, `duration`, `paused`, `ended`, `chromeVisible`, `showChrome()`, `seekTo(secs)`, `togglePause()`, `openNeighbour(file)`, `leave()`.

- [ ] **Step 1: Write the failing tests for the config helpers**

`apps/linux/src/player_config.rs`, tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::ShellPaths;
    use std::path::Path;

    #[test]
    fn player_toml_round_trips_with_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("player.toml");
        let d = load(&p);
        assert_eq!((d.volume, d.mute, d.use_my_mpv_conf), (100.0, false, false));
        save(&p, &PlayerSettings { volume: 42.5, mute: true, use_my_mpv_conf: true }).unwrap();
        let s = load(&p);
        assert_eq!((s.volume, s.mute, s.use_my_mpv_conf), (42.5, true, true));
        std::fs::write(&p, "volume = 900\n").unwrap();
        assert_eq!(load(&p).volume, 100.0, "out of range clamps");
    }

    #[test]
    fn the_owned_options_end_with_what_the_spec_lists() {
        let o = owned_options();
        let names: Vec<&str> = o.iter().map(|(n, _)| *n).collect();
        for expected in ["osc", "osd-level", "input-default-bindings", "input-vo-keyboard", "input-media-keys", "resume-playback",
                         "save-position-on-quit", "keep-open", "pause", "fullscreen", "loop-file", "loop-playlist", "ytdl", "sub-auto",
                         "audio-file-auto", "reset-on-next-file", "volume-max"] {
            assert!(names.contains(&expected), "{expected}");
        }
        assert_eq!(o.iter().find(|(n, _)| *n == "keep-open").unwrap().1, "always");
        assert_eq!(o.iter().find(|(n, _)| *n == "reset-on-next-file").unwrap().1, "sub-delay");
        assert_eq!(o.iter().find(|(n, _)| *n == "volume-max").unwrap().1, "100");
        let p = preview_options();
        assert!(p.contains(&("aid", "no")) && p.contains(&("sid", "no")) && p.contains(&("pause", "yes")) && p.contains(&("hr-seek", "yes")));
    }

    #[test]
    fn config_layers_are_the_existing_files_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ShellPaths::resolve(Some(dir.path())).unwrap();
        std::fs::create_dir_all(paths.config_dir()).unwrap();
        std::fs::create_dir_all(paths.user_mpv_conf().parent().unwrap()).unwrap();
        let bundled = dir.path().join("bundled.conf");
        std::fs::write(&bundled, "hwdec=auto\n").unwrap();
        std::fs::write(paths.user_mpv_conf(), "osc=yes\n").unwrap();
        std::fs::write(paths.anibeam_mpv_conf(), "deband=no\n").unwrap();
        // ANIBEAM_MPV_CONF points bundled_mpv_conf at the temp file for this test
        std::env::set_var("ANIBEAM_MPV_CONF", &bundled);
        let off = config_layers(&paths, false);
        assert_eq!(off, vec![bundled.clone(), paths.anibeam_mpv_conf()]);
        let on = config_layers(&paths, true);
        assert_eq!(on, vec![bundled, paths.user_mpv_conf(), paths.anibeam_mpv_conf()]);
        std::env::remove_var("ANIBEAM_MPV_CONF");
        assert!(!config_layers(&paths, true).contains(&Path::new("/usr/share/anibeam/mpv.conf").to_path_buf()) || Path::new("/usr/share/anibeam/mpv.conf").exists());
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p anibeam player_config`
Expected: compile errors.

- [ ] **Step 3: Write the item, the config module, the Player singleton and the page**

`apps/linux/mpv.conf`, verbatim from spec 5.4:

```conf
# AniBeam's base mpv configuration. The user's own mpv.conf loads after this one when
# "Use my mpv.conf" is on, and ~/.config/anibeam/mpv.conf loads last. The shell re-sets
# what it owns after every load. Scripts never load.

# nvdec on NVIDIA, vaapi on AMD, zero copy on both (#9, #18).
hwdec=auto
```

`apps/linux/cpp/videoitem.h`:

```cpp
#pragma once
// The video surface: MpvQt's MpvAbstractItem, which owns the mpv core, its thread and the
// render context. This subclass only forwards the controller's signals to QML, exposes the
// observation formats, and loads a config layer through `include` after init (spec 5.2).
#include <MpvQt/MpvAbstractItem>
#include <QtQml/qqmlregistration.h>
#include <QtCore/QString>
#include <QtCore/QVariant>

class VideoItem : public MpvAbstractItem
{
    Q_OBJECT
    QML_ELEMENT
public:
    enum Format { Flag = MPV_FORMAT_FLAG, Int64 = MPV_FORMAT_INT64, Double = MPV_FORMAT_DOUBLE, String = MPV_FORMAT_STRING, Node = MPV_FORMAT_NODE };
    Q_ENUM(Format)

    explicit VideoItem(QQuickItem *parent = nullptr);

    Q_INVOKABLE void observe(const QString &name, int format);
    /// Parses a config file as if each line were set one by one; init-only lines are ignored.
    Q_INVOKABLE void include(const QString &path);

Q_SIGNALS:
    void loaded();
    void ended(const QString &reason);
    void changed(const QString &name, const QVariant &value);
    void reconfigured();
};
```

`apps/linux/cpp/videoitem.cpp`:

```cpp
#include "videoitem.h"
#include <MpvQt/MpvController>

VideoItem::VideoItem(QQuickItem *parent)
    : MpvAbstractItem(parent)
{
    connect(mpvController(), &MpvController::fileLoaded, this, &VideoItem::loaded, Qt::QueuedConnection);
    connect(mpvController(), &MpvController::endFile, this, &VideoItem::ended, Qt::QueuedConnection);
    connect(mpvController(), &MpvController::propertyChanged, this, &VideoItem::changed, Qt::QueuedConnection);
    connect(mpvController(), &MpvController::videoReconfig, this, &VideoItem::reconfigured, Qt::QueuedConnection);
}

void VideoItem::observe(const QString &name, int format)
{
    observeProperty(name, static_cast<mpv_format>(format));
}

void VideoItem::include(const QString &path)
{
    setPropertyBlocking(QStringLiteral("include"), path);
}
```

`build.rs`: `.cpp_files(["cpp/helpers.cpp", "cpp/videoitem.h", "cpp/videoitem.cpp"])` (the header goes through `cpp_files` so it gets moc with the module URI and `QML_ELEMENT` registers `VideoItem` beside the Rust types).

`apps/linux/src/player_config.rs` (above its tests):

```rust
//! player.toml and the mpv option lists the shell owns. Spec 4.4: the layers load through
//! `include` after init, and the options the shell owns are set last so no config line
//! can take them back.

use std::path::{Path, PathBuf};

use toml_edit::{value, DocumentMut};

use crate::paths::ShellPaths;

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerSettings {
    pub volume: f64,
    pub mute: bool,
    pub use_my_mpv_conf: bool,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        PlayerSettings { volume: 100.0, mute: false, use_my_mpv_conf: false }
    }
}

pub fn load(path: &Path) -> PlayerSettings {
    let d = PlayerSettings::default();
    let Ok(text) = std::fs::read_to_string(path) else { return d };
    let Ok(doc) = text.parse::<DocumentMut>() else { return d };
    let volume = doc.get("volume").and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64))).filter(|v| (0.0..=100.0).contains(v)).unwrap_or(d.volume);
    PlayerSettings {
        volume,
        mute: doc.get("mute").and_then(|v| v.as_bool()).unwrap_or(d.mute),
        use_my_mpv_conf: doc.get("use_my_mpv_conf").and_then(|v| v.as_bool()).unwrap_or(d.use_my_mpv_conf),
    }
}

pub fn save(path: &Path, s: &PlayerSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut doc = std::fs::read_to_string(path).ok().and_then(|t| t.parse::<DocumentMut>().ok()).unwrap_or_default();
    doc["volume"] = value(s.volume);
    doc["mute"] = value(s.mute);
    doc["use_my_mpv_conf"] = value(s.use_my_mpv_conf);
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, doc.to_string())?;
    std::fs::rename(tmp, path)
}

/// Set after every include, in this order. The subtitle defaults, the language orders,
/// volume and mute follow from their own tables.
pub fn owned_options() -> Vec<(&'static str, String)> {
    [
        ("vo", "libmpv"), ("osc", "no"), ("osd-level", "0"), ("input-default-bindings", "no"), ("input-vo-keyboard", "no"),
        ("input-media-keys", "no"), ("resume-playback", "no"), ("save-position-on-quit", "no"), ("keep-open", "always"),
        ("pause", "no"), ("fullscreen", "no"), ("loop-file", "no"), ("loop-playlist", "no"), ("ytdl", "no"), ("sub-auto", "no"),
        ("audio-file-auto", "no"), ("reset-on-next-file", "sub-delay"), ("volume-max", "100"),
    ]
    .into_iter()
    .map(|(k, v)| (k, v.to_string()))
    .collect()
}

/// Haruna's MpvPreview recipe: its own core, nothing audible, nothing drawn but the frame.
pub fn preview_options() -> Vec<(&'static str, &'static str)> {
    vec![
        ("vo", "libmpv"), ("mute", "yes"), ("pause", "yes"), ("really-quiet", "yes"), ("hwdec", "auto"), ("hr-seek", "yes"),
        ("aid", "no"), ("audio-file-auto", "no"), ("sid", "no"), ("sub-auto", "no"), ("osd-level", "0"),
        ("audio-pitch-correction", "no"), ("use-text-osd", "no"), ("audio-display", "no"), ("keep-open", "always"),
    ]
}

pub fn config_layers(paths: &ShellPaths, use_my_conf: bool) -> Vec<PathBuf> {
    let mut out = vec![paths.bundled_mpv_conf()];
    if use_my_conf {
        out.push(paths.user_mpv_conf());
    }
    out.push(paths.anibeam_mpv_conf());
    out.into_iter().filter(|p| p.is_file()).collect()
}
```

`apps/linux/src/bridge/player.rs` (Tasks 11 and 13 add invokables here):

```rust
//! Player: the shell-owned player state QML reads, player.toml behind it, and the pure
//! helpers the player page calls into Rust for.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QJsonArray, QJsonValue, QList, QString, QStringList};

use crate::player_config::{self, PlayerSettings};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qjsonarray.h");
        type QJsonArray = cxx_qt_lib::QJsonArray;
        include!("cxx-qt-lib/qjsonobject.h");
        type QJsonObject = cxx_qt_lib::QJsonObject;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(f64, volume)]
        #[qproperty(bool, mute)]
        #[qproperty(bool, use_my_mpv_conf)]
        #[qproperty(QStringList, config_layers)]
        #[qproperty(QJsonArray, owned_options)]
        #[qproperty(QJsonArray, preview_options)]
        type Player = super::PlayerRust;

        #[qinvokable] fn save_volume(self: Pin<&mut Self>, volume: f64);
        #[qinvokable] fn save_mute(self: Pin<&mut Self>, mute: bool);
        #[qinvokable] fn save_use_my_mpv_conf(self: Pin<&mut Self>, on: bool);
    }
}

pub struct PlayerRust {
    volume: f64,
    mute: bool,
    use_my_mpv_conf: bool,
    config_layers: QStringList,
    owned_options: QJsonArray,
    preview_options: QJsonArray,
}

fn pairs(list: impl IntoIterator<Item = (String, String)>) -> QJsonArray {
    let mut out = QJsonArray::default();
    for (k, v) in list {
        let mut pair = QJsonArray::default();
        pair.append(&QJsonValue::from(&QString::from(&k)));
        pair.append(&QJsonValue::from(&QString::from(&v)));
        out.append(&QJsonValue::from(&pair));
    }
    out
}

fn layers(use_my_conf: bool) -> QStringList {
    let paths = crate::runtime::paths();
    QStringList::from_iter(player_config::config_layers(paths, use_my_conf).iter().map(|p| QString::from(&p.to_string_lossy().into_owned())))
}

impl Default for PlayerRust {
    fn default() -> Self {
        let s = player_config::load(&crate::runtime::paths().player_toml());
        PlayerRust {
            volume: s.volume,
            mute: s.mute,
            use_my_mpv_conf: s.use_my_mpv_conf,
            config_layers: layers(s.use_my_mpv_conf),
            owned_options: pairs(player_config::owned_options().into_iter().map(|(k, v)| (k.to_string(), v))),
            preview_options: pairs(player_config::preview_options().into_iter().map(|(k, v)| (k.to_string(), v.to_string()))),
        }
    }
}

impl qobject::Player {
    fn persist(&self) {
        let s = PlayerSettings { volume: *self.volume(), mute: *self.mute(), use_my_mpv_conf: *self.use_my_mpv_conf() };
        if let Err(e) = player_config::save(&crate::runtime::paths().player_toml(), &s) {
            eprintln!("anibeam: player.toml: {e}");
        }
    }
    pub fn save_volume(mut self: Pin<&mut Self>, volume: f64) {
        self.as_mut().set_volume(volume.clamp(0.0, 100.0));
        self.persist();
    }
    pub fn save_mute(mut self: Pin<&mut Self>, mute: bool) {
        self.as_mut().set_mute(mute);
        self.persist();
    }
    pub fn save_use_my_mpv_conf(mut self: Pin<&mut Self>, on: bool) {
        self.as_mut().set_use_my_mpv_conf(on);
        let l = layers(on);
        self.as_mut().set_config_layers(l);
        self.persist();
    }
}
```

`apps/linux/qml/PlayerButton.qml` (a round glyph button on the scrim):

```qml
import QtQuick
Corner {
    id: root
    property string glyph: ""
    property string tip: ""
    property bool active: false
    signal clicked()
    width: theme.space(9); height: width
    radius: width / 2; smoothing: theme.cornerSmoothing
    color: m.pressed ? theme.surfacePressed : (m.containsMouse ? theme.surfaceRaised : "transparent")
    opacity: enabled ? 1 : theme.disabledOpacity
    Icon { anchors.centerIn: parent; glyph: root.glyph; size: theme.space(4.5); color: root.active ? theme.accent : theme.text }
    MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; enabled: root.enabled; cursorShape: Qt.PointingHandCursor; onClicked: root.clicked() }
    Tooltip { text: root.tip }
}
```

`apps/linux/qml/PlayerPage.qml`, the base every later player task extends (the comments mark where Tasks 11 and 12 add):

```qml
// Spec 4.4: the player. The one page that takes the whole window. libmpv draws through
// VideoItem; the shell draws every overlay and handles every key. The core owns the
// rules; the shell sends ticks and shows what comes back.
import QtQuick
import QtQuick.Window
import com.marcusrosado.AniBeam

FocusScope {
    id: page
    property var props: ({})
    readonly property bool fullWindow: true
    property var session: null
    property string title: session ? session.series_title : "Player"
    property real scrollY: 0
    readonly property alias video: video

    // Observed mpv state
    property real timePos: 0
    property real duration: 0
    property bool paused: false
    property bool seeking: false
    property bool ended: false
    property bool loaded: false
    property string hwdec: ""
    property int drops: 0

    // Chrome
    property bool chromeVisible: true
    property int openMenus: 0            // pickers and the help list hold the chrome
    Timer { id: hideTimer; interval: 2500; onTriggered: if (page.openMenus === 0) page.chromeVisible = false }
    function showChrome() { chromeVisible = true; hideTimer.restart() }

    // ---- Session
    Component.onCompleted: {
        var r = Door.openPlayback(props.file)
        if (r.error) { frame.toast(r.error.message); Qt.callLater(frame.nav.back); return }
        session = r.reply.session
        if (video.isReady) start()
        showChrome()
        forceActiveFocus()
    }
    Component.onDestruction: close("Stopped")
    property bool closed: false
    function close(reason) {
        if (!session || closed) return
        closed = true
        tickTimer.stop()
        Door.closePlayback(session.session, timePos, reason)
    }
    function leave() { close("Stopped"); frame.nav.back() }
    function openNeighbour(file) { if (!file) return; close("Switched"); frame.nav.replace("player", { file: file }, page.title) }
    function escape() { leave(); return true }

    // The layers, the owned options, then the file, seeking to the resume point before
    // the first frame through the start option.
    function start() {
        var layers = Player.configLayers
        for (var i = 0; i < layers.length; i++) video.include(layers[i])
        var owned = Player.ownedOptions
        for (var j = 0; j < owned.length; j++) video.setProperty(owned[j][0], owned[j][1])
        applyDefaults()                                          // Task 11 fills this
        video.setProperty("volume", Player.volume)
        video.setProperty("mute", Player.mute)
        video.setProperty("start", session.resume_from ? String(session.resume_from) : "none")
        video.command(["loadfile", session.path])
    }
    function applyDefaults() {}

    VideoItem {
        id: video
        anchors.fill: parent
        property bool isReady: false
        onReady: {
            isReady = true
            observe("time-pos", VideoItem.Double); observe("duration", VideoItem.Double); observe("pause", VideoItem.Flag)
            observe("eof-reached", VideoItem.Flag); observe("seeking", VideoItem.Flag); observe("volume", VideoItem.Double)
            observe("mute", VideoItem.Flag); observe("hwdec-current", VideoItem.String); observe("frame-drop-count", VideoItem.Int64)
            observe("track-list", VideoItem.Node); observe("chapter-list", VideoItem.Node); observe("aid", VideoItem.String)
            observe("sid", VideoItem.String); observe("sub-delay", VideoItem.Double); observe("estimated-frame-number", VideoItem.Int64)
            if (page.session) page.start()
        }
        onLoaded: {
            page.loaded = true
            setProperty("start", "none")
            var chapters = getProperty("chapter-list") || []
            var list = []
            for (var i = 0; i < chapters.length; i++) list.push({ title: String(chapters[i].title || ""), start: Number(chapters[i].time || 0) })
            page.duration = Number(getProperty("duration") || 0)
            if (page.session && page.duration > 0) Door.reportChapters(page.session.session, list, page.duration)
            page.onFileLoaded()                                  // Task 11 picks the tracks here
            tickTimer.start()
        }
        onChanged: function(name, value) {
            if (name === "time-pos") { if (value !== null && value !== undefined) page.timePos = value }
            else if (name === "duration") { if (value) page.duration = value }
            else if (name === "pause") { var was = page.paused; page.paused = !!value; if (was !== page.paused) page.tick() }
            else if (name === "seeking") { var wasSeeking = page.seeking; page.seeking = !!value; if (wasSeeking && !page.seeking) page.tick() }
            else if (name === "eof-reached") { if (value && !page.ended) { page.ended = true; page.onEnded() } }
            else if (name === "hwdec-current") page.hwdec = value ? String(value) : ""
            else if (name === "frame-drop-count") page.drops = Number(value || 0)
            else page.onObserved(name, value)                   // Tasks 11 and 12 read the rest
        }
        MouseArea { anchors.fill: parent; onClicked: page.togglePause(); hoverEnabled: true; onPositionChanged: page.showChrome(); cursorShape: page.chromeVisible ? Qt.ArrowCursor : Qt.BlankCursor }
    }
    function onFileLoaded() {}
    function onObserved(name, value) {}
    function onEnded() { close("Ended") }                        // Task 12 adds the replay and the pill

    // ---- Ticks: once a second while playing, once on pause, once after a seek, once on close
    Timer { id: tickTimer; interval: 1000; repeat: true; running: false; onTriggered: if (!page.paused) page.tick() }
    function tick() { if (session && !closed) Door.tick(session.session, timePos, paused) }

    // ---- Transport
    function togglePause() { video.setProperty("pause", !paused); showChrome() }
    function seekTo(secs) { var t = Math.max(0, Math.min(duration > 0 ? duration : secs, secs)); video.command(["seek", String(t), "absolute"]); showChrome() }
    function setVolume(v) { v = Math.max(0, Math.min(100, v)); video.setProperty("volume", v); if (v > 0 && Player.mute) setMute(false); Player.saveVolume(v); showChrome() }
    function setMute(m) { video.setProperty("mute", m); Player.saveMute(m); showChrome() }
    function toggleFullscreen() { frame.window.visibility = frame.window.visibility === Window.FullScreen ? Window.Windowed : Window.FullScreen }

    // ---- Header
    Rectangle {
        id: header
        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
        height: theme.space(16)
        color: theme.scrim
        opacity: page.chromeVisible ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: theme.motionNormal } }
        Row {
            anchors.left: parent.left; anchors.leftMargin: theme.space(4); anchors.verticalCenter: parent.verticalCenter
            spacing: theme.space(4)
            PlayerButton { glyph: "arrow-left"; tip: "Back"; onClicked: page.leave() }
            Column {
                anchors.verticalCenter: parent.verticalCenter
                Text { text: page.session ? page.session.series_title : ""; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
                Text { text: page.session ? (page.session.episode_title || page.session.path.split("/").pop()) : ""; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold }
                Text { text: page.session ? page.session.code : ""; color: theme.textFaint; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
            }
        }
    }

    // ---- Controls island
    Corner {
        id: controls
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(6)
        width: Math.min(parent.width - theme.space(12), theme.space(220))
        height: bottomRow.height + theme.space(6)                // Task 11 adds the seek row above
        radius: theme.radiusLg; smoothing: theme.cornerSmoothing
        color: theme.scrim; borderColor: theme.line; borderWidth: 1
        opacity: page.chromeVisible ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: theme.motionNormal } }
        MouseArea { anchors.fill: parent; hoverEnabled: true; onPositionChanged: page.showChrome() }
        Item { id: seekSlot; anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: theme.space(3); height: 0 }
        Row {
            id: bottomRow
            anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.margins: theme.space(3)
            height: theme.space(9)
            spacing: theme.space(1)
            PlayerButton { glyph: "skip-back"; enabled: !!(page.session && page.session.prev && !page.session.is_extra); tip: enabled ? "Previous episode" : "No previous episode"; onClicked: page.openNeighbour(page.session.prev) }
            PlayerButton { glyph: page.paused ? "play" : "pause"; tip: page.paused ? "Play" : "Pause"; onClicked: page.togglePause() }
            PlayerButton { glyph: "skip-forward"; enabled: !!(page.session && page.session.next && !page.session.is_extra); tip: enabled ? "Next episode" : "No next episode"; onClicked: page.openNeighbour(page.session.next) }
            Text { anchors.verticalCenter: parent.verticalCenter; text: Fmt.clock(page.timePos) + " / " + Fmt.clock(page.duration); color: theme.text; font.family: theme.fontMono; font.pointSize: theme.typeSmall; leftPadding: theme.space(2); rightPadding: theme.space(2) }
            PlayerButton { glyph: Player.mute || Player.volume === 0 ? "volume-x" : "volume-2"; tip: Player.mute ? "Unmute" : "Mute"; onClicked: page.setMute(!Player.mute) }
            SliderRow { anchors.verticalCenter: parent.verticalCenter; from: 0; to: 100; value: Player.mute ? 0 : Player.volume; stepSize: 1; trackWidth: theme.space(24); onMoved: function(v) { page.setVolume(v) } }
            Item { id: rightSlot; width: parent.width - x - theme.space(1); height: parent.height   // Task 11 and 12 add the pickers, mark, help
                Row { id: rightGroup; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; spacing: theme.space(1)
                    PlayerButton { glyph: frame.window.visibility === Window.FullScreen ? "minimize" : "maximize"; tip: "Fullscreen"; onClicked: page.toggleFullscreen() } } }
        }
    }

    // ---- Keys (the base set; Task 12 completes the map)
    Keys.onPressed: function(e) {
        if (e.isAutoRepeat) { e.accepted = false; return }
        e.accepted = true
        if (e.key === Qt.Key_Space || e.key === Qt.Key_K) page.togglePause()
        else if (e.key === Qt.Key_Left) page.seekTo(page.timePos - 5)
        else if (e.key === Qt.Key_Right && !(e.modifiers & Qt.ControlModifier)) page.seekTo(page.timePos + 5)
        else if (e.key === Qt.Key_M) page.setMute(!Player.mute)
        else if (e.key === Qt.Key_F) page.toggleFullscreen()
        else if (e.key === Qt.Key_Up) page.setVolume(Player.volume + 5)
        else if (e.key === Qt.Key_Down) page.setVolume(Player.volume - 5)
        else if (e.key === Qt.Key_Escape) { if (frame.window.visibility === Window.FullScreen) frame.window.visibility = Window.Windowed; else page.leave() }
        else e.accepted = false
    }
}
```

`Frame.qml`: `player: playerPage` and `Component { id: playerPage; PlayerPage {} }`; the frame already hides the rail and the strip for a page with `fullWindow`. `build.rs`: `PlayerPage`, `PlayerButton`, `"src/bridge/player.rs"`. `bridge/mod.rs`: `pub mod player;`. `main.rs`: `mod player_config;`.

- [ ] **Step 4: Run the tests, then play a file on a monitor**

Run: `cargo test -p anibeam player_config && cargo build --release -p anibeam`
Expected: 3 tests pass; the build links `VideoItem`.

Run: `apps/linux/scripts/bench.sh player 2 keep --root /tmp/sandbox`, open a series, click an episode
Expected: the video plays within a second, hardware decoded (`hwdec` reads `nvdec` on the desktop: add a temporary `Text { text: page.hwdec + " " + page.drops }` in the header to see it, and remove it after), from its resume point when one exists; the header shows the series, the episode title and the code; Previous and Next open the neighbours at zero; click pauses; the time readout runs; the volume slider and mute persist across a relaunch (`~/.config/anibeam/player.toml` under the sandbox root); F and Escape toggle fullscreen; the controls and the cursor hide after 2.5 s. `anibeam-cli --root /tmp/sandbox events --level debug` in a terminal shows one `Tick` a second and `ResumePointChanged` lines while it plays. Leave the player: the last tick and the close arrive, and the series page shows the resume bar on the row.

- [ ] **Step 5: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the video item, the config layers, the playback session and the ticks"
```

---

### Task 11: The seek bar, the preview, tracks and the subtitle defaults

Spec 4.4 (controls, header and the seek bar; subtitle defaults; tracks and track choice). The pick rule is Rust with tests; the pickers, the delay keys and the preview are QML.

**Files:**
- Create: `apps/linux/src/tracks.rs`, `apps/linux/qml/SeekBar.qml`, `apps/linux/qml/TrackPicker.qml`
- Modify: `apps/linux/src/player_config.rs` (`subtitle_options`), `apps/linux/src/bridge/player.rs` (`pickTracks`, `trackLabel`, `trackRef`, `subtitleOptions`), `apps/linux/qml/PlayerPage.qml`, `apps/linux/build.rs`

**Interfaces:**
- Consumes: `PlaybackSession.sidecars`, `subtitle_defaults`, `track_choice`, `skip_windows`; mpv's `track-list`; `Door.setTrackChoice`.
- Produces:
  - `tracks::Track { id: i64, kind: String ("audio"|"sub"|"video"), lang: Option<String>, title: Option<String>, default: bool, external: bool }` parsed from mpv's list; `tracks::Pick { aid: Option<i64>, sid: Option<i64> }`; `tracks::pick(list: &[Track], choice: &TrackChoice, defaults: &SubtitleDefaults) -> Pick`; `tracks::track_ref(t: &Track) -> TrackRef`; `tracks::label(t: &Track) -> String`; `tracks::same_lang(a, b) -> bool` (two and three letter codes are the same).
  - `player_config::subtitle_options(d: &SubtitleDefaults) -> Vec<(&'static str, String)>`.
  - On `Player`: `pickTracks(trackList: array, trackChoice: object, defaults: object) -> { aid, sid }` (`-1` for off), `trackLabel(track) -> string`, `trackRef(track) -> object`, `subtitleOptions(defaults) -> array of [name, value]`.
  - `SeekBar` (`position`, `duration`, `windows`, `signal seeked(real)`, `signal hovered(real)`, `signal unhovered()`); `TrackPicker` (`title`, `tracks`, `selected`, `offRow`, `signal picked(int id)`, `openAt(anchor)`, `close()`).

- [ ] **Step 1: Write the failing tests**

`apps/linux/src/tracks.rs`, tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_core::{SubtitleChoice, SubtitleDefaults, TrackChoice, TrackKind, TrackRef};
    use serde_json::json;

    fn list() -> Vec<Track> {
        parse(&json!([
            { "id": 1, "type": "video" },
            { "id": 1, "type": "audio", "lang": "jpn", "title": "Japanese", "default": true },
            { "id": 2, "type": "audio", "lang": "eng", "title": "English Dub" },
            { "id": 1, "type": "sub", "lang": "eng", "title": "Signs & Songs" },
            { "id": 2, "type": "sub", "lang": "eng", "title": "Full Dialogue" },
            { "id": 3, "type": "sub", "lang": "eng", "title": "English", "external": true, "external-filename": "/x/ep.en.srt" },
            { "id": 4, "type": "sub", "lang": "spa" }
        ]))
    }

    fn defaults() -> SubtitleDefaults { SubtitleDefaults::default() }

    #[test]
    fn language_orders_prefer_a_sidecar_and_dialogue_and_the_first_audio_match() {
        let p = pick(&list(), &TrackChoice::default(), &defaults());
        assert_eq!(p.sid, Some(3), "the sidecar beats the embedded English tracks");
        assert_eq!(p.aid, Some(1), "ja is first in the audio order");
        let no_sidecar: Vec<Track> = list().into_iter().filter(|t| !t.external).collect();
        assert_eq!(pick(&no_sidecar, &TrackChoice::default(), &defaults()).sid, Some(2), "dialogue beats signs");
        let mut d = defaults();
        d.subtitle_languages = vec!["es".into()];
        d.audio_languages = vec!["en".into()];
        let p = pick(&list(), &TrackChoice::default(), &d);
        assert_eq!((p.aid, p.sid), (Some(2), Some(4)), "two letter codes match three letter tags");
    }

    #[test]
    fn a_track_choice_wins_by_kind_language_and_title_then_loosens() {
        let exact = TrackChoice { audio: Some(TrackRef { kind: TrackKind::Embedded, language: Some("eng".into()), title: Some("English Dub".into()) }),
                                  subtitle: Some(SubtitleChoice::Track { track: TrackRef { kind: TrackKind::Embedded, language: Some("en".into()), title: Some("Signs & Songs".into()) } }) };
        let p = pick(&list(), &exact, &defaults());
        assert_eq!((p.aid, p.sid), (Some(2), Some(1)));
        let loose = TrackChoice { audio: None, subtitle: Some(SubtitleChoice::Track { track: TrackRef { kind: TrackKind::Sidecar, language: Some("en".into()), title: Some("gone".into()) } }) };
        assert_eq!(pick(&list(), &loose, &defaults()).sid, Some(3), "kind and language match when the title is gone");
        let lang_only = TrackChoice { audio: None, subtitle: Some(SubtitleChoice::Track { track: TrackRef { kind: TrackKind::Embedded, language: Some("es".into()), title: None } }) };
        assert_eq!(pick(&list(), &lang_only, &defaults()).sid, Some(4));
        let off = TrackChoice { audio: None, subtitle: Some(SubtitleChoice::Off) };
        assert_eq!(pick(&list(), &off, &defaults()).sid, None, "off applies as off");
    }

    #[test]
    fn with_nothing_matching_the_first_subtitle_and_the_default_audio_play() {
        let mut d = defaults();
        d.subtitle_languages = vec!["fr".into()];
        d.audio_languages = vec!["fr".into()];
        let p = pick(&list(), &TrackChoice::default(), &d);
        assert_eq!((p.aid, p.sid), (Some(1), Some(1)));
    }

    #[test]
    fn refs_and_labels() {
        let l = list();
        let r = track_ref(&l[5]);
        assert_eq!(r.kind, TrackKind::Sidecar);
        assert_eq!(r.language.as_deref(), Some("eng"));
        assert_eq!(r.title.as_deref(), Some("English"));
        assert_eq!(label(&l[4]), "Full Dialogue (eng)");
        assert_eq!(label(&l[5]), "English (eng, sidecar)");
        assert_eq!(label(&l[6]), "spa");
        assert!(same_lang("en", "eng") && same_lang("ja", "jpn") && same_lang("EN", "en") && !same_lang("en", "es"));
    }
}
```

`apps/linux/src/player_config.rs`, one more test:

```rust
    #[test]
    fn subtitle_defaults_become_mpv_options() {
        let mut d = anibeam_core::SubtitleDefaults::default();
        let o = subtitle_options(&d);
        let get = |k: &str| o.iter().find(|(n, _)| *n == k).map(|(_, v)| v.clone()).unwrap();
        assert_eq!(get("slang"), "en");
        assert_eq!(get("alang"), "ja");
        assert_eq!(get("sub-scale"), "1");
        assert_eq!(get("sub-ass-override"), "scale");
        assert_eq!(get("sub-font"), "sans-serif");
        assert_eq!(get("sub-color"), "#FFFFFFFF");
        assert_eq!(get("sub-outline-size"), "1.65");
        assert_eq!(get("sub-outline-color"), "#FF000000");
        assert_eq!(get("sub-shadow-offset"), "0");
        assert_eq!(get("sub-bold"), "no");
        assert_eq!(get("sub-pos"), "100");
        assert_eq!(get("sub-border-style"), "outline-and-shadow");
        d.text_style.box_opacity = 0.5;
        d.ass_override = anibeam_core::AssOverride::Force;
        d.subtitle_languages = vec!["en".into(), "ja".into()];
        let o = subtitle_options(&d);
        let get = |k: &str| o.iter().find(|(n, _)| *n == k).map(|(_, v)| v.clone()).unwrap();
        assert_eq!(get("sub-border-style"), "background-box");
        assert_eq!(get("sub-back-color"), "#80000000");
        assert_eq!(get("sub-ass-override"), "force");
        assert_eq!(get("slang"), "en,ja");
    }
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p anibeam tracks player_config`
Expected: compile errors.

- [ ] **Step 3: Write the rule, the options, the bridge and the QML**

`apps/linux/src/tracks.rs` (above its tests):

```rust
//! The track pick, spec 4.4: the series' track choice first (exact kind, language and
//! title; then kind and language; then language), then the language orders (a sidecar
//! beats an embedded track, dialogue beats signs), then the first subtitle track and the
//! file's default audio, so a file never plays unsubbed by accident.

use anibeam_core::{SubtitleChoice, SubtitleDefaults, TrackChoice, TrackKind, TrackRef};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub id: i64,
    pub kind: String,
    pub lang: Option<String>,
    pub title: Option<String>,
    pub default: bool,
    pub external: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pick {
    pub aid: Option<i64>,
    pub sid: Option<i64>,
}

pub fn parse(list: &Value) -> Vec<Track> {
    list.as_array().map(|a| a.iter().filter_map(|t| {
        Some(Track {
            id: t.get("id")?.as_i64()?,
            kind: t.get("type")?.as_str()?.to_string(),
            lang: t.get("lang").and_then(Value::as_str).map(String::from),
            title: t.get("title").and_then(Value::as_str).map(String::from),
            default: t.get("default").and_then(Value::as_bool).unwrap_or(false),
            external: t.get("external").and_then(Value::as_bool).unwrap_or(false),
        })
    }).collect()).unwrap_or_default()
}

const PAIRS: [(&str, &str); 14] = [
    ("en", "eng"), ("ja", "jpn"), ("de", "ger"), ("de", "deu"), ("fr", "fre"), ("fr", "fra"), ("es", "spa"), ("it", "ita"),
    ("pt", "por"), ("ru", "rus"), ("zh", "chi"), ("zh", "zho"), ("ko", "kor"), ("ar", "ara"),
];

fn canon(l: &str) -> String {
    let l = l.to_ascii_lowercase();
    let base = l.split(['-', '_']).next().unwrap_or(&l).to_string();
    PAIRS.iter().find(|(two, three)| base == *three).map(|(two, _)| two.to_string()).unwrap_or(base).replace("und", "")
}

/// Two and three letter codes are the same to mpv, so they are the same here.
pub fn same_lang(a: &str, b: &str) -> bool {
    let (a, b) = (canon(a), canon(b));
    !a.is_empty() && a == b
}

fn lang_matches(t: &Track, lang: &str) -> bool {
    t.lang.as_deref().is_some_and(|l| same_lang(l, lang))
}

fn is_dialogue(t: &Track) -> bool {
    let title = t.title.as_deref().unwrap_or("").to_ascii_lowercase();
    !["signs", "songs", "forced", "commentary"].iter().any(|w| title.contains(w))
}

fn by_ref<'a>(tracks: &'a [Track], kind: &str, r: &TrackRef) -> Option<&'a Track> {
    let of_kind: Vec<&Track> = tracks.iter().filter(|t| t.kind == kind).collect();
    let kind_ok = |t: &Track| match r.kind { TrackKind::Sidecar => t.external, TrackKind::Embedded => !t.external };
    let lang_ok = |t: &Track| match &r.language { Some(l) => lang_matches(t, l), None => t.lang.is_none() };
    let title_ok = |t: &Track| r.title.as_deref().map(|x| x.eq_ignore_ascii_case(t.title.as_deref().unwrap_or(""))).unwrap_or(true);
    of_kind.iter().copied().find(|t| kind_ok(t) && lang_ok(t) && title_ok(t))
        .or_else(|| of_kind.iter().copied().find(|t| kind_ok(t) && lang_ok(t)))
        .or_else(|| r.language.as_deref().and_then(|l| of_kind.iter().copied().find(|t| lang_matches(t, l))))
}

pub fn pick(tracks: &[Track], choice: &TrackChoice, defaults: &SubtitleDefaults) -> Pick {
    let subs: Vec<&Track> = tracks.iter().filter(|t| t.kind == "sub").collect();
    let audio: Vec<&Track> = tracks.iter().filter(|t| t.kind == "audio").collect();

    let sid = match &choice.subtitle {
        Some(SubtitleChoice::Off) => None,
        Some(SubtitleChoice::Track { track }) if by_ref(tracks, "sub", track).is_some() => by_ref(tracks, "sub", track).map(|t| t.id),
        _ => defaults.subtitle_languages.iter().find_map(|lang| {
            let in_lang: Vec<&Track> = subs.iter().copied().filter(|t| lang_matches(t, lang)).collect();
            if in_lang.is_empty() { return None }
            in_lang.iter().copied().find(|t| t.external && is_dialogue(t))
                .or_else(|| in_lang.iter().copied().find(|t| t.external))
                .or_else(|| in_lang.iter().copied().find(|t| is_dialogue(t)))
                .or_else(|| in_lang.first().copied())
                .map(|t| t.id)
        }).or_else(|| subs.first().map(|t| t.id)),
    };

    let aid = choice.audio.as_ref().and_then(|r| by_ref(tracks, "audio", r)).map(|t| t.id)
        .or_else(|| defaults.audio_languages.iter().find_map(|lang| audio.iter().copied().find(|t| lang_matches(t, lang)).map(|t| t.id)))
        .or_else(|| audio.iter().copied().find(|t| t.default).or_else(|| audio.first().copied()).map(|t| t.id));

    Pick { aid, sid }
}

pub fn track_ref(t: &Track) -> TrackRef {
    TrackRef { kind: if t.external { TrackKind::Sidecar } else { TrackKind::Embedded }, language: t.lang.clone(), title: t.title.clone() }
}

/// "Full Dialogue (eng)", "English (eng, sidecar)", "spa", "Track 2".
pub fn label(t: &Track) -> String {
    let mut extra: Vec<&str> = Vec::new();
    if let Some(l) = &t.lang { extra.push(l) }
    if t.external { extra.push("sidecar") }
    match (&t.title, extra.is_empty()) {
        (Some(title), true) => title.clone(),
        (Some(title), false) => format!("{title} ({})", extra.join(", ")),
        (None, false) => extra.join(", "),
        (None, true) => format!("Track {}", t.id),
    }
}
```

`player_config::subtitle_options`:

```rust
fn colour(c: &anibeam_core::Colour) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}", c.a, c.r, c.g, c.b)
}

fn num(v: f64) -> String {
    if v.fract() == 0.0 { format!("{}", v as i64) } else { format!("{v}") }
}

/// Every field of SubtitleDefaults as one mpv option, spec 4.4's table.
pub fn subtitle_options(d: &anibeam_core::SubtitleDefaults) -> Vec<(&'static str, String)> {
    use anibeam_core::AssOverride;
    let s = &d.text_style;
    let mut o = vec![
        ("slang", d.subtitle_languages.join(",")),
        ("alang", d.audio_languages.join(",")),
        ("sub-scale", num(d.scale)),
        ("sub-ass-override", match d.ass_override { AssOverride::AsScripted => "no", AssOverride::ScaleOnly => "scale", AssOverride::Force => "force" }.to_string()),
        ("sub-font", s.font.clone()),
        ("sub-color", colour(&s.colour)),
        ("sub-outline-size", num(s.outline_size)),
        ("sub-outline-color", colour(&s.outline_colour)),
        ("sub-shadow-offset", num(s.shadow_offset)),
        ("sub-bold", if s.bold { "yes" } else { "no" }.to_string()),
        ("sub-pos", num(s.position)),
    ];
    if s.box_opacity > 0.0 {
        o.push(("sub-border-style", "background-box".to_string()));
        o.push(("sub-back-color", format!("#{:02X}000000", (s.box_opacity.clamp(0.0, 1.0) * 255.0).round() as u8)));
    } else {
        o.push(("sub-border-style", "outline-and-shadow".to_string()));
        o.push(("sub-back-color", "#00000000".to_string()));
    }
    o
}
```

`bridge/player.rs` additions, inside the `extern "RustQt"` block:

```rust
        #[qinvokable] fn pick_tracks(self: &Self, track_list: &QJsonArray, track_choice: &QJsonObject, defaults: &QJsonObject) -> QJsonObject;
        #[qinvokable] fn track_label(self: &Self, track: &QJsonObject) -> QString;
        #[qinvokable] fn track_ref(self: &Self, track: &QJsonObject) -> QJsonObject;
        #[qinvokable] fn subtitle_options(self: &Self, defaults: &QJsonObject) -> QJsonArray;
```

and the impls:

```rust
    pub fn pick_tracks(&self, track_list: &QJsonArray, track_choice: &QJsonObject, defaults: &QJsonObject) -> QJsonObject {
        let list = crate::tracks::parse(&Value::Array(track_list.iter().map(|v| crate::json::from_qjson(&v)).collect()));
        let choice: TrackChoice = serde_json::from_value(crate::json::from_qjson_object(track_choice)).unwrap_or_default();
        let d: SubtitleDefaults = serde_json::from_value(crate::json::from_qjson_object(defaults)).unwrap_or_default();
        let p = crate::tracks::pick(&list, &choice, &d);
        crate::json::to_qjson_object(&serde_json::json!({ "aid": p.aid.unwrap_or(-1), "sid": p.sid.unwrap_or(-1) }))
    }
    pub fn track_label(&self, track: &QJsonObject) -> QString {
        let list = crate::tracks::parse(&Value::Array(vec![crate::json::from_qjson_object(track)]));
        QString::from(&list.first().map(crate::tracks::label).unwrap_or_default())
    }
    pub fn track_ref(&self, track: &QJsonObject) -> QJsonObject {
        let list = crate::tracks::parse(&Value::Array(vec![crate::json::from_qjson_object(track)]));
        let r = list.first().map(crate::tracks::track_ref);
        crate::json::to_qjson_object(&serde_json::to_value(r).unwrap_or(Value::Null))
    }
    pub fn subtitle_options(&self, defaults: &QJsonObject) -> QJsonArray {
        let d: SubtitleDefaults = serde_json::from_value(crate::json::from_qjson_object(defaults)).unwrap_or_default();
        pairs(player_config::subtitle_options(&d).into_iter().map(|(k, v)| (k.to_string(), v)))
    }
```

(with `use anibeam_core::{SubtitleDefaults, TrackChoice}; use serde_json::Value;` and `QJsonObject` in the bridge's extern block, and `mod tracks;` in `main.rs`).

`apps/linux/qml/SeekBar.qml`:

```qml
// The seek bar: the played portion, an amber intro band and a teal outro band where skip
// windows are known, a hover position for the preview. Dragging seeks on release.
import QtQuick

Item {
    id: root
    property real position: 0
    property real duration: 0
    property var windows: []
    property real hoverAt: -1
    signal seeked(real secs)
    signal hovered(real secs)
    signal unhovered()
    height: theme.space(5)
    readonly property real played: duration > 0 ? Math.min(1, position / duration) : 0
    function at(x) { return duration > 0 ? Math.max(0, Math.min(duration, x / width * duration)) : 0 }

    Corner { id: track; anchors.verticalCenter: parent.verticalCenter; width: parent.width; height: theme.space(1.25); radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken; borderColor: theme.line; borderWidth: 1 }
    Repeater {
        model: root.windows
        Corner {
            required property var modelData
            anchors.verticalCenter: parent.verticalCenter
            x: root.duration > 0 ? modelData.start / root.duration * root.width : 0
            width: root.duration > 0 ? Math.max(2, (modelData.end - modelData.start) / root.duration * root.width) : 0
            height: track.height; radius: height / 2; smoothing: theme.cornerSmoothing
            color: modelData.kind === "Intro" ? theme.yellow : theme.cyan
            opacity: 0.7
        }
    }
    Corner { anchors.verticalCenter: parent.verticalCenter; width: track.width * root.played; height: track.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.accent }
    Corner { x: track.width * root.played - width / 2; anchors.verticalCenter: parent.verticalCenter; width: theme.space(3.5); height: width; radius: width / 2; smoothing: theme.cornerSmoothing; color: theme.accent; borderColor: theme.bg; borderWidth: theme.space(0.5); visible: mouse.containsMouse || mouse.pressed }
    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        onPositionChanged: function(m) { root.hoverAt = root.at(m.x); root.hovered(root.hoverAt) }
        onExited: { root.hoverAt = -1; root.unhovered() }
        onClicked: function(m) { root.seeked(root.at(m.x)) }
    }
}
```

`apps/linux/qml/TrackPicker.qml`:

```qml
// A track picker: Off (for subtitles) plus every track, the current one in the accent.
// Opens above its anchor; a pick closes it; Escape closes it through the frame.
import QtQuick

Item {
    id: root
    property string title: ""
    property var tracks: []
    property int selected: -1
    property bool offRow: false
    property bool open: false
    signal picked(int id)
    anchors.fill: parent
    visible: open
    z: 900
    function openAt(anchor) {
        var p = anchor.mapToItem(root, anchor.width / 2, 0)
        panel.x = Math.max(theme.space(2), Math.min(p.x - panel.width / 2, root.width - panel.width - theme.space(2)))
        panel.y = p.y - panel.height - theme.space(2)
        open = true
        page.openMenus++
        frame.escapeStack.push("popover", root)
    }
    function close() { if (!open) return; open = false; page.openMenus--; frame.escapeStack.pop(root); page.showChrome() }
    MouseArea { anchors.fill: parent; onPressed: root.close() }
    Corner {
        id: panel
        width: theme.space(70); height: Math.min(theme.space(80), column.implicitHeight + theme.space(4))
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing
        color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
        MouseArea { anchors.fill: parent }
        Flickable {
            anchors.fill: parent; anchors.margins: theme.space(2); clip: true
            contentHeight: column.implicitHeight
            Column {
                id: column
                width: parent.width
                Text { text: root.title; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall; font.weight: Font.DemiBold; height: theme.space(6); verticalAlignment: Text.AlignVCenter; leftPadding: theme.space(2) }
                Repeater {
                    model: (root.offRow ? [{ id: -1, label: "Off" }] : []).concat(root.tracks)
                    Corner {
                        required property var modelData
                        width: column.width; height: theme.controlHeight
                        radius: theme.radiusSm; smoothing: theme.cornerSmoothing
                        color: m.containsMouse ? theme.surfacePressed : "transparent"
                        Text { anchors.left: parent.left; anchors.leftMargin: theme.space(2); anchors.verticalCenter: parent.verticalCenter; text: modelData.label; color: modelData.id === root.selected ? theme.accent : theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: modelData.id === root.selected ? Font.DemiBold : Font.Normal; elide: Text.ElideRight; width: parent.width - theme.space(4) }
                        MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { root.picked(modelData.id); root.close() } }
                    }
                }
            }
        }
    }
}
```

`PlayerPage.qml` additions:

1. State: `property var trackList: []`, `property int aid: -1`, `property int sid: -1`, `property int lastSid: -1`, `property real subDelay: 0`, `property var windows: session ? session.skip_windows : []`.
2. `applyDefaults()`:

```qml
    function applyDefaults() {
        var opts = Player.subtitleOptions(session.subtitle_defaults)
        for (var i = 0; i < opts.length; i++) video.setProperty(opts[i][0], opts[i][1])
        for (var j = 0; j < session.sidecars.length; j++) {
            var s = session.sidecars[j]
            video.command(["sub-add", s.path, "auto", s.title || "", s.language || ""])
        }
    }
    Connections { target: Door; function onSettingsChanged() { if (page.session) { var r = Door.getSettings(); if (!r.error) { page.session.subtitle_defaults = r.reply.settings.subtitle_defaults; var opts = Player.subtitleOptions(page.session.subtitle_defaults); for (var i = 0; i < opts.length; i++) video.setProperty(opts[i][0], opts[i][1]) } } } }
```

3. `onFileLoaded()`:

```qml
    function onFileLoaded() {
        trackList = video.getProperty("track-list") || []
        var p = Player.pickTracks(trackList, session.track_choice, session.subtitle_defaults)
        video.setProperty("aid", p.aid >= 0 ? String(p.aid) : "no")
        video.setProperty("sid", p.sid >= 0 ? String(p.sid) : "no")
        if (p.sid >= 0) lastSid = p.sid
    }
    function onObserved(name, value) {
        if (name === "track-list") trackList = value || []
        else if (name === "aid") aid = value === "no" || value === null ? -1 : Number(value)
        else if (name === "sid") { sid = value === "no" || value === null ? -1 : Number(value); if (sid >= 0) lastSid = sid }
        else if (name === "sub-delay") subDelay = Number(value || 0)
        else if (name === "chapter-list") {}
        else onObservedMore(name, value)                      // Task 12
    }
    function onObservedMore(name, value) {}
    readonly property var audioTracks: trackList.filter(function(t) { return t.type === "audio" }).map(function(t) { return { id: t.id, label: Player.trackLabel(t), track: t } })
    readonly property var subTracks: trackList.filter(function(t) { return t.type === "sub" }).map(function(t) { return { id: t.id, label: Player.trackLabel(t), track: t } })
    function pickAudio(id) {
        video.setProperty("aid", id >= 0 ? String(id) : "no")
        var t = trackList.find(function(x) { return x.type === "audio" && x.id === id })
        Door.setTrackChoice(session.series, t ? Player.trackRef(t) : {}, session.track_choice.subtitle ? session.track_choice.subtitle : {})
    }
    function pickSubtitle(id) {
        video.setProperty("sid", id >= 0 ? String(id) : "no")
        var t = trackList.find(function(x) { return x.type === "sub" && x.id === id })
        var choice = id < 0 ? { off: true } : { Track: { track: Player.trackRef(t) } }
        Door.setTrackChoice(session.series, session.track_choice.audio ? session.track_choice.audio : {}, choice)
        session.track_choice.subtitle = id < 0 ? "Off" : choice
    }
    function toggleSubtitles() { if (sid >= 0) video.setProperty("sid", "no"); else if (lastSid >= 0) video.setProperty("sid", String(lastSid)); else if (subTracks.length) video.setProperty("sid", String(subTracks[0].id)); showChrome() }
```

`Door.setTrackChoice` takes `{}` for none and `{ off: true }` for Off (the door of Task 6 maps that to `SubtitleChoice::Off`); the session's own `track_choice.subtitle` keeps serde's shape, the string `"Off"` or `{ Track: { track } }`.

4. The seek row and the preview: replace `seekSlot`'s `height: 0` with the bar and lift `controls.height`:

```qml
        Item { id: seekSlot; anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: theme.space(3); height: seek.height
            SeekBar { id: seek; width: parent.width; position: page.timePos; duration: page.duration; windows: page.windows
                onSeeked: function(s) { page.seekTo(s) }
                onHovered: function(s) { preview.show(s, seek.mapToItem(page, seek.hoverAt / page.duration * seek.width, 0).x) }
                onUnhovered: preview.hide() } }
        height: bottomRow.height + seekSlot.height + theme.space(9)
```

and the preview item, a sibling of `video` below the controls in z order:

```qml
    // The seek preview: a second mpv core, nothing audible, moved by time-pos
    Corner {
        id: preview
        visible: false
        width: theme.space(60); height: width * 9 / 16 + theme.space(6)
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing
        color: theme.scrim; borderColor: theme.line; borderWidth: 1
        y: controls.y - height - theme.space(2)
        property bool loaded: false
        function show(secs, centerX) { x = Math.max(theme.space(2), Math.min(centerX - width / 2, page.width - width - theme.space(2))); stamp.text = Fmt.clock(secs); visible = true; if (loaded) previewVideo.setPropertyAsync("time-pos", secs) }
        function hide() { visible = false }
        VideoItem {
            id: previewVideo
            anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: theme.space(1)
            height: width * 9 / 16
            onReady: { var o = Player.previewOptions; for (var i = 0; i < o.length; i++) setProperty(o[i][0], o[i][1]); if (page.session) command(["loadfile", page.session.path]) }
            onLoaded: preview.loaded = true
        }
        Text { id: stamp; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(1); anchors.horizontalCenter: parent.horizontalCenter; color: theme.text; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
    }
```

5. The pickers, in `rightGroup` before the fullscreen button:

```qml
                    PlayerButton { id: audioBtn; glyph: "audio-lines"; tip: "Audio track"; visible: page.audioTracks.length > 1; onClicked: audioPicker.openAt(audioBtn) }
                    PlayerButton { id: subBtn; glyph: "captions"; tip: page.sid >= 0 ? "Subtitles" : "Subtitles off"; active: page.sid >= 0; visible: page.subTracks.length > 0; onClicked: subPicker.openAt(subBtn) }
```

and the two pickers at the page level: `TrackPicker { id: audioPicker; title: "Audio"; tracks: page.audioTracks; selected: page.aid; onPicked: function(id) { page.pickAudio(id) } }` and `TrackPicker { id: subPicker; title: "Subtitles"; tracks: page.subTracks; selected: page.sid; offRow: true; onPicked: function(id) { page.pickSubtitle(id) } }`.

6. Keys: in `Keys.onPressed` add `else if (e.key === Qt.Key_C && !(e.modifiers & Qt.ControlModifier)) page.toggleSubtitles()`, `else if (e.key === Qt.Key_Z && !(e.modifiers & Qt.ShiftModifier)) page.nudgeDelay(-0.1)`, `else if (e.key === Qt.Key_Z) page.nudgeDelay(0.1)` with

```qml
    function nudgeDelay(d) { var v = Math.round((subDelay + d) * 10) / 10; video.setProperty("sub-delay", v); hud.flash("subtitle delay " + (v >= 0 ? "+" : "") + v.toFixed(1) + " s"); showChrome() }
```

and a HUD the frame step shares in Task 12:

```qml
    Corner {
        id: hud
        visible: false
        anchors.top: parent.top; anchors.topMargin: theme.space(20); anchors.horizontalCenter: parent.horizontalCenter
        width: hudText.implicitWidth + theme.space(6); height: theme.controlHeight
        radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.scrim; borderColor: theme.line; borderWidth: 1
        Text { id: hudText; anchors.centerIn: parent; color: theme.text; font.family: theme.fontMono; font.pointSize: theme.typeNormal }
        Timer { id: hudTimer; interval: 1200; onTriggered: hud.visible = false }
        function flash(text) { hudText.text = text; visible = true; hudTimer.restart() }
        function clear() { visible = false; hudTimer.stop() }
    }
```

`build.rs`: `SeekBar`, `TrackPicker`; `main.rs`: `mod tracks;`.

- [ ] **Step 4: Run the tests, then play on a monitor**

Run: `cargo test -p anibeam tracks player_config`
Expected: 8 tests pass.

Run: `cargo build --release -p anibeam && apps/linux/scripts/bench.sh player-tracks 2 keep --root /tmp/sandbox`, open an episode with an ASS track and an English sidecar beside it
Expected: the sidecar plays over the embedded track; the subtitle picker lists Off plus every track with labels like `Full Dialogue (eng)`; picking one changes the subtitle at once and the next episode of the same series opens with the same pick (the series' track choice); the audio picker shows when there are two audio tracks; C toggles subtitles off and back; z and Z move the delay with the HUD line; hovering the bar shows the preview frame with its timestamp, moving as the pointer moves, without a dropped frame in the main picture; the intro and outro bands appear where `skip_windows` exist (a file with OP and ED chapters shows them on the first open; otherwise after the AniSkip answer arrives, Task 12 wires `skipWindowsReady`).

- [ ] **Step 5: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the seek bar, the preview item, the track pick and the subtitle defaults"
```

---

### Task 12: Skips, auto-next, completion, the rating prompt, frame step, the key map and help

Spec 4.4: skip windows and auto-skip; completion, resume, auto-next and the tick; frame step; the keyboard map; `?`. Plus Mark watched and the passing notices. Everything here is QML over the signals the door already emits.

**Files:**
- Modify: `apps/linux/qml/PlayerPage.qml`
- Create: `apps/linux/qml/KeyHelp.qml`, `apps/linux/qml/Notice.qml`

**Interfaces:**
- Consumes: `Door.skipWindowsReady`, `Door.marked`, `Door.viewed`, `Door.scored`, `Door.markEpisode`, `Door.setScore`, `Door.settings.auto_skip`, `ScorePicker`.
- Produces: the finished player page.

- [ ] **Step 1: Write the additions**

`apps/linux/qml/Notice.qml` (a passing notice with an optional button, used for Skipped / Undo and the tracker outcomes):

```qml
import QtQuick
Corner {
    id: root
    property string text: ""
    property string action: ""
    signal acted()
    visible: false
    anchors.horizontalCenter: parent.horizontalCenter
    anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(30)
    width: row.implicitWidth + theme.space(6); height: theme.controlHeight
    radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.scrim; borderColor: theme.line; borderWidth: 1
    Row { id: row; anchors.centerIn: parent; spacing: theme.space(3)
        Text { anchors.verticalCenter: parent.verticalCenter; text: root.text; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
        Button { visible: root.action !== ""; text: root.action; small: true; onClicked: { root.hide(); root.acted() } } }
    Timer { id: t; interval: 4000; onTriggered: root.visible = false }
    function show(text, action, seconds) { root.text = text; root.action = action || ""; visible = true; t.interval = (seconds || 4) * 1000; t.restart() }
    function hide() { visible = false; t.stop() }
}
```

`apps/linux/qml/KeyHelp.qml`:

```qml
// The key list, from the checklist and the player ticket. Escape or a click outside closes it.
import QtQuick
Item {
    id: root
    property bool open: false
    anchors.fill: parent; visible: open; z: 950
    readonly property var keys: [
        ["Space / K", "Play or pause"], ["Left / Right", "Seek 5 s"], ["Ctrl+Right", "Skip the intro or outro, else 90 s"],
        [", / .", "One frame back or forward"], ["M", "Mute"], ["F", "Fullscreen"], ["C", "Subtitles off and back"],
        ["z / Z", "Subtitle delay 100 ms earlier or later"], ["Up / Down", "Volume 5"], ["Escape", "Leave the player"], ["?", "This list"]
    ]
    function show() { open = true; page.openMenus++; frame.escapeStack.push("popover", root) }
    function close() { if (!open) return; open = false; page.openMenus--; frame.escapeStack.pop(root); page.showChrome() }
    MouseArea { anchors.fill: parent; onPressed: root.close() }
    Corner {
        anchors.centerIn: parent
        width: theme.space(100); height: column.implicitHeight + theme.space(10)
        radius: theme.radiusLg; smoothing: theme.cornerSmoothing; color: theme.surfaceRaised; borderColor: theme.lineStrong; borderWidth: 1
        MouseArea { anchors.fill: parent }
        Column {
            id: column
            anchors.centerIn: parent; width: parent.width - theme.space(10); spacing: theme.space(2)
            Text { text: "Keyboard shortcuts"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold }
            Repeater { model: root.keys; Row { required property var modelData; spacing: theme.space(3); width: column.width
                Chip { text: modelData[0]; small: true; color: theme.surface; textColor: theme.text; anchors.verticalCenter: parent.verticalCenter }
                Text { text: modelData[1]; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal; anchors.verticalCenter: parent.verticalCenter } } }
        }
    }
}
```

`PlayerPage.qml` additions:

1. Skip windows and auto-skip:

```qml
    property bool autoIntroArmed: true
    property bool autoOutroArmed: true
    property real lastPos: -1
    readonly property var intro: windows.find(function(w) { return w.kind === "Intro" }) || null
    readonly property var outro: windows.find(function(w) { return w.kind === "Outro" }) || null
    function inside(w, t) { return w && t >= w.start && t < w.end }
    Connections { target: Door; function onSkipWindowsReady(session, ws) { if (page.session && session === page.session.session) page.windows = ws } }
    function skipWindow(w) { seekTo(w.end + 1) }
    // Entering a window by playback or by the opening resume point fires the auto-skip; a
    // seek the user makes into the window does not (lastPos jumps by more than 2 s).
    onTimePosChanged: {
        var moved = lastPos >= 0 && Math.abs(timePos - lastPos) <= 2
        var settings = Door.settings.auto_skip || {}
        if (intro && settings.intro && autoIntroArmed && inside(intro, timePos) && (moved || lastPos < 0)) { autoIntroArmed = false; skipWindow(intro); notice.show("Skipped intro", "Undo") ; noticeUndo = function() { seekTo(intro.start); autoIntroArmed = false } }
        else if (outro && settings.outro && autoOutroArmed && inside(outro, timePos) && (moved || lastPos < 0)) { autoOutroArmed = false; skipWindow(outro); notice.show("Skipped outro", "Undo"); noticeUndo = function() { seekTo(outro.start); autoOutroArmed = false } }
        lastPos = timePos
        updateNext()
    }
    property var noticeUndo: null
    Notice { id: notice; parent: page; onActed: if (page.noticeUndo) page.noticeUndo() }
```

The Skip buttons, on the controls' bottom row before `rightSlot`: `Button { visible: page.inside(page.intro, page.timePos); text: "Skip Intro"; small: true; onClicked: page.skipWindow(page.intro) }` and the same for `Skip Outro`. Ctrl+Right: `else if (e.key === Qt.Key_Right && (e.modifiers & Qt.ControlModifier)) { if (inside(intro, timePos)) skipWindow(intro); else if (inside(outro, timePos)) skipWindow(outro); else seekTo(Math.min(duration, timePos + 90)) }`.

2. Auto-next, Stay, the replay button, completion:

```qml
    property bool nextVisible: false
    property bool nextCounting: false
    property bool nextDismissed: false
    function updateNext() {
        if (!session || !session.next || session.is_extra || nextDismissed || !loaded) { nextVisible = false; return }
        var remaining = duration - timePos
        var show = outro ? timePos >= outro.start : remaining <= 8
        var count = (outro && (duration - outro.end) < 20 && timePos >= outro.end) || remaining <= 3
        nextVisible = show
        if (count && !nextCounting) { nextCounting = true; nextTimer.start() }
    }
    Timer { id: nextTimer; interval: 5000; onTriggered: page.openNeighbour(page.session.next) }
    function stay() { nextDismissed = true; nextVisible = false; nextCounting = false; nextTimer.stop() }
    function onEnded() { close("Ended"); if (!nextCounting) replay.visible = true }
    Row {
        visible: page.nextVisible
        anchors.right: parent.right; anchors.bottom: controls.top; anchors.margins: theme.space(6)
        spacing: theme.space(2)
        Button { text: "Stay"; flat: true; onClicked: page.stay() }
        Corner {
            width: nextLabel.implicitWidth + theme.space(8); height: theme.controlHeight
            radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.surfaceRaised; borderColor: theme.accent; borderWidth: 1
            Corner { id: fill; width: 0; height: parent.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.accentSoft
                NumberAnimation on width { running: page.nextCounting; from: 0; to: fill.parent.width; duration: 5000 } }
            Text { id: nextLabel; anchors.centerIn: parent; text: "Next"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: page.openNeighbour(page.session.next) }
        }
    }
    PlayerButton {
        id: replay
        visible: false
        anchors.centerIn: parent
        glyph: "rotate-ccw"; tip: "Replay"
        width: theme.space(16)
        onClicked: frame.nav.replace("player", { file: page.props.file }, page.title)
    }
```

(`updateNext()` is called from `onTimePosChanged` above; `onEnded` replaces Task 10's stub; a replay opens a fresh session, since the core closed this one with `Ended`.)

3. Mark watched, the tracker notices and the final-episode rating prompt. The session carries the series id and the file id, not the episode number or the tracker ids, so the page reads the series detail once; this goes into Task 10's existing `Component.onCompleted`, right after `session = r.reply.session`:

```qml
        var d = Door.getSeries(session.series)
        if (!d.error) {
            var card = d.reply.detail.card
            trackerKnown = !!(card.match_info && (card.match_info.anilist_id || card.match_info.mal_id))
            seriesTitles = card.titles || {}
            var ep = d.reply.detail.episodes.find(function(e) { return e.file === session.file })
            episodeNumber = ep ? ep.number : -1
        }
```

with these page properties and functions:

```qml
    property bool trackerKnown: false
    property var seriesTitles: ({})
    property real episodeNumber: -1
    function markWatched() { if (!session || session.is_extra || episodeNumber < 0) return; var r = Door.markEpisode(session.series, episodeNumber); if (r.error) notice.show(r.error.message) }
    Connections {
        target: Door
        function onMarked(series, episode, outcomes) {
            if (!page.session || series !== page.session.series) return
            var ok = outcomes.filter(function(o) { return o.ok }).map(function(o) { return o.tracker + " " + (o.progress !== null && o.progress !== undefined ? "at " + o.progress : "ok") })
            var bad = outcomes.filter(function(o) { return !o.ok }).map(function(o) { return o.tracker + " " + (o.reason || o.message || "failed") })
            notice.show(ok.length ? "Tracked  " + ok.join("  ") + (bad.length ? "  " + bad.join("  ") : "") : "Tracker error  " + bad.join("  "))
            if (page.session.is_last_episode && ok.length) rating.visible = true
        }
        function onScored(series, score, outcomes) { if (page.session && series === page.session.series) notice.show(outcomes.every(function(o) { return o.ok }) ? "Rated " + Fmt.score(score) : "Score failed") }
    }
    Corner {
        id: rating
        visible: false
        anchors.horizontalCenter: parent.horizontalCenter; anchors.top: parent.top; anchors.topMargin: theme.space(24)
        width: ratingRow.implicitWidth + theme.space(8); height: theme.space(14)
        radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.scrim; borderColor: theme.line; borderWidth: 1
        Row { id: ratingRow; anchors.centerIn: parent; spacing: theme.space(3)
            Icon { glyph: "check-check"; size: theme.space(4); anchors.verticalCenter: parent.verticalCenter }
            Text { anchors.verticalCenter: parent.verticalCenter; text: "Tracked  final episode  rate this show?"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
            Button { id: rateBtn; text: "Submit"; small: true; onClicked: ratingPicker.openAt(rateBtn, -1) }
            Button { text: "Skip"; small: true; flat: true; onClicked: rating.visible = false } }
    }
    ScorePicker { id: ratingPicker; parent: page; onSaved: function(v) { rating.visible = false; var r = Door.setScore(page.session.series, v); if (r.error) notice.show(r.error.message) } }
```

The Mark watched button in `rightGroup`: `PlayerButton { glyph: "check-check"; tip: "Mark watched"; visible: page.trackerKnown && page.session && !page.session.is_extra; onClicked: page.markWatched() }`.

4. Frame step, the HUD and the help:

```qml
    property real frameNumber: 0
    function onObservedMore(name, value) { if (name === "estimated-frame-number") { frameNumber = Number(value || 0); if (stepping) hud.flash(Fmt.clockMs(timePos) + "  frame " + frameNumber) } }
    property bool stepping: false
    function step(dir) {
        if (!paused) video.setProperty("pause", true)
        stepping = true
        video.command([dir > 0 ? "frame-step" : "frame-back-step"])
        hud.flash(Fmt.clockMs(timePos) + "  frame " + frameNumber)
    }
    onPausedChanged: if (!paused) { stepping = false; hud.clear() }
    KeyHelp { id: help }
```

(`onPausedChanged` and `onTimePosChanged` each exist once on the page; Task 13 appends lines to them rather than declaring them again, since QML allows one handler per signal on an object.)

```qml
```

with, in `rightGroup`: `PlayerButton { glyph: "circle-question-mark"; tip: "Keyboard shortcuts"; onClicked: help.show() }`.

5. The whole key handler, replacing Task 10's:

```qml
    Keys.onPressed: function(e) {
        // Frame step sits above the auto-repeat guard so a held key keeps stepping
        if ((e.key === Qt.Key_Period || e.key === Qt.Key_Comma) && !(e.modifiers & (Qt.ControlModifier | Qt.AltModifier))) { e.accepted = true; page.step(e.key === Qt.Key_Period ? 1 : -1); return }
        if (e.isAutoRepeat) { e.accepted = false; return }
        e.accepted = true
        var ctrl = e.modifiers & Qt.ControlModifier
        if (e.key === Qt.Key_Space || e.key === Qt.Key_K) page.togglePause()
        else if (e.key === Qt.Key_Left) page.seekTo(page.timePos - 5)
        else if (e.key === Qt.Key_Right && ctrl) { if (page.inside(page.intro, page.timePos)) page.skipWindow(page.intro); else if (page.inside(page.outro, page.timePos)) page.skipWindow(page.outro); else page.seekTo(Math.min(page.duration, page.timePos + 90)) }
        else if (e.key === Qt.Key_Right) page.seekTo(page.timePos + 5)
        else if (e.key === Qt.Key_M) page.setMute(!Player.mute)
        else if (e.key === Qt.Key_F) { page.toggleFullscreen(); return }                     // F does not bring the chrome back
        else if (e.key === Qt.Key_C && !ctrl) page.toggleSubtitles()
        else if (e.key === Qt.Key_Z) page.nudgeDelay(e.modifiers & Qt.ShiftModifier ? 0.1 : -0.1)
        else if (e.key === Qt.Key_Up) page.setVolume(Player.volume + 5)
        else if (e.key === Qt.Key_Down) page.setVolume(Player.volume - 5)
        else if (e.key === Qt.Key_Question) help.show()
        else if (e.key === Qt.Key_Escape) {
            if (help.open) help.close()
            else if (subPicker.open) subPicker.close()
            else if (audioPicker.open) audioPicker.close()
            else if (frame.window.visibility === Window.FullScreen) frame.window.visibility = Window.Windowed
            else page.leave()
        }
        else e.accepted = false
    }
```

(`page.escape()` from the frame's stack is unreachable in practice because the page consumes Escape itself; keep it returning `true` after `leave()`.)

- [ ] **Step 2: Play through the behaviours on a monitor**

Run: `cargo build --release -p anibeam && apps/linux/scripts/bench.sh player-full 2 keep --root /tmp/sandbox`, with Auto-skip intro on in the sandbox's settings (`anibeam-cli --root /tmp/sandbox call SetAutoSkip --json '{"intro":true,"outro":false}'` before launching)
Expected, in order: opening an episode with an OP chapter jumps past the intro with the "Skipped intro" notice, Undo seeks back and the intro plays, and reopening the episode auto-skips again; Skip Intro and Skip Outro appear only inside their windows and land a second past the end; Ctrl+Right inside a window skips it and outside jumps 90 s; at the outro the Next pill appears, fills for five seconds and advances, and Stay latches it off; at the end of an episode with no next the replay button appears and replays from zero; `,` and `.` step frames with the HUD reading `m:ss.mmm  frame N`, held keys keep stepping, and play clears it; Mark watched shows the tracker notice; the last episode of a series shows the rating prompt after the mark; `?` shows the list and Escape closes it before anything else; F leaves the controls hidden.

Run: `anibeam-cli --root /tmp/sandbox events --level info` alongside
Expected: `Viewed` after 30 s of real playback, `Marked` at the outro or 85 percent, and the series page behind shows Next up moved on.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): skips and auto-skip, auto-next, the rating prompt, frame step and the key map"
```

---
### Task 13: Single instance, the application bus name, MPRIS and the now-playing lines

Spec 4.5 (single instance) and 4.4 (MPRIS), on 5.1's D-Bus owner. The guarantee is a flock under `$XDG_RUNTIME_DIR`; the bus is only the way to raise. MPRIS lives on the same connection through mpris-server, with `DesktopEntry` set to the app id. The two now-playing lines are Rust with tests, carried from `src/shared/nowPlaying.ts`.

**Files:**
- Create: `apps/linux/src/dbus/mod.rs`, `apps/linux/src/dbus/instance.rs`, `apps/linux/src/dbus/mpris.rs`, `apps/linux/src/nowplaying.rs`
- Modify: `apps/linux/src/main.rs` (the lock, the hand-off), `apps/linux/src/bridge/shell.rs` (`activateRequested`, `raiseWindow`, `quitRequested`), `apps/linux/src/bridge/player.rs` (the MPRIS invokables and the command signal), `apps/linux/cpp/helpers.h`, `apps/linux/cpp/helpers.cpp` (`raise_window`), `apps/linux/src/bridge/helpers.rs`, `apps/linux/qml/Main.qml`, `apps/linux/qml/PlayerPage.qml`

**Interfaces:**
- Consumes: `paths.lock_path()`, `runtime::runtime()`, `CxxQtThread<Shell>`, `CxxQtThread<Player>`, mpris-server 0.10, zbus 5.
- Produces:
  - `instance::Lock` (held for the life of the process), `instance::try_lock(path) -> io::Result<Option<Lock>>`, `instance::hand_off(action: Option<&str>) -> Result<(), String>` (async: `Activate` or `ActivateAction` on the running instance with the launcher's `XDG_ACTIVATION_TOKEN`), `instance::AppInterface` (serves `org.freedesktop.Application`), `instance::serve(conn: &zbus::Connection, shell: CxxQtThread<Shell>) -> zbus::Result<()>`.
  - `mpris::State { status, title, artist, art_url, length_secs, position_secs, volume, can_next, can_prev }`, `mpris::MprisPlayer` implementing `RootInterface` and `PlayerInterface`, `mpris::start(player: CxxQtThread<Player>, shell: CxxQtThread<Shell>) -> Option<mpris::Handle>` (async; `None` without a bus, after one line on stderr), `Handle::update(State)` and `Handle::seeked(secs)`.
  - `nowplaying::lines(show: &str, episode_number: Option<u32>, episode_title: Option<&str>, extra_label: Option<&str>) -> (String, String)`, `nowplaying::is_real_episode_title(title, show, number) -> bool`, `nowplaying::art_url(path: &str) -> String`.
  - `Shell` gains the signal `activateRequested(token)`, the invokable `raiseWindow(window, token)`, and the signal `quitRequested()`.
  - `Player` gains `mprisUpdate(status, title, artist, artUrl, lengthSecs, canNext, canPrev)`, `mprisPosition(secs)`, `mprisSeeked(secs)` and the signal `mprisCommand(name, value)` with names `next`, `previous`, `play`, `pause`, `playPause`, `stop`, `seek` (value: the offset in seconds), `setPosition` (value: seconds), `setVolume` (value: 0 to 100).

- [ ] **Step 1: Write the failing tests**

`apps/linux/src/nowplaying.rs`, tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_real_episode_title_is_the_title_line() {
        assert_eq!(lines("Dungeon Meshi", Some(12), Some("Red Dragon"), None), ("Red Dragon".into(), "Dungeon Meshi · Episode 12".into()));
    }

    #[test]
    fn a_bare_token_or_the_show_name_is_not_a_title() {
        for t in ["Episode 5", "Ep 5", "Ep. 5", "E05", "5", "#5", "Dungeon Meshi", "Dungeon Meshi - Episode 5", "Dungeon Meshi: E05", ""] {
            assert!(!is_real_episode_title(t, "Dungeon Meshi", Some(5)), "{t}");
            assert_eq!(lines("Dungeon Meshi", Some(5), Some(t), None), ("Dungeon Meshi".into(), "Episode 5".into()));
        }
        assert!(is_real_episode_title("Episode 6", "Dungeon Meshi", Some(5)), "a different number is a title");
    }

    #[test]
    fn extras_and_films() {
        assert_eq!(lines("Girls und Panzer", None, None, Some("Opening 1")), ("Opening 1".into(), "Girls und Panzer".into()));
        assert_eq!(lines("Koe no Katachi", None, None, None), ("Koe no Katachi".into(), String::new()));
    }

    #[test]
    fn the_art_url_is_a_file_url_with_escaped_segments() {
        assert_eq!(art_url("/home/b/.cache/anibeam/images/ab/x y.jpg"), "file:///home/b/.cache/anibeam/images/ab/x%20y.jpg");
        assert_eq!(art_url("/a/#b/c.png"), "file:///a/%23b/c.png");
    }
}
```

`apps/linux/src/dbus/instance.rs`, tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_second_lock_on_one_file_is_refused_until_the_first_drops() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("anibeam.lock");
        let first = try_lock(&path).unwrap();
        assert!(first.is_some());
        assert!(try_lock(&path).unwrap().is_none(), "held");
        drop(first);
        assert!(try_lock(&path).unwrap().is_some(), "free again");
    }

    #[test]
    fn platform_data_carries_the_token_when_set() {
        let d = platform_data(Some("tok123"));
        assert_eq!(d.get("activation-token").and_then(|v| v.downcast_ref::<String>().ok()), Some("tok123".to_string()));
        assert!(platform_data(None).is_empty());
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p anibeam nowplaying instance`
Expected: compile errors.

- [ ] **Step 3: Write the modules, the bridge additions and the QML**

`apps/linux/src/nowplaying.rs` (above its tests):

```rust
//! The two MPRIS lines, carried from Electron's `src/shared/nowPlaying.ts`. A title counts
//! as a name unless it is empty, the show's name, a bare episode token, or the show's name
//! followed by separators and such a token.

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

const SEP: &str = " · ";

fn fold(s: &str) -> String {
    s.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// "Episode 5", "Ep 5", "Ep. 5", "E05", "5", "#5", with the number equal to `number`.
fn is_episode_token(s: &str, number: Option<u32>) -> bool {
    let s = s.trim();
    let rest = ["episode", "ep.", "ep", "e", "#"].iter().find_map(|p| s.strip_prefix(p)).unwrap_or(s).trim_start();
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    match number {
        Some(n) => rest.parse::<u32>().ok() == Some(n),
        None => true,
    }
}

pub fn is_real_episode_title(title: &str, show: &str, number: Option<u32>) -> bool {
    let t = fold(title);
    if t.is_empty() {
        return false;
    }
    let s = fold(show);
    if !s.is_empty() && t == s {
        return false;
    }
    let rest = if !s.is_empty() && t.starts_with(&s) {
        t[s.len()..].trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '\u{2013}' | '\u{2014}' | ':' | '_')).to_string()
    } else {
        t
    };
    !is_episode_token(&rest, number)
}

/// (title, artist).
pub fn lines(show: &str, episode_number: Option<u32>, episode_title: Option<&str>, extra_label: Option<&str>) -> (String, String) {
    let show = show.trim().to_string();
    if let Some(extra) = extra_label.map(str::trim).filter(|s| !s.is_empty()) {
        return (extra.to_string(), show);
    }
    let Some(n) = episode_number else { return (show, String::new()) };
    let episode = format!("Episode {n}");
    if let Some(t) = episode_title.map(str::trim).filter(|s| !s.is_empty()) {
        if is_real_episode_title(t, &show, Some(n)) {
            let artist = if show.is_empty() { episode } else { format!("{show}{SEP}{episode}") };
            return (t.to_string(), artist);
        }
    }
    (show, episode)
}

const PATH_SEGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'#').add(b'%').add(b'<').add(b'>').add(b'?').add(b'[').add(b']').add(b'^').add(b'`').add(b'{').add(b'|').add(b'}');

pub fn art_url(path: &str) -> String {
    let segments: Vec<String> = path.split('/').map(|s| utf8_percent_encode(s, PATH_SEGMENT).to_string()).collect();
    format!("file://{}", segments.join("/"))
}
```

`apps/linux/src/dbus/mod.rs`: `pub mod instance; pub mod mpris;`.

`apps/linux/src/dbus/instance.rs` (above its tests):

```rust
//! Single instance, spec 4.5: a flock under $XDG_RUNTIME_DIR before anything else; a
//! second launch that loses it hands its activation token to the running window over
//! org.freedesktop.Application and exits. The core knows nothing of any of this.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::Path;

use cxx_qt::CxxQtThread;
use cxx_qt_lib::QString;
use rustix::fs::{flock, FlockOperation};
use zbus::zvariant::{OwnedValue, Value};

use crate::bridge::shell::qobject::Shell;

pub const BUS_NAME: &str = "com.marcusrosado.AniBeam";
pub const OBJECT_PATH: &str = "/com/marcusrosado/AniBeam";

pub struct Lock(#[allow(dead_code)] File);

pub fn try_lock(path: &Path) -> std::io::Result<Option<Lock>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path)?;
    match flock(&file, FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => Ok(Some(Lock(file))),
        Err(rustix::io::Errno::WOULDBLOCK) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn platform_data(token: Option<&str>) -> HashMap<String, Value<'static>> {
    let mut m = HashMap::new();
    if let Some(t) = token.filter(|t| !t.is_empty()) {
        m.insert("activation-token".to_string(), Value::from(t.to_string()));
    }
    m
}

/// The second launch's whole job: raise the running window, then exit 0.
pub async fn hand_off(action: Option<&str>) -> Result<(), String> {
    let conn = zbus::Connection::session().await.map_err(|e| format!("no session bus: {e}"))?;
    let token = std::env::var("XDG_ACTIVATION_TOKEN").ok();
    let data = platform_data(token.as_deref());
    let result = match action {
        Some(name) => conn.call_method(Some(BUS_NAME), OBJECT_PATH, Some("org.freedesktop.Application"), "ActivateAction", &(name, Vec::<Value>::new(), &data)).await,
        None => conn.call_method(Some(BUS_NAME), OBJECT_PATH, Some("org.freedesktop.Application"), "Activate", &(&data,)).await,
    };
    result.map(|_| ()).map_err(|e| format!("could not reach the running AniBeam: {e}"))
}

pub struct AppInterface {
    shell: CxxQtThread<Shell>,
}

impl AppInterface {
    fn raise(&self, platform_data: &HashMap<String, OwnedValue>) {
        let token = platform_data.get("activation-token").and_then(|v| v.downcast_ref::<String>().ok()).unwrap_or_default();
        self.shell.queue(move |shell| shell.activate_requested(QString::from(&token))).ok();
    }
}

#[zbus::interface(name = "org.freedesktop.Application")]
impl AppInterface {
    fn activate(&self, platform_data: HashMap<String, OwnedValue>) {
        self.raise(&platform_data);
    }
    /// The app opens nothing from the launcher: Open behaves as Activate.
    fn open(&self, _uris: Vec<String>, platform_data: HashMap<String, OwnedValue>) {
        self.raise(&platform_data);
    }
    /// The action table is empty today; an unknown action raises the window.
    fn activate_action(&self, action_name: String, _parameter: Vec<OwnedValue>, platform_data: HashMap<String, OwnedValue>) {
        eprintln!("anibeam: no action named {action_name}");
        self.raise(&platform_data);
    }
}

pub async fn serve(conn: &zbus::Connection, shell: CxxQtThread<Shell>) -> zbus::Result<()> {
    conn.object_server().at(OBJECT_PATH, AppInterface { shell }).await?;
    conn.request_name(BUS_NAME).await?;
    Ok(())
}
```

`apps/linux/src/dbus/mpris.rs`:

```rust
//! MPRIS through mpris-server on the connection that also carries the application name.
//! State comes from the player page through the Player singleton; commands go back to it
//! as one Qt signal.

use std::sync::{Arc, Mutex};

use cxx_qt::CxxQtThread;
use cxx_qt_lib::QString;
use mpris_server::{zbus::{fdo, Result as ZResult}, LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, Property, RootInterface, Server, Signal, Time, TrackId, Volume};

use crate::bridge::shell::qobject::Shell;

#[derive(Clone, Debug)]
pub struct State {
    pub status: PlaybackStatus,
    pub title: String,
    pub artist: String,
    pub art_url: Option<String>,
    pub length_secs: f64,
    pub position_secs: f64,
    pub volume: f64,
    pub can_next: bool,
    pub can_prev: bool,
}

impl Default for State {
    fn default() -> Self {
        State { status: PlaybackStatus::Stopped, title: String::new(), artist: String::new(), art_url: None, length_secs: 0.0, position_secs: 0.0, volume: 1.0, can_next: false, can_prev: false }
    }
}

pub struct MprisPlayer {
    state: Arc<Mutex<State>>,
    shell: CxxQtThread<Shell>,
}

impl MprisPlayer {
    /// The Player singleton registers its thread handle when the QML engine constructs it
    /// (Main.qml touches it at start); until then a command has nowhere to go.
    fn send(&self, name: &str, value: f64) {
        let Some(player) = crate::bridge::player::thread() else { return };
        let name = name.to_string();
        player.queue(move |p| p.mpris_command(QString::from(&name), value)).ok();
    }
    fn state(&self) -> State {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }
    fn metadata(&self) -> Metadata {
        let s = self.state();
        let mut b = Metadata::builder().trackid(TrackId::NO_TRACK).title(s.title.clone()).artist([s.artist.clone()]).length(Time::from_millis((s.length_secs * 1000.0) as i64));
        if let Some(url) = s.art_url { b = b.art_url(url) }
        b.build()
    }
}

impl RootInterface for MprisPlayer {
    async fn raise(&self) -> fdo::Result<()> { self.shell.queue(|s| s.activate_requested(QString::default())).ok(); Ok(()) }
    async fn quit(&self) -> fdo::Result<()> { self.shell.queue(|s| s.quit_requested()).ok(); Ok(()) }
    async fn can_quit(&self) -> fdo::Result<bool> { Ok(true) }
    async fn fullscreen(&self) -> fdo::Result<bool> { Ok(false) }
    async fn set_fullscreen(&self, _f: bool) -> ZResult<()> { Ok(()) }
    async fn can_set_fullscreen(&self) -> fdo::Result<bool> { Ok(false) }
    async fn can_raise(&self) -> fdo::Result<bool> { Ok(true) }
    async fn has_track_list(&self) -> fdo::Result<bool> { Ok(false) }
    async fn identity(&self) -> fdo::Result<String> { Ok("AniBeam".into()) }
    async fn desktop_entry(&self) -> fdo::Result<String> { Ok(crate::APP_ID.into()) }
    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> { Ok(vec![]) }
    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> { Ok(vec![]) }
}

impl PlayerInterface for MprisPlayer {
    async fn next(&self) -> fdo::Result<()> { self.send("next", 0.0); Ok(()) }
    async fn previous(&self) -> fdo::Result<()> { self.send("previous", 0.0); Ok(()) }
    async fn pause(&self) -> fdo::Result<()> { self.send("pause", 0.0); Ok(()) }
    async fn play_pause(&self) -> fdo::Result<()> { self.send("playPause", 0.0); Ok(()) }
    async fn stop(&self) -> fdo::Result<()> { self.send("stop", 0.0); Ok(()) }
    async fn play(&self) -> fdo::Result<()> { self.send("play", 0.0); Ok(()) }
    async fn seek(&self, offset: Time) -> fdo::Result<()> { self.send("seek", offset.as_micros() as f64 / 1e6); Ok(()) }
    async fn set_position(&self, _track: TrackId, position: Time) -> fdo::Result<()> { self.send("setPosition", position.as_micros() as f64 / 1e6); Ok(()) }
    async fn open_uri(&self, _uri: String) -> fdo::Result<()> { Ok(()) }
    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> { Ok(self.state().status) }
    async fn loop_status(&self) -> fdo::Result<LoopStatus> { Ok(LoopStatus::None) }
    async fn set_loop_status(&self, _l: LoopStatus) -> ZResult<()> { Ok(()) }
    async fn rate(&self) -> fdo::Result<PlaybackRate> { Ok(1.0) }
    async fn set_rate(&self, _r: PlaybackRate) -> ZResult<()> { Ok(()) }
    async fn shuffle(&self) -> fdo::Result<bool> { Ok(false) }
    async fn set_shuffle(&self, _s: bool) -> ZResult<()> { Ok(()) }
    async fn metadata(&self) -> fdo::Result<Metadata> { Ok(self.metadata()) }
    async fn volume(&self) -> fdo::Result<Volume> { Ok(self.state().volume) }
    async fn set_volume(&self, v: Volume) -> ZResult<()> { self.send("setVolume", (v * 100.0).clamp(0.0, 100.0)); Ok(()) }
    async fn position(&self) -> fdo::Result<Time> { Ok(Time::from_millis((self.state().position_secs * 1000.0) as i64)) }
    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> { Ok(1.0) }
    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> { Ok(1.0) }
    async fn can_go_next(&self) -> fdo::Result<bool> { Ok(self.state().can_next) }
    async fn can_go_previous(&self) -> fdo::Result<bool> { Ok(self.state().can_prev) }
    async fn can_play(&self) -> fdo::Result<bool> { Ok(true) }
    async fn can_pause(&self) -> fdo::Result<bool> { Ok(true) }
    async fn can_seek(&self) -> fdo::Result<bool> { Ok(self.state().length_secs > 0.0) }
    async fn can_control(&self) -> fdo::Result<bool> { Ok(true) }
}

#[derive(Clone)]
pub struct Handle {
    state: Arc<Mutex<State>>,
    server: Arc<Server<MprisPlayer>>,
}

impl Handle {
    pub fn update(&self, next: State) {
        if let Ok(mut s) = self.state.lock() { *s = next.clone(); }
        let server = self.server.clone();
        let md = server.imp().metadata();
        crate::runtime::runtime().spawn(async move {
            server.properties_changed([
                Property::PlaybackStatus(next.status), Property::Metadata(md), Property::Volume(next.volume),
                Property::CanGoNext(next.can_next), Property::CanGoPrevious(next.can_prev), Property::CanSeek(next.length_secs > 0.0),
            ]).await.ok();
        });
    }
    pub fn position(&self, secs: f64) {
        if let Ok(mut s) = self.state.lock() { s.position_secs = secs; }
    }
    pub fn seeked(&self, secs: f64) {
        self.position(secs);
        let server = self.server.clone();
        crate::runtime::runtime().spawn(async move { server.emit(Signal::Seeked { position: Time::from_millis((secs * 1000.0) as i64) }).await.ok(); });
    }
}

static HANDLE: std::sync::OnceLock<Handle> = std::sync::OnceLock::new();
pub fn install(h: Handle) { HANDLE.set(h).ok(); }
pub fn handle() -> Option<Handle> { HANDLE.get().cloned() }

/// Builds the MPRIS server, then serves org.freedesktop.Application on its connection and
/// requests the app id there. None means no session bus: one line on stderr, no media keys.
pub async fn start(shell: CxxQtThread<Shell>) -> Option<Handle> {
    let state = Arc::new(Mutex::new(State::default()));
    let imp = MprisPlayer { state: state.clone(), shell: shell.clone() };
    let server = match Server::new("anibeam", imp).await {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("anibeam: no session bus, MPRIS and media keys are off: {e}");
            return None;
        }
    };
    if let Err(e) = super::instance::serve(server.connection(), shell).await {
        eprintln!("anibeam: could not own {}: {e}", super::instance::BUS_NAME);
    }
    Some(Handle { state, server })
}
```

`bridge/shell.rs` additions: in the bridge's first extern block `include!("cxx-qt-lib/qobject.h"); type QObject = cxx_qt_lib::QObject;`; in the `extern "RustQt"` block:

```rust
        /// A second launch, or MPRIS Raise, asked for the window; the token is the launcher's.
        #[qsignal] fn activate_requested(self: Pin<&mut Self>, token: QString);
        #[qsignal] fn quit_requested(self: Pin<&mut Self>);
        /// Raises `window` (the QML Window) with the xdg-activation token.
        #[qinvokable] unsafe fn raise_window(self: &Self, window: *mut QObject, token: &QString);
```

plus `impl cxx_qt::Threading for Shell {}` and `impl cxx_qt::Initialize for Shell {}`; the impls:

```rust
impl cxx_qt::Initialize for qobject::Shell {
    fn initialize(self: Pin<&mut Self>) {
        // The bus name and MPRIS start once the Shell exists, which is once the engine loads.
        let shell = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            if let Some(h) = crate::dbus::mpris::start(shell).await {
                crate::dbus::mpris::install(h)
            }
        });
    }
}

impl qobject::Shell {
    pub unsafe fn raise_window(&self, window: *mut cxx_qt_lib::QObject, token: &QString) {
        crate::bridge::helpers::ffi::raise_window(window, token);
    }
}
```

`bridge/player.rs`: `impl cxx_qt::Threading for Player {}` and `impl cxx_qt::Initialize for Player {}` in the bridge, a static `THREAD: OnceLock<CxxQtThread<Player>>` that `initialize` fills (`THREAD.set(self.qt_thread()).ok()`), `pub fn thread() -> Option<CxxQtThread<Player>> { THREAD.get().cloned() }`, and the MPRIS invokables:

```rust
        #[qinvokable] fn mpris_update(self: &Self, status: &QString, title: &QString, artist: &QString, art_url: &QString, length_secs: f64, can_next: bool, can_prev: bool);
        #[qinvokable] fn mpris_position(self: &Self, secs: f64);
        #[qinvokable] fn mpris_seeked(self: &Self, secs: f64);
        #[qsignal] fn mpris_command(self: Pin<&mut Self>, name: QString, value: f64);
```

with `mpris::install(handle)` storing the `Handle` in a `OnceLock` and the invokables forwarding when it is set:

```rust
    pub fn mpris_update(&self, status: &QString, title: &QString, artist: &QString, art_url: &QString, length_secs: f64, can_next: bool, can_prev: bool) {
        let Some(h) = crate::dbus::mpris::handle() else { return };
        let status = match status.to_string().as_str() { "Playing" => PlaybackStatus::Playing, "Paused" => PlaybackStatus::Paused, _ => PlaybackStatus::Stopped };
        let art = art_url.to_string();
        h.update(State { status, title: title.to_string(), artist: artist.to_string(), art_url: if art.is_empty() { None } else { Some(art) }, length_secs, position_secs: 0.0, volume: *self.volume() / 100.0, can_next, can_prev });
    }
    pub fn mpris_position(&self, secs: f64) { if let Some(h) = crate::dbus::mpris::handle() { h.position(secs) } }
    pub fn mpris_seeked(&self, secs: f64) { if let Some(h) = crate::dbus::mpris::handle() { h.seeked(secs) } }
```

The now-playing lines cross the same way:

```rust
        #[qinvokable] fn now_playing(self: &Self, show: &QString, episode_number: i32, episode_title: &QString, extra_label: &QString) -> QStringList;
        #[qinvokable] fn art_url(self: &Self, path: &QString) -> QString;
```

```rust
    pub fn now_playing(&self, show: &QString, episode_number: i32, episode_title: &QString, extra_label: &QString) -> QStringList {
        let (t, a) = crate::nowplaying::lines(&show.to_string(), if episode_number >= 0 { Some(episode_number as u32) } else { None },
            Some(&episode_title.to_string()).filter(|s| !s.is_empty()), Some(&extra_label.to_string()).filter(|s| !s.is_empty()));
        QStringList::from_iter([QString::from(&t), QString::from(&a)])
    }
    pub fn art_url(&self, path: &QString) -> QString { QString::from(&crate::nowplaying::art_url(&path.to_string())) }
```

`Main.qml` touches the singleton at start so it exists before any MPRIS command arrives: `readonly property real bootVolume: Player.volume`.

`cpp/helpers.h` and `.cpp`:

```cpp
#include <QtCore/QObject>
// Raises a window with the xdg-activation token a launcher or a second launch passed.
void raise_window(QObject *window, const QString &token);
```

```cpp
#include <QtGui/QWindow>

void raise_window(QObject *window, const QString &token)
{
    auto *w = qobject_cast<QWindow *>(window);
    if (!w)
        return;
    if (!token.isEmpty())
        qputenv("XDG_ACTIVATION_TOKEN", token.toUtf8());
    w->show();
    w->raise();
    w->requestActivate();
}
```

and in `bridge/helpers.rs`: `include!("cxx-qt-lib/qobject.h"); type QObject = cxx_qt_lib::QObject;` and `unsafe fn raise_window(window: *mut QObject, token: &QString);`.

`main.rs`, after the paths are resolved and before the core opens:

```rust
    let lock = match dbus::instance::try_lock(&paths.lock_path()) {
        Ok(Some(lock)) => lock,
        Ok(None) => {
            let result = runtime::runtime().block_on(dbus::instance::hand_off(args.action.as_deref()));
            match result {
                Ok(()) => std::process::exit(0),
                Err(e) => {
                    eprintln!("anibeam: another AniBeam is running and {e}");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("anibeam: lock {}: {e}", paths.lock_path().display());
            std::process::exit(2);
        }
    };
```

and keep `lock` alive until the end of `main` (`drop(lock)` after `shutdown`). `mod dbus; mod nowplaying;`.

`Main.qml`: `Connections { target: Shell; function onActivateRequested(token) { Shell.raiseWindow(window, token) } function onQuitRequested() { Qt.quit() } }`.

`PlayerPage.qml`: the MPRIS wiring, using the series' romaji title from the `GetSeries` reply the page already fetched (`titles.romaji || titles.english || titles.folder`):

```qml
    function episodeNumberOf() { return session.is_extra || episodeNumber < 0 ? -1 : Math.round(episodeNumber) }
    function publishNowPlaying() {
        if (!session) return
        var show = seriesTitles.romaji || seriesTitles.english || seriesTitles.folder || session.series_title
        var lines = Player.nowPlaying(show, episodeNumberOf(), session.episode_title || "", session.is_extra ? session.episode_title || session.code : "")
        Player.mprisUpdate(paused ? "Paused" : "Playing", lines[0], lines[1], session.artwork ? Player.artUrl(session.artwork) : "", duration, !!session.next && !session.is_extra, !!session.prev && !session.is_extra)
    }
    onDurationChanged: publishNowPlaying()
    onSessionChanged: publishNowPlaying()
    // appended to the page's existing handlers: `publishNowPlaying()` at the end of
    // `onPausedChanged`, `Player.mprisPosition(timePos)` at the end of `onTimePosChanged`,
    // and the line below at the end of `Component.onDestruction`
    Connections {
        target: Player
        function onMprisCommand(name, value) {
            if (name === "next") page.openNeighbour(page.session.next)
            else if (name === "previous") page.openNeighbour(page.session.prev)
            else if (name === "play") video.setProperty("pause", false)
            else if (name === "pause") video.setProperty("pause", true)
            else if (name === "playPause") page.togglePause()
            else if (name === "stop") page.leave()
            else if (name === "seek") { page.seekTo(page.timePos + value); Player.mprisSeeked(page.timePos + value) }
            else if (name === "setPosition") { page.seekTo(value); Player.mprisSeeked(value) }
            else if (name === "setVolume") page.setVolume(value)
        }
    }
    Player.mprisUpdate("Stopped", "", "", "", 0, false, false)
```

(`seriesTitles` is the property Task 12 fills from the `GetSeries` reply.)

- [ ] **Step 4: Run the tests and prove the three mechanisms**

Run: `cargo test -p anibeam nowplaying instance`
Expected: 6 tests pass.

Run: `cargo build --release -p anibeam && apps/linux/scripts/bench.sh instance 2 keep`, then in a terminal `target/release/anibeam; echo exit=$?`
Expected: the second launch prints nothing, exits 0, and the running window is raised or marked urgent (Hyprland's `misc:focus_on_activate` decides which). `busctl --user list | grep marcusrosado` shows the name owned once.

Run: `env -u DBUS_SESSION_BUS_ADDRESS target/release/anibeam; echo exit=$?`
Expected: one line on stderr and exit 1.

Run: play an episode, then `busctl --user introspect org.mpris.MediaPlayer2.anibeam /org/mpris/MediaPlayer2` and `playerctl -p anibeam metadata`
Expected: `DesktopEntry` reads `com.marcusrosado.AniBeam`; the title and artist follow the spec's rule (an episode with a real title shows it as the title and `Series · Episode N` as the artist); `mpris:artUrl` is a `file://` URL; `playerctl -p anibeam play-pause`, `next`, `position 30` act on the player; the desktop's media keys pause and resume.

- [ ] **Step 5: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): single instance over a flock, the app bus name, MPRIS and the now-playing lines"
```

---

### Task 14: The status strip and the activity log drawer

Spec 4.5 (the status strip and the activity log) and 4.1 unit 10. The strip is wired since Task 7; this task adds the drawer: the core's event stream at Info and above, filtered by stage and level, consecutive identical events folded with a count, rows expandable, Copy, Clear, Close, and the unseen-errors count that clears when the drawer opens.

**Files:**
- Create (copied from the prototype and edited): `apps/linux/qml/ActivityDrawer.qml`
- Modify: `apps/linux/qml/Frame.qml` (`drawerSlot` becomes the drawer, `toggleDrawer()`), `apps/linux/build.rs`

**Interfaces:**
- Consumes: `Door.recentEvents(2000)`, `Door.event`, `Door.clearEvents()`, `Door.markLogSeen()`, `Door.unseenErrors`.
- Produces: `ActivityDrawer` with `open`, `toggle()`, `close()`, `entries`; `frame.toggleDrawer()`.

- [ ] **Step 1: Write the drawer**

Copy `spikes/home-grid-qml/qml/ActivityDrawer.qml` and make these edits:

1. The stage and level lists become the contract's, shown in lower case:

```qml
    readonly property var stages: ["Library", "Metadata", "Trackers", "Franchise", "Playback", "Store", "System"]
    readonly property var levels: ["Info", "Warn", "Error"]
```

and every place the prototype compares `e.stage`/`e.level` against the lists keeps working since the entries below carry the contract's names; the chips render `modelData.toLowerCase()`.

2. Entries come from the door. Replace `property var entries: []` with:

```qml
    property var entries: []
    property bool backfilled: false
    function backfill() {
        var r = Door.recentEvents(2000)
        if (r.error) return
        entries = r.reply.events.slice().reverse().map(toEntry)
        backfilled = true
    }
    function toEntry(e) { return { time: Qt.formatTime(new Date(e.at * 1000), "hh:mm:ss"), stage: e.stage, level: e.level, msg: e.message, seq: e.seq } }
    Connections {
        target: Door
        function onEvent(envelope) {
            if (envelope.level === "Debug") return
            if (!root.backfilled) return
            root.entries = [root.toEntry(envelope)].concat(root.entries).slice(0, 2000)
        }
    }
```

(`recentEvents` returns oldest first; the drawer lists newest first. The run-folding in `rows` keys on `stage`, `level` and `msg` as before.)

3. `toggle()` and `close()` register on the frame's escape stack and mark the log seen:

```qml
    function toggle() { open ? close() : show() }
    function show() { if (!backfilled) backfill(); open = true; forceActiveFocus(); Door.markLogSeen(); frame.escapeStack.push("drawer", root) }
    function close() { if (!open) return; open = false; frame.escapeStack.pop(root); closed() }
```

and remove the prototype's `Keys.onEscapePressed` (the frame handles Escape).

4. Clear calls the core and empties the list: the Clear button's handler becomes `onClicked: { Door.clearEvents(); root.entries = [] }`. Copy stays the hidden `TextEdit` route.

5. `levelColor` compares against `"Error"` and `"Warn"`.

`Frame.qml`: replace `drawerSlot` with

```qml
        ActivityDrawer {
            id: drawer
            visible: !frame.fullWindow && openness > 0.001
            anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: strip.top
            maxHeight: Math.round(page.height * 0.6)
        }
```

and `function toggleDrawer() { drawer.toggle() }`. The strip's `onClicked` and Ctrl+L already call it. `build.rs`: `ActivityDrawer`.

- [ ] **Step 2: Build and capture**

Run: `cargo build -p anibeam && ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh drawer --page library`, with `Main.qml`'s shoot path opening the drawer when `Shell.page` ends in `:drawer` (split on `:`; the suffix `drawer` calls `frame.item.toggleDrawer()` before the grab)
Expected: the drawer over the library at sixty percent of the content height: the stage chips `library`, `metadata`, `trackers`, `franchise`, `playback`, `store`, `system`, the level chips, Copy, Clear and Close at the top right, the Ready line and the scan lines from the sandbox's launch with their times, identical consecutive lines folded with a `+N` count.

Run on a monitor: click the strip, press Escape, press Ctrl+L, click a folded row, click Copy and paste into a terminal, click Clear, and let a scan run (`anibeam-cli` cannot write while the shell runs; use Settings later, or touch a file under the sandbox's source to trigger the watcher)
Expected: the drawer rises and falls with the smoothing; Escape closes it before anything else; the folded row expands; the clipboard holds one line per event; Clear empties the drawer and the core's ring; the error count on the strip goes to zero when the drawer opens and counts new errors after it closes.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the activity log drawer over the status strip"
```

---
### Task 15: The settings page, the Library tab and the trackers

Spec 4.5 (the settings page, Library) and 4.1 unit 6. The layout is the prototype's `qml/SettingsPage.qml`, split into files and fed by the core: four tabs in a segmented switch with Lucide icons, two panel columns filling the viewport, a panel per column that grows, scrolling only when the natural height exceeds the viewport. The inline components the prototype kept inside `SettingsPage.qml` become files.

**Files:**
- Create: `apps/linux/qml/SettingsPage.qml`, `apps/linux/qml/SettingsTab.qml`, `apps/linux/qml/SettingsPair.qml`, `apps/linux/qml/Note.qml`, `apps/linux/qml/Tiles.qml`, `apps/linux/qml/UsageBar.qml`, `apps/linux/qml/SourceRow.qml`, `apps/linux/qml/TrackerRow.qml`, `apps/linux/qml/SettingsLibraryTab.qml`, plus placeholders `SettingsAppearanceTab.qml`, `SettingsPlaybackTab.qml`, `SettingsDataTab.qml` (each a `SettingsTab` with one `Note { text: "Task N" }` until its task)
- Modify: `apps/linux/qml/Frame.qml` (`settings: settingsPage`, `property int settingsTab: 0`, `property var settingsScroll: [0, 0, 0, 0]`), `apps/linux/build.rs`

**Interfaces:**
- Consumes: `Door.listSources`, `addSource`, `removeSource`, `scan`, `revealHidden`, `trackers`, `setTrackerCredentials`, `connectTracker`, `disconnectTracker`, `setMainTracker`, `cancelJob`, `listSeries`, the `sourceChanged`, `sourceRemoved`, `scanFinished`, `authUrlReady`, `trackerConnected`, `trackersChanged`, `jobFinished` signals; `InlineConfirm`; `QtQuick.Dialogs.FolderDialog`.
- Produces: `SettingsPage` (`props.tab` optional: `library`, `appearance`, `playback`, `data`), `SettingsTab` (the scrolling tab base with `footInset`, `viewport`, the centred block), `SettingsPair` (`split`, `left`, `right` slots, `twoUp`), `Note`, `Tiles` (`tiles: [{ value, caption }]`), `UsageBar` (`parts: [{ label, value, color }]`), `SourceRow` (`source`, signals `open()`, `rescan()`, `remove()`), `TrackerRow` (`tracker` (`"Anilist"` or `"Mal"`), `account`, `main`, signals `login()`, `disconnect()`).

- [ ] **Step 1: Write the page, the pieces and the Library tab**

From `spikes/home-grid-qml/qml/SettingsPage.qml`, lift these inline components into their own files, each unchanged except as noted: `Note` into `Note.qml`; `Tiles` into `Tiles.qml`; `UsageBar` into `UsageBar.qml`; `Tab` into `SettingsTab.qml` (it keeps `footInset`, `viewport`, the `blockX`/`blockWidth` cap at `theme.space(560)`, the focus sink, the scroll bar); `Pair` into `SettingsPair.qml` (it keeps `split`, `twoUp`, the `grows` propagation); `SourceRow` into `SourceRow.qml`; `TrackerRow` into `TrackerRow.qml`. `Confirm` is not lifted: `InlineConfirm.qml` from Task 7 replaces it, with the same `accepted` and `kept` signals. The corner glyph `Component` moves into the Appearance tab (Task 16).

`SourceRow.qml` edits: the row takes `property var source` (a `Source`: `id`, `path`, `available`, `series_count`, `movie_folders`); the path text binds `source.path`; the Unavailable chip shows when `!source.available`; the meta line reads `Fmt.plural(source.series_count, "series", "series") + (source.movie_folders.length ? " · " + Fmt.plural(source.movie_folders.length, "movie folder", "movie folders") : "")`; the three flat buttons emit `open()`, `rescan()` and `remove()`; the inline confirm is owned by the tab (below), so the row exposes `property bool confirming: false` and shows an `InlineConfirm { question: "Remove " + source.path.split("/").pop() + ", " + Fmt.plural(source.series_count, "series", "series") + " and their history?"; onAccepted: root.removeAccepted(); onKept: root.confirming = false }` in place of its buttons while `confirming`; add `signal removeAccepted()`.

`TrackerRow.qml` edits: `property string tracker` and `property var account` (a `TrackerAccount`); the name reads `tracker === "Anilist" ? "AniList" : "MyAnimeList"`; the initials `AL` / `MAL`; the connection line reads `account.connected ? "Connected as " + (account.username || "?") + " · synced " + (account.last_sync ? Fmt.relative(account.last_sync, Date.now() / 1000) : "never") : "Not connected"`; the control slot shows, in this order: while `waiting` a `Note { text: "Waiting for browser authorization…" }` and a `Button { text: "Cancel"; flat: true; onClicked: root.cancel() }`; else when `account.connected` a `Button { text: "Disconnect"; icon: "log-out"; onClicked: root.confirming = true }` giving way to `InlineConfirm { question: "Disconnect " + name + "? Your access token will be removed."; confirmText: "Disconnect"; confirmIcon: "log-out"; onAccepted: root.disconnect(); onKept: root.confirming = false }`; else when `account.bundled_credentials || account.client_id !== ""` a `Button { text: "Log in to " + name; icon: "log-in"; onClicked: root.login("", "") }`; else the credentials block: a `Note` with the register help (AniList: `Create a new client. Paste the redirect URL below into AniList's "Redirect URL" field exactly, port and trailing /callback included.`; MAL: `Create an app (App Type: "Web"). Paste the redirect URL below into MAL's "App Redirect URL" field.`), a row `Text { text: "Redirect URL" }` with `Chip { text: root.redirectUrl; clickable: true; onClicked: { clipboard.text = root.redirectUrl; clipboard.selectAll(); clipboard.copy(); copied.restart() } }` (a hidden `TextEdit { id: clipboard }` and a `Timer { id: copied; interval: 1200 }` that flips the chip's icon from `copy` to `check` while running), a `Field { id: clientId; placeholder: "Client ID"; mono: true }`, for MAL a `Field { id: secret; placeholder: "Client Secret"; mono: true }`, and `Button { text: "Connect"; icon: "log-in"; onClicked: root.login(clientId.text, secret.text) }`. Signals: `login(string clientId, string clientSecret)`, `disconnect()`, `cancel()`; properties `waiting: false`, `confirming: false`, `redirectUrl: "http://127.0.0.1:53682/callback"` (the core's `DEFAULT_OAUTH_PORT`, 53682, and its `/callback` path; the `authUrlReady` event carries the same URL once a connect starts).

`apps/linux/qml/SettingsPage.qml`:

```qml
// Spec 4.5: Settings as four tabs in a segmented switch, each two panel columns that fill
// the viewport. The tab and each tab's scroll are session state kept on the frame.
import QtQuick
import com.marcusrosado.AniBeam

FocusScope {
    id: page
    property var props: ({})
    property string title: "Settings"
    property real scrollY: 0
    readonly property var tabNames: ["Library", "Appearance", "Playback", "Data"]
    readonly property var tabIcons: ["folder", "palette", "play", "hard-drive"]
    property int tab: frame.settingsTab
    onTabChanged: frame.settingsTab = tab
    Component.onCompleted: {
        var want = tabNames.map(function(n) { return n.toLowerCase() }).indexOf(String(props.tab || "").toLowerCase())
        if (want >= 0) tab = want
        forceActiveFocus()
    }
    function escape() { return false }

    Column {
        id: head
        anchors.top: parent.top; anchors.topMargin: theme.space(7)
        x: content.item ? content.item.blockX + theme.space(8) : theme.space(8)
        spacing: theme.space(4)
        Text { text: "Settings"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold }
        Seg {
            options: page.tabNames.map(function(n, i) { return { text: n, icon: page.tabIcons[i] } })
            index: page.tab
            onPicked: function(i) { page.tab = i }
        }
    }
    Loader {
        id: content
        anchors.top: head.bottom; anchors.topMargin: theme.space(6)
        anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
        sourceComponent: [libraryTab, appearanceTab, playbackTab, dataTab][page.tab]
        onLoaded: { item.contentY = frame.settingsScroll[page.tab] || 0 }
        Connections { target: content.item; function onContentYChanged() { var s = frame.settingsScroll.slice(); s[page.tab] = content.item.contentY; frame.settingsScroll = s } }
    }
    Component { id: libraryTab; SettingsLibraryTab {} }
    Component { id: appearanceTab; SettingsAppearanceTab {} }
    Component { id: playbackTab; SettingsPlaybackTab {} }
    Component { id: dataTab; SettingsDataTab {} }
}
```

`apps/linux/qml/SettingsLibraryTab.qml`:

```qml
// The Library tab: sources with their counts and the Movies folders under them, Scan all,
// Add folder through the native picker, Rescan and Remove per source, Show hidden shows,
// the Subscriptions row; and the Trackers panel.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    property var sources: []
    property var stats: ({ series: 0, films: 0, episodes: 0, lastScan: "never" })
    property var waitingJob: ({})           // tracker name -> connect job id
    function reload() {
        var r = Door.listSources()
        if (!r.error) sources = r.reply.sources
        var all = Door.listSeries("All", "", "Alpha", "Asc", Door.revealHidden)
        if (!all.error) {
            var s = all.reply.series
            var eps = 0; s.forEach(function(c) { eps += c.episodes_on_disk || 0 })
            var ev = Door.recentEvents(2000)
            var last = "never"
            if (!ev.error) ev.reply.events.forEach(function(e) { if (e.kind === "ScanFinished") last = Qt.formatTime(new Date(e.at * 1000), "hh:mm") })
            stats = { series: s.filter(function(c) { return c.kind === "Show" }).length, films: s.filter(function(c) { return c.kind === "Movie" }).length, episodes: eps, lastScan: last }
        }
    }
    Component.onCompleted: reload()
    Timer { id: debounce; interval: 250; onTriggered: tab.reload() }
    Connections {
        target: Door
        function onSourceChanged(s) { debounce.restart() }
        function onSourceRemoved(s) { debounce.restart() }
        function onScanFinished(s, a, c, r) { debounce.restart() }
        function onSeriesChanged(c) { debounce.restart() }
        function onAuthUrlReady(tracker, openUrl, redirectUrl) { Qt.openUrlExternally(openUrl) }
        function onTrackerConnected(tracker, username) { frame.toast("Connected to " + (tracker === "Anilist" ? "AniList" : "MyAnimeList") + " as " + username); var w = JSON.parse(JSON.stringify(tab.waitingJob)); delete w[tracker]; tab.waitingJob = w }
        function onJobFinished(job, kind, ok) { if (kind === "ConnectTracker") { var w = {}; for (var k in tab.waitingJob) if (tab.waitingJob[k] !== job) w[k] = tab.waitingJob[k]; tab.waitingJob = w; if (!ok) frame.toast("Connect failed") } }
    }
    function login(tracker, clientId, clientSecret) {
        if (clientId !== "") { var c = Door.setTrackerCredentials(tracker, clientId, clientSecret); if (c.error) { frame.toast(c.error.message); return } }
        var r = Door.connectTracker(tracker)
        if (r.error) { frame.toast(r.error.message); return }
        var w = JSON.parse(JSON.stringify(waitingJob)); w[tracker] = r.reply.job; waitingJob = w
    }

    SettingsPair {
        split: 3 / 5
        left: [ libraryPanel ]
        right: [ trackersPanel ]
    }
    Component {
        id: libraryPanel
        Panel {
            title: "Library"; icon: "folder-open"; grows: true
            Tiles { tiles: [{ value: String(tab.stats.series), caption: "Series" }, { value: String(tab.stats.films), caption: "Films" }, { value: tab.stats.episodes.toLocaleString(), caption: "Episodes" }, { value: tab.stats.lastScan, caption: "Last scan" }] }
            stretch: Corner {
                implicitHeight: list.implicitHeight + theme.space(4)
                radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken; borderColor: theme.line; borderWidth: 1
                Column {
                    id: list
                    x: theme.space(2); y: theme.space(2); width: parent.width - theme.space(4)
                    Repeater {
                        model: tab.sources
                        SourceRow {
                            required property var modelData
                            source: modelData
                            onOpen: Qt.openUrlExternally("file://" + modelData.path)
                            onRescan: { var r = Door.scan(modelData.id); frame.toast(r.error ? r.error.message : "Rescan started") }
                            onRemove: confirming = true
                            onRemoveAccepted: { var r = Door.removeSource(modelData.id); if (r.error) frame.toast(r.error.message) }
                        }
                    }
                    Text { visible: tab.sources.length === 0; text: "No folders yet. Click Add folder to point AniBeam at your collection."; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal; padding: theme.space(3) }
                }
            }
            foot: [
                Row { spacing: theme.space(2)
                    Button { text: "Add folder"; icon: "folder-plus"; onClicked: folderDialog.open() }
                    Button { text: "Scan all"; icon: "refresh-cw"; enabled: tab.sources.length > 0; onClicked: { var r = Door.scan(-1); frame.toast(r.error ? r.error.message : "Scan started") } } },
                Note { text: "AniBeam scans these folders for video files. A folder is a series; a file at the top level of a Movies folder is a film." },
                SettingRow { label: "Show hidden shows"; helper: "Shows hidden series on every page until AniBeam closes."
                    Switch { checked: Door.revealHidden; onToggled: function(on) { Door.revealHidden = on } } },
                SettingRow { label: "Subscriptions"; helper: "The feeds anirss watches for you."
                    Button { text: "Open"; icon: "arrow-up-right"; flat: true; onClicked: frame.go("subscriptions") } }
            ]
        }
    }
    Component {
        id: trackersPanel
        Panel {
            title: "Trackers"; icon: "user-check"
            helper: "Episodes are marked on every connected tracker when you reach the outro or mark them by hand. Counts only go up."
            TrackerRow { tracker: "Anilist"; account: Door.trackers.anilist || ({}); waiting: tab.waitingJob["Anilist"] !== undefined
                onLogin: function(id, secret) { tab.login("Anilist", id, secret) }
                onDisconnect: { var r = Door.disconnectTracker("Anilist"); if (r.error) frame.toast(r.error.message) }
                onCancel: Door.cancelJob(tab.waitingJob["Anilist"]) }
            TrackerRow { tracker: "Mal"; account: Door.trackers.mal || ({}); waiting: tab.waitingJob["Mal"] !== undefined
                onLogin: function(id, secret) { tab.login("Mal", id, secret) }
                onDisconnect: { var r = Door.disconnectTracker("Mal"); if (r.error) frame.toast(r.error.message) }
                onCancel: Door.cancelJob(tab.waitingJob["Mal"]) }
            SettingRow { label: "Main tracker"; helper: "Whose count the cards show. The other tracker still receives every mark."
                Seg { options: ["AniList", "MyAnimeList"]; index: Door.trackers.main === "Mal" ? 1 : 0; onPicked: function(i) { var r = Door.setMainTracker(i === 1 ? "Mal" : "Anilist"); if (r.error) frame.toast(r.error.message) } } }
        }
    }
    FolderDialog {
        id: folderDialog
        title: "Add a folder"
        onAccepted: { var r = Door.addSource(decodeURIComponent(String(selectedFolder).replace("file://", ""))); frame.toast(r.error ? r.error.message : "Folder added, scanning") }
    }
}
```

`SettingsPair` takes `left` and `right` as lists of `Component`s it instantiates into its two `ColumnLayout`s (an edit to the lifted `Pair`: two `Repeater`s over the lists with `Loader`s, or `Instantiator`s; the panels' `Layout.*` attached properties then apply since the Loaders forward `Layout.fillWidth`, `Layout.fillHeight: item.grows`, `Layout.minimumHeight: item.implicitHeight`).

`Frame.qml`: `settings: settingsPage`, `Component { id: settingsPage; SettingsPage {} }`, `property int settingsTab: 0`, `property var settingsScroll: [0, 0, 0, 0]`, and `Ctrl+,` stays as is (it opens the page, which reads `frame.settingsTab`). `build.rs`: the twelve new QML files.

- [ ] **Step 2: Build, capture, and connect a tracker**

Run: `cargo build -p anibeam && ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh settings-library --page settings`
Expected: the page as in `docs/prototypes/home-grid-qml/a1-library.jpg` with the sandbox's real source, its counts, the four tiles, Add folder and Scan all, Show hidden shows, Subscriptions, and the two tracker rows, AniList and MyAnimeList both Not connected, Main tracker on AniList.

Run with `W=2500 H=1400` and again with `W=1000 H=1400`
Expected: two columns at the wide size, one column stacked at the narrow one; the tab fills the height and shows no scroll bar at the wide size.

Run on a monitor against the real data (`bench.sh settings 2 keep`, no `--root`): Add folder picks a folder through the portal's picker and a scan starts on the strip; Remove shows the inline confirm and Keep restores the row; Log in to AniList opens the browser, the row reads "Waiting for browser authorization…" with Cancel, and after the browser returns the row shows the username and the toast
Expected as listed; Disconnect asks inline and the row returns to Not connected.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the settings page, the Library tab with sources and the trackers"
```

---

### Task 16: The Appearance tab and the preview panes

Spec 4.5 (Appearance) and 4.3. Every knob writes through the `Theme` singleton; the two preview panes each hold their own `Tokens` forced to a mode, so the pair being chosen is what is shown.

**Files:**
- Create: `apps/linux/qml/SettingsAppearanceTab.qml` (replacing the placeholder), `apps/linux/qml/LookPreview.qml`, `apps/linux/qml/LookPane.qml` (from the prototype, edited)
- Modify: `apps/linux/build.rs`

**Interfaces:**
- Consumes: `Theme.*` and the `pick*` invokables, `Door.listSeries` for sample cards, `Card`, `Tokens`.
- Produces: the tab; `LookPreview` (`samples`), `LookPane` (`mode`, `samples`).

- [ ] **Step 1: Write the tab and the panes**

`LookPane.qml`: copy the prototype's and replace its `Theme { id: theme; ... }` block with `Tokens { id: theme; mode: pane.mode }`; drop the `host` property (the Tokens item reads the singleton itself); its sample `Card`s take the real `SeriesCard` records in `samples` and `nowMs: Date.now()`; keep the header, the search pill, the Seg, the chips, the three type tiers, the switch and buttons and the one-line strip. `LookPreview.qml`: copy the prototype's; `samples` are the first eight cards with a poster from `Door.listSeries("All", "", "LastViewed", "Desc", false).reply.series`, in-progress ones first (a card with `watched` between 0 and `total_episodes`).

`apps/linux/qml/SettingsAppearanceTab.qml`:

```qml
// The Appearance tab: Colours (mode, colour source, the theme pair, the accent), Shape
// (density, poster size, corners), and the two preview panes.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    readonly property var darkThemes: Theme.themes.filter(function(t) { return t.mode === "dark" })
    readonly property var lightThemes: Theme.themes.filter(function(t) { return t.mode === "light" })
    function indexOf(list, stem) { for (var i = 0; i < list.length; i++) if (list[i].stem === stem) return i; return 0 }

    Component {
        id: cornerGlyph
        Corner {
            width: theme.space(5); height: width
            radius: width * 0.45
            smoothing: option.smoothing
            borderColor: tint; borderWidth: 1.5; color: "transparent"
        }
    }

    SettingsPair {
        split: 2 / 5
        left: [ coloursPanel, shapePanel, footNote ]
        right: [ previewPanel ]
    }
    Component {
        id: coloursPanel
        Panel {
            title: "Colours"; icon: "palette"
            SettingRow { label: "Mode"
                Seg { options: [{ text: "Dark", icon: "moon" }, { text: "Light", icon: "sun" }, { text: "System", icon: "monitor" }]
                    index: ["dark", "light", "system"].indexOf(Theme.mode); onPicked: function(i) { Theme.pickMode(["dark", "light", "system"][i]) } } }
            SettingRow { label: "Colour source"; helper: "System reads your terminal's colours, or the desktop's scheme and accent when it finds no terminal config."
                Seg { options: ["System", "Theme"]; index: Theme.source === "theme" ? 1 : 0; onPicked: function(i) { Theme.pickSource(i === 1 ? "theme" : "system") } } }
            SettingRow { label: "Dark theme"; opacity: Theme.source === "theme" ? 1 : theme.disabledOpacity
                Dropdown { options: tab.darkThemes.map(function(t) { return t.name }); index: tab.indexOf(tab.darkThemes, Theme.themeDark); enabled: Theme.source === "theme"
                    onPicked: function(i) { Theme.pickTheme("dark", tab.darkThemes[i].stem) } } }
            SettingRow { label: "Light theme"; helper: "Base16 and kitty files in ~/.config/anibeam/themes appear here."; opacity: Theme.source === "theme" ? 1 : theme.disabledOpacity
                Dropdown { options: tab.lightThemes.map(function(t) { return t.name }); index: tab.indexOf(tab.lightThemes, Theme.themeLight); enabled: Theme.source === "theme"
                    onPicked: function(i) { Theme.pickTheme("light", tab.lightThemes[i].stem) } } }
            SettingRow { label: "Accent"
                Swatches { slot: Theme.accent; onPicked: function(s) { Theme.pickAccent(s) } } }
        }
    }
    Component {
        id: shapePanel
        Panel {
            title: "Shape"; icon: "shapes"
            SettingRow { label: "Density"
                Seg { options: ["Compact", "Normal", "Comfortable"]; index: ["compact", "normal", "comfortable"].indexOf(Theme.density); onPicked: function(i) { Theme.pickDensity(["compact", "normal", "comfortable"][i]) } } }
            SettingRow { label: "Poster size"
                Seg { options: ["S", "M", "L"]; index: ["s", "m", "l"].indexOf(Theme.poster); onPicked: function(i) { Theme.pickPoster(["s", "m", "l"][i]) } } }
            SettingRow { label: "Corners"
                Seg { options: [{ text: "Smooth", delegate: cornerGlyph, smoothing: 0.6 }, { text: "Plain", delegate: cornerGlyph, smoothing: 0 }]
                    index: Theme.corners === "plain" ? 1 : 0; onPicked: function(i) { Theme.pickCorners(i === 1 ? "plain" : "smooth") } } }
        }
    }
    Component { id: footNote; Note { text: "All of this lives in ~/.config/anibeam/theme.toml and reloads when the file changes." } }
    Component {
        id: previewPanel
        Panel {
            title: "Preview"; icon: "eye"; grows: true
            stretch: LookPreview {}
        }
    }
}
```

`build.rs`: `LookPreview`, `LookPane`.

- [ ] **Step 2: Build and capture**

Run: `cargo build -p anibeam && ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh settings-appearance --page settings:appearance` (the page suffix goes into `props.tab`; `Main.qml`'s `onLoaded` splits `Shell.page` on `:` and passes `{ tab }`)
Expected: `docs/prototypes/home-grid-qml/a2-appearance.jpg` with the sandbox's posters in both panes; the dark pane and the light pane each rendered from their mode's tokens.

Run on a monitor: switch Mode, Colour source, the two theme dropdowns, the accent swatches, density, poster size and corners
Expected: the whole app recolours or reshapes at once, both panes follow, `~/.config/anibeam/theme.toml` holds the choices, and a hand edit to that file applies live.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the Appearance tab with live preview panes"
```

---

### Task 17: The Playback tab and the subtitle preview through mpv

Spec 4.5 (Playback) and 4.4 (subtitle defaults, Use my mpv.conf, auto-skip). The preview is the player's own item: a frame of the episode watched last, paused at its resume point, with that file's subtitle track, the defaults re-applied on every change; with no history, a black lavfi source with one sample line from a generated SRT, still through libass.

**Files:**
- Create: `apps/linux/qml/SettingsPlaybackTab.qml` (replacing the placeholder), `apps/linux/qml/SubtitlePreview.qml` (new, over `VideoItem`)
- Modify: `apps/linux/src/bridge/player.rs` (`samplePreview()`), `apps/linux/build.rs`

**Interfaces:**
- Consumes: `Door.settings`, `Door.setSubtitleDefaults`, `Door.setAutoSkip`, `Player.useMyMpvConf`, `Player.saveUseMyMpvConf`, `Player.subtitleOptions`, `Player.previewOptions`, `Player.configLayers`, `Door.listSeries`, `Door.getSeries`, `VideoItem`.
- Produces: the tab; `SubtitlePreview` (`defaults`, `path`, `subtitle`, `startAt`); `Player.samplePreview() -> { path, subtitle }` (writes `<cache_dir>/sample.srt` once and returns the lavfi source `av://lavfi:color=c=0x101216:s=1280x720:d=3600` with it).

- [ ] **Step 1: Write the tab, the preview and the sample source**

`bridge/player.rs` addition:

```rust
        #[qinvokable] fn sample_preview(self: &Self) -> QJsonObject;
```

```rust
    pub fn sample_preview(&self) -> QJsonObject {
        let dir = std::path::PathBuf::from(&crate::runtime::paths().core.cache_dir);
        let srt = dir.join("sample.srt");
        if !srt.exists() {
            std::fs::create_dir_all(&dir).ok();
            std::fs::write(&srt, "1\n00:00:00,000 --> 01:00:00,000\nSample subtitle line\n").ok();
        }
        crate::json::to_qjson_object(&serde_json::json!({ "path": "av://lavfi:color=c=0x101216:s=1280x720:d=3600", "subtitle": srt.to_string_lossy() }))
    }
```

`apps/linux/qml/SubtitlePreview.qml`:

```qml
// The subtitle preview: the player's own item, paused on a frame, re-applying the
// defaults on every change. A QML approximation is rejected by spec 4.5.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    id: root
    property var defaults: ({})
    property string path: ""
    property string subtitle: ""
    property real startAt: 0
    radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken; borderColor: theme.line; borderWidth: 1
    fillItem: video.isReady ? video : null
    onDefaultsChanged: apply()
    function apply() {
        if (!video.isReady) return
        var opts = Player.subtitleOptions(defaults)
        for (var i = 0; i < opts.length; i++) video.setProperty(opts[i][0], opts[i][1])
    }
    VideoItem {
        id: video
        width: parent.width; height: parent.height
        property bool isReady: false
        onReady: {
            var layers = Player.configLayers
            for (var i = 0; i < layers.length; i++) include(layers[i])
            var o = Player.previewOptions
            for (var j = 0; j < o.length; j++) if (o[j][0] !== "sid" && o[j][0] !== "sub-auto") setProperty(o[j][0], o[j][1])
            setProperty("sid", "auto")
            isReady = true
            root.apply()
            if (root.path !== "") {
                setProperty("start", root.startAt > 0 ? String(root.startAt) : "none")
                command(["loadfile", root.path])
            }
        }
        onLoaded: { if (root.subtitle !== "") command(["sub-add", root.subtitle, "select"]); root.apply() }
    }
}
```

`apps/linux/qml/SettingsPlaybackTab.qml`:

```qml
// The Playback tab: auto-skip, Use my mpv.conf, the language orders, the subtitle
// defaults, and the preview rendered through mpv.
import QtQuick
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    property var defaults: JSON.parse(JSON.stringify(Door.settings.subtitle_defaults || {}))
    property var preview: ({ path: "", subtitle: "", startAt: 0 })
    Connections { target: Door; function onSettingsChanged() { tab.defaults = JSON.parse(JSON.stringify(Door.settings.subtitle_defaults)) } }
    Timer { id: save; interval: 300; onTriggered: { var r = Door.setSubtitleDefaults(tab.defaults); if (r.error) frame.toast(r.error.message) } }
    function edit(f) { var d = JSON.parse(JSON.stringify(defaults)); f(d); defaults = d; save.restart() }
    function isHex(s) { return /^#[0-9a-fA-F]{6}$/.test(s) }
    function hexOf(c) { function h(n) { return ("0" + n.toString(16)).slice(-2) } return "#" + h(c.r) + h(c.g) + h(c.b) }
    function colourOf(hex) { return { r: parseInt(hex.substr(1, 2), 16), g: parseInt(hex.substr(3, 2), 16), b: parseInt(hex.substr(5, 2), 16), a: 255 } }
    Component.onCompleted: {
        // The episode watched last, at its resume point; else the sample source
        var recent = Door.listSeries("All", "", "LastViewed", "Desc", false)
        var chosen = null
        if (!recent.error) for (var i = 0; i < recent.reply.series.length && !chosen; i++) {
            if (!recent.reply.series[i].last_viewed_at) break
            var d = Door.getSeries(recent.reply.series[i].id)
            if (d.error) continue
            var eps = d.reply.detail.episodes
            var withResume = eps.filter(function(e) { return e.resume })
            var ep = withResume.length ? withResume[0] : (eps.length ? eps[0] : null)
            if (ep) chosen = { path: ep.path, subtitle: "", startAt: ep.resume ? ep.resume.position : 0 }
        }
        preview = chosen || Player.samplePreview()
    }

    SettingsPair {
        split: 2 / 5
        left: [ playbackPanel, tracksPanel, subtitlePanel ]
        right: [ previewPanel ]
    }
    Component {
        id: playbackPanel
        Panel {
            title: "Playback"; icon: "play"
            SettingRow { label: "Auto-skip intro"; helper: "Jumps the intro when the file's chapters or AniSkip know where it is. Undo in the player turns it off for the session."
                Switch { checked: !!(Door.settings.auto_skip && Door.settings.auto_skip.intro); onToggled: function(on) { Door.setAutoSkip(on, !!(Door.settings.auto_skip && Door.settings.auto_skip.outro)) } } }
            SettingRow { label: "Auto-skip outro"; helper: "Jumps the outro when the file's chapters or AniSkip know where it is. Undo in the player turns it off for the session."
                Switch { checked: !!(Door.settings.auto_skip && Door.settings.auto_skip.outro); onToggled: function(on) { Door.setAutoSkip(!!(Door.settings.auto_skip && Door.settings.auto_skip.intro), on) } } }
            SettingRow { label: "Use my mpv.conf"; helper: "Loads ~/.config/mpv/mpv.conf under AniBeam's own settings. Lines that only apply at start-up, scripts, input-conf and config-dir, are ignored, and no script ever loads."
                Switch { checked: Player.useMyMpvConf; onToggled: function(on) { Player.saveUseMyMpvConf(on) } } }
        }
    }
    Component {
        id: tracksPanel
        Panel {
            title: "Tracks"; icon: "languages"
            SettingRow { label: "Subtitle languages"
                Field { text: (tab.defaults.subtitle_languages || []).join(", "); width: theme.space(30); onEdited: function(t) { tab.edit(function(d) { d.subtitle_languages = t.split(",").map(function(s) { return s.trim() }).filter(function(s) { return s }) }) } } }
            SettingRow { label: "Audio languages"; helper: "Comma separated, first match wins."
                Field { text: (tab.defaults.audio_languages || []).join(", "); width: theme.space(30); onEdited: function(t) { tab.edit(function(d) { d.audio_languages = t.split(",").map(function(s) { return s.trim() }).filter(function(s) { return s }) }) } } }
        }
    }
    Component {
        id: subtitlePanel
        Panel {
            title: "Subtitle defaults"; icon: "captions"
            helper: "What every session starts from. Change tracks in the player and AniBeam remembers them per series."
            SettingRow { label: "Scale"
                SliderRow { from: 0.5; to: 2.0; stepSize: 0.05; decimals: 2; value: tab.defaults.scale || 1; onMoved: function(v) { tab.edit(function(d) { d.scale = v }) } } }
            SettingRow { label: "ASS override"; helper: "Force applies the text style to styled subtitles and may break signs and karaoke."
                Seg { options: ["As scripted", "Scale only", "Force"]; index: ["AsScripted", "ScaleOnly", "Force"].indexOf(tab.defaults.ass_override || "ScaleOnly"); onPicked: function(i) { tab.edit(function(d) { d.ass_override = ["AsScripted", "ScaleOnly", "Force"][i] }) } } }
            Note { text: "TEXT STYLE, FOR SRT AND VTT" }
            SettingRow { label: "Font"
                Field { text: tab.defaults.text_style ? tab.defaults.text_style.font : ""; width: theme.space(30); onEdited: function(t) { if (t.trim() !== "") tab.edit(function(d) { d.text_style.font = t.trim() }) } } }
            SettingRow { label: "Colour"
                Row { spacing: theme.space(2)
                    Corner { width: theme.space(6); height: width; radius: theme.radiusSm; smoothing: theme.cornerSmoothing; color: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.colour) : "#ffffff"; borderColor: theme.line; borderWidth: 1 }
                    Field { text: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.colour).toUpperCase() : ""; mono: true; width: theme.space(24); onEdited: function(t) { if (tab.isHex(t)) tab.edit(function(d) { d.text_style.colour = tab.colourOf(t) }) } } } }
            SettingRow { label: "Outline"
                Row { spacing: theme.space(2)
                    Field { text: tab.defaults.text_style ? String(tab.defaults.text_style.outline_size) : ""; mono: true; width: theme.space(14); onEdited: function(t) { var v = Number(t); if (v >= 0) tab.edit(function(d) { d.text_style.outline_size = v }) } }
                    Corner { width: theme.space(6); height: width; radius: theme.radiusSm; smoothing: theme.cornerSmoothing; color: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.outline_colour) : "#000000"; borderColor: theme.line; borderWidth: 1 }
                    Field { text: tab.defaults.text_style ? tab.hexOf(tab.defaults.text_style.outline_colour).toUpperCase() : ""; mono: true; width: theme.space(24); onEdited: function(t) { if (tab.isHex(t)) tab.edit(function(d) { d.text_style.outline_colour = tab.colourOf(t) }) } } } }
            SettingRow { label: "Shadow"
                Field { text: tab.defaults.text_style ? String(tab.defaults.text_style.shadow_offset) : ""; mono: true; width: theme.space(14); onEdited: function(t) { var v = Number(t); if (v >= 0) tab.edit(function(d) { d.text_style.shadow_offset = v }) } } }
            SettingRow { label: "Box opacity"
                SliderRow { from: 0; to: 1; stepSize: 0.05; decimals: 2; value: tab.defaults.text_style ? tab.defaults.text_style.box_opacity : 0; onMoved: function(v) { tab.edit(function(d) { d.text_style.box_opacity = v }) } } }
            SettingRow { label: "Bold"
                Switch { checked: !!(tab.defaults.text_style && tab.defaults.text_style.bold); onToggled: function(on) { tab.edit(function(d) { d.text_style.bold = on }) } } }
            SettingRow { label: "Position"
                SliderRow { from: 0; to: 150; stepSize: 1; value: tab.defaults.text_style ? tab.defaults.text_style.position : 100; onMoved: function(v) { tab.edit(function(d) { d.text_style.position = v }) } } }
        }
    }
    Component {
        id: previewPanel
        Panel {
            title: "Preview"; icon: "eye"; grows: true
            stretch: Item {
                implicitHeight: theme.space(60)
                SubtitlePreview {
                    anchors.centerIn: parent
                    width: Math.min(parent.width, parent.height * 16 / 9); height: width * 9 / 16
                    defaults: tab.defaults; path: tab.preview.path; subtitle: tab.preview.subtitle; startAt: tab.preview.startAt
                }
            }
        }
    }
}
```

`build.rs`: `SettingsPlaybackTab`, `SubtitlePreview`.

- [ ] **Step 2: Build and look on a monitor**

Run: `cargo build --release -p anibeam && apps/linux/scripts/bench.sh settings-playback 2 keep --root /tmp/sandbox` and open Settings, Playback
Expected: the preview shows a frame of the episode watched last with its subtitle track (the sandbox has history once Task 12's run played something); Scale, ASS override and every text style field change the picture within a moment; with a fresh sandbox the preview is the flat backdrop with "Sample subtitle line" through libass; Auto-skip and Use my mpv.conf toggle and persist (`anibeam-cli --root /tmp/sandbox call GetSettings` shows the auto-skip pair; `player.toml` shows the toggle); a player opened afterwards uses the new defaults.

Run: `ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh settings-playback --page settings:playback`
Expected: the layout as `a4-playback.jpg`, the preview panel showing whatever the offscreen GL context can render (the preview may stay black offscreen; the monitor run is the check).

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the Playback tab with the subtitle preview through mpv"
```

---

### Task 18: The Data tab: storage, export, import, about

Spec 4.5 (Data) and 4.1 unit 6 (Storage, Export, Import). Pickers are the shell's (`QtQuick.Dialogs`); the export and the import are core jobs whose outcome arrives as events.

**Files:**
- Create: `apps/linux/qml/SettingsDataTab.qml` (replacing the placeholder)
- Modify: `apps/linux/src/bridge/shell.rs` (`fileSize(path) -> number`, `homeShort(path) -> string`), `apps/linux/build.rs`

**Interfaces:**
- Consumes: `Door.getStorage`, `clearImages`, `about`, `recentEvents`, `exportLibrary`, `importLibrary`, `trackers`, the `imagesCleared`, `exportFinished`, `importFinished`, `jobFailed` signals; `FileDialog`.
- Produces: the tab; `Shell.fileSize(path)` (bytes, 0 when missing), `Shell.homeShort(path)` (`~` for the home prefix).

- [ ] **Step 1: Write the tab**

`bridge/shell.rs` additions: `#[qinvokable] fn file_size(self: &Self, path: &QString) -> f64;` returning `std::fs::metadata(path).map(|m| m.len() as f64).unwrap_or(0.0)`, and `#[qinvokable] fn home_short(self: &Self, path: &QString) -> QString;` replacing a leading `$HOME` with `~`.

`apps/linux/qml/SettingsDataTab.qml`:

```qml
// The Data tab: Storage with its tiles, the usage bar, Clear images and the paths;
// Export and import; About.
import QtQuick
import QtQuick.Dialogs
import com.marcusrosado.AniBeam

SettingsTab {
    id: tab
    property var storage: ({ image_count: 0, image_bytes: 0 })
    property real dbBytes: 0
    property int eventsKept: 0
    property bool privateData: false
    property string lastExport: ""
    property string lastImport: ""
    property bool clearing: false
    readonly property var about: Door.about
    function reload() {
        var r = Door.getStorage(); if (!r.error) storage = r.reply
        dbBytes = Shell.fileSize(about.db_path || "")
        var ev = Door.recentEvents(2000); if (!ev.error) eventsKept = ev.reply.events.length
    }
    Component.onCompleted: reload()
    Connections {
        target: Door
        function onImagesCleared(removed) { frame.toast(Fmt.plural(removed, "image", "images") + " removed"); tab.reload() }
        function onExportFinished(path) { tab.lastExport = "Wrote " + Shell.homeShort(path); frame.toast(tab.lastExport) }
        function onImportFinished(s) { tab.lastImport = "Imported " + s.series_created + " series, " + s.matches_applied + " matches, " + s.sources_added + " sources" + (s.fields_ignored.length ? ", " + s.fields_ignored.length + " fields ignored" : ""); frame.toast(tab.lastImport); tab.reload() }
        function onJobFailed(job, kind, error) { if (kind === "Export" || kind === "Import" || kind === "ClearImages") frame.toast(kind + " failed: " + error.message) }
    }
    function dateStamp() { var d = new Date(); return d.getFullYear() + "-" + ("0" + (d.getMonth() + 1)).slice(-2) + "-" + ("0" + d.getDate()).slice(-2) }
    readonly property string exportName: (privateData ? "anibeam-export-full-" : "anibeam-export-") + dateStamp() + ".json"
    function pathOf(url) { return decodeURIComponent(String(url).replace("file://", "")) }

    SettingsPair {
        split: 1 / 2
        left: [ storagePanel ]
        right: [ transferPanel ]
    }
    Component {
        id: storagePanel
        Panel {
            title: "Storage"; icon: "hard-drive"; grows: true
            Tiles { tiles: [{ value: Fmt.bytes(tab.storage.image_bytes), caption: "Images" }, { value: Fmt.bytes(tab.dbBytes), caption: "Database" }, { value: String(tab.eventsKept), caption: "Events kept" }, { value: String(tab.storage.image_count), caption: "Posters" }] }
            UsageBar { parts: [{ label: "Images", value: tab.storage.image_bytes, color: theme.blue }, { label: "Database", value: tab.dbBytes, color: theme.purple }] }
            SettingRow { label: "Images"; helper: "Posters come back on the next launch."
                Button { visible: !tab.clearing; text: "Clear images"; icon: "trash-2"; danger: true; onClicked: tab.clearing = true }
                InlineConfirm { visible: tab.clearing; question: "Clear " + Fmt.plural(tab.storage.image_count, "cached image", "cached images") + "?"; confirmText: "Clear"
                    onAccepted: { tab.clearing = false; var r = Door.clearImages(); if (r.error) frame.toast(r.error.message) } onKept: tab.clearing = false } }
            foot: [
                SettingRow { label: "Database"; line: Shell.homeShort(tab.about.db_path || ""); Button { text: "Open"; icon: "folder-open"; flat: true; onClicked: Qt.openUrlExternally("file://" + (tab.about.db_path || "").replace(/\/[^/]*$/, "")) } },
                SettingRow { label: "Data"; line: Shell.homeShort(tab.about.data_dir || ""); Button { text: "Open"; icon: "folder-open"; flat: true; onClicked: Qt.openUrlExternally("file://" + tab.about.data_dir) } },
                SettingRow { label: "Config"; line: Shell.homeShort(tab.about.config_dir || ""); Button { text: "Open"; icon: "folder-open"; flat: true; onClicked: Qt.openUrlExternally("file://" + tab.about.config_dir) } },
                SettingRow { label: "Cache"; line: Shell.homeShort(tab.about.cache_dir || ""); Button { text: "Open"; icon: "folder-open"; flat: true; onClicked: Qt.openUrlExternally("file://" + tab.about.cache_dir) } }
            ]
        }
    }
    Component {
        id: transferPanel
        Panel {
            title: "Export and import"; icon: "archive"; grows: true
            SettingRow { label: "Include private data"; helper: "Tracker logins, API keys, watch history and preferences, in plain text."
                Switch { checked: tab.privateData; onToggled: function(on) { tab.privateData = on } } }
            SettingRow { label: "Export"; helper: "Writes " + tab.exportName + "."; status: tab.lastExport
                Button { text: "Export"; icon: "upload"; onClicked: { exportDialog.currentFile = "file://" + (tab.about.data_dir ? "" : "") + tab.exportName; exportDialog.open() } } }
            SettingRow { label: "Import"; helper: "Merges a file into this library. The file wins for matches and accounts, the newer entry wins for history, nothing is deleted."; status: tab.lastImport
                Button { text: "Import"; icon: "download"; onClicked: importDialog.open() } }
            foot: [
                Panel {
                    title: "About"; icon: "info"
                    Row { spacing: theme.space(4)
                        Corner { width: theme.space(16); height: width; radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.accentSoft
                            Image { anchors.centerIn: parent; width: parent.width * 0.7; height: width; source: "qrc:/qt/qml/com/marcusrosado/AniBeam/assets/icon.png"; sourceSize: Qt.size(128, 128) } }
                        Column { spacing: theme.space(1); anchors.verticalCenter: parent.verticalCenter
                            Row { spacing: theme.space(2)
                                Text { text: "AniBeam"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
                                Chip { text: Shell.version; small: true; anchors.verticalCenter: parent.verticalCenter }
                                Chip { text: "GPL-3.0-or-later"; small: true; mono: false; anchors.verticalCenter: parent.verticalCenter } }
                            Text { text: "Made by Marcus Rosado"; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
                            Text { text: "A local anime library: it scans folders, matches them against AniList, plays the files and keeps your trackers up to date."; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall; wrapMode: Text.Wrap; width: theme.space(100) }
                            Row { spacing: theme.space(2)
                                Button { text: "Repository"; icon: "git-branch"; flat: true; small: true; onClicked: Qt.openUrlExternally("https://github.com/marcusbandit/AniBeam") }
                                Button { text: "marcusrosado.com"; icon: "globe"; flat: true; small: true; onClicked: Qt.openUrlExternally("https://marcusrosado.com") }
                                Button { visible: !!(Door.trackers.anilist && Door.trackers.anilist.username); text: "AniList"; icon: "heart"; flat: true; small: true; onClicked: Qt.openUrlExternally("https://anilist.co/user/" + Door.trackers.anilist.username) } } } }
                }
            ]
        }
    }
    FileDialog { id: exportDialog; title: "Export the library"; fileMode: FileDialog.SaveFile; nameFilters: ["AniBeam export (*.json)"]; defaultSuffix: "json"
        onAccepted: { var r = Door.exportLibrary(tab.pathOf(selectedFile), tab.privateData); frame.toast(r.error ? r.error.message : "Export started") } }
    FileDialog { id: importDialog; title: "Import an AniBeam export"; nameFilters: ["AniBeam export (*.json)", "All files (*)"]
        onAccepted: { var r = Door.importLibrary(tab.pathOf(selectedFile)); frame.toast(r.error ? r.error.message : "Import started") } }
}
```

The About panel sits in the transfer panel's `foot` here so it spans the right column; if it must span both columns as the prototype draws it (`a5-data.jpg`), move it under the `SettingsPair` as a third row of the tab's column: `SettingsTab` accepts children after the pair, laid out below it at full block width.

`build.rs`: `SettingsDataTab`.

- [ ] **Step 2: Build, capture and round-trip an export**

Run: `cargo build -p anibeam && ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh settings-data --page settings:data`
Expected: `a5-data.jpg` with the sandbox's real sizes and paths, the usage bar, About with the version chip.

Run on a monitor: Export with the checkbox off, then Import the same file into a second sandbox root (`bench.sh import 2 keep --root /tmp/sandbox2`)
Expected: the save picker opens with `anibeam-export-<date>.json` proposed; the export's status line names the file; the import into the empty root creates the series (missing until their paths exist) and the summary line reads the counts; `Clear images` asks inline and the tiles drop to zero.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the Data tab with storage, export, import and about"
```

---
### Task 19: The Metadata page

Spec 4.1 unit 7 (the table, the filters, Attach sources, Refresh all, the per-row actions, Forget, the crawl bar). `ListMetadata` gives the rows and the counts; the page keeps them in a `RecordModel` and reloads, debounced, on the events that change a series.

**Files:**
- Create: `apps/linux/qml/MetadataPage.qml`, `apps/linux/qml/MetadataRow.qml`
- Modify: `apps/linux/qml/Frame.qml` (`metadata: metadataPage`), `apps/linux/build.rs`

**Interfaces:**
- Consumes: `Door.listMetadata`, `autoMatch`, `refreshAll`, `refreshSeries`, `clearMatch`, `forgetSeries`, `revealHidden`, `runningJobs`, the `seriesChanged`, `seriesRemoved`, `matchApplied`, `autoMatchFinished`, `refreshFinished`, `jobFinished` signals; `InlineConfirm`; `MatchModal` (Task 20; a stub `Item { function open(seriesId, title) {} }` until then).
- Produces: `MetadataPage` with `props.q`; `MetadataRow` (`row`, `missing`, signals `match()`, `refresh()`, `clear()`, `forget()`).

- [ ] **Step 1: Write the page**

`apps/linux/qml/MetadataRow.qml`:

```qml
// One table row: thumbnail, title with the alternate beneath, the type pill, the source
// chip, the files bar with an amber +N, and the actions. A missing row offers Forget.
import QtQuick
import com.marcusrosado.AniBeam

Corner {
    id: root
    property var row: ({})
    property bool missing: false
    property bool confirming: false
    signal match()
    signal refresh()
    signal clear()
    signal forget()
    readonly property var card: row.series || ({})
    readonly property int have: row.have || 0
    readonly property int expected: row.expected || 0
    readonly property int extra: row.extra_on_disk || 0
    readonly property real fraction: expected > 0 ? Math.min(1, have / expected) : 0
    width: parent ? parent.width : implicitWidth
    height: theme.space(14)
    radius: theme.radiusSm; smoothing: theme.cornerSmoothing
    color: hover.containsMouse ? theme.surface : "transparent"
    MouseArea { id: hover; anchors.fill: parent; hoverEnabled: true; z: -1 }

    Row {
        anchors.left: parent.left; anchors.leftMargin: theme.space(2)
        anchors.right: actions.left; anchors.rightMargin: theme.space(3)
        anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(3)
        Corner { anchors.verticalCenter: parent.verticalCenter; width: theme.space(8); height: width * 1.5; radius: theme.radiusSm; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken
            fillItem: thumb.status === Image.Ready ? thumb : null
            Image { id: thumb; visible: false; width: parent.width; height: parent.height; source: card.poster ? "file://" + card.poster : ""; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize.width: 96 } }
        Column {
            anchors.verticalCenter: parent.verticalCenter
            width: theme.space(80)
            Text { width: parent.width; text: card.title || ""; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold; elide: Text.ElideRight
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.match() } Tooltip { text: "Match to a different show" } }
            Text { visible: !!row.alt_title; width: parent.width; text: row.alt_title || ""; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall; elide: Text.ElideRight }
        }
        Chip { anchors.verticalCenter: parent.verticalCenter; text: card.kind === "Movie" ? "Movie" : "Series"; small: true; mono: false; textColor: theme.hue(Theme.formatHue(card.kind === "Movie" ? "MOVIE" : "TV")) }
        Chip { anchors.verticalCenter: parent.verticalCenter; small: true; mono: false
            text: row.provider === "Anilist" ? "AniList" : row.provider === "Mal" ? "MAL" : row.provider === "Tmdb" ? "TMDB" : "none"
            textColor: row.provider ? theme.text : theme.textFaint }
        Chip { visible: root.missing; anchors.verticalCenter: parent.verticalCenter; text: "Missing"; small: true; mono: false; textColor: theme.yellow }
        Row { anchors.verticalCenter: parent.verticalCenter; spacing: theme.space(2)
            Corner { anchors.verticalCenter: parent.verticalCenter; width: theme.space(24); height: theme.space(1.5); radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken; borderColor: theme.line; borderWidth: 1
                Corner { width: parent.width * root.fraction; height: parent.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: root.extra > 0 ? theme.yellow : theme.accent } }
            Text { anchors.verticalCenter: parent.verticalCenter; text: root.have + "/" + (root.expected > 0 ? root.expected : "?"); color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall }
            Chip { visible: root.extra > 0; anchors.verticalCenter: parent.verticalCenter; icon: "triangle-alert"; text: "+" + root.extra; small: true; textColor: theme.yellow
                Tooltip { text: Fmt.plural(root.extra, "file", "files") + " beyond the expected " + root.expected + ", needs attention" } }
        }
    }
    Row {
        id: actions
        anchors.right: parent.right; anchors.rightMargin: theme.space(2); anchors.verticalCenter: parent.verticalCenter
        spacing: theme.space(1)
        visible: !root.confirming
        Button { text: "Match"; icon: "link"; flat: true; small: true; onClicked: root.match() }
        Button { text: "Refresh"; icon: "refresh-cw"; flat: true; small: true; enabled: !!row.provider; onClicked: root.refresh() }
        Button { text: "Clear match"; icon: "x"; flat: true; small: true; enabled: !!row.provider; onClicked: root.clear() }
        Button { visible: root.missing; text: "Forget"; icon: "trash-2"; flat: true; small: true; danger: true; onClicked: root.confirming = true }
    }
    InlineConfirm { visible: root.confirming; anchors.right: parent.right; anchors.rightMargin: theme.space(2); anchors.verticalCenter: parent.verticalCenter
        question: "Forget " + (card.title || "") + " and its history?"; confirmText: "Forget"
        onAccepted: { root.confirming = false; root.forget() } onKept: root.confirming = false }
}
```

`apps/linux/qml/MetadataPage.qml`:

```qml
// Spec 4.1 unit 7: the table of every series, the filters with counts, the text filter
// seeded from To Metadata, Attach sources, Refresh all with an inline confirm, the per-row
// actions, Forget on missing rows, and the crawl bar.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    property var props: ({})
    property real scrollY: list.contentY
    onScrollYChanged: if (Math.abs(list.contentY - scrollY) > 1) list.contentY = scrollY
    readonly property var filters: [["All", "All"], ["Series", "Series"], ["Movies", "Movies"], ["MissingFiles", "Missing files"]]
    property string filter: "All"
    property string query: props.q || ""
    property var counts: ({ all: 0, series: 0, movies: 0, missing_files: 0 })
    property bool confirmingRefresh: false
    property int refreshJob: -1
    readonly property var crawl: Door.runningJobs.find(function(j) { return j.kind === "Crawl" }) || null
    readonly property int sourceless: { var n = 0; for (var i = 0; i < rows.count; i++) if (!rows.at(i).provider) n++; return n }
    RecordModel { id: rows; idKey: "series.id"; roles: ["series", "alt_title", "provider", "have", "expected", "extra_on_disk"] }
    function reload() {
        var keep = list.contentY
        var r = Door.listMetadata(filter, query, Door.revealHidden)
        if (r.error) { frame.toast(r.error.message); return }
        rows.reset(r.reply.rows)
        counts = r.reply.counts
        list.contentY = Math.min(keep, Math.max(0, list.contentHeight - list.height))
    }
    function contextItems() { return [] }
    Component.onCompleted: { search.text = query; reload() }
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Timer { id: queryDebounce; interval: 150; onTriggered: { page.query = search.text; page.reload() } }
    Connections {
        target: Door
        function onSeriesChanged(c) { debounce.restart() }
        function onSeriesRemoved(i) { debounce.restart() }
        function onMatchApplied(s) { debounce.restart() }
        function onAutoMatchFinished(backfilled, matched, unmatched) { frame.toast("Attached " + Fmt.plural(backfilled + matched, "source", "sources") + (unmatched ? ". " + unmatched + " still had no match, left for manual matching." : ".")); debounce.restart() }
        function onRefreshFinished(refreshed, failed) { frame.toast("Refresh complete: " + refreshed + " successful, " + failed + " failed"); debounce.restart() }
        function onRevealHiddenChanged() { debounce.restart() }
    }

    Column {
        id: header
        anchors.top: parent.top; anchors.topMargin: theme.space(7)
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        anchors.right: parent.right; anchors.rightMargin: theme.space(8)
        spacing: theme.space(4)
        Row { spacing: theme.space(3)
            Text { text: "Metadata"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
            Chip { anchors.verticalCenter: parent.verticalCenter; text: rows.count + " rows"; small: true; color: theme.surface; textColor: theme.textDim } }
        Row {
            width: parent.width; spacing: theme.space(3)
            Seg {
                options: page.filters.map(function(f) { return f[1] + "  " + ({ All: page.counts.all, Series: page.counts.series, Movies: page.counts.movies, MissingFiles: page.counts.missing_files })[f[0]] })
                index: page.filters.map(function(f) { return f[0] }).indexOf(page.filter)
                onPicked: function(i) { page.filter = page.filters[i][0]; page.reload() }
            }
            SearchField { id: search; placeholder: "Filter by title"; hint: ""; width: theme.space(80); onTextChanged: queryDebounce.restart(); onCleared: { page.query = ""; page.reload() } }
            Item { width: theme.space(2); height: 1 }
            Button { text: "Attach sources" + (page.sourceless ? " (" + page.sourceless + ")" : ""); icon: "link"; enabled: page.sourceless > 0 && !page.crawl; onClicked: { var r = Door.autoMatch(); frame.toast(r.error ? r.error.message : "Attaching sources") }
                Tooltip { text: "Give a source to every title showing none. Matched shows get their label back; only unmatched titles are searched." } }
            Button { visible: !page.confirmingRefresh; text: "Refresh all"; icon: "refresh-cw"; onClicked: page.confirmingRefresh = true }
            InlineConfirm { visible: page.confirmingRefresh; question: "Refresh metadata for all " + rows.count + " rows? This may take a while."; confirmText: "Refresh"; confirmIcon: "refresh-cw"
                onAccepted: { page.confirmingRefresh = false; var r = Door.refreshAll(); frame.toast(r.error ? r.error.message : "Refreshing everything") } onKept: page.confirmingRefresh = false }
        }
        // The crawl bar while a crawl runs
        Column {
            visible: page.crawl !== null
            width: parent.width; spacing: theme.space(1)
            Row { width: parent.width
                Text { text: "Franchise crawl"; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
                Text { anchors.right: parent.right; text: page.crawl ? page.crawl.done + " / " + (page.crawl.total > 0 ? page.crawl.total : "?") : ""; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall } }
            Corner { width: parent.width; height: theme.space(1); radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.surfaceSunken
                Corner { width: page.crawl && page.crawl.total > 0 ? parent.width * page.crawl.done / page.crawl.total : 0; height: parent.height; radius: height / 2; smoothing: theme.cornerSmoothing; color: theme.accent } }
        }
    }
    ListView {
        id: list
        anchors.top: header.bottom; anchors.topMargin: theme.space(4)
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        anchors.right: parent.right; anchors.rightMargin: theme.space(8)
        anchors.bottom: parent.bottom
        clip: true
        model: rows
        spacing: theme.space(0.5)
        delegate: MetadataRow {
            required property int index
            row: rows.at(index)
            missing: !!(row.series && row.series.missing)
            onMatch: matchModal.open(row.series.id, row.series.title)
            onRefresh: { var r = Door.refreshSeries(row.series.id); frame.toast(r.error ? r.error.message : "Refreshing " + row.series.title) }
            onClear: { var r = Door.clearMatch(row.series.id); if (r.error) frame.toast(r.error.message) }
            onForget: { var r = Door.forgetSeries(row.series.id); if (r.error) frame.toast(r.error.message) }
        }
        footer: Item { width: 1; height: theme.space(10) }
        EmptyState { visible: rows.count === 0; icon: page.counts.all === 0 ? "book-open" : "search"; title: page.counts.all === 0 ? "No metadata yet" : "No matches"; body: page.counts.all === 0 ? "Your library is empty. Add a folder in Settings and scan it to get started." : "No series match your filters." }
    }
    MatchModal { id: matchModal; parent: frame.overlay }
}
```

Until Task 20, `MatchModal.qml` is `Item { function open(seriesId, title) { frame.toast("Match modal, Task 20") } }`. `Frame.qml`: `metadata: metadataPage`. `build.rs`: `MetadataPage`, `MetadataRow`, `MatchModal`.

- [ ] **Step 2: Build and capture**

Run: `cargo build -p anibeam && ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh metadata --page metadata`
Expected: the table with every series, the four filters with their counts, the source chips reading AniList or none, the files bars with a `+N` where the disk holds more than the match expects.

Run on a monitor: filter to Missing files after moving a folder out of the sandbox's source (the watcher marks it missing), press Forget, Keep, then Forget and confirm; press Refresh all and Keep; press Attach sources
Expected: the missing row shows the Missing chip and Forget; Keep restores it; confirming removes the row and the series for good; Refresh all's inline confirm sits in the toolbar; Attach sources reports the outcome in a toast.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the metadata table with filters, attach, refresh all, clear match and forget"
```

---

### Task 20: The match modal

Spec 4.1 unit 7 (the match modal): AniList only, a search box seeded with the title, debounced 250 ms, at least two characters; result rows with cover, titles, format, year and episodes; pasting an AniList or MyAnimeList link applies it directly, a MAL id resolving through AniList with a clear message when it cannot; applying replaces the metadata and marks the match confirmed; closing is blocked while an apply is in flight.

**Files:**
- Create: `apps/linux/qml/Modal.qml`, `apps/linux/qml/MatchModal.qml` (replacing the stub)
- Modify: `apps/linux/build.rs`

**Interfaces:**
- Consumes: `Door.searchProvider`, `resolveLink`, `applyMatch`, the `searchFinished`, `linkResolved`, `matchApplied`, `jobFailed` signals.
- Produces: `Modal` (`open`, `title`, `subtitle`, `blocked`, `close()`, `show()`; registers as a popover; ignores Escape and the backdrop while `blocked`); `MatchModal` with `open(seriesId, title)`.

- [ ] **Step 1: Write the modal**

`apps/linux/qml/Modal.qml`:

```qml
// A sheet over the page: a raised surface centred in the overlay with a title, a subtitle
// and an X. Escape and the backdrop close it unless it is blocked.
import QtQuick

Item {
    id: root
    property bool open: false
    property bool blocked: false
    property string title: ""
    property string subtitle: ""
    default property alias content: body.data
    property real sheetWidth: theme.space(140)
    property real sheetHeight: theme.space(150)
    anchors.fill: parent
    visible: open
    z: 950
    function show() { open = true; frame.escapeStack.push("popover", root); sheet.forceActiveFocus() }
    function close() { if (!open || blocked) return; open = false; frame.escapeStack.pop(root) }
    MouseArea { anchors.fill: parent; onPressed: root.close() }
    Rectangle { anchors.fill: parent; color: theme.scrim; opacity: 0.6 }
    Corner {
        id: sheet
        anchors.centerIn: parent
        width: Math.min(root.sheetWidth, parent.width - theme.space(8)); height: Math.min(root.sheetHeight, parent.height - theme.space(8))
        radius: theme.radiusXl; smoothing: theme.cornerSmoothing; color: theme.surface; borderColor: theme.lineStrong; borderWidth: 1
        MouseArea { anchors.fill: parent }
        Column {
            id: head
            x: theme.space(6); y: theme.space(6); width: parent.width - theme.space(12)
            spacing: theme.space(1)
            Text { text: root.title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold }
            Text { visible: root.subtitle !== ""; text: root.subtitle; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal; wrapMode: Text.Wrap; width: parent.width }
        }
        PlayerButton { anchors.right: parent.right; anchors.top: parent.top; anchors.margins: theme.space(4); glyph: "x"; tip: "Close"; enabled: !root.blocked; onClicked: root.close() }
        Item { id: body; anchors.top: head.bottom; anchors.topMargin: theme.space(4); anchors.left: head.left; anchors.right: head.right; anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(6) }
    }
}
```

`apps/linux/qml/MatchModal.qml`:

```qml
// The match modal: AniList only. Search, or paste an AniList or MyAnimeList link.
import QtQuick
import com.marcusrosado.AniBeam

Modal {
    id: modal
    title: "Match metadata"
    subtitle: "Pick a title or paste a link. Its data replaces the current entry."
    property real seriesId: -1
    property string seriesTitle: ""
    property int searchJob: -1
    property int linkJob: -1
    property int applyJob: -1
    property var results: []
    property string message: ""
    property var pendingLink: null
    blocked: applyJob >= 0
    readonly property var anilistLink: /^(?:https?:\/\/)?(?:www\.)?anilist\.co\/anime\/(\d+)/i
    readonly property var malLink: /^(?:https?:\/\/)?(?:www\.)?myanimelist\.net\/anime(?:\/(\d+)|\.php\?id=(\d+))/i
    function open(id, title) {
        seriesId = id; seriesTitle = title; results = []; message = ""; applyJob = -1; searchJob = -1; linkJob = -1; pendingLink = null
        show()
        search.text = title
        Qt.callLater(function() { search.focusInput() })
    }
    function trimmed() { return search.text.trim() }
    function runSearch() {
        var q = trimmed()
        var a = anilistLink.exec(q), m = malLink.exec(q)
        if (a) { applyTarget({ Anilist: { id: Number(a[1]), season: null } }, "AniList #" + a[1]); return }
        if (m) { var r = Door.resolveLink(q); if (r.error) { message = r.error.message; return } linkJob = r.reply.job; pendingLink = q; message = "Resolving through AniList…"; return }
        if (q.length < 2) { results = []; message = "Type at least 2 characters."; return }
        var s = Door.searchProvider(q, 12)
        if (s.error) { message = s.error.message; return }
        searchJob = s.reply.job
        message = "Searching…"
    }
    function applyTarget(target, label) {
        var r = Door.applyMatch(seriesId, target)
        if (r.error) { message = r.error.message; return }
        applyJob = r.reply.job
        message = "Applying " + label + "…"
    }
    Timer { id: debounce; interval: 250; onTriggered: modal.runSearch() }
    Connections {
        target: Door
        function onSearchFinished(job, list) { if (job === modal.searchJob) { modal.results = list; modal.message = list.length ? "" : "No matches."; modal.searchJob = -1 } }
        function onLinkResolved(job, target) { if (job === modal.linkJob) { modal.linkJob = -1; modal.applyTarget(target, target.Anilist ? "AniList #" + target.Anilist.id : "MyAnimeList #" + target.Mal.id) } }
        function onMatchApplied(series) { if (series === modal.seriesId && modal.applyJob >= 0) { modal.applyJob = -1; frame.toast("Matched " + modal.seriesTitle); modal.close() } }
        function onJobFailed(job, kind, error) {
            if (job === modal.applyJob) { modal.applyJob = -1; modal.message = "Could not apply match: " + error.message }
            else if (job === modal.linkJob) { modal.linkJob = -1; modal.message = error.kind === "Provider" && error.status === 404 ? "AniList has no entry for that MyAnimeList id." : "Couldn't read that link. Paste an AniList or MyAnimeList page URL." }
            else if (job === modal.searchJob) { modal.searchJob = -1; modal.message = error.message }
        }
    }

    Column {
        anchors.fill: parent
        spacing: theme.space(3)
        SearchField { id: search; width: parent.width; placeholder: "Search AniList, or paste a link…"; hint: ""; enabled: !modal.blocked; onTextChanged: debounce.restart() }
        Text { visible: modal.message !== ""; text: modal.message; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
        ListView {
            width: parent.width; height: parent.height - search.height - theme.space(12)
            clip: true; spacing: theme.space(1)
            model: modal.results
            delegate: Corner {
                required property var modelData
                width: ListView.view.width; height: theme.space(20)
                radius: theme.radiusMd; smoothing: theme.cornerSmoothing
                color: m.containsMouse ? theme.surfaceRaised : theme.surfaceSunken; borderColor: theme.line; borderWidth: 1
                opacity: modal.blocked ? theme.disabledOpacity : 1
                Row {
                    anchors.fill: parent; anchors.margins: theme.space(2); spacing: theme.space(3)
                    Corner { width: theme.space(11); height: parent.height; radius: theme.radiusSm; smoothing: theme.cornerSmoothing; color: theme.surface
                        fillItem: cover.status === Image.Ready ? cover : null
                        Image { id: cover; visible: false; width: parent.width; height: parent.height; source: modelData.cover_url || ""; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize.width: 120 }
                        Text { visible: !modelData.cover_url; anchors.centerIn: parent; text: "?"; color: theme.textFaint } }
                    Column { anchors.verticalCenter: parent.verticalCenter; width: parent.width - theme.space(14); spacing: theme.space(0.5)
                        Text { width: parent.width; text: modelData.title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold; elide: Text.ElideRight }
                        Text { visible: !!modelData.alt_title && modelData.alt_title !== modelData.title; width: parent.width; text: modelData.alt_title || ""; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall; elide: Text.ElideRight }
                        Text { text: [modelData.format, modelData.year, modelData.episodes ? modelData.episodes + " ep" : null].filter(function(x) { return x }).join("  "); color: theme.textFaint; font.family: theme.fontMono; font.pointSize: theme.typeSmall } }
                }
                MouseArea { id: m; anchors.fill: parent; hoverEnabled: true; enabled: !modal.blocked; cursorShape: Qt.PointingHandCursor; onClicked: modal.applyTarget({ Anilist: { id: modelData.id, season: null } }, modelData.title) }
            }
        }
    }
}
```

`build.rs`: `Modal`, `MatchModal`.

- [ ] **Step 2: Build and match a series on a monitor**

Run: `cargo build --release -p anibeam && apps/linux/scripts/bench.sh match 2 keep --root /tmp/sandbox`, open Metadata, click a title
Expected: the modal opens seeded with the title and the results arrive after the debounce; typing one character shows "Type at least 2 characters."; pasting `https://anilist.co/anime/21` applies at once and the modal closes on `matchApplied`; pasting a MAL link resolves first; pasting a MAL id AniList lacks shows the clear message; while an apply runs Escape, the backdrop and the X do nothing; the row's source chip and files bar update after the apply.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the match modal over AniList with link paste"
```

---

### Task 21: The feed page

Spec 4.1 unit 8 (Feed) and 3.7. The core orders the cards and names the reason; the page shows Recently released or Coming soon, persists the choice in the core's preferences, badges the highest episode on disk for a scheduled card, and draws the divider.

**Files:**
- Create: `apps/linux/qml/FeedPage.qml`
- Modify: `apps/linux/qml/Card.qml` (`episodeBadge` and `metaLeftText` overrides), `apps/linux/qml/Frame.qml` (`feed: feedPage`), `apps/linux/build.rs`

- [ ] **Step 1: Write the page**

`Card.qml` gains two override properties: `property string episodeBadge: ""` (when non-empty it replaces `epBadge`: `readonly property string epBadge: episodeBadge !== "" ? episodeBadge : (item.code || "")`) and `property string metaLeftText: ""` with `property string metaLeftTip: ""` (`metaLeft` becomes `metaLeftText !== "" ? metaLeftText : (...)` and the meta text gets `Tooltip { text: root.metaLeftTip }`).

`apps/linux/qml/FeedPage.qml`:

```qml
// Spec 4.1 unit 8: one card per series. Recently released orders by the latest aired
// episode, else the newest file, and says which; Coming soon lists scheduled series
// soonest first, then everything else after a divider.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    property var props: ({})
    property real scrollY: grid.contentY
    onScrollYChanged: if (Math.abs(grid.contentY - scrollY) > 1) grid.contentY = scrollY
    property string sort: Door.preferences.feed_sort || "Recent"
    property var cards: []
    property real nowMs: Date.now()
    readonly property int dividerAt: sort === "Upcoming" ? cards.findIndex(function(c) { return c.reason.kind !== "Scheduled" }) : -1
    function reload() {
        var r = Door.listFeed(sort)
        if (r.error) { frame.toast(r.error.message); return }
        cards = r.reply.cards.map(function(c) { var k = typeof c.reason === "string" ? c.reason : Object.keys(c.reason)[0]; var v = typeof c.reason === "string" ? {} : c.reason[k]; return { series: c.series, highest: c.highest_on_disk, reason: { kind: k, episode: v.episode, at: v.at } } })
    }
    function pickSort(i) { sort = i === 1 ? "Upcoming" : "Recent"; var p = JSON.parse(JSON.stringify(Door.preferences)); p.feed_sort = sort; Door.setPreferences(p); reload() }
    Component.onCompleted: reload()
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Timer { interval: 30000; running: true; repeat: true; onTriggered: page.nowMs = Date.now() }
    Connections { target: Door; function onSeriesChanged(c) { debounce.restart() } function onSeriesRemoved(i) { debounce.restart() } }

    Column {
        id: header
        anchors.top: parent.top; anchors.topMargin: theme.space(7)
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        anchors.right: parent.right; anchors.rightMargin: theme.space(8)
        spacing: theme.space(4)
        Row { spacing: theme.space(3)
            Text { text: "Feed"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
            Chip { anchors.verticalCenter: parent.verticalCenter; text: page.cards.length + " series"; small: true; color: theme.surface; textColor: theme.textDim } }
        Row { spacing: theme.space(3)
            Seg { options: [{ text: "Recently released", icon: "clock" }, { text: "Coming soon", icon: "calendar-clock" }]; index: page.sort === "Upcoming" ? 1 : 0; onPicked: function(i) { page.pickSort(i) } }
            Text { anchors.verticalCenter: parent.verticalCenter; text: page.sort === "Upcoming" ? "Upcoming episodes, soonest first." : "Your library, ordered by latest episode release."; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall } }
    }
    GridView {
        id: grid
        anchors.top: header.bottom; anchors.topMargin: theme.space(6)
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        anchors.right: parent.right; anchors.rightMargin: theme.space(8)
        anchors.bottom: parent.bottom
        clip: true
        readonly property real gapX: theme.space(5)
        readonly property real gapY: theme.space(6)
        readonly property int columns: Math.max(1, Math.floor((width + gapX) / (theme.posterWidth + gapX)))
        cellWidth: Math.floor((width + gapX) / columns)
        readonly property real cardWidth: cellWidth - gapX
        cellHeight: Math.ceil(cardWidth * 1.5 + theme.space(2) + theme.typeNormal * 2 * 1.5 + theme.typeSmall * 1.5 + theme.space(1)) + gapY
        model: page.cards
        cacheBuffer: 1200
        delegate: Item {
            required property int index
            required property var modelData
            width: grid.cellWidth; height: grid.cellHeight + (index === page.dividerAt && index > 0 ? theme.space(12) : 0)
            Column {
                visible: index === page.dividerAt && index > 0
                width: grid.width; spacing: theme.space(2)
                Text { text: "Everything else"; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall; font.weight: Font.DemiBold }
                Rectangle { width: parent.width; height: 1; color: theme.line }
                Item { width: 1; height: theme.space(2) }
            }
            Card {
                y: index === page.dividerAt && index > 0 ? theme.space(12) : 0
                item: modelData.series; posterWidth: grid.cardWidth; nowMs: page.nowMs
                episodeBadge: modelData.reason.kind === "Scheduled" && modelData.highest ? "EP " + String(Math.floor(modelData.highest)).padStart(2, "0") : ""
                metaLeftText: modelData.reason.kind === "Aired" || modelData.reason.kind === "Downloaded" ? Fmt.relative(modelData.reason.at, page.nowMs / 1000) : ""
                metaLeftTip: modelData.reason.kind === "Aired" ? "Episode aired" : modelData.reason.kind === "Downloaded" ? "File downloaded" : ""
                onOpened: frame.go("series", { id: modelData.series.id }, modelData.series.title)
            }
        }
        footer: Item { width: 1; height: theme.space(10) }
        EmptyState { visible: page.cards.length === 0; icon: "activity"; title: "Nothing here yet"; body: "Add a folder in Settings to build your feed." }
    }
}
```

(`FeedReason` serialises as `"None"` or `{ "Aired": { episode, at } }`; the page flattens it.) `Frame.qml`: `feed: feedPage`. `build.rs`: `FeedPage`.

- [ ] **Step 2: Build and capture**

Run: `cargo build -p anibeam && ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh feed --page feed`
Expected: the cards newest first with the aired or downloaded time on the meta line; switching to Coming soon (`--page feed` after setting `feed_sort` through `anibeam-cli call SetPreferences`) shows the scheduled series first, the divider "Everything else", and `EP NN` badges of the highest episode on disk on scheduled cards.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the feed page"
```

---

### Task 22: The watching page

Spec 4.1 unit 8 (Watching) and 3.7. The cached list paints at once, a refresh runs behind, a failed refresh keeps the list. Owned series are normal cards; the rest are external cards opening AniList; an owned but hidden series is dropped.

**Files:**
- Create: `apps/linux/qml/WatchingPage.qml`, `apps/linux/qml/ExternalCard.qml`
- Modify: `apps/linux/qml/Frame.qml` (`watching: watchingPage`), `apps/linux/build.rs`

- [ ] **Step 1: Write the page**

`apps/linux/qml/ExternalCard.qml` (an AniList-only entry, dashed frame, the AniList pill):

```qml
import QtQuick
import com.marcusrosado.AniBeam
Item {
    id: root
    property var entry: ({})
    property real posterWidth: 180
    property real nowMs: Date.now()
    signal opened()
    width: posterWidth
    implicitHeight: posterWidth * 1.5 + theme.space(2) + info.implicitHeight
    Image { id: poster; visible: false; width: frame.width; height: frame.height; source: entry.poster ? "file://" + entry.poster : ""; sourceSize.width: 480; fillMode: Image.PreserveAspectCrop; asynchronous: true }
    Corner {
        id: frame
        width: root.posterWidth; height: width * 1.5
        radius: theme.radiusLg; smoothing: theme.cornerSmoothing; color: theme.surface
        fillItem: poster.status === Image.Ready ? poster : null
        borderColor: hover.containsMouse ? theme.lineStrong : theme.line; borderWidth: 1; dashed: 1
        Chip { x: theme.space(2); y: theme.space(2); text: "AniList"; small: true; mono: false; textColor: theme.textDim }
        Chip { anchors.right: parent.right; anchors.rightMargin: theme.space(2); y: theme.space(2); text: entry.progress + "/" + (entry.total || "?"); textColor: theme.textDim }
        Chip { visible: entry.score !== null && entry.score !== undefined; x: theme.space(2); anchors.bottom: parent.bottom; anchors.bottomMargin: theme.space(2); small: true; text: Fmt.score(entry.score || 0); textColor: theme.accent }
    }
    Column {
        id: info
        anchors.top: frame.bottom; anchors.topMargin: theme.space(2); width: root.posterWidth; spacing: theme.space(0.5)
        Text { width: parent.width; text: entry.title || ""; color: hover.containsMouse ? theme.accent : theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold; elide: Text.ElideRight; maximumLineCount: 2; wrapMode: Text.Wrap }
        Item { width: parent.width; height: meta.implicitHeight
            Text { id: meta; text: entry.updated_at ? Fmt.relative(entry.updated_at, root.nowMs / 1000) : "-"; color: theme.textFaint; font.family: theme.fontMono; font.pointSize: theme.typeSmall; Tooltip { text: "Last updated on AniList" } }
            Text { anchors.right: parent.right; text: entry.next_airing && entry.next_airing.at * 1000 > root.nowMs ? Fmt.countdown(entry.next_airing.at - root.nowMs / 1000) : ""; color: theme.accent; font.family: theme.fontMono; font.pointSize: theme.typeSmall } }
    }
    MouseArea { id: hover; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.opened() }
}
```

`apps/linux/qml/WatchingPage.qml`:

```qml
// Spec 4.1 unit 8: the AniList watching and repeating list, most recently updated first.
import QtQuick
import com.marcusrosado.AniBeam

Item {
    id: page
    property var props: ({})
    property real scrollY: grid.contentY
    onScrollYChanged: if (Math.abs(grid.contentY - scrollY) > 1) grid.contentY = scrollY
    property var entries: []
    property bool refreshing: false
    property string failure: ""
    property real nowMs: Date.now()
    readonly property bool connected: !!(Door.trackers.anilist && Door.trackers.anilist.connected)
    property var owned: ({})
    function reload() {
        var r = Door.listWatching()
        if (r.error) { failure = r.error.message; return }
        entries = r.reply.list.entries
        refreshing = r.reply.refreshing !== null && r.reply.refreshing !== undefined
        // The owned cards come from the library so they draw like every other card
        var map = {}
        var all = Door.listSeries("All", "", "Alpha", "Asc", false)
        if (!all.error) all.reply.series.forEach(function(c) { map[c.id] = c })
        owned = map
    }
    Component.onCompleted: reload()
    Timer { interval: 30000; running: true; repeat: true; onTriggered: page.nowMs = Date.now() }
    Timer { id: debounce; interval: 250; onTriggered: page.reload() }
    Connections {
        target: Door
        function onWatchingRefreshed(list) { page.entries = list.entries; page.refreshing = false; page.failure = "" }
        function onJobFailed(job, kind, error) { if (kind === "RefreshWatching") { page.refreshing = false; page.failure = error.message } }
        function onSeriesChanged(c) { debounce.restart() }
        function onTrackersChanged(s) { debounce.restart() }
    }
    readonly property var visibleEntries: entries.filter(function(e) { return !(e.owned && !owned[e.owned]) })

    Column {
        id: header
        anchors.top: parent.top; anchors.topMargin: theme.space(7)
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        anchors.right: parent.right; anchors.rightMargin: theme.space(8)
        spacing: theme.space(2)
        Row { spacing: theme.space(3)
            Text { text: "Watching"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
            Chip { anchors.verticalCenter: parent.verticalCenter; text: page.visibleEntries.length + " series"; small: true; color: theme.surface; textColor: theme.textDim }
            Chip { visible: page.refreshing; anchors.verticalCenter: parent.verticalCenter; text: "refreshing"; small: true; mono: false; color: theme.surface; textColor: theme.textFaint } }
        Text { text: "Your AniList watching list. Dashed cards are AniList only; click to open them there." + (page.failure ? "  Last refresh failed: " + page.failure : ""); color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
    }
    GridView {
        id: grid
        anchors.top: header.bottom; anchors.topMargin: theme.space(6)
        anchors.left: parent.left; anchors.leftMargin: theme.space(8)
        anchors.right: parent.right; anchors.rightMargin: theme.space(8)
        anchors.bottom: parent.bottom
        clip: true
        readonly property real gapX: theme.space(5)
        readonly property real gapY: theme.space(6)
        readonly property int columns: Math.max(1, Math.floor((width + gapX) / (theme.posterWidth + gapX)))
        cellWidth: Math.floor((width + gapX) / columns)
        readonly property real cardWidth: cellWidth - gapX
        cellHeight: Math.ceil(cardWidth * 1.5 + theme.space(2) + theme.typeNormal * 2 * 1.5 + theme.typeSmall * 1.5 + theme.space(1)) + gapY
        model: page.visibleEntries
        delegate: Item {
            required property var modelData
            width: grid.cellWidth; height: grid.cellHeight
            Loader {
                sourceComponent: modelData.owned && page.owned[modelData.owned] ? ownedCard : externalCard
                property var entry: modelData
            }
        }
        Component { id: ownedCard; Card { item: page.owned[entry.owned]; posterWidth: grid.cardWidth; nowMs: page.nowMs; onOpened: frame.go("series", { id: entry.owned }, item.title) } }
        Component { id: externalCard; ExternalCard { entry: parent.entry; posterWidth: grid.cardWidth; nowMs: page.nowMs; onOpened: Qt.openUrlExternally(entry.site_url || ("https://anilist.co/anime/" + entry.anilist_id)) } }
        footer: Item { width: 1; height: theme.space(10) }
        EmptyState { visible: !page.connected; icon: "eye"; title: "AniList not connected"; body: "Connect AniList in Settings, Trackers, to see your watching list."
            Button { text: "Settings"; icon: "settings"; onClicked: frame.go("settings", { tab: "library" }) } }
        EmptyState { visible: page.connected && page.visibleEntries.length === 0 && page.failure === ""; icon: "eye"; title: "Nothing on your watching list"; body: "Mark a show as Watching on AniList to see it here." }
        EmptyState { visible: page.connected && page.visibleEntries.length === 0 && page.failure !== ""; icon: "circle-alert"; title: "Couldn't load watching list"; body: page.failure }
    }
}
```

(The `Loader` delegate reads `entry` off its own property in both components; in `ownedCard`, `entry` resolves through the Loader's context.) `Frame.qml`: `watching: watchingPage`. `build.rs`: `WatchingPage`, `ExternalCard`.

- [ ] **Step 2: Build and capture, connected and not**

Run: `cargo build -p anibeam && ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh watching --page watching`
Expected, with no AniList connected in the sandbox: the not-connected state with a Settings button. Against the real data on a monitor with AniList connected: the last list paints at once, the `refreshing` chip shows while the job runs, owned series are normal cards and the rest dashed with the AniList pill, ordered by updated time; a click on an external card opens the browser.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the watching page from the cached AniList list"
```

---

### Task 23: The subscriptions page

Spec 4.1 unit 11 and 3.7. A read-only view of anirss, reachable from Settings only.

**Files:**
- Create: `apps/linux/qml/SubscriptionsPage.qml`
- Modify: `apps/linux/qml/Frame.qml` (`subscriptions: subscriptionsPage`), `apps/linux/build.rs`

- [ ] **Step 1: Write the page**

```qml
// Spec 4.1 unit 11: what anirss watches, read-only. Refresh runs the job again.
import QtQuick
import com.marcusrosado.AniBeam

PageScroll {
    id: page
    property var props: ({})
    property string title: "Subscriptions"
    property var feeds: []
    property string state: "loading"       // loading, ok, Missing, NeedsAuth, Timeout, error
    property string error: ""
    function refresh() { state = "loading"; var r = Door.listSubscriptions(); if (r.error) { state = "error"; error = r.error.message } }
    Component.onCompleted: refresh()
    Connections {
        target: Door
        function onSubscriptionsListed(result) { if (result.kind === "Ok") { page.feeds = result.feeds; page.state = "ok" } else page.state = result.kind }
        function onJobFailed(job, kind, err) { if (kind === "Subscriptions") { page.state = "error"; page.error = err.message } }
    }
    Row { spacing: theme.space(3)
        Chip { text: frame.nav.backLabel; icon: "arrow-left"; mono: false; clickable: true; color: theme.surface; textColor: theme.textDim; onClicked: frame.nav.back(); anchors.verticalCenter: parent.verticalCenter }
        Text { text: "Subscriptions"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold; anchors.verticalCenter: parent.verticalCenter }
        Button { text: "Refresh"; icon: "refresh-cw"; small: true; enabled: page.state !== "loading"; onClicked: page.refresh(); anchors.verticalCenter: parent.verticalCenter } }
    Text { text: "RSS feeds anirss is watching for you."; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeSmall }
    Text { visible: page.state === "loading"; text: "Reading anirss…"; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
    Text { visible: page.state === "Missing"; text: "anirss is not installed or not on PATH."; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
    Text { visible: page.state === "NeedsAuth"; text: "qBittorrent session needed. Run anirss -Sy in a terminal to log in to qBittorrent. AniBeam picks up the cached session."; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; wrapMode: Text.Wrap; width: parent.width }
    Text { visible: page.state === "Timeout"; text: "anirss timed out. Is qBittorrent reachable?"; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
    Text { visible: page.state === "error"; text: "Couldn't read subscriptions: " + page.error; color: theme.red; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
    Text { visible: page.state === "ok" && page.feeds.length === 0; text: "No subscriptions yet. Run anirss in a terminal to subscribe to a feed."; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
    Repeater {
        model: page.state === "ok" ? page.feeds : []
        Corner {
            required property var modelData
            width: parent.width; height: theme.space(16)
            radius: theme.radiusMd; smoothing: theme.cornerSmoothing; color: theme.surface; borderColor: theme.line; borderWidth: 1
            opacity: modelData.active ? 1 : theme.disabledOpacity
            Column {
                anchors.left: parent.left; anchors.leftMargin: theme.space(4); anchors.verticalCenter: parent.verticalCenter
                width: parent.width - theme.space(40); spacing: theme.space(0.5)
                Row { spacing: theme.space(2)
                    Icon { glyph: "rss"; size: theme.space(4); anchors.verticalCenter: parent.verticalCenter }
                    Text { text: modelData.name; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal; font.weight: Font.DemiBold; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideRight; width: theme.space(80) }
                    Chip { text: modelData.active ? "active" : "paused"; small: true; mono: false; textColor: modelData.active ? theme.accent : theme.textDim; anchors.verticalCenter: parent.verticalCenter }
                    Chip { text: Fmt.plural(modelData.torrents, "torrent", "torrents"); small: true; anchors.verticalCenter: parent.verticalCenter; Tooltip { text: "torrents in qBittorrent" } } }
                Text { visible: modelData.query !== ""; text: "query  " + modelData.query; color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall; elide: Text.ElideRight; width: parent.width; Tooltip { text: modelData.url } }
                Row { spacing: theme.space(1); Icon { glyph: "folder-open"; size: theme.space(3.5); color: theme.textFaint; anchors.verticalCenter: parent.verticalCenter }
                    Text { text: modelData.save_path; color: theme.textFaint; font.family: theme.fontMono; font.pointSize: theme.typeSmall; elide: Text.ElideMiddle; width: theme.space(100); anchors.verticalCenter: parent.verticalCenter } }
            }
            Button { anchors.right: parent.right; anchors.rightMargin: theme.space(3); anchors.verticalCenter: parent.verticalCenter; text: "open feed"; icon: "external-link"; flat: true; small: true; onClicked: Qt.openUrlExternally(modelData.url) }
        }
    }
}
```

`Frame.qml`: `subscriptions: subscriptionsPage`. `build.rs`: `SubscriptionsPage`.

- [ ] **Step 2: Build and check the three states**

Run: `cargo build -p anibeam && ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh subscriptions --page subscriptions` with `PATH` lacking anirss
Expected: "anirss is not installed or not on PATH." On the desktop with anirss present and qBittorrent logged in, the rows with name, state, torrent count, the decoded query and the save path; with a stale session, the `anirss -Sy` line. Back reads `Settings` when opened from the Library tab's row.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the subscriptions page over anirss"
```

---

### Task 24: The franchise graph in Related

Spec 4.1 unit 9 and 3.6 (what the shell draws). Drawn once from the core's layout: nodes as `Corner` cards at the core's positions inside a scaled item, edges as one `Shape` with a path per edge, pan by drag, zoom by scroll and pinch, a click on an owned node opens the series in-app, else AniList. Nothing else responds.

**Files:**
- Create: `apps/linux/qml/FranchiseGraph.qml`
- Modify: `apps/linux/qml/SeriesPage.qml` (Related loads `FranchiseGraph`), `apps/linux/build.rs`

- [ ] **Step 1: Write the graph**

```qml
// Spec 4.1 unit 9: the simplified franchise graph. The core computed the layout; this
// draws it once. GraphNode.w and h are 180 by 420 layout units; the shell scales.
import QtQuick
import QtQuick.Shapes
import com.marcusrosado.AniBeam

Item {
    id: root
    property real seriesId: -1
    property var layout: null
    property real zoom: 0.5
    property real panX: 0
    property real panY: 0
    clip: true
    function load() {
        var r = Door.getFranchiseGraph(seriesId)
        if (r.error || !r.reply.layout) return
        layout = r.reply.layout
        centreOnCurrent()
    }
    function centreOnCurrent() {
        var n = layout.nodes.find(function(x) { return x.current }) || layout.nodes[0]
        if (!n) return
        panX = width / 2 - (n.x + n.w / 2) * zoom
        panY = height / 2 - (n.y + n.h / 2) * zoom
    }
    Component.onCompleted: load()
    Connections { target: Door; function onGraphChanged(rootId) { if (root.layout && rootId === root.layout.root) root.load() } }

    Rectangle { anchors.fill: parent; color: theme.surfaceSunken }
    Item {
        id: canvas
        x: root.panX; y: root.panY
        scale: root.zoom
        transformOrigin: Item.TopLeft
        Shape {
            preferredRendererType: Shape.CurveRenderer
            Repeater {
                model: root.layout ? root.layout.edges : []
                ShapePath {
                    required property var modelData
                    readonly property var a: root.layout.nodes.find(function(n) { return n.anilist_id === modelData.from })
                    readonly property var b: root.layout.nodes.find(function(n) { return n.anilist_id === modelData.to })
                    strokeColor: theme.lineStrong; strokeWidth: 2; fillColor: "transparent"
                    startX: a ? a.x + a.w / 2 : 0; startY: a ? a.y + a.h / 2 : 0
                    PathLine { x: b ? b.x + b.w / 2 : 0; y: b ? b.y + b.h / 2 : 0 }
                }
            }
        }
        Repeater {
            model: root.layout ? root.layout.nodes : []
            Corner {
                required property var modelData
                x: modelData.x; y: modelData.y; width: modelData.w; height: modelData.h
                radius: theme.radiusLg * 2; smoothing: theme.cornerSmoothing
                color: theme.surface
                borderColor: modelData.current ? theme.accent : (modelData.root ? theme.lineStrong : theme.line)
                borderWidth: modelData.current || modelData.root ? 4 : 2
                opacity: modelData.pending ? 0.6 : 1
                Column {
                    anchors.fill: parent; anchors.margins: 12; spacing: 8
                    Corner { width: parent.width; height: width * 1.5; radius: theme.radiusMd * 2; smoothing: theme.cornerSmoothing; color: theme.surfaceRaised
                        fillItem: art.status === Image.Ready ? art : null
                        Image { id: art; visible: false; width: parent.width; height: parent.height; source: modelData.poster ? "file://" + modelData.poster : ""; fillMode: Image.PreserveAspectCrop; asynchronous: true; sourceSize.width: 320 }
                        Chip { x: 8; y: 8; text: modelData.owned ? "Owned" : "AniList"; small: true; mono: false; textColor: modelData.owned ? theme.accent : theme.textDim }
                        Chip { visible: !modelData.released; anchors.right: parent.right; anchors.rightMargin: 8; y: 8; text: "Not yet released"; small: true; mono: false; textColor: theme.yellow }
                        StatusDot { anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.margins: 10; status: modelData.list_status || "" } }
                    Text { width: parent.width; text: modelData.title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeNormal * 2; font.weight: Font.DemiBold; elide: Text.ElideRight; maximumLineCount: 2; wrapMode: Text.Wrap }
                    Text { width: parent.width; text: [modelData.relation ? modelData.relation.replace(/_/g, " ").toLowerCase() : null, modelData.format, modelData.year].filter(function(x) { return x }).join("  "); color: theme.textDim; font.family: theme.fontMono; font.pointSize: theme.typeSmall * 2; elide: Text.ElideRight }
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: modelData.owned ? frame.go("series", { id: modelData.owned }, modelData.title) : Qt.openUrlExternally(modelData.site_url || ("https://anilist.co/anime/" + modelData.anilist_id)) }
            }
        }
    }
    MouseArea {
        anchors.fill: parent
        z: -1
        property real lastX: 0
        property real lastY: 0
        onPressed: function(m) { lastX = m.x; lastY = m.y }
        onPositionChanged: function(m) { if (pressed) { root.panX += m.x - lastX; root.panY += m.y - lastY; lastX = m.x; lastY = m.y } }
        onWheel: function(w) {
            var factor = w.angleDelta.y > 0 ? 1.1 : 1 / 1.1
            var next = Math.max(0.15, Math.min(2, root.zoom * factor))
            // zoom around the pointer
            root.panX = w.x - (w.x - root.panX) * (next / root.zoom)
            root.panY = w.y - (w.y - root.panY) * (next / root.zoom)
            root.zoom = next
        }
    }
    PinchHandler { target: null; onScaleChanged: function(delta) { root.zoom = Math.max(0.15, Math.min(2, root.zoom * delta)) } }
}
```

`SeriesPage.qml`: the Related loader's `sourceComponent` becomes `Component { id: relatedGraph; FranchiseGraph { seriesId: page.props.id } }` and the section's count reads `(related.item && related.item.layout ? related.item.layout.nodes.length - 1 : -1)`. `build.rs`: `FranchiseGraph`.

- [ ] **Step 2: Build and look**

Run: `cargo build -p anibeam && ANIBEAM_ROOT=/tmp/sandbox apps/linux/scripts/shoot.sh graph --page series --height 2200` on a series with relations (the sandbox's crawl must have run: `anibeam-cli --root /tmp/sandbox call GetFranchiseGraph --json '{"series": N}'` shows a layout)
Expected: the Related section with the nodes at the core's positions, the current series ringed in the accent and the root in the strong line, edges between centres, Owned and AniList marks, a Not yet released tag where it applies. On a monitor: drag pans, the wheel zooms around the pointer, a click on an owned node opens it, on an unowned one the browser.

- [ ] **Step 3: Commit**

```bash
git add apps/linux
git commit -m "feat(shell): the simplified franchise graph in Related"
```

---
### Task 25: Packaging, the install on both machines, and the switch-line hand-off

Spec 5.3 and 1.5. The PKGBUILD builds the enclosing checkout with `pkgver` from `git describe`, `package.sh` runs it under `target/` so git stays clean, and the package installs the binary, the CLI, the entry, the icon, the bundled `mpv.conf`, the thirty themes and the licence. Then the install on the desktop and the laptop, and the checklist the owner walks. Phase 2's exit is the owner's: every switch-line item green on the real library on both machines.

**Files:**
- Create: `apps/linux/packaging/PKGBUILD`, `apps/linux/packaging/package.sh`
- Modify: `.gitignore` (`target/` is already ignored), `apps/linux/README.md`, `CLAUDE.md` (the native line's commands), `README.md` at the root if it describes running the app

**Interfaces:**
- Consumes: everything above.
- Produces: `/usr/bin/anibeam`, `/usr/bin/anibeam-cli`, `/usr/share/applications/com.marcusrosado.AniBeam.desktop`, `/usr/share/icons/hicolor/512x512/apps/com.marcusrosado.AniBeam.png`, `/usr/share/anibeam/mpv.conf`, `/usr/share/anibeam/themes/*`, `/usr/share/licenses/anibeam/LICENSE`, and the `anibeam-debug` package.

- [ ] **Step 1: Write the PKGBUILD and package.sh**

`apps/linux/packaging/PKGBUILD`, verbatim from spec 5.3:

```bash
# Maintainer: Marcus Rosado
# Builds the enclosing checkout; package.sh copies this file under target/ and runs makepkg
# there, so makepkg's pkgver rewrite, src/, pkg/ and the package files never touch git.
pkgname=anibeam
pkgver=2.0.0   # rewritten by pkgver() on every build
pkgrel=1
pkgdesc="Browse, play, and track your local anime library"
arch=(x86_64)
url="https://github.com/marcusbandit/AniBeam"
license=(GPL-3.0-or-later)
depends=(qt6-base qt6-declarative qt6-svg mpvqt mpv gcc-libs glibc hicolor-icon-theme)
optdepends=('xdg-desktop-portal: colour source fallback when no terminal config is found'
            'gnome-keyring: Secret Service store for tracker tokens')
makedepends=(rust lld git)
# makepkg's default lto option puts -flto=auto into CXXFLAGS; cc-rs passes it to g++ and the
# lld linker cxx-qt-build forces cannot read GCC LTO objects, so every C++ bridge symbol
# comes back undefined. Rust's own LTO is a Cargo profile matter, not a makepkg one.
options=(!lto)
source=()
sha256sums=()

_repo="$(git -C "$startdir" rev-parse --show-toplevel)"
_linux="$_repo/apps/linux"
_target="$_repo/target/makepkg-cargo"

pkgver() {
  git -C "$_repo" describe --tags --dirty | sed 's/^v//;s/-dirty$/.dirty/;s/\([^-]*-g\)/r\1/;s/-/./g'
}

prepare() {
  cd "$_repo"
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd "$_repo"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR="$_target"
  cargo build --frozen --release -p anibeam -p anibeam-cli
}

package() {
  install -Dm755 "$_target/release/anibeam"     "$pkgdir/usr/bin/anibeam"
  install -Dm755 "$_target/release/anibeam-cli" "$pkgdir/usr/bin/anibeam-cli"
  install -Dm644 "$_linux/com.marcusrosado.AniBeam.desktop" \
    "$pkgdir/usr/share/applications/com.marcusrosado.AniBeam.desktop"
  install -Dm644 "$_linux/assets/icon.png" \
    "$pkgdir/usr/share/icons/hicolor/512x512/apps/com.marcusrosado.AniBeam.png"
  install -Dm644 "$_linux/mpv.conf" "$pkgdir/usr/share/anibeam/mpv.conf"
  install -Dm644 -t "$pkgdir/usr/share/anibeam/themes" "$_linux"/themes/*.yaml
  install -Dm644 "$_repo/LICENSE" "$pkgdir/usr/share/licenses/anibeam/LICENSE"
}
```

(One edit against the spec's text: `themes/*.yaml`, so `themes/UPSTREAM` stays out of the package.)

`apps/linux/packaging/package.sh`, verbatim from spec 5.3:

```bash
#!/usr/bin/env bash
# The native line's "bun run package": build the checkout, install it, leave git untouched.
set -euo pipefail
repo="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
mkdir -p "$repo/target/makepkg"
cp "$repo/apps/linux/packaging/PKGBUILD" "$repo/target/makepkg/"
cd "$repo/target/makepkg"
makepkg -fi
```

`chmod +x apps/linux/packaging/package.sh`. `makepkg -i` prompts for sudo at the install step; on this desktop the face scan answers it (run it in a pty: `script -qec apps/linux/packaging/package.sh /dev/null`, after telling the owner to look at the camera), or the owner runs it.

`CLAUDE.md` (the project's): under Commands, add a "Native line" block:

```markdown
### Native line (Rust core + Qt shell)

    cargo test --workspace                       # the core and the shell's unit tests
    cargo build -p anibeam                       # the Linux shell, target/debug/anibeam
    apps/linux/scripts/shoot.sh <name> --page X  # one offscreen capture into apps/linux/captures/
    apps/linux/scripts/bench.sh <name> <ws> keep # the real window on a workspace
    apps/linux/packaging/package.sh              # build, package and install (the launcher runs /usr/bin/anibeam)

The shell's plan is `docs/superpowers/plans/2026-09-05-shell-phase-2.md`; the spec is chapters 4 and 5 of
`docs/superpowers/specs/2026-09-04-native-line-design.md`. The Electron app is frozen on the `electron` branch.
```

- [ ] **Step 2: Build the package on the desktop and prove the launcher run**

Run: `desktop-file-validate apps/linux/com.marcusrosado.AniBeam.desktop`
Expected: no output.

Run: `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check && bun run typecheck`
Expected: green; nothing under `src/` changed.

Run: `script -qec apps/linux/packaging/package.sh /dev/null` (owner at the camera) or ask the owner to run it
Expected: `makepkg` builds under `target/makepkg-cargo`, produces `anibeam-1.0.0.r<n>.g<hash>-1-x86_64.pkg.tar.zst` and its `-debug` sibling, and installs both; `pacman -Ql anibeam` lists the seven kinds of file above; `anibeam --version` and `anibeam-cli --version` agree with `pkgver`.

Run: `gtk-launch com.marcusrosado.AniBeam`, then `journalctl --user -n 50 | grep -i anibeam`
Expected: the window maps with the icon from the entry, no `Could not register app ID` line, and the theme follows the terminal palette; a second `gtk-launch` raises the first window.

- [ ] **Step 3: Install on the laptop**

Run: `ssh kangaeru 'cd ~/Projects/WebApps/AniBeam && git pull && apps/linux/packaging/package.sh'` (the laptop's makepkg asks for sudo; run it in a session where the owner can answer, or hand the command to the owner), then `ssh kangaeru gtk-launch com.marcusrosado.AniBeam`
Expected: the same package builds on the AMD laptop against its Qt 6.11.2; playback lands on `vaapi` (`hwdec-current`); the theme reads the laptop's kitty chain.

- [ ] **Step 4: Hand the switch line to the owner**

Post the checklist as one issue, "Switch line: units 1 to 4, 6 and 7 on both machines", with a checkbox per item of spec 4.1's switch line, two columns (desktop, laptop), and the install day steps from spec 5.3: run `package.sh`, rename the Electron entry's `Name` to `AniBeam (Electron)` in `~/.local/share/applications/anibeam.desktop`, and note that the wrapper's `^anibeam$` match keeps finding only Electron. When the owner marks every item green, `v2.0.0` is cut: set the workspace version to `2.0.0`, tag `v2.0.0` annotated on `main`, and rebuild so `pkgver` reads `2.0.0`.

```bash
gh issue create --title "Switch line: units 1 to 4, 6 and 7 on both machines" --body-file apps/linux/captures/switch-line.md
```

where `switch-line.md` is written from spec 4.1's switch line, one `- [ ]` per bullet under each unit heading, the two machine columns as two copies of the list.

- [ ] **Step 5: Commit and merge**

```bash
git add apps/linux CLAUDE.md
git commit -m "feat(shell): the PKGBUILD, package.sh and the install"
git checkout main && git merge --no-ff feat/shell-phase-2 -m "Merge feat/shell-phase-2: the Linux shell, phase 2 of the native line"
git push origin main && git branch -d feat/shell-phase-2 && git push origin --delete feat/shell-phase-2
```

The merge happens once the package installs on the desktop and the tree is green; the laptop install and the owner's walkthrough follow on `main`.

---

## Self-review notes

Spec coverage, section by section, against the tasks:

- 4.1 unit 1 (frame): Tasks 1, 5, 7, 13, 16. The empty home's Import: Task 8.
- 4.1 unit 2 (library): Task 8.
- 4.1 unit 3 (series): Task 9, the graph in Task 24.
- 4.1 unit 4 (player): Tasks 10 to 13. MPRIS in 13.
- 4.1 unit 6 (settings): Tasks 15 to 18.
- 4.1 unit 7 (metadata, match modal): Tasks 19, 20.
- 4.1 units 8 to 11 (retire line): Tasks 21, 22, 24, 14, 23. Built in phase 2 as spec 1.5 lists them, gating nothing.
- 4.2 (theme model): Tasks 3, 4, 5, 16.
- 4.3 (the look): Task 5 (the primitives, the numbers), Task 8 (the card and the grid), Task 24 (the graph with the same pieces).
- 4.4 (player): Tasks 10, 11, 12, 13.
- 4.5 (single instance, first frame, strip and drawer, settings): Tasks 13, 1 and 7, 7 and 14, 15 to 18.
- 5.1 (stack): Tasks 1, 6. 5.2 (video surface): Task 10. 5.3 (packaging): Task 25. 5.4 (mpv.conf): Task 10.
- Section 3.8's ticks and the two close reasons: Task 10; `Ended`: Task 12.

Two things this plan leaves to the owner or a later ticket, on purpose: the parsers for foot, alacritty and ghostty (spec 4.2 ships kitty first), and the volume gain above 100 (spec 1.6). One thing it asks the executor to watch: cxx-qt 0.10.0 has been built against Qt 6.11.1 on the desktop and never against 6.11.2, which the laptop runs; Task 25's laptop build is where a Qt point release would bite, and the fix is a pinned cxx-qt patch release, never a caret.
