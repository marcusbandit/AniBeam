# AniBeam Linux shell

Qt 6.11 QML over the Rust core through cxx-qt 0.10, built with Cargo alone. Spec: chapters 4 and 5 of
`docs/superpowers/specs/2026-09-04-native-line-design.md`; plan: `docs/superpowers/plans/2026-09-05-shell-phase-2.md`.

    cargo build -p anibeam                          # needs qmake6 on PATH, lld, mpvqt, qt6-svg
    target/debug/anibeam --root /tmp/sandbox        # a sandboxed run; without --root the real XDG dirs
    target/debug/anibeam --version
    scripts/shoot.sh library --page library         # one offscreen capture into captures/
    scripts/shoot.sh series --page series --props '{"id":5}'   # a specific series, not the first alphabetically
    scripts/shoot.sh drawer --page library:drawer   # the activity drawer open over a page
    scripts/bench.sh player 2 keep                  # the real window on the main monitor's workspace 2
    packaging/package.sh                            # build, package, install (Task 25)

Environment the shell sets for itself: QSG_RENDER_LOOP=threaded (not on a Mac, where AppKit
forbids an OpenGL context off the main thread), QT_XCB_GL_INTEGRATION=xcb_egl.
ANIBEAM_THEMES_DIR and ANIBEAM_MPV_CONF point a dev run at the checkout's themes/ and mpv.conf.

`ANIBEAM_QML_DIR=qml` (run from `apps/linux/`, like every command here) loads the shell's QML from that directory instead of the copy compiled
into the binary, so a QML-only edit shows up on the next run with no cargo build (a relink of the
debug binary is minutes; the QML parse is a fraction of a second). It exists in debug builds only. A
URL interceptor in `cpp/helpers.cpp` redirects each `qrc:/qt/qml/com/marcusrosado/AniBeam/qml/<name>.qml`
the engine asks for to `<dir>/<name>.qml` when that file exists, so `Main.qml`, the pages and the shared
components all come from disk. Edit, then run with the binary as it stands:

    ANIBEAM_NO_BUILD=1 ANIBEAM_QML_DIR=qml scripts/shoot.sh library --page library

Measured on a loaded machine, a library shot took 1.65 s from the resources and 1.69 s from disk with
Qt's compile cache warm, 1.8 s to 1.9 s with it cold or off. `shoot.sh` and `drive.sh` keep that cache
in the sandbox root's `qmlcache/` rather than `~/.cache`. What still needs a cargo build: Rust and C++
changes, a QML file added to or removed from `build.rs`, and the build's own QML checks, so a syntax
error in a disk file shows up at run time as a load failure in the `.log`, not at build time. The scripts
turn a relative dir into an absolute one against the directory they run from; a dir without `Main.qml` is
reported in the `.log` and ignored, and a file missing from the dir is reported once and loaded from the
compiled copy. Build before the last run of a task all the same: the binary the launcher starts has the QML compiled in.

`shoot.sh` and `drive.sh` build into one shared target dir instead of the checkout's own `target/`:
`scripts/target-dir.sh` points `CARGO_TARGET_DIR` at the main checkout's `target/` (when that is an
AniBeam checkout), so a fresh worktree reuses the dependency build instead of starting cold (logged
cold builds ran 99 s median, 518 s worst). A `CARGO_TARGET_DIR` already in the environment wins.
Right after each build the scripts copy the binary to the checkout's own
`target/debug/anibeam-staged` and run that copy, so a capture always runs this checkout's last build
even when another worktree links the shared `debug/anibeam` meanwhile. `ANIBEAM_NO_BUILD=1` skips the
build and reuses that staged copy. The tradeoff: builds from different checkouts queue on one cargo
lock. Release builds and `bench.sh` stay per checkout, so a worktree never replaces the binary the
launcher starts.

