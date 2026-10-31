#!/usr/bin/env bash
# Bakes the shell's shaders into .qsb packs with Qt's qsb, for every graphics API the shell
# runs on (OpenGL, OpenGL ES, Vulkan, Direct3D, Metal). The packs are committed, so a build
# needs no qsb; run this after editing a .vert or .frag.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
qsb="${QSB:-$(command -v qsb || echo /usr/lib/qt6/bin/qsb)}"
# -b on a vertex shader adds the batchable variant the scene graph's renderer asks for
for f in "$here"/*.vert; do
  "$qsb" -b --glsl "100 es,120,150" --hlsl 50 --msl 12 -o "$f.qsb" "$f"
done
for f in "$here"/*.frag; do
  "$qsb" --glsl "100 es,120,150" --hlsl 50 --msl 12 -o "$f.qsb" "$f"
done
