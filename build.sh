#!/usr/bin/env bash
# Builds the desktop app from this checkout: checks what it needs, then runs
#   cargo build --release -p anibeam
# into target/release/anibeam. Any argument is handed to cargo (./build.sh --locked).
#
# A build from source shows the dev icon, the blueprint in assets/logo/, so it never passes
# for a release. ANIBEAM_ICON=release ./build.sh keeps the release icon.
# docs/build-from-source.md has the rest.
set -euo pipefail
root="$(cd "$(dirname "$0")" && pwd)"

missing=()
need() { command -v "$1" >/dev/null 2>&1 || missing+=("$2"); }
need cargo "Rust 1.95 or newer (rustup.rs)"
need qmake6 "Qt 6.11 or newer, with qmake6 on PATH (base, declarative, svg)"
need pkg-config "pkg-config"
if command -v pkg-config >/dev/null 2>&1 && ! pkg-config --exists mpv; then
  missing+=("libmpv and its headers")
fi
if [ ! -d "${MPVQT_PREFIX:-/usr}/include/MpvQt" ]; then
  missing+=("KDE's MpvQt (or MPVQT_PREFIX pointing at it)")
fi
if command -v qmake6 >/dev/null 2>&1; then
  qt="$(qmake6 -query QT_VERSION)"
  if [ "$(printf '%s\n6.11\n' "$qt" | sort -V | head -n1)" != "6.11" ]; then
    missing+=("Qt 6.11 or newer (qmake6 reports $qt)")
  fi
fi
if [ ${#missing[@]} -gt 0 ]; then
  echo "build.sh: missing:" >&2
  printf '  - %s\n' "${missing[@]}" >&2
  echo "On Arch: pacman -S rust lld qt6-base qt6-declarative qt6-svg mpv mpvqt" >&2
  exit 1
fi

cd "$root"
cargo build --release -p anibeam "$@"

cat <<EOF

Built target/release/anibeam. Run it from here with its themes and mpv.conf:

  ANIBEAM_THEMES_DIR=apps/linux/themes ANIBEAM_MPV_CONF=apps/linux/mpv.conf target/release/anibeam

Add --root <dir> to keep its data apart from an installed Anibeam.
EOF
