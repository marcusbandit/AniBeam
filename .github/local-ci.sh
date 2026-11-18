#!/usr/bin/env bash
# The Linux checks a pull request has to pass, run on your own machine instead of a runner
# (#986). A push to a PR runs only the fast checks on GitHub; this is the full Linux pass:
# what CI, Strings, No sites and the Linux workflow check, the Qt shell included, minus
# the AppImage and its distro smoke tests, which the `full ci` label runs before merge. It
# needs what building the shell needs (docs/build-from-source.md) and runs from anywhere:
#
#   .github/local-ci.sh              every check, then a tester build to try by hand
#   .github/local-ci.sh --no-build   the checks only
#   .github/local-ci.sh --quick      the checks that need git and no compiler, in seconds
#   .github/local-ci.sh --cargo      clippy, the tests and the build; needs no .git, so a
#                                    build box can run it on a copy of the files
#   .github/local-ci.sh --all        clippy and test every crate, not only the changed ones
#   --base REV                       compare with REV as it is, with no fetch, instead of
#                                    origin/main; CI passes the merge commit's first parent
#   --skip NAME                      leave a crate out of every step, e.g. one this machine
#                                    cannot build; CI skips the Qt shell
#
# ci.yml runs `--cargo --no-build` itself, so this file is the one place the cargo checks
# are written down.
#
# Clippy and the tests cover the crates changed since the merge base with origin/main (or
# --base), plus
# every crate that builds on them (.github/affected-crates.py, #1075); a file only one test
# target reads runs that target alone (#1086). The tests run under cargo-nextest when it is
# installed, as in CI, and under cargo test when not. A copy without .git
# reads the changed files from LOCAL_CI_FILES, one per line, which localci sets on the box
# with the history; without that either, it checks every crate.
#
# Each step prints a "== <step>" line first, so a wrapper can show where it is. The exit
# status is the first failing step's; the steps after a failure still run, so one pass lists
# everything that is wrong.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

build=1
quick=0
cargo=0
all=0
base=""
skip=()
skipped=()
while [ $# -gt 0 ]; do
  case "$1" in
    --no-build) build=0 ;;
    --quick) quick=1 ;;
    --cargo) cargo=1 ;;
    --all) all=1 ;;
    --base) base="${2:?local-ci: --base needs a revision}"; shift ;;
    --skip) skipped+=("${2:?local-ci: --skip needs a crate}"); skip+=(--skip "$2"); shift ;;
    -h | --help) awk 'NR > 1 && /^#/ { sub(/^# ?/, ""); print; next } NR > 1 { exit }' "$0"; exit 0 ;;
    *) echo "local-ci: unknown option $1" >&2; exit 2 ;;
  esac
  shift
done

# Neither half named means both.
[ $quick -eq 0 ] && [ $cargo -eq 0 ] && quick=1 && cargo=1

status=0
fetched=0
step() {
  local name="$1"
  shift
  [ -n "${GITHUB_ACTIONS:-}" ] && echo "::group::$name"
  echo "== $name"
  local start=$SECONDS
  "$@"
  local code=$?
  [ -n "${GITHUB_ACTIONS:-}" ] && echo "::endgroup::"
  if [ $code -eq 0 ]; then
    echo "== ok $name ($((SECONDS - start)) s)"
  else
    echo "== FAILED $name ($((SECONDS - start)) s, exit $code)"
    [ -n "${GITHUB_ACTIONS:-}" ] && echo "::error::$name failed"
    [ $status -eq 0 ] && status=$code
  fi
}

if [ -n "$base" ] && [ $all -eq 1 ]; then
  echo "local-ci: --all checks every crate, so --base $base only sets the patch note check's base"
fi
# The checks compare with main as it is now, not as it was at the last fetch.
fetch_main() {
  [ $fetched -eq 1 ] && return
  git fetch -q origin main 2>/dev/null || echo "local-ci: could not fetch origin/main; comparing with the last fetch"
  fetched=1
}
if [ $quick -eq 1 ]; then
  [ -z "$base" ] && fetch_main
  step fmt cargo fmt --all --check
  # A base with no merge base fails here rather than letting the script fall back to a stale
  # origin/main.
  step changelog bash -c 'mb="$(git merge-base HEAD "$1")" && .github/changelog-newest.sh "$mb"' _ "${base:-origin/main}"
  step strings python3 .github/strings-check.py
  step "no sites" .github/no-site-names.sh
  step "one spelling" .github/one-spelling.sh
  step "affected crates self-test" python3 .github/affected-crates.py --self-test
