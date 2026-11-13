#!/bin/sh
# AniBeam's installer for Linux and Mac:  curl -fsSL https://anibeam.app/install.sh | sh
# The newest nightly instead:              curl -fsSL https://anibeam.app/install.sh | ANIBEAM_CHANNEL=nightly sh
#
# It installs AniBeam the way the system installs things, so updates arrive with the rest of it.
# On a Mac (Apple silicon): the Homebrew cask when brew is there, otherwise the dmg's app copied
# into /Applications. On Linux it reads /etc/os-release:
#   Arch and its family    anibeam-bin from the AUR with yay or paru; without either, the AppImage
#   Ubuntu, Debian         the apt repo on anibeam.app, then apt install anibeam
#   Fedora                 the dnf repo on anibeam.app, then dnf install anibeam
#   anything else          the AppImage in ~/.local/bin, with a menu entry
# A nightly comes as the AppImage on every Linux, and as the dmg on a Mac: the package repos, the
# AUR and the Homebrew cask carry stable releases only, and the AppImage and the dmg's app update
# themselves on whichever track they came from.
# On Linux it asks for sudo only for the system's own package manager; on a Mac it needs none. Read it first if you like:
# https://anibeam.app/install.sh
# Everything runs from main at the very end, so a download cut off half way runs nothing.
#   ANIBEAM_BASE      where the repos live (default https://anibeam.app), for testing
#   ANIBEAM_APPIMAGE  set to 1 to take the AppImage route on any system
#   ANIBEAM_CHANNEL   stable (the default) or nightly
set -eu

base="${ANIBEAM_BASE:-https://anibeam.app}"
repo="marcusbandit/AniBeam"
channel="${ANIBEAM_CHANNEL:-stable}"

say() { printf '\033[1;33m==>\033[0m %s\n' "$*"; }
fail() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" > /dev/null 2>&1; }

# The release links the site keeps (tools/releases.py), read once and only by the routes that
# download a file; empty when they cannot be had.
releases="" read=""
read_releases() {
  [ -n "$read" ] && return
  read=1
  releases="$(curl -fsSL --max-time 20 "$base/releases.json" 2> /dev/null)" || releases=""
}

# The newest file of one kind (appimage, dmg) on the channel's release, from those links. When
# they cannot be had, GitHub's newest release, or its newest nightly.
asset() {
  read_releases
  if [ "$channel" = nightly ]; then range='/"nightly":/,/"phone":/p'; else range='/"stable":/,/"nightly":/p'; fi
  url="$(printf '%s\n' "$releases" | sed -n "$range" \
    | grep -o "\"$1\": *\"[^\"]*\"" | head -n 1 | sed 's/.*"\(https[^"]*\)"/\1/')" || url=""
  [ -n "$url" ] && return
  if [ "$channel" = nightly ]; then
    # GitHub lists the newest release first. A stable release newer than every nightly is the
    # newest build there is, the same rule releases.py keeps, so it counts too; test builds
    # (-pr.) do not.
    url="$(curl -fsSL "https://api.github.com/repos/$repo/releases?per_page=30" \
      | grep -oE "\"browser_download_url\": *\"[^\"]*/AniBeam-[0-9.]+(-nightly\.[0-9.]+)?-$2\"" | head -n 1 \
      | sed 's/.*"\(https[^"]*\)"/\1/')" || url=""
    return
  fi
  url="$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" \
    | grep -o "\"browser_download_url\": *\"[^\"]*$2\"" | head -n 1 | sed 's/.*"\(https[^"]*\)"/\1/')" || url=""
}

# Nightly asked for, but the site says no nightly is newer than the stable release: the stable
# release goes in the way a nightly would, and Settings moves it onto Nightly later.
pick_channel() {
  case "$channel" in
    stable) return ;;
    nightly) nightly_route=1 ;;
    *) fail "ANIBEAM_CHANNEL is stable or nightly, not $channel" ;;
  esac
  read_releases
  if printf '%s\n' "$releases" | grep -q '"nightly": *null'; then
    say "No nightly is newer than the stable release yet, so the stable release goes in."
    say "Pick Nightly in Settings > About > Updates to move to the next nightly once it is out"
    channel=stable
  fi
}
nightly_route=""

# The dmg's app, not notarized: macOS would stop its first launch, so the quarantine flag that
# a browser or Homebrew sets on it is taken off, as the README tells people to do by hand.
unquarantine() {
  say "Clearing the quarantine flag on $1: the app is not notarized, so macOS would block its first launch"
  xattr -dr com.apple.quarantine "$1" 2> /dev/null || true
}

