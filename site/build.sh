#!/usr/bin/env bash
# Assembles the website into one folder: the pages in site/ plus the app screenshots the
# README already keeps in docs/readme, so the images live in one place. The Pages workflow
# runs this; run it yourself to look at the site locally:
#   site/build.sh /some/dir && open /some/dir/index.html
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$root/target/site}"

rm -rf "$out"
mkdir -p "$out/img"
cp -r "$root/site/." "$out/"
rm -f "$out/build.sh"
cp "$root"/docs/readme/{home,series,library}.webp "$out/img/"
cp "$root/assets/icon.png" "$out/img/icon.png"
echo "site assembled in $out"