fi
if [ $cargo -eq 1 ]; then
  pick=(--all)
  if [ $all -eq 0 ]; then
    if git rev-parse --git-dir >/dev/null 2>&1; then
      [ -z "$base" ] && fetch_main
      pick=(--base "${base:-origin/main}")
    elif [ -n "${LOCAL_CI_FILES+set}" ]; then
      # A copy on a build box has no .git; localci sends the changed files along instead,
      # one per line, worked out where the history is.
      files=()
      while IFS= read -r f; do if [ -n "$f" ]; then files+=("$f"); fi; done <<<"$LOCAL_CI_FILES"
      echo "local-ci: no .git here; using the ${#files[@]} changed files localci sent"
      pick=(--files "${files[@]}")
    else
      echo "local-ci: no .git here, so every crate is checked"
    fi
    if [ -n "$base" ] && [ "${pick[0]}" != --base ]; then
      echo "local-ci: no .git here, so --base $base is not used"
    fi
  fi
  picked="$(mktemp)"
  step "affected crates" python3 .github/affected-crates.py "${pick[@]}" "${skip[@]}" --output "$picked"
  # A failed selection, which its step reports, checks every crate rather than none.
  grep -q '^crates=' "$picked" || python3 .github/affected-crates.py --all "${skip[@]}" --output "$picked" >/dev/null
  crates=() alone=() core=false packages=() targets=() docs=() lock=false
  if grep -q '^crates=' "$picked"; then
    read -ra crates <<<"$(sed -n 's/^crates=//p' "$picked")"
    read -ra alone <<<"$(sed -n 's/^alone=//p' "$picked")"
    read -ra targets <<<"$(sed -n 's/^tests=//p' "$picked")"
    read -ra doc <<<"$(sed -n 's/^doc=//p' "$picked")"
    lock="$(sed -n 's/^lock=//p' "$picked")"
    core="$(sed -n 's/^core=//p' "$picked")"
    for crate in "${crates[@]}"; do packages+=(-p "$crate"); done
    for crate in "${doc[@]}"; do docs+=(-p "$crate"); done
  else
    # Not even cargo metadata worked: let cargo say why.
    packages=(--workspace) docs=(--workspace) core=true
    for crate in "${skipped[@]}"; do packages+=(--exclude "$crate") docs+=(--exclude "$crate"); done
  fi
  if cargo nextest --version >/dev/null 2>&1; then
    run_tests=(cargo nextest run --locked --profile ci)
  else
    if [ ${#packages[@]} -gt 0 ] || [ ${#targets[@]} -gt 0 ]; then
      echo "local-ci: cargo-nextest is not installed, so the tests run under cargo test, one binary at a time"
    fi
    run_tests=(cargo test --locked)
    docs=()
  fi
  rm -f "$picked"
  if [ ${#packages[@]} -gt 0 ]; then
    step clippy cargo clippy --locked "${packages[@]}" --all-targets -- -D warnings
    # Each picked crate another one builds on, by itself: one cargo call over several crates
    # merges their features, and would hide a use the crate does not turn on itself, such as
    # rusqlite in the contract, whose sqlite feature only the core turns on.
    for crate in "${alone[@]}"; do
      step "clippy $crate alone" cargo clippy --locked -p "$crate" --all-targets -- -D warnings
    done
    # Only the phone build compiles the core's phone and cast features, and CI builds no phone
    # on a push, so without this a desktop change can break the APK and stay green (#309).
    # The host target is enough: these features need no NDK.
    [ "$core" = true ] && step "clippy phone features" cargo clippy --locked -p anibeam-core --features phone,cast --all-targets -- -D warnings
    # The phone's torrent engine and casting have tests behind the downloads and playback
    # crates' own features, which the core's turn on for the libraries only, so nothing else
    # compiles those tests. With no list, every crate was picked.
    if [ ${#crates[@]} -eq 0 ] || [[ " ${crates[*]} " == *" anibeam-downloads "* ]]; then
      step "clippy downloads with phone" cargo clippy --locked -p anibeam-downloads --features phone --all-targets -- -D warnings
    fi
    if [ ${#crates[@]} -eq 0 ] || [[ " ${crates[*]} " == *" anibeam-playback "* ]]; then
      step "clippy playback with cast" cargo clippy --locked -p anibeam-playback --features cast --all-targets -- -D warnings
    fi
    step test "${run_tests[@]}" "${packages[@]}"
    # nextest skips doctests; cargo test ran them with the rest.
    [ ${#docs[@]} -gt 0 ] && step doctests cargo test --locked --doc "${docs[@]}"
  elif [ ${#targets[@]} -eq 0 ]; then
    echo "== no crate changed: clippy and the tests have nothing to check"
  fi
  # A test target picked by itself, such as the core's changelog test for a patch note.
  for t in "${targets[@]}"; do
    step "test ${t%%:*} --test ${t#*:}" "${run_tests[@]}" -p "${t%%:*}" --test "${t#*:}"
  done
  # A lock change that reaches no crate ran no cargo step above, so --locked never checked it
  # against the manifests. This does, without building anything.
  if [ "$lock" = true ]; then
    step "lock is current" bash -c 'cargo metadata --locked --format-version 1 >/dev/null'
  fi
  if [ $build -eq 1 ]; then
    shells=()
    for crate in anibeam anibeam-cli; do
      [[ " ${skipped[*]} " == *" $crate "* ]] || shells+=(-p "$crate")
    done
    [ ${#shells[@]} -gt 0 ] && step "tester build" cargo build --locked --profile tester "${shells[@]}"
  fi
fi

if [ $status -eq 0 ]; then echo "== passed"; else echo "== failed"; fi
exit $status
