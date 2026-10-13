#!/usr/bin/env python3
"""Cut Archivo's variable TTF into three files the shell registers as one family.

Qt registers only a variable font's default instance (Archivo's is wght 600) and never maps
font.weight onto the wght axis, so with the single file every Text that sets a weight and no
variableAxes renders SemiBold. Each cut keeps the full wdth axis and a slice of wght around
its own default, so font.weight picks the nearest cut (400, 600, 800) and a role's
variableAxes (Tokens.roles) still lands anywhere inside it.

    python3 scripts/cut-fonts.py <Archivo[wdth,wght].ttf> <out dir>     (needs fonttools)
"""
import sys

from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

CUTS = [("Regular", 400, (100, 400, 500)), ("SemiBold", 600, (500, 600, 700)), ("ExtraBold", 800, (700, 800, 900))]


def main(src, out):
    for name, weight, wght in CUTS:
        font = instancer.instantiateVariableFont(TTFont(src), {"wght": wght}, updateFontNames=False)
        font["OS/2"].usWeightClass = weight
        font["OS/2"].fsSelection &= ~(1 | 32 | 64)
        if name == "Regular":
            font["OS/2"].fsSelection |= 64
        font["head"].macStyle = 0
        for rec in font["name"].names:
            if rec.nameID in (2, 17):
                rec.string = name
            elif rec.nameID == 4:
                rec.string = "Archivo" if name == "Regular" else "Archivo " + name
            elif rec.nameID == 6:
                rec.string = "Archivo-" + name
        path = f"{out}/Archivo-Variable-{name}.ttf"
        font.save(path)
        print(path)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
