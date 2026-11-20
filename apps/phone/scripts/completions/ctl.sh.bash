# bash completion for apps/phone/scripts/ctl.sh, control over adb (#1207). bash finds a
# completion by the command's last part too, so any path to the script works. Install it once:
#   ln -s "$PWD/apps/phone/scripts/completions/ctl.sh.bash" \
#     ~/.local/share/bash-completion/completions/ctl.sh
# click, tap, hold, wait, find, tree and fill complete the labels the last `ctl.sh tree` showed.

_anibeam_ctl() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local first=1
  if [[ "${COMP_WORDS[1]:-}" == "-s" ]]; then
    if (( COMP_CWORD == 2 )); then
      mapfile -t COMPREPLY < <(compgen -W "$(adb devices 2>/dev/null | awk 'NR > 1 && $1 { print $1 }')" -- "$cur")
      return
    fi
    first=3
  fi
  local pos=$(( COMP_CWORD - first ))
  local command="${COMP_WORDS[first]:-}"
  if (( pos == 0 )); then
    local all="go where tree click tap hold type fill key back scroll swipe shot onboarding find wait state perf help"
    (( first == 1 )) && all+=" -s"
    mapfile -t COMPREPLY < <(compgen -W "$all" -- "$cur")
    return
  fi
  local words=""
  case "$command" in
    go) (( pos == 1 )) && words="home calendar library downloads more series" ;;
    key) words="back enter tab escape backspace up down left right" ;;
    onboarding) (( pos == 1 )) && words="next back step finish start" ;;
    perf) (( pos == 1 )) && words="2s 5s 10s" ;;
    shot) (( pos == 1 )) && { mapfile -t COMPREPLY < <(compgen -f -- "$cur"); return; } ;;
    scroll)
      if (( pos == 1 )); then words="up down left right"
      elif [[ "${COMP_WORDS[COMP_CWORD-1]}" != at ]]; then words="at 300 600 1200"
      else command=click
      fi ;;
  esac
  case "$command" in
    click|press|tap|hold|wait|find|fill|tree)
      local cache="${XDG_CACHE_HOME:-$HOME/.cache}/anibeam-ctl/labels"
      [[ -r "$cache" ]] || return
      local IFS=$'\n' label
      COMPREPLY=()
      while read -r label; do
        [[ "$label" == "$cur"* ]] && COMPREPLY+=("$(printf '%q' "$label")")
      done < "$cache"
      return ;;
  esac
  mapfile -t COMPREPLY < <(compgen -W "$words" -- "$cur")
}

complete -F _anibeam_ctl ctl.sh
