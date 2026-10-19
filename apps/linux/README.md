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

Environment the shell sets for itself: QSG_RENDER_LOOP=threaded, QT_XCB_GL_INTEGRATION=xcb_egl.
ANIBEAM_THEMES_DIR and ANIBEAM_MPV_CONF point a dev run at the checkout's themes/ and mpv.conf.

`shoot.sh` and `drive.sh` build into one shared target dir instead of the checkout's own `target/`:
`scripts/target-dir.sh` points `CARGO_TARGET_DIR` at the main checkout's `target/`, so a fresh worktree
reuses the dependency build instead of starting cold (logged cold builds ran 99 s median, 518 s worst).
A `CARGO_TARGET_DIR` already in the environment wins. The tradeoff: builds from different checkouts
queue on one cargo lock, and `target/debug/anibeam` is whichever checkout built last, so an
`ANIBEAM_NO_BUILD=1` capture shows that checkout's code, not yours. Release builds and `bench.sh` stay
per checkout, so a worktree never replaces the binary the launcher starts.

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

    {"wait": ms}                     {"click": [x, y] | "objectName", "button": "right", "double": true}
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
