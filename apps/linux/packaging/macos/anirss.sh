#!/usr/bin/env bash
# usage: anirss.sh <AniBeam.app>
# Puts anirss and a Python to run it into <app>/Contents/Resources/anirss:
#   python/      python-build-standalone 3.13.16 (release 20261013), the install_only_stripped
#                build for Apple silicon, minus what anirss never imports (Tk, IDLE, pip, the
#                man pages and headers). It is its own island: bin/python3.13 finds
#                libpython through @rpath, and fix-dylibs.py leaves those references alone.
#   app/         anirss v0.4.2: the anirss launcher and anirss_lib, the whole program.
#   bin/anirss   a two-line shell launcher, so the app has one path to start it by.
# Every Mach-O in the tree is ad-hoc signed here. Apple silicon kills unsigned code, and
# codesign --strict refuses unsigned code in Resources, so bundle.sh's deep sign of the app is
# not enough for files it does not treat as nested code. The .pyc files are written now, as
# unchecked-hash caches: Python then never writes into the sealed app, and a copy that lost
# its modification times (a drag out of the dmg) still trusts them.
set -euo pipefail

PBS_RELEASE=20261013
PBS_PYTHON=3.13.16
PBS_SHA256=16a5332fb0dcb77f163fc8f8cbccd95be1644cc53e68e5d2a79f50afd6680a0e
ANIRSS_VERSION=0.4.2
ANIRSS_SHA256=0f8e3a7e184a6100d4e2bb05d081187353ea64fde8b35cbc136215e83f95c9fc

app="$(cd "$1" && pwd)"
dest="$app/Contents/Resources/anirss"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
rm -rf "$dest"
mkdir -p "$dest/bin" "$dest/app"

fetch() { # url sha256 out
  curl -fsSL -o "$3" "$1"
  echo "$2  $3" | shasum -a 256 -c -
}

echo "== Python $PBS_PYTHON (python-build-standalone $PBS_RELEASE)"
pbs="cpython-$PBS_PYTHON+$PBS_RELEASE-aarch64-apple-darwin-install_only_stripped.tar.gz"
fetch "https://github.com/astral-sh/python-build-standalone/releases/download/$PBS_RELEASE/${pbs//+/%2B}" \
  "$PBS_SHA256" "$work/python.tar.gz"
tar -xz -C "$work" -f "$work/python.tar.gz"
mv "$work/python" "$dest/python"

py="$dest/python"
lib="$py/lib/python3.13"
# Tk and its Tcl, IDLE, pip, the test and demo trees, dbm (it links a library that is not
# in the tree), headers, man pages and the build-time config. None of it is imported by
# anirss, and the Tk libraries alone are 5 MB.
rm -rf "$py/include" "$py/share" "$py/lib/pkgconfig" \
  "$py"/lib/libtcl* "$py"/lib/tcl9* "$py"/lib/tk9* "$py"/lib/itcl* "$py"/lib/thread* \
  "$py"/bin/idle* "$py"/bin/pip* "$py"/bin/pydoc* "$py"/bin/python3*-config \
  "$lib/tkinter" "$lib/idlelib" "$lib/ensurepip" "$lib/turtledemo" "$lib/turtle.py" \
  "$lib/pydoc_data" "$lib/config-3.13-darwin" "$lib/dbm" "$lib/test" "$lib/venv" \
  "$lib"/lib-dynload/_tkinter* "$lib"/lib-dynload/_dbm*
rm -rf "$lib"/site-packages/*

echo "== anirss $ANIRSS_VERSION"
fetch "https://github.com/marcusbandit/anirss/archive/refs/tags/v$ANIRSS_VERSION.tar.gz" \
  "$ANIRSS_SHA256" "$work/anirss.tar.gz"
tar -xz -C "$work" -f "$work/anirss.tar.gz"
src="$work/anirss-$ANIRSS_VERSION"
cp "$src/anirss" "$dest/app/anirss"
cp -R "$src/anirss_lib" "$dest/app/anirss_lib"
chmod 755 "$dest/app/anirss"
cat > "$dest/bin/anirss" <<'SH'
#!/bin/sh
# AniBeam's own anirss: the release it was built with, on the Python beside it.
here="$(cd "$(dirname "$0")/.." && pwd)"
exec "$here/python/bin/python3" "$here/app/anirss" "$@"
SH
chmod 755 "$dest/bin/anirss"

echo "== byte code"
"$py/bin/python3" -I -m compileall -q -f --invalidation-mode unchecked-hash "$lib" "$dest/app/anirss_lib"

echo "== signing the Python tree"
find "$py" -type f -print0 | while IFS= read -r -d '' f; do
  if [ "$(head -c 4 "$f" | od -An -tx1 | tr -d ' ')" = "cffaedfe" ]; then
    codesign --force --sign - --timestamp=none "$f"
  fi
done

"$dest/bin/anirss" --version
du -sh "$dest" "$py"
