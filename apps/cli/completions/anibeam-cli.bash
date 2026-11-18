# bash completion for anibeam-cli, Anibeam's core from the terminal.

_anibeam_cli() {
  local cur prev command i
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"
  case "$prev" in
    --root) COMPREPLY=($(compgen -d -- "$cur")); return ;;
    --level) COMPREPLY=($(compgen -W "debug info warn error" -- "$cur")); return ;;
    --tab) COMPREPLY=($(compgen -W "all series movies watched hidden" -- "$cur")); return ;;
    --sort) COMPREPLY=($(compgen -W "alpha lastViewed progress communityScore myScore downloaded" -- "$cur")); return ;;
    --direction) COMPREPLY=($(compgen -W "asc desc" -- "$cur")); return ;;
    --json|--source|--query) return ;;
  esac
  command=""
  for ((i = 1; i < COMP_CWORD; i++)); do
    case "${COMP_WORDS[i]}" in
      --root) ((i++)) ;;
      -*) ;;
      *) command="${COMP_WORDS[i]}"; break ;;
    esac
  done
  case "$command" in
    "") COMPREPLY=($(compgen -W "call events sources scan list show jobs import export help --root --help --version" -- "$cur")) ;;
    call) COMPREPLY=($(compgen -W "--json --wait --help" -- "$cur")) ;;
    events) COMPREPLY=($(compgen -W "--follow --level --help" -- "$cur")) ;;
    scan) COMPREPLY=($(compgen -W "--source --wait --help" -- "$cur")) ;;
    list) COMPREPLY=($(compgen -W "--tab --sort --direction --query --help" -- "$cur")) ;;
    import) COMPREPLY=($(compgen -f -W "--wait --help" -- "$cur")) ;;
    export) COMPREPLY=($(compgen -f -W "--private --wait --help" -- "$cur")) ;;
    help) COMPREPLY=($(compgen -W "call events sources scan list show jobs import export" -- "$cur")) ;;
    *) COMPREPLY=($(compgen -W "--help" -- "$cur")) ;;
  esac
}
complete -F _anibeam_cli anibeam-cli
