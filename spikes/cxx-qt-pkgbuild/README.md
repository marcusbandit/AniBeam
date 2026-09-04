# Cargo-only cxx-qt app through a PKGBUILD: throwaway spike

Wayfinder ticket #10. Findings live in `docs/spikes/cxx-qt-pkgbuild.md`. This code is a probe, not a
starting point for the shell; it exists so the numbers in that document can be reproduced.

One crate, no CMake: a Rust QObject singleton (`src/spike.rs`) with a property, an invokable and a
signal that a tokio worker fires through `CxxQtThread::queue`; a C++ `MpvAbstractItem` subclass
(`cpp/spikevideo.h`) registered into the same QML module with `QML_ELEMENT`; `qml/Main.qml` and the app
icon compiled in as resources. `packaging/PKGBUILD` builds the checkout and installs the binary, the
`.desktop` entry and the icon.

Build and run (needs `rust`, `lld`, `qt6-base`, `qt6-declarative`, `mpvqt`, `mpv` from the Arch repos):

    cargo build --release
    QT_FORCE_STDERR_LOGGING=1 target/release/anibeam-spike

The job starts by itself once the window is up and logs `SPIKE ...` lines: the status, the five ticks
with the worker thread id, and the mpv version the C++ item read from its handle.

Package and install:

    cd packaging && makepkg -f
    sudo pacman -U anibeam-spike-0.1.0-1-x86_64.pkg.tar.zst
    gtk-launch anibeam-spike

`options=(!lto)` in the PKGBUILD is not optional; see the write-up for why.
