"""Builds the nightly logo: the release mark on yellow graph paper, inked by hand.

Every outline is cut from inside its shape (the shape minus a shrunk, slightly warped copy of
itself), so nothing grows past the release logo's silhouette and the line width wanders like a
pen's. The grid is drawn line by line, a little off true. A fixed seed keeps it repeatable.

usage: python3 site/tools/nightly_logo.py site/img/logo.svg site/img/logo-nightly.svg
"""
import random
import re
import sys

src, out = sys.argv[1], sys.argv[2]
glyph = re.findall(r'd="([^"]*)"', open(src).read())[-1]
rnd = random.Random(12)

PAPER, GRIDLINE, EDGE, INK, FILL = "#f3e073", "#c7ad42", "#a88f24", "#121212", "#96841e"
X0, Y0, SIZE, R = 14, 14, 996, 225

# The grid: about 7 squares across, each line its own stroke, slightly tilted and bowed.
lines = []
step = SIZE / 7
for i in range(1, 7):
    for vertical in (True, False):
        p = (X0 if vertical else Y0) + i * step + rnd.uniform(-6, 6)
        a, b = p + rnd.uniform(-7, 7), p + rnd.uniform(-7, 7)
        mid = (a + b) / 2 + rnd.uniform(-5, 5)
        w = rnd.uniform(3.5, 6.5)
        o = rnd.uniform(0.45, 0.75)
        if vertical:
            d = f"M{a:.1f} 0 Q{mid:.1f} 512 {b:.1f} 1024"
        else:
            d = f"M0 {a:.1f} Q512 {mid:.1f} 1024 {b:.1f}"
        lines.append(f'<path d="{d}" stroke-width="{w:.1f}" opacity="{o:.2f}"/>')

svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
<!-- The nightly logo, after the owner's sketch: the release mark on yellow graph paper, inked by
     hand. Built by site/tools/nightly_logo.py. Outlines are cut from inside each shape, so the silhouette
     is exactly the release logo's. -->
<defs>
  <clipPath id="sq"><rect x="{X0}" y="{Y0}" width="{SIZE}" height="{SIZE}" rx="{R}"/></clipPath>
  <!-- ink(r, warp): the shape minus a copy shrunk by r and warped, so a ring of wandering width
       inside the edge -->
  <filter id="ink-edge" x="0" y="0" width="100%" height="100%" color-interpolation-filters="sRGB">
    <feMorphology in="SourceAlpha" operator="erode" radius="11" result="in"/>
    <feTurbulence type="fractalNoise" baseFrequency="0.006" numOctaves="2" seed="4" result="n"/>
    <feDisplacementMap in="in" in2="n" scale="18" xChannelSelector="R" yChannelSelector="G" result="w"/>
    <feComposite in="SourceAlpha" in2="w" operator="out" result="ring"/>
    <feFlood flood-color="{EDGE}"/><feComposite in2="ring" operator="in"/>
  </filter>
  <filter id="ink-bold" x="0" y="0" width="100%" height="100%" color-interpolation-filters="sRGB">
    <feMorphology in="SourceAlpha" operator="erode" radius="7" result="in"/>
    <feTurbulence type="fractalNoise" baseFrequency="0.011" numOctaves="2" seed="9" result="n"/>
    <feDisplacementMap in="in" in2="n" scale="11" xChannelSelector="R" yChannelSelector="G" result="w"/>
    <feComposite in="SourceAlpha" in2="w" operator="out" result="ring"/>
    <feFlood flood-color="{INK}"/><feComposite in2="ring" operator="in"/>
  </filter>
  <filter id="ink-fine" x="0" y="0" width="100%" height="100%" color-interpolation-filters="sRGB">
    <feMorphology in="SourceAlpha" operator="erode" radius="2.5" result="in"/>
    <feTurbulence type="fractalNoise" baseFrequency="0.02" numOctaves="1" seed="3" result="n"/>
    <feDisplacementMap in="in" in2="n" scale="3" xChannelSelector="R" yChannelSelector="G" result="w"/>
    <feComposite in="SourceAlpha" in2="w" operator="out" result="ring"/>
    <feFlood flood-color="{INK}"/><feComposite in2="ring" operator="in"/>
  </filter>
  <!-- the bold ink fades out along the sweep, leaving the fine line on the spikes -->
  <linearGradient id="taper" gradientUnits="userSpaceOnUse" x1="600" y1="540" x2="410" y2="720">
    <stop offset="0" stop-color="#fff"/><stop offset="1" stop-color="#fff" stop-opacity="0"/>
  </linearGradient>
  <mask id="pen"><rect width="1024" height="1024" fill="url(#taper)"/></mask>
</defs>
<rect x="{X0}" y="{Y0}" width="{SIZE}" height="{SIZE}" rx="{R}" fill="{PAPER}"/>
<g clip-path="url(#sq)" fill="none" stroke="{GRIDLINE}" stroke-linecap="round">
  {chr(10).join("  " + l for l in lines).strip()}
</g>
<rect x="{X0}" y="{Y0}" width="{SIZE}" height="{SIZE}" rx="{R}" filter="url(#ink-edge)"/>
<g clip-path="url(#sq)"><path d="{glyph}" transform="translate(12 14)" fill="{INK}" opacity=".22"/></g>
<path d="{glyph}" fill="{FILL}"/>
<path d="{glyph}" filter="url(#ink-fine)"/>
<path d="{glyph}" filter="url(#ink-bold)" mask="url(#pen)"/>
</svg>
'''
open(out, "w").write(svg)
print("ok", len(svg))
