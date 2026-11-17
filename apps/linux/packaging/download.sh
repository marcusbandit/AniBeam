# Sourced by every platform's packaging scripts: downloads that outlive one bad mirror (#953,
# #1140). A mirror that never answers, a 504 from a release host or a transfer that stalls
# fails one attempt, never the build. Each source is tried six times, curl waiting 1, 2, 4, 8
# and 16 seconds between them, and then the next source is tried the same way.
# It runs under bash on Linux, on macOS with BSD tools and in Git Bash on Windows, so it uses
# nothing GNU-only without a fallback: the checksum tool and timeout are looked up below.
# shellcheck shell=bash

# download <file> <url> [<fallback url>...]: writes <file> only once a download finished.
download() {
  local file="$1" url
  shift
  for url in "$@"; do
    # --connect-timeout: a dead mirror took 135 s to fail the first try on #952.
    # --speed-limit/--speed-time: a transfer that hangs at under 1 KB/s for a minute counts
    # as failed and is retried. --max-time caps one try and --retry-max-time the retries,
    # a long Retry-After included. --retry-all-errors retries a 404 too, which costs half a
    # minute per source on a mistyped version before the same error.
    # -o truncates the part file on every retry.
    if curl -fL --no-progress-meter --connect-timeout 20 \
        --speed-limit 1024 --speed-time 60 --max-time 900 \
        --retry 5 --retry-all-errors --retry-max-time 300 \
        -o "$file.part" "$url"; then
      mv -f "$file.part" "$file"
      return 0
    fi
    echo "download: $url failed after 6 tries" >&2
  done
  rm -f "$file.part"
  echo "download: no source answered for $(basename "$file")" >&2
  return 1
}

# sha256_of <file>: the file's sha256 in hex. macOS has shasum but no sha256sum. Reading
# stdin keeps a Windows path out of the output, which sha256sum would escape.
sha256_of() {
  if command -v sha256sum > /dev/null 2>&1; then
    sha256sum < "$1" | cut -d' ' -f1
  else
    shasum -a 256 < "$1" | cut -d' ' -f1
  fi
}

# verify <file> <sha256>: fails, and removes the file, when it is not the pinned one.
verify() {
  local got
  got="$(sha256_of "$1")"
  [ "$got" = "$2" ] && return 0
  echo "checksum mismatch for $(basename "$1"): got $got, want $2" >&2
  rm -f "$1"
  return 1
}

# fetch <url> <sha256> <name>: into the caller's $downloads directory, checked every time, so
# a cached file that went bad is fetched again on the next run.
fetch() {
  local file="${downloads:?fetch needs the caller to set downloads}/$3"
  mkdir -p "$downloads"
  [ -f "$file" ] || download "$file" "$1"
  verify "$file" "$2" || exit 1
}

# with_timeout <seconds> <command>...: runs the command, killed after <seconds> where a GNU
# timeout exists. macOS has none unless Homebrew's coreutils put gtimeout there, and Windows'
# own timeout.exe is a different program, so each is tried before it is trusted. Without one
# the command runs uncapped; git's low-speed limit still ends a stalled clone.
with_timeout() {
  local secs="$1"
  shift
  if timeout 1 true > /dev/null 2>&1; then
    timeout "$secs" "$@"
  elif gtimeout 1 true > /dev/null 2>&1; then
    gtimeout "$secs" "$@"
  else
    "$@"
  fi
}

# clone <dir> <tag> <commit> <url> [<fallback url>...]: a shallow clone of <tag>, three tries
# per source, checked to be <commit> so a fallback cannot hand over other code. A try that
# moves under 1 KB/s for a minute, or runs past five minutes, fails.
clone() {
  local dir="$1" tag="$2" commit="$3" url try
  shift 3
  for url in "$@"; do
    for try in 1 2 3; do
      rm -rf "$dir"
      if with_timeout 300 git -c advice.detachedHead=false -c http.lowSpeedLimit=1024 -c http.lowSpeedTime=60 \
          clone --quiet --depth 1 --branch "$tag" "$url" "$dir"; then
        if [ "$(git -C "$dir" rev-parse HEAD)" = "$commit" ]; then
          return 0
        fi
        echo "clone: $url has $tag at $(git -C "$dir" rev-parse HEAD), not $commit" >&2
        rm -rf "$dir"
        return 1
      fi
      [ "$try" = 3 ] || sleep $((try * 5))
    done
    echo "clone: $url failed after 3 tries" >&2
  done
  echo "clone: no source answered for $tag" >&2
  return 1
}
