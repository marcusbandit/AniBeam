#!/usr/bin/env bash
# Assembles the website into one folder: the pages in site/, their translations, the release
# links and What's new. The Pages workflow runs this; run it yourself to look at the site:
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
# The logos live in assets/logo/ with the app's icons; the pages name them by these paths.
cp "$root/assets/logo/logo.svg" "$root/assets/logo/logo-nightly.svg" \
  "$root/assets/logo/logo-dev.svg" "$out/img/"
cp "$root/assets/logo/logo.png" "$out/img/icon.png"
cp "$root/assets/logo/favicon.ico" "$out/favicon.ico"
cp "$root/assets/logo/logo-nightly.png" "$out/img/icon-nightly.png"
cp "$root/assets/logo/logo-dev.png" "$out/img/icon-dev.png"
# Every language in site/i18n/ gets the home page and the download page, and every page a menu.
PYTHONDONTWRITEBYTECODE=1 python3 "$root/site/tools/i18n.py" build "$out"
rm -rf "$out/i18n"
# The release links the download buttons, the download page, install.sh and What's new read.
PYTHONDONTWRITEBYTECODE=1 python3 "$root/site/tools/releases.py" "$out"
# What's new: a page per version, from the apps' own patch notes. Its nav comes from the
# translated download page, so it runs after.
PYTHONDONTWRITEBYTECODE=1 python3 "$root/site/whats-new.py" "$root" "$out"
# The wiki: its pages from the fragments in site/wiki/, in the same nav and footer.
PYTHONDONTWRITEBYTECODE=1 python3 "$root/site/wiki.py" "$root" "$out"
rm -rf "$out/whats-new.py" "$out/wiki.py" "$out/__pycache__" "$out/tools"
echo "site assembled in $out"
