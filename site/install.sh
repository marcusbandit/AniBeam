#!/bin/sh
# AniBeam's installer for Linux and Mac:  curl -fsSL https://anibeam.app/install.sh | sh
#
# It installs AniBeam the way the system installs things, so updates arrive with the rest of it.
# On a Mac (Apple silicon): the Homebrew cask when brew is there, otherwise the dmg's app copied
# into /Applications. On Linux it reads /etc/os-release:
#   Arch and its family    anibeam-bin from the AUR with yay or paru; without either, the AppImage
#   Ubuntu, Debian         the apt repo on anibeam.app, then apt install anibeam
#   Fedora                 the dnf repo on anibeam.app, then dnf install anibeam
#   anything else          the AppImage in ~/.local/bin, with a menu entry
# On Linux it asks for sudo only for the system's own package manager; on a Mac it needs none. Read it first if you like:
# https://anibeam.app/install.sh
#   ANIBEAM_BASE      where the repos live (default https://anibeam.app), for testing
#   ANIBEAM_APPIMAGE  set to 1 to take the AppImage route on any system
set -eu

base="${ANIBEAM_BASE:-https://anibeam.app}"
repo="marcusbandit/AniBeam"

say() { printf '\033[1;33m==>\033[0m %s\n' "$*"; }
fail() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" > /dev/null 2>&1; }

have curl || fail "curl is needed"

# The dmg's app, not notarized: macOS would stop its first launch, so the quarantine flag that
# a browser or Homebrew sets on it is taken off, as the README tells people to do by hand.
unquarantine() { xattr -dr com.apple.quarantine "$1" 2> /dev/null || true; }

mac() {
  [ "$(uname -m)" = arm64 ] || fail "AniBeam for Mac is built for Apple silicon only."
  if have brew; then
    say "Installing AniBeam with Homebrew"
    brew tap marcusbandit/anibeam "https://github.com/$repo"
    brew install --cask marcusbandit/anibeam/anibeam
    unquarantine /Applications/AniBeam.app
    return
  fi
  say "Finding the newest dmg"
  url="$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" \
    | grep -o '"browser_download_url": *"[^"]*macos-arm64\.dmg"' | head -n 1 | sed 's/.*"\(https[^"]*\)"/\1/')"
  [ -n "$url" ] || fail "Could not find the dmg on the newest release: https://github.com/$repo/releases/latest"
  work="$(mktemp -d)"
  say "Downloading $(basename "$url")"
  curl -fL --progress-bar "$url" -o "$work/AniBeam.dmg"
  hdiutil attach -nobrowse -quiet -mountpoint "$work/mnt" "$work/AniBeam.dmg"
  apps=/Applications
  [ -w "$apps" ] || { apps="$HOME/Applications"; mkdir -p "$apps"; }
  rm -rf "$apps/AniBeam.app"
  ditto "$work/mnt/AniBeam.app" "$apps/AniBeam.app"
  hdiutil detach -quiet "$work/mnt"
  rm -rf "$work"
  unquarantine "$apps/AniBeam.app"
  say "AniBeam is in $apps. It updates itself from Settings > About"
}

if [ "$(uname -s)" = Darwin ]; then
  mac
  say "AniBeam is installed. Open it from Launchpad or Applications"
  exit 0
fi
[ "$(uname -s)" = Linux ] || fail "This installer is for Linux and Mac. Windows: $base/#install"
[ "$(uname -m)" = x86_64 ] || fail "AniBeam is built for x86_64 Linux only for now."

# Only the system package managers need root; the AppImage goes in the home folder.
sudo=""
need_root() {
  [ "$(id -u)" -eq 0 ] && return
  have sudo || fail "sudo is needed to install with the system's package manager. ANIBEAM_APPIMAGE=1 installs the AppImage in your home folder instead"
  sudo=sudo
}

# A repo file goes in place only once it has fully arrived, so a failed download never leaves
# an empty one behind for apt or dnf to trip on.
put() {
  tmp="$(mktemp)"
  curl -fsSL "$1" -o "$tmp" || { rm -f "$tmp"; fail "Could not download $1"; }
  $sudo install -m 644 "$tmp" "$2"
  rm -f "$tmp"
}

id="" like=""
if [ -r /etc/os-release ]; then
  # shellcheck disable=SC1091
  . /etc/os-release
  id="${ID:-}"
  like="${ID_LIKE:-}"
fi
family() { case " $id $like " in *" $1 "*) return 0 ;; esac; return 1; }

# `curl | sh` hands the script to sh on stdin, so nothing here may stop and read an answer
# from it: the helpers run with --noconfirm.
arch() {
  for helper in paru yay; do
    if have "$helper"; then
      say "Installing anibeam-bin from the AUR with $helper"
      "$helper" -S --needed --noconfirm anibeam-bin
      return
    fi
  done
  # anibeam-bin needs anirss, an AUR package too, which makepkg alone cannot fetch.
  say "No AUR helper (yay or paru) here, so AniBeam goes in as the AppImage"
  appimage
}

debian() {
  need_root
  say "Adding the AniBeam apt repo"
  $sudo install -d -m 755 /etc/apt/sources.list.d
  put "$base/apt/anibeam.sources" /etc/apt/sources.list.d/anibeam.sources
  $sudo apt-get update
  say "Installing AniBeam"
  $sudo apt-get install -y anibeam
}

fedora() {
  need_root
  say "Adding the AniBeam dnf repo"
  put "$base/rpm/anibeam.repo" /etc/yum.repos.d/anibeam.repo
  say "Installing AniBeam"
  $sudo dnf install -y anibeam
}

appimage() {
  say "Finding the newest AppImage"
  url="$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" \
    | grep -o '"browser_download_url": *"[^"]*x86_64\.AppImage"' | head -n 1 | sed 's/.*"\(https[^"]*\)"/\1/')"
  [ -n "$url" ] || fail "Could not find the AppImage on the newest release: https://github.com/$repo/releases/latest"
  bin="$HOME/.local/bin"
  mkdir -p "$bin" "$HOME/.local/share/applications" "$HOME/.local/share/icons/hicolor/512x512/apps"
  say "Downloading $(basename "$url")"
  curl -fL --progress-bar "$url" -o "$bin/anibeam.part"
  chmod 755 "$bin/anibeam.part"
  mv "$bin/anibeam.part" "$bin/anibeam"
  curl -fsSL "$base/img/icon.png" -o "$HOME/.local/share/icons/hicolor/512x512/apps/com.marcusrosado.AniBeam.png" || true
  cat > "$HOME/.local/share/applications/com.marcusrosado.AniBeam.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=AniBeam
Comment=Your anime folder, as a library you can watch and track
Exec=$bin/anibeam %U
Icon=com.marcusrosado.AniBeam
Categories=AudioVideo;Video;Player;
Terminal=false
EOF
  case ":$PATH:" in *":$bin:"*) ;; *) say "Add $bin to your PATH to start it as anibeam" ;; esac
  say "The AppImage updates itself from Settings > About"
}

if [ "${ANIBEAM_APPIMAGE:-}" = 1 ]; then
  appimage
elif family arch && have pacman; then
  arch
elif family debian && have apt-get; then
  debian
elif { family fedora || [ "$id" = fedora ]; } && have dnf; then
  fedora
else
  appimage
fi

say "AniBeam is installed. Open it from your app menu, or run anibeam"
