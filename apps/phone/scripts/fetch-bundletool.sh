#!/usr/bin/env bash
# Fetch bundletool, which turns the base module build.sh lays out into the App Bundle Google
# Play takes, into vendor/bundletool/ and print the jar's path.
#
# Pinned by version and checked by hash on every run, so every bundle is built by the same
# bundletool. vendor/ is not in git.
#
#   java -jar "$(scripts/fetch-bundletool.sh)" validate --bundle build/Anibeam-release.aab
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=1.18.3
SHA=a099cfa1543f55593bc2ed16a70a7c67fe54b1747bb7301f37fdfd6d91028e29

jar=vendor/bundletool/bundletool-all-$VERSION.jar
if [[ ! -f $jar ]]; then
  mkdir -p vendor/bundletool
  echo "fetch-bundletool.sh: bundletool $VERSION" >&2
  curl -fsSL -o "$jar.part" \
    "https://github.com/google/bundletool/releases/download/$VERSION/bundletool-all-$VERSION.jar"
  mv "$jar.part" "$jar"
fi
if ! echo "$SHA  $jar" | sha256sum -c --quiet >&2; then
  rm -f "$jar"
  echo "fetch-bundletool.sh: $jar does not match its pinned hash; removed it" >&2
  exit 1
fi
echo "$PWD/$jar"
