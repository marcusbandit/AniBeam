#!/bin/bash
# matrix-nv.sh BLOCK MON WS: runs one block (fhd or uhd) of the quality matrix fullscreen on MON/WS.
S=${SPIKE:-$HOME/spike-libmpv}
BLOCK=$1; export MON=${2:-HDMI-A-1} WS=${3:-3}
FHD=$S/media/gup03.mkv
UHD=$S/media/gup03-4k.mkv
declare -A CFG=(
  [base]=""
  [hq]="--set profile=high-quality"
  [scale-ewa]="--set scale=ewa_lanczossharp"
  [cscale-ewa]="--set cscale=ewa_lanczossharp"
  [dscale-mit]="--set dscale=mitchell"
  [dither8]="--set dither-depth=8"
  [deband]="--set deband=yes"
  [interp]="--set interpolation=yes --set video-sync=display-resample"
  [base2]=""
)
ORDER=(base hq scale-ewa cscale-ewa dscale-mit dither8 deband interp base2)
for c in "${ORDER[@]}"; do
  case $BLOCK in
    fhd) FULL=1 $S/quality-nv.sh "fhd-$c" "$FHD" --start 300 --play 60 --stills 380,420,460 ${CFG[$c]} ;;
    uhd) FULL=1 $S/quality-nv.sh "uhd-$c" "$UHD" --start 5 --play 60 --stills 90,120,150 ${CFG[$c]} ;;
  esac
done
echo "=== BLOCK $BLOCK DONE on $MON ws $WS"
