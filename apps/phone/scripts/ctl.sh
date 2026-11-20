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
# failed, 2 when the app did not answer, 64 when ctl.sh itself was called wrong.
set -euo pipefail
# The script's own folder, through a symlink too (one in ~/.local/bin, say).
here="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"

if [[ "${1:-}" == "-s" ]]; then
  [[ -n "${2:-}" ]] || { echo "ctl.sh: -s needs a serial" >&2; exit 64; }
  export ANDROID_SERIAL="$2"
  shift 2
fi
source "$here/env.sh"
(( $# )) || set -- help

# shot <file.png>: the device's own pixels, from the display that is powered (env.sh). It
# runs here, not in the app, so it stands on its own; inside a ; line the app says so.
if [[ "$1" == "shot" ]]; then
  [[ $# == 2 ]] || { echo "ctl.sh: shot takes a file name" >&2; exit 64; }
  shot_to "$2" || exit 1
  echo "$2"
  exit 0
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

# How long ctl.sh waits for an answer: past the app's own 45 s for a line. Android 14 and
# later hold broadcasts back from an app sitting cached in the background, and `am` waits
# with them.
ANSWER_WAIT=60

# ask <line>: sends a line and sets $code (the result code) and $data (the answer). Exits 2
# when the app did not answer.
ask() {
  # adb shell runs the command through the device's sh: the line goes in single quotes.
  local quoted="'${1//\'/\'\\\'\'}'" out rest status=0
  local -a limit=()
  command -v timeout > /dev/null && limit=(timeout "$ANSWER_WAIT")
  # `sent` is the phone's clock as its own sh runs the command: the app drops a broadcast
  # that arrives over 10 s late, so after ctl.sh gives up nothing is left to run.
  out="$("${limit[@]}" "$ADB" shell "am broadcast -a com.marcusrosado.anibeam.CONTROL -p $PKG --es cmd $quoted --el sent \$(date +%s)" 2>&1)" || status=$?
  if (( status == 124 )); then
    echo "ctl.sh: no answer within ${ANSWER_WAIT} s. Is Anibeam in the background? Android holds broadcasts to a cached app; bring it to the front. The command will not run late, so sending it again is safe." >&2
    exit 2
  elif (( status != 0 )); then
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
}

ask "$line"
# A wait the line's time cut short (8 s on Android 13 and older, where every app's
# broadcasts wait behind a long one) ends with `again: <the wait to send next>`, worked out
# by the app; it is sent as it is until the label shows or the time is used up.
for _ in {1..20}; do
  next="${data##*$'\n'}"
  [[ "$code" != 0 && "$next" == "again: "* ]] || break
  ask "${next#again: }"
done
printf '%s\n' "$data"

# The labels a whole tree shows, for the shells' completion of click, tap and the rest. Only
# an answer that holds a tree (its `shown: N` line) replaces the last one, so a line that
# failed first keeps the old labels. Rust writes a label with \" and \\ for a quote and a
# backslash.
if [[ "$line" =~ (^|\;)[[:space:]]*(tree|dump)[[:space:]]*($|\;) ]] \
  && grep -q '^shown: ' <<<"$data"; then
  cache="${XDG_CACHE_HOME:-$HOME/.cache}/anibeam-ctl"
  mkdir -p "$cache"
  # Bash's own regex, so any sed will do for the unescape (BSD sed has no \| either).
  while IFS= read -r l; do
    [[ "$l" =~ ^\ \ [a-z]+\ \"((\\.|[^\"\\])*)\" ]] && printf '%s\n' "${BASH_REMATCH[1]}"
  done <<<"$data" | sed 's/\\\(["\\]\)/\1/g' | sort -u > "$cache/labels.new" || true
  [[ -s "$cache/labels.new" ]] && mv -f "$cache/labels.new" "$cache/labels"
  rm -f "$cache/labels.new"
fi

[[ "$code" == 0 ]] || exit 1
