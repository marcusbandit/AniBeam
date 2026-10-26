#!/usr/bin/env bash
# usage: anirss.sh <stage-dir>
# Puts anirss and a Python to run it into <stage-dir>/anirss, the Windows counterpart of the
# Mac's anirss.sh:
#   python/      python-build-standalone 3.13.17 (release 20261021), the install_only_stripped
#                build for x86_64 Windows, minus what anirss never imports (Tk, IDLE, pip,
#                the tests and headers). It carries its own vcruntime140.dll.
#   app/         anirss: the anirss script and anirss_lib, the whole program.
# Windows cannot start the bare script, so there is no launcher here: the shell names
# python\python.exe in ANIBEAM_PYTHON and the core runs app\anirss through it.
set -euo pipefail

PBS_RELEASE=20261021
PBS_PYTHON=3.13.17
PBS_SHA256=56cde3b1a1dcc36576753cb408ee0c3568d211baafa0eee40423adc147c480f2
ANIRSS_VERSION=0.6.1
ANIRSS_SHA256=ee700625e89e8002c6a39b89c3faabee1ac3004d7e9313232ff2913701b7d22a

dest="$(cd "$1" && pwd)/anirss"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
rm -rf "$dest"
mkdir -p "$dest/app"

fetch() { # url sha256 out
  curl -fsSL --retry 3 -o "$3" "$1"
  echo "$2  $3" | sha256sum -c -
}

echo "== Python $PBS_PYTHON (python-build-standalone $PBS_RELEASE)"
pbs="cpython-$PBS_PYTHON+$PBS_RELEASE-x86_64-pc-windows-msvc-install_only_stripped.tar.gz"
fetch "https://github.com/astral-sh/python-build-standalone/releases/download/$PBS_RELEASE/${pbs//+/%2B}" \
  "$PBS_SHA256" "$work/python.tar.gz"
tar -xzf "$work/python.tar.gz" -C "$work"
mv "$work/python" "$dest/python"

py="$dest/python"
lib="$py/Lib"
rm -rf "$py/tcl" "$py/include" "$py/Scripts" "$py/libs" \
  "$lib/tkinter" "$lib/idlelib" "$lib/ensurepip" "$lib/turtledemo" "$lib/turtle.py" \
  "$lib/pydoc_data" "$lib/test" "$lib/venv" \
  "$py"/DLLs/_tkinter* "$py"/DLLs/tcl*.dll "$py"/DLLs/tk*.dll
rm -rf "$lib"/site-packages/*

echo "== anirss $ANIRSS_VERSION"
fetch "https://github.com/marcusbandit/anirss/archive/refs/tags/v$ANIRSS_VERSION.tar.gz" \
  "$ANIRSS_SHA256" "$work/anirss.tar.gz"
tar -xzf "$work/anirss.tar.gz" -C "$work"
src="$work/anirss-$ANIRSS_VERSION"
cp "$src/anirss" "$dest/app/anirss"
cp -R "$src/anirss_lib" "$dest/app/anirss_lib"

echo "== byte code"
# Unchecked-hash caches, as on the Mac: Python never writes into Program Files-style
# installs, and a copy that lost its modification times still trusts them.
"$py/python.exe" -I -m compileall -q -f --invalidation-mode unchecked-hash "$lib" "$dest/app/anirss_lib"

"$py/python.exe" -I "$dest/app/anirss" --version
du -sh "$dest" "$py"
