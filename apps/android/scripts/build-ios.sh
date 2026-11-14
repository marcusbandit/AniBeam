#!/usr/bin/env bash
# Builds the iPhone app: fetches the player's libmpv (scripts/fetch-mpv-ios.sh), generates
# the Xcode project from ios/project.yml with xcodegen, then builds it unsigned, for the iOS
# Simulator or, as an IPA, for a device. Needs Xcode, xcodegen and the aarch64-apple-ios-sim
# (or, for the IPA, aarch64-apple-ios) Rust target.
#
#   scripts/build-ios.sh            a Debug build for the simulator
#   scripts/build-ios.sh Release    a Release build for the simulator
#   scripts/build-ios.sh ipa        an unsigned Release IPA for a device
#
# The simulator app lands in build/ios/Build/Products/<configuration>-iphonesimulator/AniBeam.app.
# The IPA lands in build/ios/AniBeam.ipa, and carries no signature: AltStore and LiveContainer
# sign it themselves when they install it. It is built as ANIBEAM_RELEASE_VERSION when CI
# sets one (2.4.3 or 2.4.3-nightly.20261113.41), else as Cargo.toml's version. The bundle's
# short version is the base, 2.4.3, which is all iOS takes, and its build number the run
# number of a nightly.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
mode="${1:-Debug}"
command -v xcodegen >/dev/null || {
  echo "build-ios.sh: needs xcodegen (brew install xcodegen)" >&2
  exit 1
}
crate_version="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$here/Cargo.toml" | head -1)"
release_version="${ANIBEAM_RELEASE_VERSION:-$crate_version}"
version="${release_version%%-*}"
if [ "$version" != "$crate_version" ]; then
  echo "build-ios.sh: ANIBEAM_RELEASE_VERSION $release_version is not a build of $crate_version (Cargo.toml)." >&2
  exit 1
fi
build_number=1
case "$release_version" in
  "$version"-nightly.*.*)
    build_number="${release_version##*.}"
    [[ "$build_number" =~ ^[1-9][0-9]*$ ]] || build_number=1
    ;;
esac

# The player's libmpv, linked statically (build.rs); fetched once, then only when its pin
# changes.
"$here/scripts/fetch-mpv-ios.sh" || {
  echo "build-ios.sh: could not fetch libmpv for iOS (scripts/fetch-mpv-ios.sh)" >&2
  exit 1
}

xcodegen generate --spec "$here/ios/project.yml" --quiet

if [ "$mode" = ipa ]; then
  xcodebuild -project "$here/ios/AniBeam.xcodeproj" -scheme AniBeam -configuration Release \
    -sdk iphoneos -destination 'generic/platform=iOS' \
    -derivedDataPath "$here/build/ios" CODE_SIGNING_ALLOWED=NO MARKETING_VERSION="$version" \
    CURRENT_PROJECT_VERSION="$build_number" -quiet build
  # An IPA is a zip whose root is Payload/, holding the .app.
  stage="$here/build/ios/ipa"
  rm -rf "$stage" "$here/build/ios/AniBeam.ipa"
  mkdir -p "$stage/Payload"
  ditto "$here/build/ios/Build/Products/Release-iphoneos/AniBeam.app" "$stage/Payload/AniBeam.app"
  (cd "$stage" && zip -qry "$here/build/ios/AniBeam.ipa" Payload)
  rm -rf "$stage"
  echo "build-ios.sh: $here/build/ios/AniBeam.ipa"
  exit 0
fi

xcodebuild -project "$here/ios/AniBeam.xcodeproj" -scheme AniBeam -configuration "$mode" \
  -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' \
  -derivedDataPath "$here/build/ios" CODE_SIGNING_ALLOWED=NO MARKETING_VERSION="$version" \
  CURRENT_PROJECT_VERSION="$build_number" -quiet build
echo "build-ios.sh: $here/build/ios/Build/Products/$mode-iphonesimulator/AniBeam.app"
