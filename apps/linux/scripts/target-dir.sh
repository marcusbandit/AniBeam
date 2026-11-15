# Sourced by shoot.sh and drive.sh. Two jobs.
#
# 1. Picks this checkout's own target/ as the target dir, unless CARGO_TARGET_DIR names one.
#    Each checkout and worktree keeps its own target dir: cargo keys a workspace member by its
#    path relative to the workspace root, so two checkouts sharing a target dir share one
#    artifact for anibeam-core, and a build in one can call the other's stale build fresh
#    (#250).
# 2. anibeam_stage builds (unless ANIBEAM_NO_BUILD is set) and copies the debug binary to
#    <checkout>/target/debug/anibeam-staged, then sets ANIBEAM_BIN to that copy. A capture runs
#    the staged copy, the build this script made, which is what ANIBEAM_NO_BUILD=1 reuses.
#
# The build gets the target dir as --target-dir, with CARGO_TARGET_DIR (and its config-env
# twin CARGO_BUILD_TARGET_DIR) taken out of its environment (#263). A compiler cache such as sccache keys every compile on the CARGO_*
# variables it sees, so a CARGO_TARGET_DIR that differs per checkout made every third-party
# crate a miss in every fresh worktree. Without it, a worktree's first build takes the
# registry crates another checkout already compiled, and still builds its own members.
#
# A target dir outside this checkout gets a warning: if another checkout builds there too,
# #250 is back, and staging does not help, since the build itself links the stale core.
#
# The build and the copy share one lock, so another capture script cannot link between them;
# a plain cargo build run by hand in this checkout still can.
# Release builds (bench.sh, the launcher) are not touched.
_ab_checkout="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
# Resolved, so a relative CARGO_TARGET_DIR means the same dir to cargo, cp and the check below
_ab_target="$(realpath -m "${CARGO_TARGET_DIR:-$_ab_checkout/target}")"
case "$_ab_target" in
  "$_ab_checkout"/*) ;;
  *) echo "target-dir.sh: CARGO_TARGET_DIR=$CARGO_TARGET_DIR ($_ab_target) is outside this" \
       "checkout; never share it with another checkout (#250)" >&2 ;;
esac

anibeam_stage() {
  local staged="$_ab_checkout/target/debug/anibeam-staged"
  if [ -z "${ANIBEAM_NO_BUILD:-}" ]; then
    mkdir -p "$(dirname "$staged")" "$_ab_target"
    # One lock around build and copy, so no other capture script links between them
    # Each step is chained, since set -e does not hold in a subshell followed by ||: a failed
    # build must not stage the last good binary as if it were this one.
    (
      flock 9 \
        && env -u CARGO_TARGET_DIR -u CARGO_BUILD_TARGET_DIR cargo build -p anibeam --quiet \
          --manifest-path "$_ab_checkout/Cargo.toml" --target-dir "$_ab_target" \
        && cp "$_ab_target/debug/anibeam" "$staged.$$" \
        && mv -f "$staged.$$" "$staged"
    ) 9> "$_ab_target/.anibeam-stage.lock" || return 1
  fi
  if [ ! -x "$staged" ]; then
    echo "no staged build at $staged; run once without ANIBEAM_NO_BUILD" >&2
    return 1
  fi
  ANIBEAM_BIN="$staged"
}
