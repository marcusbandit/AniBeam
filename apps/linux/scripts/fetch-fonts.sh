#!/usr/bin/env bash
# Fetch the shell's one face, Archivo (variable in width and weight), and cut it into the
# three files in assets/fonts/ (scripts/cut-fonts.py says why), so the packaged app never
# touches the network for typography. The variable TTF comes from the google/fonts repo
# (OFL, the cut artifacts are committed); needs fonttools; run once, from anywhere:
#   apps/linux/scripts/fetch-fonts.sh
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
out="$here/assets/fonts"; mkdir -p "$out"
base="https://raw.githubusercontent.com/google/fonts/main/ofl"
fetch() { curl -sL --fail -o "$out/$2" "$base/$1" && echo "$2 ok"; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
curl -sL --fail -o "$tmp/Archivo.ttf" "$base/archivo/Archivo%5Bwdth%2Cwght%5D.ttf"
python3 -I "$here/scripts/cut-fonts.py" "$tmp/Archivo.ttf" "$out"
fetch "archivo/OFL.txt" "OFL-Archivo.txt"
file "$out"/*.ttf | grep -q TrueType || { echo "not TTFs"; exit 1; }
