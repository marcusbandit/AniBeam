#!/usr/bin/env python3
"""Draws every Anibeam icon from glyph.svg: the three logos in this folder, the desktop's SVGs
and PNGs, the hicolor sizes, and the Android launcher icons with their picker previews.

Run it after glyph.svg or a colour changes, then commit what it wrote. It needs resvg and
ImageMagick's magick on PATH, and numpy for the nightly mark.

The icons:
  default    the yellow ground with the dark glyph
  dark       the same, colours swapped, for people who keep a dark home screen
  line       Android only: line.svg, a simpler glyph with two speed lines for streaks,
             stroked on its own and drawn in the phone's Material You
             colours on Android 12 and later
"""

import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

here = Path(__file__).resolve().parent
root = here.parents[1]
linux = root / "apps/linux"
phone = root / "apps/phone"
res = phone / "android/res"

YELLOW = "#FDC901"
INK = "#0D0E11"
# The icon's rounded square inside the 1024 canvas.
SQUARE = '<rect x="14" y="14" width="996" height="996" rx="225" fill="{}"/>'
# The Mac icons are these drawings shrunk onto Apple's grid, a tile 824 wide in the 1024
# canvas (100 px of transparent margin all round) where a full-bleed icon looks oversized in
# the Dock. 996 is the tile above, which nightly.py and dev.py draw at as well.
MAC_SCALE = 824 / 996
# The outline's width in the glyph's 1024 units: about 2 dp on Android's 108 dp canvas.
LINE = 32

glyph = re.search(r' d="([^"]+)"', (here / "glyph.svg").read_text()).group(1)
line = re.search(r' d="([^"]+)"', (here / "line.svg").read_text()).group(1)
# line.svg's drawing sits low and left in its square, and its streaks reach the corner, so it
# gets its own fit: its middle (489, 576) on the canvas's middle, and every point within
# 31 dp of it, inside the 33 dp circle every launcher mask keeps.
LINE_SCALE = 0.058
LINE_MIDDLE = (489, 576)


def fit(scale: float) -> str:
    """An SVG transform shrinking a 1024 drawing about the canvas's middle."""
    return f"translate(512 512) scale({scale:.6f}) translate(-512 -512)"


def icon(ground: str, ink: str) -> str:
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">'
        + SQUARE.format(ground)
        + f'<path fill="{ink}" d="{glyph}"/></svg>\n'
    )


def mac_grid(svg: str) -> str:
    """A 1024 logo's svg put on the Mac grid, nested whole."""
    root = re.match(r"<svg[^>]*>", svg.strip()).group(0)
    sized = re.sub(r'\s(width|height)="[^"]*"', "", root).replace("<svg ", '<svg width="1024" height="1024" ', 1)
    inner = sized + svg.strip()[len(root):]
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">'
        f'<g transform="{fit(MAC_SCALE)}">{inner}</g></svg>\n'
    )


def outline(ink: str) -> str:
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">'
        f'<path fill="none" stroke="{ink}" stroke-width="{LINE}" stroke-linejoin="round" '
        f'stroke-linecap="round" transform="{line_fit(1024 / 72)}" d="{line}"/></svg>\n'
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
# The three logos every page and README takes, in this folder: release, nightly and dev. The
# nightly and dev marks are drawn from the release one.
# The pages size it themselves, so it carries a viewBox and no width or height.
(here / "logo.svg").write_text(default.replace(' width="1024" height="1024"', ""))
png(default, here / "logo.png", 512)
for kind in ("nightly", "dev"):
    svg = here / f"logo-{kind}.svg"
    subprocess.run([sys.executable, str(here / f"{kind}.py"), str(here / "logo.svg"), str(svg)], check=True)
    subprocess.run(["resvg", "-w", "512", "-h", "512", str(svg), str(here / f"logo-{kind}.png")], check=True)
