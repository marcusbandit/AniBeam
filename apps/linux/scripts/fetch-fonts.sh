#!/usr/bin/env bash
# Fetch the two lobby faces into assets/fonts/ so the packaged app never touches the
# network for typography. Variable TTFs from the google/fonts repo (OFL, artifacts are
# committed); run once, from anywhere:
#   apps/linux/scripts/fetch-fonts.sh
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
out="$here/assets/fonts"; mkdir -p "$out"
base="https://raw.githubusercontent.com/google/fonts/main/ofl"
fetch() { curl -sL --fail -o "$out/$2" "$base/$1" && echo "$2 ok"; }
fetch "fraunces/Fraunces%5BSOFT%2CWONK%2Copsz%2Cwght%5D.ttf" "Fraunces-Variable.ttf"
fetch "fraunces/OFL.txt" "OFL-Fraunces.txt"
fetch "schibstedgrotesk/SchibstedGrotesk%5Bwght%5D.ttf" "SchibstedGrotesk-Variable.ttf"
fetch "schibstedgrotesk/OFL.txt" "OFL-SchibstedGrotesk.txt"
file "$out"/*.ttf | grep -q TrueType || { echo "not TTFs"; exit 1; }
