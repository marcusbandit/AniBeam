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

`shoot.sh` forces the RHI backend (QT_QUICK_BACKEND=rhi, QSG_RHI_BACKEND=opengl) under the offscreen
platform when a DISPLAY is set, because Qt's software scene graph cannot paint a Shapes `fillItem`
and every poster comes back white without it, and RHI's GL backend needs GLX for its context. With a
DISPLAY the capture paints through the GPU; without one, forcing RHI aborts instead of falling back
(no context, no PNG), so `shoot.sh` leaves the software backend running instead, and poster frames
come back white.
