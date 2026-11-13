#!/usr/bin/env bash
# Assembles the website into one folder: the pages in site/ plus the app screenshots the
# README already keeps in docs/readme, so the images live in one place. The Pages workflow
# runs this; run it yourself to look at the site locally:
#   site/build.sh /some/dir && open /some/dir/index.html
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$root/target/site}"

# It replaces the output folder, so it only takes a new folder or one it built before, and
# never one inside site/ itself.
case "$(realpath -m "$out")/" in
  "$root/site/"*) echo "build.sh: the output can't be inside site/" >&2; exit 1 ;;
esac
if [ -e "$out" ] && ! { [ -f "$out/index.html" ] && [ -f "$out/app.js" ]; }; then
  echo "build.sh: $out exists and isn't an earlier site build; pick a new folder" >&2
  exit 1
fi
rm -rf "$out"
mkdir -p "$out/img"
cp -r "$root/site/." "$out/"
rm -f "$out/build.sh"
cp "$root/assets/icon.png" "$out/img/icon.png"
# Every language in site/i18n/ gets the home page and the download page, and every page a menu.
PYTHONDONTWRITEBYTECODE=1 python3 "$root/site/tools/i18n.py" build "$out"
rm -rf "$out/i18n"
# The release links the download buttons, the download page, install.sh and What's new read.
PYTHONDONTWRITEBYTECODE=1 python3 "$root/site/tools/releases.py" "$out"
# What's new: a page per version, from the apps' own patch notes. Its nav comes from the
# translated download page, so it runs after.
PYTHONDONTWRITEBYTECODE=1 python3 "$root/site/whats-new.py" "$root" "$out"
rm -rf "$out/whats-new.py" "$out/__pycache__" "$out/tools"
echo "site assembled in $out"
