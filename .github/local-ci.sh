#!/usr/bin/env bash
# The Linux checks a pull request has to pass, run on your own machine instead of a runner
# (#986). A push to a PR runs only the fast checks on GitHub; this is the full Linux pass:
# what CI, Strings, No sites and the Linux workflow check, over the whole workspace, minus
# the AppImage and its distro smoke tests, which the `full ci` label runs before merge. It
# needs what building the shell needs (docs/build-from-source.md) and runs from anywhere:
#
#   .github/local-ci.sh              every check, then a tester build to try by hand
#   .github/local-ci.sh --no-build   the checks only
#   .github/local-ci.sh --quick      the checks that need git and no compiler, in seconds
#   .github/local-ci.sh --cargo      clippy, the tests and the build; needs no .git, so a
#                                    build box can run it on a copy of the files
#
# Each step prints a "== <step>" line first, so a wrapper can show where it is. The exit
# status is the first failing step's; the steps after a failure still run, so one pass lists
# everything that is wrong.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

build=1
quick=0
cargo=0
for arg in "$@"; do
  case "$arg" in
    --no-build) build=0 ;;
    --quick) quick=1 ;;
    --cargo) cargo=1 ;;
    -h | --help) sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "local-ci: unknown option $arg" >&2; exit 2 ;;
  esac
done

# Neither half named means both.
[ $quick -eq 0 ] && [ $cargo -eq 0 ] && quick=1 && cargo=1

status=0
step() {
  local name="$1"
  shift
  echo "== $name"
  local start=$SECONDS
  "$@"
  local code=$?
  if [ $code -eq 0 ]; then
    echo "== ok $name ($((SECONDS - start)) s)"
  else
    echo "== FAILED $name ($((SECONDS - start)) s, exit $code)"
    [ $status -eq 0 ] && status=$code
  fi
}

if [ $quick -eq 1 ]; then
  # The patch note check compares with main as it is now, not as it was at the last fetch.
  git fetch -q origin main 2>/dev/null || echo "local-ci: could not fetch origin/main; comparing with the last fetch"
  base="$(git merge-base HEAD origin/main)"
  step fmt cargo fmt --all --check
  step changelog .github/changelog-newest.sh "$base"
  step strings python3 .github/strings-check.py
  step "no sites" .github/no-site-names.sh
  step "one spelling" .github/one-spelling.sh
fi
if [ $cargo -eq 1 ]; then
  step clippy cargo clippy --locked --workspace --all-targets -- -D warnings
  # The leaf crates on their own: one cargo call with the core merges their features and
  # would hide a use the crate does not turn on itself (ci.yml says more).
  for crate in anibeam-contract anibeam-foundation anibeam-net anibeam-library anibeam-releases anibeam-metadata anibeam-trackers; do
    step "clippy $crate alone" cargo clippy --locked -p "$crate" --all-targets -- -D warnings
  done
  step "clippy phone features" cargo clippy --locked -p anibeam-core --features phone,cast --all-targets -- -D warnings
  step test cargo test --locked --workspace
  [ $build -eq 1 ] && step "tester build" cargo build --locked --profile tester -p anibeam -p anibeam-cli
fi

if [ $status -eq 0 ]; then echo "== passed"; else echo "== failed"; fi
exit $status
