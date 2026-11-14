#!/usr/bin/env bash
# The Linux checks a pull request has to pass, run on your own machine instead of a runner
# (#986). A push to a PR runs only the fast checks on GitHub; this is the full Linux pass:
# what CI, Strings, No sites and the Linux workflow check, minus the AppImage and its distro
# smoke tests, which the `full ci` label runs once before merge. It needs what building the
# shell needs (docs/build-from-source.md) and runs from anywhere in the checkout:
#
#   .github/local-ci.sh              every check, then a tester build to try by hand
#   .github/local-ci.sh --no-build   the checks only
#
# Each step prints a "== <step>" line first, so a wrapper can show where it is. The exit
# status is the first failing step's; the steps after a failure still run, so one pass lists
# everything that is wrong.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

build=1
for arg in "$@"; do
  case "$arg" in
    --no-build) build=0 ;;
    -h | --help) sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "local-ci: unknown option $arg" >&2; exit 2 ;;
  esac
done

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

base="$(git merge-base HEAD origin/main 2>/dev/null || echo origin/main)"

step fmt cargo fmt --all --check
step changelog .github/changelog-newest.sh "$base"
step strings python3 .github/strings-check.py
step "no sites" .github/no-site-names.sh
step clippy cargo clippy --locked --workspace --all-targets -- -D warnings
# The leaf crates on their own: one cargo call with the core merges their features and would
# hide a use the crate does not turn on itself (ci.yml says more).
for crate in anibeam-contract anibeam-foundation anibeam-net; do
  step "clippy $crate alone" cargo clippy --locked -p "$crate" --all-targets -- -D warnings
done
step "clippy phone features" cargo clippy --locked -p anibeam-core --features phone,cast --all-targets -- -D warnings
step test cargo test --locked --workspace
[ $build -eq 1 ] && step "tester build" cargo build --locked --profile tester -p anibeam -p anibeam-cli

if [ $status -eq 0 ]; then echo "== passed"; else echo "== failed"; fi
exit $status