`--props '<json>'` hands a shot page its props (Main.qml parses it and passes it to `nav.replace`),
for a page that needs more than a bare name to draw anything real: `--page series` with no `id` in
`--props` falls back to the first series alphabetically, so `scripts/shoot.sh series --page series`
still works with a plain sandbox and no `--props` at all. `target/debug/anibeam-cli --root
/tmp/sandbox list` prints series ids to pick one deliberately.

`shoot.sh` forces the RHI backend (QT_QUICK_BACKEND=rhi, QSG_RHI_BACKEND=opengl) under the offscreen
platform when a DISPLAY is set, because Qt's software scene graph cannot paint a Shapes `fillItem`
and every poster comes back white without it, and RHI's GL backend needs GLX for its context. With a
DISPLAY the capture paints through the GPU; without one, forcing RHI aborts instead of falling back
(no context, no PNG), so `shoot.sh` leaves the software backend running instead, and poster frames
come back white.

`--page` takes an optional `:<action>` suffix for a shoot that needs the frame in a state a bare
page name cannot reach on its own. `library:drawer` opens the library page as usual, then calls
`frame.toggleDrawer()` once the page is up and before the grab, so the shoot's own settle delay
doubles as the drawer's rise time. `Main.qml` splits the suffix off before ever comparing against a
page name, so `--page library:drawer` still opens `library`, not a page literally named that.

`--drive <script.json>` is the interactive sibling of `--shoot` (the two are exclusive): once the
frame is up it runs a JSON array of steps against the offscreen window and exits 0, or prints
`DRIVE FAIL: <reason>` and exits 1. `scripts/drive.sh <script.json> [anibeam args...]` wraps it
like `shoot.sh` does (same `ANIBEAM_ROOT`, `W`/`H`; shots land in `captures/` unless
`ANIBEAM_CAPTURES` or `--out <dir>` says otherwise) and prints the `DRIVE STEP <n>: ...` log.
`scripts/drive/smoke.json` is the worked example. `scripts/drive/downloads.json` drives the Downloads
page against the real anirss and qBittorrent on the machine (a search, Best fit, the rule preview; it
adds nothing), so it needs both installed and skips nothing when they are not.

    {"wait": ms}                     {"until": "js", "timeout": ms, "expect": <json>, "as": "k"}
    {"click": [x, y] | "objectName", "button": "right", "double": true}
    {"hover": [x, y] | "objectName"} {"wheel": [x, y, dy]}  or  {"wheel": "objectName", "dy": -120}
    {"key": "Escape" | "Ctrl+K" | "Alt+Left" | "Shift+Tab" | "a"}     {"type": "some text"}
    {"shot": "name"}                 {"log": "text"}        {"page": "settings", "props": {}}
    {"eval": "frame.item.nav.current.page", "expect": "settings", "as": "k"}

Input enters through QtGui's window-system entry points (`qt_handleMouseEvent` and friends, the
ones QtTest uses, declared in `cpp/helpers.cpp`), so Shortcut, Keys, MouseArea hover, pointer
handlers, Popup close-on-press-outside and double-click synthesis all behave as with a real
pointer. A click is move, press, release 30 ms apart. An objectName target is the first visible
item of that name in the window's tree, open Popups included; `Rail`, `Seg` (cells are
`<name>-<label>`) and `Dropdown` (`<name>-popup`, `<name>-option-<i>`) carry names for this.
`eval` runs in Main.qml's scope (`window`, `frame`, `Theme`, `driver.item.findItem(name)`), and
`vars.k` reads what an earlier `"as": "k"` stored.

