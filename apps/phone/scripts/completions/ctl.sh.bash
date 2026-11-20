# bash completion for apps/phone/scripts/ctl.sh, control over adb (#1207). bash finds a
# completion by the command's last part too, so any path to the script works. Install it once:
#   ln -s "$PWD/apps/phone/scripts/completions/ctl.sh.bash" \
#     ~/.local/share/bash-completion/completions/ctl.sh
# click, tap, hold, wait, find, tree and fill complete the labels the last `ctl.sh tree` showed.

# The adb ctl.sh talks through (env.sh): the SDK's, else the one on PATH. Another build's
# `adb devices` could restart the server under ctl.sh.
_anibeam_ctl_adb() {
  local sdk="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}/platform-tools/adb"
  if [[ -x "$sdk" ]]; then "$sdk" "$@"; else adb "$@"; fi
}

_anibeam_ctl() {
  local cur cword
  local -a w words
  # bash splits words at ':' too, so a label such as Re:Zero would complete from "Zero". The
  # bash-completion package puts the words back together; without it, the split stands.
  if declare -F _get_comp_words_by_ref > /dev/null; then
    _get_comp_words_by_ref -n : cur words cword
    w=("${words[@]}")
  else
    cur="${COMP_WORDS[COMP_CWORD]}"
    w=("${COMP_WORDS[@]}")
    cword=$COMP_CWORD
  fi
  # Only Anibeam's ctl.sh: a script of the same name elsewhere gets bash's own completion.
  local cmd="${w[0]}"
  [[ "$cmd" == */* ]] || cmd="$(type -P -- "$cmd" 2> /dev/null)"
  case "$(realpath -m -- "${cmd:-.}" 2> /dev/null)" in
    */apps/phone/scripts/ctl.sh) ;;
    *) compopt -o default 2> /dev/null; COMPREPLY=(); return ;;
  esac
  local first=1
  if [[ "${w[1]:-}" == "-s" ]]; then
    if (( cword == 2 )); then
      mapfile -t COMPREPLY < <(compgen -W "$(_anibeam_ctl_adb devices 2>/dev/null | awk 'NR > 1 && $1 { print $1 }')" -- "$cur")
      return
    fi
    first=3
  fi
  local pos=$(( cword - first ))
  local command="${w[first]:-}"
  if (( pos == 0 )); then
    local all="go where tree click tap hold type fill key back scroll swipe shot onboarding find wait state perf help"
    (( first == 1 )) && all+=" -s"
    mapfile -t COMPREPLY < <(compgen -W "$all" -- "$cur")
    return
  fi
  local choices=""
  case "$command" in
    go)
      (( pos == 1 )) && choices="home calendar library downloads more series"
      (( pos >= 2 )) && [[ "${w[first+1]}" == series ]] && command=find ;;
    key) choices="back enter tab escape backspace up down left right" ;;
    onboarding)
      (( pos == 1 )) && choices="next back step finish start"
      (( pos == 2 )) && [[ "${w[cword-1]}" == step ]] && choices="0 1 2 3 4 5 6 7" ;;
    perf) (( pos == 1 )) && choices="2s 5s 10s" ;;
    shot) (( pos == 1 )) && { mapfile -t COMPREPLY < <(compgen -f -- "$cur"); return; } ;;
    scroll)
      if (( pos == 1 )); then choices="up down left right"
      elif [[ "${w[cword-1]}" != at ]]; then choices="at 300 600 1200"
      else command=click
      fi ;;
  esac
  # After a label: --nth for the controls, a time for wait.
  if (( pos >= 2 )) && [[ "${w[cword-1]}" != --nth ]]; then
    case "$command" in
      click|press|tap|hold)
        [[ "$cur" == -* ]] && { mapfile -t COMPREPLY < <(compgen -W "--nth" -- "$cur"); return; } ;;
      wait)
        [[ "$cur" == [0-9]* ]] && { mapfile -t COMPREPLY < <(compgen -W "5s 10s 30s" -- "$cur"); return; } ;;
    esac
  fi
  [[ "${w[cword-1]}" == --nth ]] && { mapfile -t COMPREPLY < <(compgen -W "1 2 3 4 5" -- "$cur"); return; }
  case "$command" in
    click|press|tap|hold|wait|find|fill|tree)
      local cache="${XDG_CACHE_HOME:-$HOME/.cache}/anibeam-ctl/labels"
      [[ -r "$cache" ]] || return
      # The word as typed holds the shell's escapes; the cache holds the bare labels. A word
      # that opens a quote is compared bare, any other against each label written as %q
      # writes it (Kaguya-sama\ \(Season\ 2\)).
      local want="$cur" quoted=0
      if [[ "$cur" == [\"\']* ]]; then
        want="${cur:1}"
        quoted=1
      fi
      local IFS=$'\n' label q
      COMPREPLY=()
      while read -r label; do
        q="$(printf '%q' "$label")"
        if (( quoted )); then
          [[ "$label" == "$want"* ]] && COMPREPLY+=("$q")
        else
          [[ "$q" == "$want"* ]] && COMPREPLY+=("$q")
        fi
      done < "$cache"
      # What bash will put after the last ':' is all the reply may hold.
      declare -F __ltrim_colon_completions > /dev/null && __ltrim_colon_completions "$want"
      return ;;
  esac
  mapfile -t COMPREPLY < <(compgen -W "$choices" -- "$cur")
}

complete -F _anibeam_ctl ctl.sh
