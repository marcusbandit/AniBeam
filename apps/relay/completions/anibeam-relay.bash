# bash completion for the AniBeam watch together relay.

_anibeam_relay() {
  local cur prev
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"
  case "$prev" in
    --listen) COMPREPLY=($(compgen -W "127.0.0.1:8795 0.0.0.0:8795" -- "$cur")); return ;;
  esac
  COMPREPLY=($(compgen -W "--listen --help --version" -- "$cur"))
}
complete -F _anibeam_relay anibeam-relay
