# bash completion for the AniBeam desktop app.

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
  local w
  for w in "${COMP_WORDS[@]:1:COMP_CWORD-1}"; do
    if [[ $w == serve ]]; then
      COMPREPLY=($(compgen -W "--root" -- "$cur"))
      return
    fi
  done
  if (( COMP_CWORD == 1 )); then
    COMPREPLY=($(compgen -W "serve --version --root --shoot --page --props --width --height --action --drive --out" -- "$cur"))
  else
    COMPREPLY=($(compgen -W "--root --shoot --page --props --width --height --action --drive --out" -- "$cur"))
  fi
}
complete -F _anibeam anibeam
