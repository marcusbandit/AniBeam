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
command -v cargo >/dev/null 2>&1 || missing+=("Rust 1.95 or newer (rustup.rs)")
# The same qmake the build takes: QMAKE when set, else qmake6 on PATH.
qmake="${QMAKE:-qmake6}"
if ! command -v "$qmake" >/dev/null 2>&1; then
  missing+=("Qt 6.11 or newer, with qmake6 on PATH or QMAKE naming it (base, declarative, svg)")
else
  qt="$("$qmake" -query QT_VERSION 2>/dev/null || true)"
  if [ -z "$qt" ] || [ "$(printf '%s\n6.11\n' "$qt" | sort -V | head -n1)" != "6.11" ]; then
    missing+=("Qt 6.11 or newer ($qmake reports ${qt:-no version})")
  fi
fi
# libmpv: under MPV_PREFIX when set, as the build looks for it, else wherever pkg-config knows.
if [ -n "${MPV_PREFIX:-}" ]; then
  [ -f "$MPV_PREFIX/include/mpv/client.h" ] || missing+=("libmpv under MPV_PREFIX ($MPV_PREFIX)")
elif command -v pkg-config >/dev/null 2>&1; then
  pkg-config --exists mpv || missing+=("libmpv and its headers")
elif [ ! -f /usr/include/mpv/client.h ]; then
  missing+=("libmpv and its headers")
fi
if [ ! -d "${MPVQT_PREFIX:-/usr}/include/MpvQt" ]; then
  missing+=("KDE's MpvQt (or MPVQT_PREFIX pointing at it)")
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
