#!/usr/bin/env bash
# Drive the running Anibeam over adb by name, without tapping coordinates (#1207). The
# sideload build listens for commands from the shell user and answers with what it did.
#
#   scripts/ctl.sh go downloads
#   scripts/ctl.sh click "Get episodes"
#   scripts/ctl.sh tree
#   scripts/ctl.sh shot captures/a.png
#   scripts/ctl.sh 'go series Frieren; wait Play 5s; tree'   one argument is a whole line
#   scripts/ctl.sh -s <serial> state                         another device than $ANDROID_SERIAL
#   scripts/ctl.sh help                                      every command
#
# The words are the desktop's `anibeam ctl` ones where the apps do the same thing.
#
# Talks to $ANDROID_SERIAL, or this project's emulator (env.sh). Exits 1 when a command
# failed, 2 when the app did not answer.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"

if [[ "${1:-}" == "-s" ]]; then
  [[ -n "${2:-}" ]] || { echo "ctl.sh: -s needs a serial" >&2; exit 2; }
  export ANDROID_SERIAL="$2"
  shift 2
fi
source "$here/env.sh"
(( $# )) || set -- help

# shot <file.png>: the device's own pixels, from the display that is powered (env.sh). It
# runs here, not in the app, so it stands on its own rather than in a ; line.
if [[ "$1" == "shot" ]]; then
  [[ $# == 2 ]] || { echo "ctl.sh: shot takes a file name" >&2; exit 2; }
  shot_to "$2" || exit 1
  echo "$2"
  exit 0
fi
if (( $# == 1 )) && [[ "$1" =~ (^|\;)[[:space:]]*shot([[:space:]]|$|\;) ]]; then
  echo "ctl.sh: shot runs on its own, not in a ; line: ctl.sh shot <file.png>" >&2
  exit 2
fi

# A word as the app's reader takes it: in double quotes when it holds a blank, a quote, a
# backslash or a ;.
word() {
  local w="$1"
  if [[ -z "$w" || "$w" =~ [[:space:]\"\'\\\;] ]]; then
    w="${w//\\/\\\\}"
    w="${w//\"/\\\"}"
    printf '"%s"' "$w"
  else
    printf '%s' "$w"
  fi
}

if (( $# == 1 )); then
  line="$1"
else
  line="$1"
  shift
  for a in "$@"; do line+=" $(word "$a")"; done
fi

# adb shell runs the command through the device's sh: the line goes in single quotes.
quoted="'${line//\'/\'\\\'\'}'"
if ! out="$("$ADB" shell "am broadcast -a com.marcusrosado.anibeam.CONTROL -p $PKG --es cmd $quoted" 2>&1)"; then
  echo "ctl.sh: adb failed: $out" >&2
  exit 2
fi
out="${out//$'\r'/}"

rest="${out#*Broadcast completed: result=}"
if [[ "$rest" == "$out" ]]; then
  echo "ctl.sh: $out" >&2
  exit 2
fi
code="${rest%%[!0-9]*}"
if [[ "$rest" != *', data="'* ]]; then
  echo "ctl.sh: no answer. Is Anibeam open, and a sideload build? The Play build has no control." >&2
  exit 2
fi
data="${rest#*, data=\"}"
data="${data%"${data##*[![:space:]]}"}"
data="${data%\"}"
printf '%s\n' "$data"
[[ "$line" == help ]] && echo "shot <file.png>                             the screen, from the device; on its own, not in a ; line"

# The labels a tree shows, for the shells' completion of click, tap and the rest.
if [[ "$line" =~ (^|\;)[[:space:]]*(tree|dump)[[:space:]]*($|\;) ]]; then
  cache="${XDG_CACHE_HOME:-$HOME/.cache}/anibeam-ctl"
  mkdir -p "$cache"
  sed -n 's/^  [a-z]* "\(\([^"\\]\|\\.\)*\)".*/\1/p' <<<"$data" | sed 's/\\"/"/g' | sort -u > "$cache/labels" || true
fi

[[ "$code" == 0 ]] || exit 1
