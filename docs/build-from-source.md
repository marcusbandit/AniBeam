# Build AniBeam from source

Most people want an installer from [Releases](https://github.com/marcusbandit/AniBeam/releases).
This page is for building the app yourself or working on it.

## What you need

Rust 1.95 or newer, Qt 6.11 or newer (`qmake6` on PATH; base, declarative, svg), libmpv,
KDE's MpvQt and the lld linker. On Arch:

```bash
pacman -S rust lld qt6-base qt6-declarative qt6-svg mpv mpvqt
```

## Build and run

```bash
git clone https://github.com/marcusbandit/AniBeam.git
cd AniBeam
cargo build --release -p anibeam     # target/release/anibeam
```

An installed build reads its themes and its bundled `mpv.conf` from `/usr/share/anibeam`. A
run straight from the checkout points at the checkout's copies instead. `--root` keeps all of
its data in one directory, so it never touches a real install's:

```bash
ANIBEAM_THEMES_DIR=apps/linux/themes ANIBEAM_MPV_CONF=apps/linux/mpv.conf \
  target/release/anibeam --root /tmp/anibeam-dev
```

A build from source finds anirss and qbittorrent-nox on your PATH (anirss 0.6.0 or newer,
`yay -S anirss qbittorrent-nox` on Arch).

It also has no AniList or MyAnimeList app registration of its own, so the Trackers tab asks
for a client ID the first time you connect. To bundle your own, copy `.env.example` to
`.env.local` and fill it in before building; it explains where to register.

On a Mac, `apps/linux/packaging/macos/deps.sh` builds libmpv and MpvQt, and `bundle.sh` turns
the binary into `AniBeam.app` and a dmg. On Windows, `apps/linux/packaging/windows/deps.sh`
and `bundle.sh` do the same for the installer and the zip. `.github/workflows/macos.yml` and
`windows.yml` run them.

## Working on it

The Rust core lives in `core/`, the Qt shell in `apps/linux/` (it builds for the Mac and
Windows too), the phone app in `apps/android/`, and a terminal shell over the same core in
`apps/cli/`.

```bash
cargo build --release -p anibeam-cli   # target/release/anibeam-cli
cargo test --workspace                 # unit tests plus core/tests/
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

The CLI is the quickest way to drive the core: `sources`, `scan`, `list`, `show`, `jobs`,
`import`, `export`, `events --follow`, and `call <Name> --json '{...}'` for anything else.
`--root <dir>` puts the database, config, cache and secrets under one directory instead of
the XDG ones.

```bash
anibeam-cli call AddSource --json '{"path": "/mnt/media/anime"}'
anibeam-cli scan --wait
anibeam-cli list --tab all --sort alpha --direction asc
```

## The phone's door

With Share over Tailscale on, AniBeam listens on port 8790 on this computer's Tailscale
addresses only. See what it serves with `curl http://<address>:8790/api/v1/hello` and
`/api/v1/snapshot`; `host-api/src/lib.rs` describes the JSON. `ANIBEAM_HOST_BIND=127.0.0.1:8790`
picks the address by hand, and `0.0.0.0:8790` lets an Android emulator reach it at
`10.0.2.2`.

## Where to read next

- `CONTRIBUTING.md`: issues, branch and commit names, the PR flow, patch notes and releases.
- `apps/linux/README.md`: sandboxed runs, offscreen captures with `scripts/shoot.sh` and
  scripted drives with `scripts/drive.sh`.
- `apps/android/README.md`: building and sideloading the phone app.
- `CONTEXT.md`: the vocabulary the code and the docs share.