`{"wait": ms}` is a fixed pause. Use it only for time that is the point, such as an animation
settling or a hover tip's own delay; a wait for the app to get somewhere is `{"until": "js"}`.
`until` polls the JS (same scope as `eval`) every 25 ms and goes on the moment it is truthy, or
equals `"expect"` when one is given; `"as"` keeps the value. It fails the run with `DRIVE FAIL:
step <n>: until timed out after <ms> ms: <js> (last: <value>)` once `"timeout"` passes (5000 ms
unless set), so a state that never arrives costs the timeout and a loud failure, not a silent
sleep. A throw while polling counts as not ready, because the page or item it reads may not exist
yet. Give slow work such as a network search a generous timeout:

    {"key": "Return"},
    {"until": "frame.item.pageItem.searching === false && frame.item.pageItem.state !== null", "timeout": 60000}

Two proxies flip before the thing is ready, so do not wait on them alone. `nav.current.page`
changes at once while Frame swaps the page in a tick later: wait on
`driver.item.pageReady('series')`, which is true once that page's item is the one loaded. The
activity drawer sets `open` at once and eases `openness` over about 300 ms: wait on
`frame.item.drawerPanel.open === true && frame.item.drawerPanel.openness > 0.99` to see it up,
and on `drawerPanel.open === false && drawerPanel.visible === false` to see it gone.

An `until` is a readiness check, not an assertion that something stays put: where a script waits
to prove nothing happens (a tip that must not show, a click that must not close a panel), keep
the `wait`.

To capture a set of pages, drive them in one run instead of calling `shoot.sh` once per page: each
`shoot.sh` pays a full app start plus its own settle delay, and a drive run pays them once. A `page`
step with a `shot` opens the page, waits until the frame shows it and a new page item stands in it,
settles 300 ms (`"settle": ms` changes that), and grabs `<name>.png`. `"until": "js"` adds a
readiness condition for a page whose data arrives late (it takes `"expect"` and `"as"` like a plain
`until`), and `"timeout"` bounds the wait (15000 ms
unless set). `"pages"` takes a list of such steps:

    {"pages": [
      {"page": "library", "shot": "set-library"},
      {"page": "series", "props": {"id": 1}, "shot": "set-series"},
      {"page": "settings", "shot": "set-settings"}
    ]}

`scripts/drive/capture-pages.json` is that, ready to copy and edit. It opens series id 1, so it
needs an `ANIBEAM_ROOT` that holds a library with that series; against the default empty root its
first step fails with a plain message instead of shooting an error page, and the other pages
would only show empty states. `target/debug/anibeam-cli --root <dir> list` prints the ids, and
`anibeam-cli --root <dir> import <export.json> --wait` seeds a root from an export file:

    ANIBEAM_ROOT=<dir> scripts/drive.sh scripts/drive/capture-pages.json   # one PNG per page in captures/

## macOS

The same shell builds for Apple silicon, from `.github/workflows/macos.yml` on a macos-15
runner; nothing here builds it on Linux. `packaging/macos/` holds the steps the workflow runs:

    deps.sh <prefix>        libmpv 0.41 (lean: no Lua, VapourSynth or Vulkan) and MpvQt 1.2 into a prefix
    MPVQT_PREFIX=<prefix> MPV_PREFIX=<prefix> QMAKE=$(brew --prefix qtbase)/bin/qmake \
      cargo build --release -p anibeam -p anibeam-cli
    DEPS_PREFIX=<prefix> packaging/macos/bundle.sh dist   AniBeam.app and the dmg, ad-hoc signed
    packaging/macos/smoke.sh dist/AniBeam.app target/release/anibeam-cli smoke

`bundle.sh` runs macdeployqt, then `fix-dylibs.py`, which copies in whatever still loads from
outside the bundle and fails if anything does. `smoke.sh` seeds a library of generated clips,
shoots the pages and drives the player from the bundle; the workflow runs it with Homebrew
moved away, and `crash-report.py` prints the crashed thread when a run dies. On a Mac the
themes and the bundled mpv.conf come from `Contents/Resources`, the lock from `$TMPDIR`, and
tokens go to the login keychain. A runner's only OpenGL is Apple's software renderer, so the
smoke's second player run uses a cheap mpv profile to show the picture reaching the window.
