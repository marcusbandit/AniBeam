#!/usr/bin/env bash
# usage: bin-tarball.sh <version> <outdir>        e.g. bin-tarball.sh 2.5.0 dist
# Builds the release binary and packs what the AUR's anibeam-bin installs into
# <outdir>/AniBeam-<version>-linux-x86_64.tar.zst, with a .sha256 beside it. The tarball holds
# a usr/ tree laid out by install-tree.sh, the same files the anibeam package installs, so
# anibeam-bin's package() is one copy. Build it on Arch: the binary links against the system's
# Qt, MpvQt and libmpv, which is what anibeam-bin depends on. CI runs it in an Arch container
# (arch.yml); ANIBEAM_SKIP_BUILD=1 packs an existing target/release/anibeam.
set -euo pipefail
version="${1:?usage: bin-tarball.sh <version> <outdir>}"
out="${2:?usage: bin-tarball.sh <version> <outdir>}"
src="$(cd "$(dirname "$0")/../../../.." && pwd)"
target="${CARGO_TARGET_DIR:-$src/target}"

if [ -z "${ANIBEAM_SKIP_BUILD:-}" ]; then
  cargo build --release --locked -p anibeam --manifest-path "$src/Cargo.toml"
fi
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
"$src/apps/linux/packaging/linux/install-tree.sh" "$src" "$stage" "$target/release/anibeam"
# makepkg strips the binary anyway, as it does for anibeam, so ship it stripped: the download
# is a sixth of the size.
strip --strip-unneeded "$stage/usr/bin/anibeam"

mkdir -p "$out"
name="AniBeam-$version-linux-x86_64.tar.zst"
# Sorted, owned by root and dated to the commit, so the same build packs the same bytes.
stamp="$(git -C "$src" log -1 --format=%ct 2>/dev/null || date +%s)"
tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$stamp" \
  -C "$stage" -cf - usr | zstd -q -19 -T0 -o "$out/$name" -f
(cd "$out" && sha256sum "$name" > "$name.sha256")
cat "$out/$name.sha256"
