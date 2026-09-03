# Spike: a Cargo-only cxx-qt app packages through a PKGBUILD

Resolves wayfinder ticket #10 on the native line map (#2). Run on 2026-09-03 on banditbox: Arch, Hyprland 0.56.1, RTX 3090 with nvidia-utils 610.43.03, Rust 1.92.0 stable through rustup 1.29.0 (pacman sees the rustup package as providing rust and cargo), GCC 16.1.1, lld 22.1.8, pacman 7.1.0 with its makepkg, qt6-base 6.11.1-1, qt6-declarative 6.11.1-3, mpvqt 1.2.0-1, mpv 0.41.0-3.

Code: `spikes/cxx-qt-pkgbuild/` on this branch. One Cargo crate and no CMake anywhere: a Rust QObject singleton with a property, an invokable and a signal fired from a tokio worker; a C++ `MpvAbstractItem` subclass registered into the same QML module; a QML window and the app icon compiled in as resources; a PKGBUILD under `packaging/` that builds the crate and installs the binary, a `.desktop` entry and the icon.

## Answer

Yes. `cargo build --release` links the binary, `makepkg` turns it into `anibeam-spike-0.1.0-1-x86_64.pkg.tar.zst`, and the desktop entry starts it from a launcher. Nothing needed CMake. Two things `find_package(MpvQt)` used to supply silently now sit in build.rs by hand: the MpvQt include directory and the two link lines. One PKGBUILD option is mandatory: `options=(!lto)`. makepkg's default LTO flag reaches g++ through cc-rs, and the lld linker that cxx-qt-build forces cannot read GCC LTO objects, so the link ends with every C++ bridge symbol undefined.

`pacman -U` installed the package on 2026-09-04, three files: the binary, the entry and the icon. `gtk-launch anibeam-spike` against the entry under `/usr/share/applications` then started `/usr/bin/anibeam-spike`: the window mapped 818 ms after exec, the journal carried the same five ticks and the mpv version, and the portal app-id complaint from the bare run did not appear. Before the install, the same entry copied under the home directory had already started the app the same way; that copy was removed so it could not shadow the package's.

## The crate

```
spikes/cxx-qt-pkgbuild/
  Cargo.toml              cxx-qt, cxx-qt-lib and cxx-qt-build pinned with "=0.10.0"; tokio rt-multi-thread + time
  build.rs                the whole build, below
  src/main.rs             the tokio runtime in a OnceLock, QGuiApplication, QQmlApplicationEngine
  src/spike.rs            the bridge: the Spike singleton and the two C++ helper declarations
  cpp/spikevideo.h/.cpp   SpikeVideo : MpvAbstractItem, QML_ELEMENT, one property
  cpp/helpers.h/.cpp      QQuickWindow::setGraphicsApi(OpenGL) and QGuiApplication::setDesktopFileName
  qml/Main.qml            the window
  assets/icon.png         the current app icon, 512 px, compiled into the binary and installed for the entry
  anibeam-spike.desktop
  packaging/PKGBUILD
```

Locked versions: cxx-qt, cxx-qt-lib, cxx-qt-build and qt-build-utils 0.10.0, cxx 1.0.199, cc 1.4.4, tokio 1.53.1.

build.rs, complete:

```rust
use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("dev.anibeam.spike")
            .version(1, 0)
            .qml_file("qml/Main.qml"),
    )
    .qt_module("Quick")
    .files(["src/spike.rs"])
    .include_dir("cpp")
    .include_dir("/usr/include/MpvQt")
    .cpp_files(["cpp/spikevideo.h", "cpp/spikevideo.cpp", "cpp/helpers.cpp"])
    .qrc_resources(["assets/icon.png"])
    .build();

    println!("cargo:rustc-link-lib=MpvQt");
    println!("cargo:rustc-link-lib=mpv");
}
```

What that one call does, read off the build output: finds Qt through `qmake6`, writes the qmldir and the qrc, runs moc on the generated bridge header and on `spikevideo.h`, rcc on the resources, qmlcachegen on Main.qml, qmltyperegistrar on the moc JSON, compiles the generated C++ plus the three listed files with `c++ -std=c++17 -O3` and the Qt include paths, and emits the link lines for Qt6Quick, Qt6OpenGL, Qt6Qml, Qt6Network, Qt6Gui and Qt6Core. The Rust link goes through `-fuse-ld=lld` because the system `ld` is GNU bfd.

Paths that follow from the module URI: Main.qml is `qrc:/qt/qml/dev/anibeam/spike/qml/Main.qml` and the icon is `qrc:/qt/qml/dev/anibeam/spike/assets/icon.png`. Every Rust file holding a bridge must sit in one directory per QML module (cxx-qt panics otherwise, citing QTBUG-93443), so the bridge lives in `src/spike.rs` alone and `src/main.rs` carries none.

