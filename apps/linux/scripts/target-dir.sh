# Sourced by shoot.sh and drive.sh. Two jobs.
#
# 1. Sets CARGO_TARGET_DIR so every checkout and worktree builds into one shared target dir
#    (the main checkout's target/) and a fresh worktree does not start from a cold build.
#    An existing CARGO_TARGET_DIR wins. The shared dir is only trusted when it belongs to an
#    AniBeam checkout; otherwise the checkout's own target/ is used.
# 2. anibeam_stage builds (unless ANIBEAM_NO_BUILD is set) and copies the debug binary to
#    <checkout>/target/debug/anibeam-staged, then sets ANIBEAM_BIN to that copy. The shared
#    debug/anibeam is whichever checkout linked last, so a capture must not run it directly:
#    another worktree's build can land between this checkout's build and its run. The staged
#    copy is this checkout's last build, which is what ANIBEAM_NO_BUILD=1 reuses.
#
# Tradeoff: builds from different checkouts queue on one cargo lock. The copy follows the
# build within milliseconds, so the window for another link to land is not zero but is tiny.
# Release builds (bench.sh, the launcher) are not touched.
_ab_checkout="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
if [ -z "${CARGO_TARGET_DIR:-}" ]; then
  _ab_common="$(cd "$_ab_checkout" && git rev-parse --path-format=absolute --git-common-dir 2>/dev/null || true)"
  if [ "$(basename "$_ab_common")" = .git ] \
    && [ -f "$(dirname "$_ab_common")/apps/linux/scripts/target-dir.sh" ]; then
    CARGO_TARGET_DIR="$(dirname "$_ab_common")/target"
  else
    CARGO_TARGET_DIR="$_ab_checkout/target"
  fi
  export CARGO_TARGET_DIR
fi

anibeam_stage() {
  local staged="$_ab_checkout/target/debug/anibeam-staged"
  if [ -z "${ANIBEAM_NO_BUILD:-}" ]; then
    cargo build -p anibeam --quiet
    mkdir -p "$(dirname "$staged")"
    cp "$CARGO_TARGET_DIR/debug/anibeam" "$staged.$$" && mv -f "$staged.$$" "$staged"
  fi
  if [ ! -x "$staged" ]; then
    echo "no staged build at $staged; run once without ANIBEAM_NO_BUILD" >&2
    return 1
  fi
  ANIBEAM_BIN="$staged"
}
