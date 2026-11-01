#!/usr/bin/env python3
"""Draws every AniBeam icon from glyph.svg: the desktop's SVGs and PNGs, the hicolor sizes,
the README's PNG, and the Android launcher icons with their picker previews.

Run it after glyph.svg or a colour changes, then commit what it wrote. It needs resvg and
ImageMagick's magick on PATH.

The icons:
  default    the yellow ground with the dark glyph
  dark       the same, colours swapped, for people who keep a dark home screen
  line       Android only: the glyph's outline on its own, drawn in the phone's Material You
             colours on Android 12 and later
"""

import re
import subprocess
import sys
import tempfile
from pathlib import Path

here = Path(__file__).resolve().parent
root = here.parents[1]
linux = root / "apps/linux"
android = root / "apps/android"
res = android / "android/res"

YELLOW = "#FDC901"
INK = "#0D0E11"
# The icon's rounded square inside the 1024 canvas.
SQUARE = '<rect x="14" y="14" width="996" height="996" rx="225" fill="{}"/>'
# The outline's width in the glyph's 1024 units: about 2 dp on Android's 108 dp canvas.
LINE = 32

glyph = re.search(r' d="([^"]+)"', (here / "glyph.svg").read_text()).group(1)


def icon(ground: str, ink: str) -> str:
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">'
        + SQUARE.format(ground)
        + f'<path fill="{ink}" d="{glyph}"/></svg>\n'
    )


def outline(ink: str) -> str:
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">'
        f'<path fill="none" stroke="{ink}" stroke-width="{LINE}" stroke-linejoin="round" '
        f'stroke-linecap="round" d="{glyph}"/></svg>\n'
    )


def png(svg: str, out: Path, size: int) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile("w", suffix=".svg") as f:
        f.write(svg)
        f.flush()
        subprocess.run(["resvg", "-w", str(size), "-h", str(size), f.name, str(out)], check=True)


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


default = icon(YELLOW, INK)
dark = icon(INK, YELLOW)

# ---- Desktop: the window and the Rail read the 512 PNGs from qrc, packaging takes the rest.
write(linux / "assets/icon.svg", default)
write(linux / "assets/icon-dark.svg", dark)
png(default, linux / "assets/icon.png", 512)
png(dark, linux / "assets/icon-dark.png", 512)
png(default, root / "assets/icon.png", 512)
# The Mac's icns takes a 1024 for its 512@2x slot.
png(default, linux / "packaging/macos/icon-1024.png", 1024)
# Every hicolor size, for the packages to install and for the app to lay the dark set over
# them in the user's own icon folder when Dark is picked.
for size in (16, 24, 32, 48, 64, 128, 256, 512):
    png(default, linux / f"assets/hicolor/default/{size}.png", size)
    png(dark, linux / f"assets/hicolor/dark/{size}.png", size)
with tempfile.TemporaryDirectory() as tmp:
    sizes = (256, 128, 64, 48, 32, 16)
    parts = []
    for size in sizes:
        p = Path(tmp) / f"{size}.png"
        png(default, p, size)
        parts.append(str(p))
    subprocess.run(["magick", *parts, str(linux / "packaging/windows/anibeam.ico")], check=True)

# ---- Android: adaptive icons from vector layers. The 1024 square maps onto the middle 62 of
# the 108 dp canvas, which keeps the glyph inside the 66 dp circle every launcher mask shows.
SCALE = 62 / 1024
SHIFT = (108 - 62) / 2


def vector(body: str) -> str:
    return (
        '<?xml version="1.0" encoding="utf-8"?>\n'
        "<!-- Written by assets/logo/make.py; edit that, not this. -->\n"
        '<vector xmlns:android="http://schemas.android.com/apk/res/android"\n'
        '    android:width="108dp" android:height="108dp"\n'
        '    android:viewportWidth="108" android:viewportHeight="108">\n'
        f'    <group android:translateX="{SHIFT}" android:translateY="{SHIFT}"\n'
        f'        android:scaleX="{SCALE}" android:scaleY="{SCALE}">\n'
        f"        <path {body}\n"
        f'            android:pathData="{glyph}" />\n'
        "    </group>\n"
        "</vector>\n"
    )


fill = 'android:fillColor="{}"'
stroke = (
    'android:strokeColor="{}" android:strokeWidth="' + str(LINE) + '"\n'
    '            android:strokeLineJoin="round" android:strokeLineCap="round"'
)
write(res / "drawable/ic_glyph.xml", vector(fill.format("@color/icon_ink")))
write(res / "drawable/ic_glyph_dark.xml", vector(fill.format("@color/icon_ground")))
write(res / "drawable/ic_glyph_mono.xml", vector(fill.format("#FFFFFFFF")))
write(res / "drawable/ic_glyph_line.xml", vector(stroke.format("@color/line_ink")))
write(res / "drawable/ic_glyph_line_mono.xml", vector(stroke.format("#FFFFFFFF")))


def adaptive(note: str, ground: str, fore: str, mono: str) -> str:
    return (
        '<?xml version="1.0" encoding="utf-8"?>\n'
        f"<!-- {note} Written by assets/logo/make.py. -->\n"
        '<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">\n'
        f'    <background android:drawable="{ground}" />\n'
        f'    <foreground android:drawable="{fore}" />\n'
        f'    <monochrome android:drawable="{mono}" />\n'
        "</adaptive-icon>\n"
    )


anydpi = res / "mipmap-anydpi-v26"
write(anydpi / "ic_launcher.xml", adaptive(
    "The desktop's icon: the dark glyph on the yellow ground.",
    "@color/icon_ground", "@drawable/ic_glyph", "@drawable/ic_glyph_mono"))
write(anydpi / "ic_launcher_dark.xml", adaptive(
    "The default with its colours swapped: the yellow glyph on the dark ground.",
    "@color/icon_ink", "@drawable/ic_glyph_dark", "@drawable/ic_glyph_mono"))
write(anydpi / "ic_launcher_line.xml", adaptive(
    "The glyph's outline alone, in the colours values-v31 takes from Material You.",
    "@color/line_ground", "@drawable/ic_glyph_line", "@drawable/ic_glyph_line_mono"))

# The picker's previews: the two fixed icons as they are, and the outline in white for the
# app to tint with the phone's own colours.
icons = android / "ui/icons"
png(default, icons / "app.png", 256)
png(dark, icons / "app-dark.png", 256)
png(outline("#FFFFFF"), icons / "app-line.png", 256)

print("icons written", file=sys.stderr)
