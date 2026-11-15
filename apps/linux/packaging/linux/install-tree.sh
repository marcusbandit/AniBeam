#!/usr/bin/env bash
# usage: install-tree.sh <checkout> <destdir> <anibeam binary>
# Lays out what a Linux package installs under <destdir>/usr: the binary, the launcher entry,
# the icons, mpv.conf, the themes, the host's user unit and the shell completions. The AUR's
# anibeam (package()) and the anibeam-bin tarball (bin-tarball.sh) both call it, so the two
# packages install the same files.
set -euo pipefail
src="${1:?usage: install-tree.sh <checkout> <destdir> <binary>}"
dest="${2:?usage: install-tree.sh <checkout> <destdir> <binary>}"
bin="${3:?usage: install-tree.sh <checkout> <destdir> <binary>}"
shell="$src/apps/linux"

install -Dm755 "$bin" "$dest/usr/bin/anibeam"
install -Dm644 "$shell/com.marcusrosado.AniBeam.desktop" \
  "$dest/usr/share/applications/com.marcusrosado.AniBeam.desktop"
for size in 16 24 32 48 64 128 256 512; do
  install -Dm644 "$shell/assets/hicolor/default/$size.png" \
    "$dest/usr/share/icons/hicolor/${size}x${size}/apps/com.marcusrosado.AniBeam.png"
done
install -Dm644 "$shell/assets/icon.svg" \
  "$dest/usr/share/icons/hicolor/scalable/apps/com.marcusrosado.AniBeam.svg"
install -Dm644 "$shell/mpv.conf" "$dest/usr/share/anibeam/mpv.conf"
install -Dm644 "$shell/packaging/linux/anibeam-host.service" \
  "$dest/usr/lib/systemd/user/anibeam-host.service"
install -Dm644 "$shell/packaging/linux/completions/_anibeam" \
  "$dest/usr/share/zsh/site-functions/_anibeam"
install -Dm644 "$shell/packaging/linux/completions/anibeam.bash" \
  "$dest/usr/share/bash-completion/completions/anibeam"
mkdir -p "$dest/usr/share/anibeam"
cp -r "$shell/themes" "$dest/usr/share/anibeam/themes"
find "$dest/usr/share/anibeam/themes" -type d -exec chmod 755 {} +
find "$dest/usr/share/anibeam/themes" -type f -exec chmod 644 {} +
