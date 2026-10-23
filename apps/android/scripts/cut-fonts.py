#!/usr/bin/env python3
"""Cut the desktop's Archivo roles into static fonts for the phone.

The desktop sets Archivo's width and weight axes per role (apps/linux/qml/Tokens.qml,
`roles`). Slint cannot set a variable font's axes, so each role becomes its own static
family here, named after the role, and the pages pick a role by family name.

    python3 -I scripts/cut-fonts.py      (from apps/android, needs fonttools)
"""
import pathlib
import sys

from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

FONTS = pathlib.Path(__file__).resolve().parent.parent / "ui" / "fonts"

# family, source file, wdth, wght; the values are Tokens.qml's roles.
ROLES = [
    ("Archivo Display", "Archivo-Variable-Regular.ttf", 68, 330),
    ("Archivo Title", "Archivo-Variable-SemiBold.ttf", 86, 600),
    ("Archivo Ledger", "Archivo-Variable-Regular.ttf", 88, 500),
    ("Archivo UI", "Archivo-Variable-Regular.ttf", 100, 450),
    ("Archivo UI Strong", "Archivo-Variable-SemiBold.ttf", 100, 600),
]


def rename(font: TTFont, family: str, weight: int) -> None:
    names = font["name"]
    postscript = family.replace(" ", "") + "-Regular"
    for record in list(names.names):
        if record.nameID in (1, 2, 3, 4, 6, 16, 17, 25):
            names.removeNames(nameID=record.nameID)
    for name_id, value in ((1, family), (2, "Regular"), (3, postscript), (4, family), (6, postscript)):
        names.setName(value, name_id, 3, 1, 0x409)
        names.setName(value, name_id, 1, 0, 0)
    font["OS/2"].usWeightClass = weight
    font["OS/2"].usWidthClass = 5
    font["OS/2"].fsSelection = (font["OS/2"].fsSelection & ~0b1100001) | 0b1000000
    font["head"].macStyle = 0


def main() -> int:
    for family, source, wdth, wght in ROLES:
        font = TTFont(FONTS / source)
        static = instancer.instantiateVariableFont(font, {"wdth": wdth, "wght": wght})
        rename(static, family, wght)
        out = FONTS / (family.replace(" ", "-") + ".ttf")
        static.save(out)
        print(f"{out.name}: wdth {wdth}, wght {wght}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
