#compdef ctl.sh
# zsh completion for apps/phone/scripts/ctl.sh, control over adb (#1207). zsh completes
# scripts/ctl.sh by its last part, so any path to the script works. Install it once:
#   ln -s "$PWD/apps/phone/scripts/completions/_ctl.sh" ~/.local/share/zsh/site-functions/
# click, tap, hold, wait, find, tree and fill complete the labels the last `ctl.sh tree` showed.

_ctl.sh_labels() {
  local cache=${XDG_CACHE_HOME:-$HOME/.cache}/anibeam-ctl/labels
  [[ -r $cache ]] || return 1
  local -a labels
  labels=("${(@f)$(<$cache)}")
  compadd -a labels
}

# The adb ctl.sh talks through (env.sh): the SDK's, else the one on PATH. Another build's
# `adb devices` could restart the server under ctl.sh.
_ctl.sh_adb() {
  local sdk=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}/platform-tools/adb
  if [[ -x $sdk ]]; then $sdk "$@"; else adb "$@"; fi
}

_ctl.sh() {
  # Only Anibeam's ctl.sh: a script of the same name elsewhere gets its files completed.
  local cmd=$words[1]
  [[ $cmd == */* ]] || cmd=${commands[$cmd]:-$cmd}
  if [[ ${cmd:A} != */apps/phone/scripts/ctl.sh ]]; then
    _files
    return
  fi
  local -a commands
  commands=(
    'go:switch tab, open a series, search Downloads'
    'where:the page and what is open over it'
    'tree:the page and every control shown'
    'click:a control by its words, or x,y'
    'tap:a tap on a control or at x,y'
    'hold:a long press on a control or at x,y'
    'type:keys into the focused field'
    'fill:set a field by number or placeholder'
    'key:press keys'
    'back:Android Back'
    'scroll:scroll up, down, left or right'
    'swipe:drag from x1 y1 to x2 y2'
    'shot:the screen to a PNG'
    'onboarding:step through onboarding'
    'find:every element whose label holds a text'
    'wait:until a control with a label shows'
    'state:threads, memory, frames, core jobs'
    'perf:sample the UI loop for a while'
    'help:every command'
  )
  # -s <serial> first picks the device.
  if [[ $words[2] == -s ]]; then
    if (( CURRENT == 3 )); then
      local -a serials
      serials=(${${(f)"$(_ctl.sh_adb devices 2>/dev/null)"}[2,-1]%%[[:space:]]*})
      compadd -a serials
      return
    fi
    words=($words[1] $words[4,-1])
    (( CURRENT -= 2 ))
  fi
  if (( CURRENT == 2 )); then
    if [[ $words[2] == -* ]]; then
      compadd -- -s
    else
      _describe -t commands 'command' commands
    fi
    return
  fi
  case $words[2] in
    go)
      if (( CURRENT == 3 )); then compadd home calendar library downloads more series
      elif [[ $words[3] == series ]]; then _ctl.sh_labels
      fi ;;
    key) compadd back enter tab escape backspace up down left right ;;
    # After a label, --nth for the controls and a time for wait, beside more labels.
    click|press|tap|hold)
      if [[ $words[CURRENT-1] == --nth ]]; then compadd 1 2 3 4 5
      else
        _ctl.sh_labels
        (( CURRENT > 3 )) && compadd -- --nth
      fi ;;
    wait)
      _ctl.sh_labels
      (( CURRENT > 3 )) && compadd 5s 10s 30s ;;
    find|fill|tree) _ctl.sh_labels ;;
    shot) (( CURRENT == 3 )) && _files -g '*.png' ;;
    scroll)
      if (( CURRENT == 3 )); then compadd up down left right
      elif [[ $words[CURRENT-1] == at ]]; then _ctl.sh_labels
      else compadd at 300 600 1200
      fi ;;
    onboarding)
      if (( CURRENT == 3 )); then compadd next back step finish start
      elif (( CURRENT == 4 )) && [[ $words[3] == step ]]; then compadd 0 1 2 3 4 5 6 7
      fi ;;
    perf) (( CURRENT == 3 )) && compadd 2s 5s 10s ;;
  esac
}

_ctl.sh "$@"
