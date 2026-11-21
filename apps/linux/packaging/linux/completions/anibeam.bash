# bash completion for the Anibeam desktop app.

_anibeam() {
  local cur prev
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"
  case "$prev" in
    --root|--out) COMPREPLY=($(compgen -d -- "$cur")); return ;;
    --shoot) COMPREPLY=($(compgen -f -X '!*.png' -- "$cur")); return ;;
    --drive) COMPREPLY=($(compgen -f -X '!*.json' -- "$cur")); return ;;
    --page) COMPREPLY=($(compgen -W "home library series airing watching downloads metadata settings tokens" -- "$cur")); return ;;
    --props|--width|--height|--action) return ;;
  esac
  # A watch together link: anibeam://room/<code>
  if [[ $cur == a* ]] && [[ anibeam://room/ == "$cur"* ]]; then
    COMPREPLY=("anibeam://room/")
    compopt -o nospace 2>/dev/null
    return
  fi
  # A magnet link, which opens in Downloads
  if [[ $cur == m* ]] && [[ magnet:? == "$cur"* ]]; then
    COMPREPLY=("magnet:?")
    compopt -o nospace 2>/dev/null
    return
  fi
  local w i
  # anibeam ctl <command> ...: drives the open window
  for (( i = 1; i < COMP_CWORD; i++ )); do
    if [[ ${COMP_WORDS[i]} == ctl ]]; then
      _anibeam_ctl "$i"
      return
    fi
  done
  for w in "${COMP_WORDS[@]:1:COMP_CWORD-1}"; do
    if [[ $w == serve ]]; then
      COMPREPLY=($(compgen -W "--root" -- "$cur"))
      return
    fi
  done
  if (( COMP_CWORD == 1 )); then
    COMPREPLY=($(compgen -W "serve ctl --version --root --shoot --page --props --width --height --action --drive --out" -- "$cur"))
  else
    COMPREPLY=($(compgen -W "--root --shoot --page --props --width --height --action --drive --out" -- "$cur"))
  fi
}
_anibeam_ctl() {
  local at=$1 cur="${COMP_WORDS[COMP_CWORD]}" prev="${COMP_WORDS[COMP_CWORD-1]}" cmd="" j
  case "$prev" in
    --root) COMPREPLY=($(compgen -d -- "$cur")); return ;;
    --timeout|--nth|--props) return ;;
  esac
  for (( j = at + 1; j < COMP_CWORD; j++ )); do
    case "${COMP_WORDS[j]}" in
      --root|--timeout|--nth|--props) (( j++ )) ;;
      -*) ;;
      *) cmd=${COMP_WORDS[j]}; break ;;
    esac
  done
  if [[ -z $cmd ]]; then
    COMPREPLY=($(compgen -W "open go back forward where pages tree click hover scroll key type shot eval wait run quit help --root --json --timeout" -- "$cur"))
    return
  fi
  local args=$(( COMP_CWORD - j - 1 ))
  case "$cmd" in
    go|open)
      if (( args == 0 )); then
        COMPREPLY=($(compgen -W "home library airing feed watching stats metadata downloads settings series player show recommended --offscreen --props" -- "$cur"))
      elif [[ ${COMP_WORDS[j+1]} == settings ]] && (( args == 1 )); then
        COMPREPLY=($(compgen -W "library accounts appearance playback downloads keys about developer" -- "$cur"))
      else
        COMPREPLY=($(compgen -W "--props --offscreen --timeout --json" -- "$cur"))
      fi ;;
    click) COMPREPLY=($(compgen -W "--nth --right --double --json --timeout" -- "$cur")) ;;
    hover|scroll) COMPREPLY=($(compgen -W "--nth --json --timeout" -- "$cur")) ;;
    key) COMPREPLY=($(compgen -W "--focus Escape Return Tab Shift+Tab Space Backspace Up Down Left Right Home End PageUp PageDown Ctrl+K Alt+Left Alt+Right F11" -- "$cur")) ;;
    shot) COMPREPLY=($(compgen -f -X '!*.png' -- "$cur")) ;;
    run) COMPREPLY=($(compgen -f -X '!*.json' -- "$cur") -) ;;
    wait) COMPREPLY=($(compgen -W "--timeout" -- "$cur")) ;;
    type) COMPREPLY=($(compgen -W "--focus" -- "$cur")) ;;
    *) COMPREPLY=($(compgen -W "--json --timeout" -- "$cur")) ;;
  esac
}
complete -F _anibeam anibeam
