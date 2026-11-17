#!/usr/bin/env python3
"""Builds the dev build's logo: the release mark as an unfinished blueprint.

Where the nightly logo is the mark inked by hand on graph paper, a dev build is the drawing
before anyone got round to it: white outline on blueprint blue, construction lines running
off the edges, a dimension nobody filled in, hatching that stops halfway, and a strip of
tape saying DEV. A fixed seed keeps it repeatable.

usage: python3 assets/logo/dev.py assets/logo/logo.svg assets/logo/logo-dev.svg
       rsvg-convert -w 512 -h 512 assets/logo/logo-dev.svg -o assets/logo/logo-dev.png

make.py runs both lines and copies the PNG into the desktop app, for a build from source.

Every word on it is drawn as strokes, not set in a font, so it looks the same wherever the SVG
is opened. Only the glyph's bounding box is read here; nightly.py needs the full outline.
"""
import random
import re
import sys

src, out = sys.argv[1], sys.argv[2]
glyph_d = re.findall(r'd="([^"]*)"', open(src).read())[-1]
rnd = random.Random(7)

BLUE, DEEP, MINOR, MAJOR = "#1f5aa8", "#123c78", "#ffffff1f", "#ffffff40"
LINE, GUIDE, TAPE, RED = "#f4f8ff", "#9fd0ff", "#f1e3b0", "#e2453c"
X0, Y0, SIZE, R = 14, 14, 996, 225


def points(d):
    """The glyph's points, for its bounding box: absolute M, then relative c and l."""
    toks = re.findall(r"[MmCcLlZz]|-?\d*\.?\d+(?:e-?\d+)?", d)
    out, cmd, i, x, y = [], None, 0, 0.0, 0.0
    while i < len(toks):
        t = toks[i]
        if t.isalpha():
            cmd = t
            i += 1
            continue
        if cmd == "M":
            x, y = float(toks[i]), float(toks[i + 1])
            i += 2
        elif cmd == "l":
            x, y = x + float(toks[i]), y + float(toks[i + 1])
            i += 2
        elif cmd == "c":
            x, y = x + float(toks[i + 4]), y + float(toks[i + 5])
            i += 6
        else:
            i += 1
        out.append((x, y))
    return out


pts = points(glyph_d)
gx0, gx1 = min(p[0] for p in pts), max(p[0] for p in pts)
gy0, gy1 = min(p[1] for p in pts), max(p[1] for p in pts)


def wobble(x1, y1, x2, y2, amount=1.6):
    """A line drawn with a ruler that slipped a little."""
    mx, my = (x1 + x2) / 2 + rnd.uniform(-amount, amount), (y1 + y2) / 2 + rnd.uniform(-amount, amount)
    return f"M{x1:.1f} {y1:.1f} Q{mx:.1f} {my:.1f} {x2:.1f} {y2:.1f}"


grid = []
# Centred on the square, so neither edge gets a sliver
G0 = X0 + (SIZE % 32) // 2
for v in range(G0, X0 + SIZE, 32):
    major = (v - G0) % 128 == 0
    grid.append(f'<path d="M{v} {Y0}V{Y0 + SIZE}M{X0} {v}H{X0 + SIZE}" stroke="{MAJOR if major else MINOR}" '
                f'stroke-width="{2 if major else 1}"/>')

# Construction lines off the glyph's extremes, running past the edges
guides = []
for x in (gx0, gx1):
    guides.append(f'<path d="{wobble(x, 0, x, 1024)}" stroke="{GUIDE}" stroke-width="2" stroke-dasharray="14 10" opacity=".7"/>')
for y in (gy0, gy1):
    guides.append(f'<path d="{wobble(0, y, 1024, y)}" stroke="{GUIDE}" stroke-width="2" stroke-dasharray="14 10" opacity=".7"/>')

# A dimension above the mark, its number never filled in: two question marks, drawn
def question(x, y):
    """A question mark about 50 units tall, its top at y."""
    return (f'<path d="M{x - 12:.1f} {y + 12:.1f}C{x - 12:.1f} {y - 4:.1f} {x + 14:.1f} {y - 4:.1f} '
            f'{x + 14:.1f} {y + 12:.1f}C{x + 14:.1f} {y + 24:.1f} {x:.1f} {y + 22:.1f} {x:.1f} {y + 34:.1f}" '
            f'fill="none" stroke="{LINE}" stroke-width="7" stroke-linecap="round"/>'
            f'<circle cx="{x:.1f}" cy="{y + 46:.1f}" r="4.5" fill="{LINE}"/>')