# The desktop shows the nightly's or dev's mark on those builds (apps/linux/build.rs).
for kind in ("nightly", "dev"):
    shutil.copyfile(here / f"logo-{kind}.png", linux / f"assets/icon-{kind}.png")
# ---- Mac: every icon the Mac shows on the Mac grid. The window's Dock icon and the Dark
# bundle icon take the 512s in assets/mac/ (build.rs puts them in the qrc on a Mac); the
# bundle's icns takes the 1024s, the one for the channel being built (bundle.sh), and the
# Dark bundle icon has a 1024 of its own.
for name, svg in (
    ("icon", default),
    ("icon-dark", dark),
    ("icon-nightly", (here / "logo-nightly.svg").read_text()),
    ("icon-dev", (here / "logo-dev.svg").read_text()),
):
    png(mac_grid(svg), linux / f"assets/mac/{name}.png", 512)
    if name != "icon-dark":
        png(mac_grid(svg), linux / f"packaging/macos/{name.replace('icon', 'icon-1024')}.png", 1024)
    else:
        # NSWorkspace sets the Dark bundle icon from this, so it is drawn at the icns's size.
        png(mac_grid(svg), linux / "assets/mac/icon-dark-1024.png", 1024)
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
# The website's favicon.ico, which Google and browsers ask for at the site's root: the
# hicolor 48, 32 and 16 written above (site/build.sh copies it in).
subprocess.run(
    ["magick", *(str(linux / f"assets/hicolor/default/{s}.png") for s in (48, 32, 16)), str(here / "favicon.ico")],
    check=True,
)

# ---- Android: adaptive icons from vector layers. The 1024 square maps onto the middle 62 of
# the 108 dp canvas, which keeps the glyph inside the 66 dp circle every launcher mask shows.
SCALE = 62 / 1024
SHIFT = (108 - 62) / 2


def line_fit(unit: float) -> str:
    """line.svg as the launcher shows it, as an SVG transform: the middle 72 dp of the canvas,
    which is what a mask leaves visible, at `unit` pixels per dp."""
    k = LINE_SCALE * unit
    x, y = ((54 - 18 - m * LINE_SCALE) * unit for m in LINE_MIDDLE)
    return f"translate({x:.2f} {y:.2f}) scale({k:.5f})"


def vector(body: str, path: str = glyph, scale: float = SCALE, at: tuple = (SHIFT, SHIFT)) -> str:
    return (
        '<?xml version="1.0" encoding="utf-8"?>\n'
        "<!-- Written by assets/logo/make.py; edit that, not this. -->\n"
        '<vector xmlns:android="http://schemas.android.com/apk/res/android"\n'
        '    android:width="108dp" android:height="108dp"\n'
        '    android:viewportWidth="108" android:viewportHeight="108">\n'
        f'    <group android:translateX="{at[0]:.2f}" android:translateY="{at[1]:.2f}"\n'
        f'        android:scaleX="{scale}" android:scaleY="{scale}">\n'
        f"        <path {body}\n"
        f'            android:pathData="{path}" />\n'
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
line_at = tuple(54 - m * LINE_SCALE for m in LINE_MIDDLE)
write(res / "drawable/ic_glyph_line.xml", vector(stroke.format("@color/line_ink"), line, LINE_SCALE, line_at))
write(res / "drawable/ic_glyph_line_mono.xml", vector(stroke.format("#FFFFFFFF"), line, LINE_SCALE, line_at))


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
    "The glyph's outline alone on a clear ground, inked in Material You by values-v31.",
    "@color/line_ground", "@drawable/ic_glyph_line", "@drawable/ic_glyph_line_mono"))

# The picker's previews: the two fixed icons as they are, and the outline in white for the
# app to tint with the phone's own colours.
icons = phone / "ui/icons"
png(default, icons / "app.png", 256)
png(dark, icons / "app-dark.png", 256)
png(outline("#FFFFFF"), icons / "app-line.png", 256)

print("icons written", file=sys.stderr)
