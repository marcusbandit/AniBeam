#!/bin/bash
S=$HOME/spike-libmpv
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
  FULL=1 $S/quality.sh "fhd-$c" "$FHD" --start 300 --play 60 --stills 380,420,460 ${CFG[$c]}
done
for c in "${ORDER[@]}"; do
  FULL=1 $S/quality.sh "uhd-$c" "$UHD" --start 5 --play 60 --stills 90,120,150 ${CFG[$c]}
done
for c in base hq scale-ewa dscale-mit base2; do
  $S/quality.sh "win-$c" "$FHD" --start 300 --play 60 --stills 380,420,460 ${CFG[$c]}
done
echo "=== MATRIX DONE"