## The bridge and the signal path

The bridge declares `Spike` as `#[qobject] #[qml_element] #[qml_singleton]` with `#[qproperty(i32, counter)]` and `#[qproperty(QString, status)]`, a `#[qsignal] tick(n: i32, worker_thread: QString)`, a `#[qinvokable] start_job(steps: i32)`, and `impl cxx_qt::Threading for Spike {}`. `#[auto_cxx_name]` on the block gives QML `startJob` for `start_job`. The QML engine constructs and owns the singleton, so the bridge reaches the tokio runtime through a `OnceLock` in main.rs rather than a constructor argument.

The invokable returns at once. It sets `status` on the calling thread, clones `self.qt_thread()` and spawns a task on the runtime. The task sleeps 300 ms per step and posts each step with `qt.queue(move |mut spike: Pin<&mut Spike>| { spike.as_mut().set_counter(n); spike.as_mut().tick(n, worker); })`. The closure runs on the Qt thread, which is the only place the Rust struct is touched. A `queue` that returns `Err` means the QObject is gone, and the task returns.

Observed in the QML handler, which logs `Spike.counter` and the worker's thread id on every tick:

```
SPIKE status job of 5 started on ThreadId(1)
SPIKE tick 1 counter 1 worker ThreadId(3)
SPIKE tick 2 counter 2 worker ThreadId(3)
...
SPIKE tick 5 counter 5 worker ThreadId(3)
SPIKE status job of 5 finished
```

Thread 1 is the main thread that owns QGuiApplication; thread 3 (thread 2 on the launcher run) is a tokio worker named `anibeam-core`. The property already held the new value when the signal handler ran, and the five ticks arrived in order.

Two things needed a C++ line each because cxx-qt-lib does not wrap them: `QQuickWindow::setGraphicsApi(QSGRendererInterface::OpenGL)` before the application exists (the mpv item is a QQuickFramebufferObject and wants the OpenGL scene graph), and `QGuiApplication::setDesktopFileName`. Both are free functions in `helpers.cpp`, declared to Rust inside the same `#[cxx_qt::bridge]` in an `unsafe extern "C++"` block with `include!("helpers.h")`.

## The C++ item

`SpikeVideo` subclasses `MpvAbstractItem`, carries `Q_OBJECT` and `QML_ELEMENT`, and exposes one `mpvVersion` property filled from `getProperty("mpv-version")` when the item's `ready` signal fires. It goes through `cpp_files` like any other C++ source. The header gets moc with the module URI attached, and the generated `dev_anibeam_spike_qmltyperegistration.cpp` registers it beside the Rust type:

```cpp
qmlRegisterTypesAndRevisions<Spike>("dev.anibeam.spike", 1);
qmlRegisterTypesAndRevisions<SpikeVideo>("dev.anibeam.spike", 1);
```

Main.qml instantiates it as a 96 px strip. The item constructs an mpv handle on its own thread and reports `mpv v0.41.0` about half a second after the window maps. Nothing is loaded into it; rendering was the libmpv spike's job.

## What CMake was hiding

The first build failed inside MpvQt's own headers: `mpvqt_export.h` has `#include <mpvqt_version.h>` with no directory, which the CMake target satisfied through its interface include path. mpvqt ships CMake config files and no `.pc` file, so build.rs names `/usr/include/MpvQt` as an include directory and links `MpvQt` and `mpv` by hand (mpv itself does ship `mpv.pc`; it was not needed). That is the whole list of things CMake did for the mpv spike that Cargo does not do here.

## The PKGBUILD

```
depends=(qt6-base qt6-declarative mpvqt mpv gcc-libs glibc hicolor-icon-theme)
makedepends=(rust lld)
options=(!lto)
```

`prepare()` copies the crate into `$srcdir` and runs `cargo fetch --locked` for the host target; `build()` exports `RUSTUP_TOOLCHAIN=stable` and `CARGO_TARGET_DIR=target` and runs `cargo build --frozen --release`; `package()` installs the binary to `/usr/bin`, the entry to `/usr/share/applications` and the icon to `/usr/share/icons/hicolor/512x512/apps`. The spike builds the checkout one directory up because it has no tag; the shell's PKGBUILD fetches a tagged tarball and keeps the rest.

The PKGBUILD lives in `packaging/` rather than beside Cargo.toml because makepkg creates `src/` and `pkg/` next to the PKGBUILD, and `src/` is the crate's source directory.

