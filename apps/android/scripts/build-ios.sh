#!/usr/bin/env bash
# Builds the iPhone app for the iOS Simulator: generates the Xcode project from
# ios/project.yml with xcodegen, then builds it unsigned. Needs Xcode, xcodegen and the
# aarch64-apple-ios-sim Rust target.
#
#   scripts/build-ios.sh            a Debug build
#   scripts/build-ios.sh Release    a Release build
#
# The app lands in build/ios/Build/Products/<configuration>-iphonesimulator/AniBeam.app.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
config="${1:-Debug}"
command -v xcodegen >/dev/null || {
  echo "build-ios.sh: needs xcodegen (brew install xcodegen)" >&2
  exit 1
}
version="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$here/Cargo.toml" | head -1)"

xcodegen generate --spec "$here/ios/project.yml" --quiet
xcodebuild -project "$here/ios/AniBeam.xcodeproj" -scheme AniBeam -configuration "$config" \
  -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' \
  -derivedDataPath "$here/build/ios" CODE_SIGNING_ALLOWED=NO MARKETING_VERSION="$version" \
  -quiet build
echo "build-ios.sh: $here/build/ios/Build/Products/$config-iphonesimulator/AniBeam.app"
