# Sourced by shoot.sh and drive.sh: sets CARGO_TARGET_DIR so every checkout and worktree
# builds the debug binary into one shared target dir (the main checkout's target/) and a
# fresh worktree does not start from a cold build. An existing CARGO_TARGET_DIR wins.
# Tradeoff: builds from different checkouts queue on one cargo lock, and the debug binary
# is whichever checkout built last, so a capture run with ANIBEAM_NO_BUILD=1 shows that
# checkout's code. Release builds (bench.sh, the launcher) are not touched.
if [ -z "${CARGO_TARGET_DIR:-}" ]; then
  common="$(cd "$here" && git rev-parse --path-format=absolute --git-common-dir 2>/dev/null || true)"
  if [ "$(basename "$common")" = .git ]; then
    CARGO_TARGET_DIR="$(dirname "$common")/target"
  else
    CARGO_TARGET_DIR="$here/../../target"
  fi
  export CARGO_TARGET_DIR
fi