dy = gy0 - 52
mid = (gx0 + gx1) / 2
dim = (f'<path d="M{gx0:.1f} {dy:.1f}H{gx1:.1f}" stroke="{LINE}" stroke-width="3"/>'
       f'<path d="M{gx0:.1f} {dy:.1f}l22 -11v22zM{gx1:.1f} {dy:.1f}l-22 -11v22z" fill="{LINE}"/>'
       f'<path d="M{gx0:.1f} {dy - 24:.1f}v48M{gx1:.1f} {dy - 24:.1f}v48" stroke="{LINE}" stroke-width="3"/>'
       f'<rect x="{mid - 46:.1f}" y="{dy - 30:.1f}" width="92" height="60" rx="8" fill="url(#bg)"/>'
       + question(mid - 17, dy - 25) + question(mid + 19, dy - 25))

# A compass arc that was meant to round the hook and missed
cx, cy = gx1 - 40, gy0 + 210
arc = (f'<path d="M{cx + 150:.1f} {cy:.1f} A150 150 0 0 1 {cx - 60:.1f} {cy + 138:.1f}" fill="none" '
       f'stroke="{GUIDE}" stroke-width="3" stroke-dasharray="4 9" stroke-linecap="round"/>'
       f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="7" fill="none" stroke="{GUIDE}" stroke-width="3"/>')

# Hatching inside the mark that stops halfway down
hatch = []
for k in range(-1100, 1100, 26):
    hatch.append(wobble(k, 0, k + 1100, 1100, 3))
hatch_svg = (f'<g clip-path="url(#half)"><g clip-path="url(#glyph)">'
             f'<path d="{" ".join(hatch)}" stroke="{LINE}" stroke-width="5" opacity=".55"/></g></g>')

half_y = gy0 + (gy1 - gy0) * 0.48

svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
<!-- Made by assets/logo/dev.py from assets/logo/logo.svg; edit the script, not this file. -->
<defs>
<clipPath id="sq"><rect x="{X0}" y="{Y0}" width="{SIZE}" height="{SIZE}" rx="{R}"/></clipPath>
<clipPath id="glyph"><path d="{glyph_d}"/></clipPath>
<clipPath id="half"><rect x="0" y="0" width="1024" height="{half_y:.0f}"/></clipPath>
<radialGradient id="bg" gradientUnits="userSpaceOnUse" cx="460" cy="410" r="820"><stop offset="0" stop-color="{BLUE}"/><stop offset="1" stop-color="{DEEP}"/></radialGradient>
</defs>
<g clip-path="url(#sq)">
<rect x="{X0}" y="{Y0}" width="{SIZE}" height="{SIZE}" fill="url(#bg)"/>
<g fill="none">{"".join(grid)}</g>
<g fill="none">{"".join(guides)}</g>
{arc}
{hatch_svg}
<path d="{glyph_d}" fill="none" stroke="{LINE}" stroke-width="9" stroke-linejoin="round"/>
<path d="{glyph_d}" fill="none" stroke="url(#bg)" stroke-width="12" stroke-dasharray="60 99999" stroke-dashoffset="-1500" stroke-linecap="round"/>
{dim}
<g transform="translate(690 852) rotate(-12)">
<rect x="-150" y="-50" width="300" height="100" fill="{TAPE}" opacity=".93"/>
<path d="M-150 -50l-10 12 10 12 -10 12 10 12 -10 12 10 12 -10 12 10 16M150 -50l10 14 -10 12 10 12 -10 12 10 12 -10 12 10 12 -10 14" fill="{TAPE}" opacity=".93"/>
<path d="M-87 -30V30M-87 -30H-67C-37 -30 -37 30 -67 30H-87M13 -30H-19V30H13M-19 0H7M35 -30L61 30L87 -30" fill="none" stroke="{RED}" stroke-width="14" stroke-linecap="round" stroke-linejoin="round"/>
</g>
</g>
<rect x="{X0 + 3}" y="{Y0 + 3}" width="{SIZE - 6}" height="{SIZE - 6}" rx="{R - 3}" fill="none" stroke="{GUIDE}" stroke-width="6" opacity=".6"/>
</svg>
'''
open(out, "w").write(svg)
