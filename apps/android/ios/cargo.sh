#!/usr/bin/env bash
# Xcode's build step for the iPhone app (project.yml): builds the Rust executable for the
# SDK Xcode is building for and puts it where Xcode expects the app's executable. After
# Slint's scripts/build_for_ios_with_cargo.bash, for arm64 only.
set -euo pipefail

# Xcode's PATH has no cargo; rustup's reads the checkout's rust-toolchain.toml.
export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH:$HOME/.cargo/bin"
app="$(cd "$(dirname "$0")/.." && pwd)"

if [[ "${LLVM_TARGET_TRIPLE_SUFFIX:-}" == "-simulator" ]]; then
  target=aarch64-apple-ios-sim
else
  target=aarch64-apple-ios
fi
if [[ "$CONFIGURATION" == "Debug" ]]; then
  profile=dev out=debug
else
  profile=release out=release
fi

cd "$app"
cargo build --profile "$profile" --target "$target" --bin anibeam-ios
cp "target/$target/$out/anibeam-ios" "$TARGET_BUILD_DIR/$EXECUTABLE_PATH"
