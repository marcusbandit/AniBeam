#!/usr/bin/env bash
# Installs AniBeam desktop integration (launcher entry + icon) for the
# current user. No root needed; everything lands under ~/.local/share.
#
# Works from either location:
#   - inside a packaged build (this script ships in <app>/resources/),
#     e.g. an unzipped GitHub release
#   - a repo checkout after `bun run package` (bun run install:desktop)
#
# Usage: install-desktop.sh [path-to-app-dir]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# --- Locate the app directory (the one holding the `anibeam` binary).
APP_DIR="${1:-}"
if [[ -z "$APP_DIR" ]]; then
  if [[ -x "$SCRIPT_DIR/../anibeam" ]]; then
    # Packaged build: we are in <app>/resources/.
    APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
  elif [[ -x "$SCRIPT_DIR/../out/AniBeam-linux-x64/anibeam" ]]; then
    # Repo checkout: we are in <repo>/scripts/.
    APP_DIR="$(cd "$SCRIPT_DIR/../out/AniBeam-linux-x64" && pwd)"
  else
    echo "error: could not find the packaged app." >&2
    echo "Run 'bun run package' first, or pass the app directory explicitly:" >&2
    echo "  $0 /path/to/AniBeam-linux-x64" >&2
    exit 1
  fi
fi
BIN="$APP_DIR/anibeam"
if [[ ! -x "$BIN" ]]; then
  echo "error: $BIN not found or not executable" >&2
  exit 1
fi

# --- Icon: prefer the copy packaged into resources/, fall back to the repo asset.
ICON_SRC="$APP_DIR/resources/icon.png"
[[ -f "$ICON_SRC" ]] || ICON_SRC="$SCRIPT_DIR/../assets/icon.png"
if [[ ! -f "$ICON_SRC" ]]; then
  echo "error: icon.png not found next to the app or in the repo" >&2
  exit 1
fi

DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}"
ICON_DEST="$DATA_DIR/icons/hicolor/512x512/apps/anibeam.png"
DESKTOP_DEST="$DATA_DIR/applications/anibeam.desktop"

install -Dm644 "$ICON_SRC" "$ICON_DEST"

# --- Exec line. When the machine has a focus-or-launch wrapper, route the
# launch through it so activating the entry focuses a running AniBeam window
# instead of spawning a second instance. The doubled backslash in the regex
# is Desktop Entry escaping: the launcher unescapes it to ^anibeam\$.
WRAPPER=""
if [[ -x "$HOME/.local/bin/focus-or-launch" ]]; then
  WRAPPER="$HOME/.local/bin/focus-or-launch"
elif command -v focus-or-launch >/dev/null 2>&1; then
  WRAPPER="$(command -v focus-or-launch)"
fi
if [[ -n "$WRAPPER" ]]; then
  EXEC_LINE="$WRAPPER \"^anibeam\\\\\$\" \"$BIN\" %U"
else
  EXEC_LINE="\"$BIN\" %U"
fi

mkdir -p "$(dirname "$DESKTOP_DEST")"
cat > "$DESKTOP_DEST" <<EOF
[Desktop Entry]
Type=Application
Name=AniBeam
Comment=Browse, play, and track your local anime library
Exec=$EXEC_LINE
Path=$APP_DIR
Icon=anibeam
Terminal=false
Categories=AudioVideo;Video;Player;
StartupWMClass=AniBeam
Keywords=anibeam;anime;media;video;
EOF

command -v update-desktop-database >/dev/null 2>&1 \
  && update-desktop-database "$DATA_DIR/applications" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null 2>&1 \
  && gtk-update-icon-cache -f -t "$DATA_DIR/icons/hicolor" >/dev/null 2>&1 || true

echo "Installed:"
echo "  $DESKTOP_DEST"
echo "  $ICON_DEST"
echo "Exec: $EXEC_LINE"
