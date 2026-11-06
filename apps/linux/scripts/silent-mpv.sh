# Sourced by shoot.sh, drive.sh and bench.sh. Their runs never make a sound (#564).
#
# anibeam_silent_mpv_conf <bundled mpv.conf> <dir> writes <dir>/mpv-silent.conf, a copy of the
# bundled file with ao=null appended, and sets ANIBEAM_SILENT_MPV_CONF to it. The scripts point
# ANIBEAM_MPV_CONF there, so a drive that plays a video sends its audio nowhere instead of to
# the speakers of whoever sits at the machine. Audio still decodes, so tracks, volume and mute
# behave as they do in a normal run.
#
# The copy goes through a temp file and a rename, so parallel captures sharing one dir never
# read a half-written file.
anibeam_silent_mpv_conf() {
  local bundled="$1" dir="$2" tmp
  mkdir -p "$dir"
  tmp="$(mktemp "$dir/.mpv-silent.XXXXXX")"
  { cat "$bundled"; printf '\n# Added by scripts/silent-mpv.sh: capture runs make no sound.\nao=null\n'; } > "$tmp"
  mv -f "$tmp" "$dir/mpv-silent.conf"
  ANIBEAM_SILENT_MPV_CONF="$dir/mpv-silent.conf"
}