mac() {
  [ "$(uname -m)" = arm64 ] || fail "AniBeam for Mac is built for Apple silicon only."
  if [ -n "$nightly_route" ]; then
    have brew && say "Nightly comes as the dmg, not the Homebrew cask, which carries stable releases only"
  elif have brew; then
    say "Installing AniBeam with Homebrew"
    brew tap marcusbandit/anibeam "https://github.com/$repo" < /dev/null
    brew install --cask marcusbandit/anibeam/anibeam < /dev/null
    unquarantine /Applications/AniBeam.app
    return
  fi
  say "Finding the newest $channel dmg"
  asset dmg 'macos-arm64\.dmg'
  [ -n "$url" ] || fail "Could not find the dmg on the newest $channel release: https://github.com/$repo/releases"
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
family() { case " $id $like " in *" $1 "*) return 0 ;; esac; return 1; }

# `curl | sh` hands the script to sh on stdin, so nothing here may stop and read an answer
# from it, nor eat the rest of it: every package manager runs non-interactive with stdin closed.
arch() {
  for helper in paru yay; do
    if have "$helper"; then
      say "Installing anibeam-bin from the AUR with $helper"
      "$helper" -S --needed --noconfirm anibeam-bin < /dev/null
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
  $sudo apt-get update < /dev/null
  say "Installing AniBeam"
  $sudo apt-get install -y anibeam < /dev/null
}

fedora() {
  need_root
  say "Adding the AniBeam dnf repo"
  put "$base/rpm/anibeam.repo" /etc/yum.repos.d/anibeam.repo
  say "Installing AniBeam"
  $sudo dnf install -y anibeam < /dev/null
}

appimage() {
  say "Finding the newest $channel AppImage"
  asset appimage 'x86_64\.AppImage'
  [ -n "$url" ] || fail "Could not find the AppImage on the newest $channel release: https://github.com/$repo/releases"
  bin="$HOME/.local/bin"
  mkdir -p "$bin" "$HOME/.local/share/applications" "$HOME/.local/share/icons/hicolor/512x512/apps"
  say "Downloading $(basename "$url")"
  curl -fL --progress-bar "$url" -o "$bin/anibeam.part"
  chmod 755 "$bin/anibeam.part"
  mv "$bin/anibeam.part" "$bin/anibeam"
  curl -fsSL "$base/img/icon.png" -o "$HOME/.local/share/icons/hicolor/512x512/apps/com.marcusrosado.AniBeam.png" || true
  # APPIMAGELAUNCHER_DISABLE: AppImageLauncher, where it is installed, would otherwise offer to
  # "integrate" the file on its first run and move it to ~/Applications, away from this entry
  cat > "$HOME/.local/share/applications/com.marcusrosado.AniBeam.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=AniBeam
Comment=Your anime folder, as a library you can watch and track
Exec=env APPIMAGELAUNCHER_DISABLE=1 "$bin/anibeam" %U
Icon=com.marcusrosado.AniBeam
Categories=AudioVideo;Video;Player;
Terminal=false
EOF
  case ":$PATH:" in *":$bin:"*) ;; *) say "Add $bin to your PATH to start it as anibeam" ;; esac
  say "The AppImage updates itself from Settings > About"
}

main() {
  have curl || fail "curl is needed"
  read_releases
  pick_channel
  if [ "$(uname -s)" = Darwin ]; then
    mac
    say "AniBeam is installed. Open it from Launchpad or Applications"
    return
  fi
  [ "$(uname -s)" = Linux ] || fail "This installer is for Linux and Mac. Windows: $base/download/"
  [ "$(uname -m)" = x86_64 ] || fail "AniBeam is built for x86_64 Linux only for now."
  if [ -r /etc/os-release ]; then
    # shellcheck disable=SC1091
    . /etc/os-release
    id="${ID:-}"
    like="${ID_LIKE:-}"
  fi
  if [ "${ANIBEAM_APPIMAGE:-}" = 1 ]; then
    appimage
  elif [ -n "$nightly_route" ]; then
    case " $id $like " in
      *" arch "* | *" debian "* | *" fedora "*)
        say "Nightly comes as the AppImage: the AUR and the apt and dnf repos carry stable releases only" ;;
    esac
    # A copy from the system's packages stays as it is: the menu entry this writes opens the
    # AppImage, but anibeam typed in a shell may still find the packaged one first
    if [ -x /usr/bin/anibeam ]; then
      say "AniBeam from the system's packages stays in /usr/bin; the menu opens the nightly AppImage. Remove the package to keep one copy"
    fi
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
}

main "$@"