makepkg's `CFLAGS` and `CXXFLAGS` reach the C++ half through cc-rs, so the generated bridge and the item compile with `-march=x86-64 -O2 -fstack-clash-protection -fcf-protection -D_FORTIFY_SOURCE=3 -D_GLIBCXX_ASSERTIONS` and the rest of the distribution set. Nothing from makepkg reaches rustc: cargo ignores `LDFLAGS`, and this box's makepkg.conf sets no `RUSTFLAGS`.

The LTO failure, for whoever hits it next: makepkg's default `OPTIONS` include `lto`, which appends `LTOFLAGS="-flto=auto"` to `CXXFLAGS`. g++ then writes GIMPLE bytecode objects into the static archive cxx-qt-build produces, rust-lld cannot read them, and the link fails with 19 undefined symbols of the form `cxxbridge1$199$use_opengl_scene_graph`, `rust$cxxqtlib1$cxxbridge1$199$qguiapplication_new` and `cxxbridge1$unique_ptr$QQmlApplicationEngine$load`. `options=(!lto)` was the only change between the failing run and the passing one. Rust's own LTO stays a Cargo profile matter.

makepkg's packaging checks reported nothing. It also produced an `anibeam-spike-debug` package from the stripped symbols, as the default `debug` option does.

## Numbers

| Measure | Value |
|---|---|
| Clean `cargo build --release --frozen`, 12 threads | 87 s wall |
| Clean `makepkg -f` (prepare, build, strip, debug package, zstd) | 104 s wall |
| Incremental rebuild after a QML or Rust edit | under 4 s |
| Binary, cargo release, unstripped | 2,760,416 bytes |
| Binary as installed by the package, stripped | 1,952,952 bytes |
| Package file | 884,000 bytes |
| Installed size reported by pacman | 2,242 KiB |
| Window mapped after exec, both runs | 816 ms |

Shared libraries the binary asks for beyond libc: Qt6Quick, Qt6Gui, Qt6Qml, Qt6Core, Qt6QmlMeta, Qt6QmlModels, Qt6QmlWorkerScript, Qt6Network, Qt6OpenGL, Qt6DBus, MpvQt (soname 3), mpv (soname 2), stdc++, zstd. Every one is owned by a package in `depends`.

## Desktop entry

`anibeam-spike.desktop` has `Exec=anibeam-spike`, `Icon=anibeam-spike`, `Categories=AudioVideo;Video;` and `StartupWMClass=anibeam-spike`; `desktop-file-validate` passes it. The binary calls `setDesktopFileName("anibeam-spike")`, so the Wayland app id is `anibeam-spike` and Hyprland reports the window's class as such, which is what pairs the window with the entry's icon in a panel.

Without an entry installed, Qt logs `qt.qpa.services: Failed to register with host portal ... Could not register app ID: App info not found for 'anibeam-spike'` at startup. With the entry under `~/.local/share/applications` and the icon under `~/.local/share/icons/hicolor/512x512/apps`, `gtk-launch anibeam-spike` started the app, the line was gone from the journal, and the window behaved as in the direct run. That user-level copy was removed afterwards so it cannot shadow the package's entry.

Screenshots, both on the portrait DP-1 monitor: [cargo-run.jpg](cxx-qt-pkgbuild/cargo-run.jpg) from the plain binary and [launcher-run.jpg](cxx-qt-pkgbuild/launcher-run.jpg) from the desktop entry. The icon at the top left is the qrc resource, the black strip is the mpv item's framebuffer.

## Noise worth knowing about

GCC 16 prints a `-Wsfinae-incomplete` warning about `QChar` for every C++ file that includes `QString`. It is harmless and comes from Qt's headers; `-Wno-sfinae-incomplete` through `cc_builder` would silence it.

Qt 6.11 deprecates reading a property inside its own change handler without qualification (`onTextChanged: console.log(text)` warns about injected parameters). Qualify with the item's id.

The scene graph runs the basic render loop by default on this NVIDIA box and gets an OpenGL ES 3.2 context, the same as the libmpv spike observed; the shell forces the threaded loop as that spike decided.

## Environment notes for whoever reruns this

```
cd spikes/cxx-qt-pkgbuild
cargo build --release                   # needs qmake6 on PATH, lld installed
QT_FORCE_STDERR_LOGGING=1 target/release/anibeam-spike
cd packaging && makepkg -f              # writes anibeam-spike-0.1.0-1-x86_64.pkg.tar.zst
sudo pacman -U anibeam-spike-0.1.0-1-x86_64.pkg.tar.zst
gtk-launch anibeam-spike                # or pick "AniBeam cxx-qt spike" in the launcher
sudo pacman -Rns anibeam-spike          # when done
```

The window maps on the focused monitor; the runs here moved it to DP-1 workspace 6 with `hyprctl dispatch 'hl.dsp.window.move({ workspace = 6, silent = true, window = "class:anibeam-spike" })'` and shot it with `grim -o DP-1`.
