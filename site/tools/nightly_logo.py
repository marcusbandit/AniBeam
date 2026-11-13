#!/usr/bin/env python3
"""Builds the nightly logo: the release mark on yellow graph paper, inked with a brush.

The ink is real brush strokes in vector, not a filter. A stroke runs along each outline just
inside the shape, its width rising and falling with the pressure, and it is made of bristles:
thin lines side by side, some of which skip where the brush runs dry, with now and then a stray
hair. Everything is clipped to the shape, so the silhouette is exactly the release logo's. The
grid is drawn line by line, a little off true. A fixed seed keeps it repeatable.

usage: python3 site/tools/nightly_logo.py site/img/logo.svg site/img/logo-nightly.svg
"""
import math
import random
import re
import sys

import numpy as np

src, out = sys.argv[1], sys.argv[2]
glyph_d = re.findall(r'd="([^"]*)"', open(src).read())[-1]
rnd = random.Random(12)

PAPER, GRIDLINE, EDGE, INK, FILL = "#f3e073", "#c7ad42", "#9c831c", "#141414", "#96841e"
X0, Y0, SIZE, R = 14, 14, 996, 225


# ---------------------------------------------------------------- geometry

def parse(d):
    """The glyph path (M, c, l, z, relative after the first point) as one closed polyline."""
    toks = re.findall(r"[MmCcLlZz]|-?\d*\.?\d+(?:e-?\d+)?", d)
    pts, cmd, i, cur = [], None, 0, np.zeros(2)
    while i < len(toks):
        t = toks[i]
        if t.isalpha():
            cmd = t
            i += 1
            if cmd in "Zz":
                continue
            continue
        if cmd == "M":
            cur = np.array([float(toks[i]), float(toks[i + 1])])
            pts.append(cur.copy())
            i += 2
        elif cmd == "l":
            cur = cur + np.array([float(toks[i]), float(toks[i + 1])])
            pts.append(cur.copy())
            i += 2
        elif cmd == "c":
            c = [float(x) for x in toks[i:i + 6]]
            p0, p1, p2, p3 = cur, cur + c[0:2], cur + c[2:4], cur + c[4:6]
            for t in np.linspace(0, 1, 9)[1:]:
                pts.append((1 - t) ** 3 * p0 + 3 * (1 - t) ** 2 * t * p1 + 3 * (1 - t) * t * t * p2 + t ** 3 * p3)
            cur = p3
            i += 6
        else:
            i += 1
    return np.array(pts)


def rounded_rect(x, y, w, h, r, n=48):
    pts = []
    for cx, cy, a0 in ((x + w - r, y + r, -90), (x + w - r, y + h - r, 0), (x + r, y + h - r, 90), (x + r, y + r, 180)):
        for a in np.linspace(a0, a0 + 90, n):
            pts.append((cx + r * math.cos(math.radians(a)), cy + r * math.sin(math.radians(a))))
    return np.array(pts)


def resample(ring, step):
    """Points every `step` units along the closed ring, with arc length and inward normals."""
    ring = np.vstack([ring, ring[:1]])
    seg = np.diff(ring, axis=0)
    lens = np.hypot(seg[:, 0], seg[:, 1])
    keep = lens > 1e-6
    ring = np.vstack([ring[:-1][keep], ring[:1]])
    seg, lens = seg[keep], lens[keep]
    cum = np.concatenate([[0], np.cumsum(lens)])
    total = cum[-1]
    s = np.arange(0, total, step)
    xs = np.interp(s, cum, ring[:, 0])
    ys = np.interp(s, cum, ring[:, 1])
    p = np.stack([xs, ys], 1)
    # tangents from neighbours, normals pointing inside (by the ring's orientation)
    t = np.roll(p, -1, 0) - np.roll(p, 1, 0)
    t /= np.hypot(t[:, 0], t[:, 1])[:, None] + 1e-9
    area = 0.5 * np.sum(ring[:-1, 0] * ring[1:, 1] - ring[1:, 0] * ring[:-1, 1])
    sign = 1 if area > 0 else -1  # y points down: positive area is clockwise on screen
    n = sign * np.stack([-t[:, 1], t[:, 0]], 1)
    return p, n, s, total


def wave(s, total, parts):
    """Smooth periodic noise along the ring, a sum of a few sines that close up on themselves."""
    v = np.zeros_like(s)
    for k, amp in parts:
        v += amp * np.sin(2 * math.pi * k * s / total + rnd.uniform(0, 2 * math.pi))
    return v


def runs(mask):
    """Index runs where mask is true, for drawing a bristle in pieces."""
    out, start = [], None
    for i, m in enumerate(mask):
        if m and start is None:
            start = i
        if not m and start is not None:
            out.append((start, i))
            start = None
    if start is not None:
        out.append((start, len(mask)))
    return out


def path_of(points):
    if len(points) < 2:
        return ""
    head = f"M{points[0][0]:.0f} {points[0][1]:.0f}"
    return head + "".join(f"L{x:.0f} {y:.0f}" for x, y in points[1:])


def brush(ring, width, bristles, color, taper=None, dry=0.25, strays=3):
    """SVG for one brush stroke along `ring`, just inside it."""
    p, n, s, total = resample(ring, 4.5)
    # pressure: a slow swell and a quicker wobble
    pressure = 1 + wave(s, total, [(2, 0.22), (5, 0.12), (11, 0.06)])
    w = width * np.clip(pressure, 0.45, 1.5)
    if taper is not None:
        w *= taper(p)
    parts = []
    for k in range(bristles):
        frac = (k + 0.5) / bristles
        off = w * frac
        q = p + n * off[:, None]
        # the inner bristles run dry first; each skips where its own noise dips
        gap = wave(s, total, [(rnd.randint(7, 13), 0.5), (rnd.randint(19, 31), 0.35), (rnd.randint(40, 60), 0.25)])
        mask = gap > (-1 + 2 * dry * frac * frac * 2)
        bw = (w / bristles) * rnd.uniform(1.05, 1.5)
        alpha = rnd.uniform(0.72, 1.0)
        for a, b in runs(mask):
            if b - a < 2:
                continue
            seg = q[a:b]
            parts.append(f'<path d="{path_of(seg)}" stroke-width="{np.mean(bw[a:b]):.2f}" opacity="{alpha:.2f}"/>')
    # stray hairs: a thin line that leaves the stroke and drifts inward for a short way
    for _ in range(strays):
        i0 = rnd.randrange(len(p))
        ln = rnd.randint(10, 23)
        idx = np.arange(i0, i0 + ln) % len(p)
        drift = w[idx] * (1.05 + np.linspace(0, rnd.uniform(0.4, 0.9), ln))
        q = p[idx] + n[idx] * drift[:, None]
        parts.append(f'<path d="{path_of(q)}" stroke-width="{rnd.uniform(1.2, 2.2):.2f}" opacity="{rnd.uniform(0.5, 0.85):.2f}"/>')
    return f'<g fill="none" stroke="{color}" stroke-linecap="round" stroke-linejoin="round">{"".join(parts)}</g>'


# ---------------------------------------------------------------- the drawing

glyph = parse(glyph_d)
square = rounded_rect(X0, Y0, SIZE, SIZE, R)

# Along the sweep the brush lifts, so the spikes at the end get only a light touch.
A, B = np.array([600.0, 540.0]), np.array([410.0, 720.0])


def lift(p):
    t = np.clip(((p - A) @ (B - A)) / ((B - A) @ (B - A)), 0, 1)
    t = t * t * (3 - 2 * t)
    return 1 - 0.68 * t


ink_glyph = brush(glyph, 17, 7, INK, taper=lift, dry=0.28, strays=4)
ink_edge = brush(square, 24, 8, EDGE, dry=0.3, strays=5)

lines = []
step = SIZE / 7
for i in range(1, 7):
    for vertical in (True, False):
        pos = (X0 if vertical else Y0) + i * step + rnd.uniform(-6, 6)
        a, b = pos + rnd.uniform(-7, 7), pos + rnd.uniform(-7, 7)
        mid = (a + b) / 2 + rnd.uniform(-5, 5)
        d = f"M{a:.1f} 0 Q{mid:.1f} 512 {b:.1f} 1024" if vertical else f"M0 {a:.1f} Q512 {mid:.1f} 1024 {b:.1f}"
        lines.append(f'<path d="{d}" stroke-width="{rnd.uniform(3.5, 6.5):.1f}" opacity="{rnd.uniform(0.45, 0.75):.2f}"/>')

svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
<!-- The nightly logo, after the owner's sketch: the release mark on yellow graph paper, inked
     with a brush. Built by site/tools/nightly_logo.py; the ink sits inside each shape, so the
     silhouette is exactly the release logo's. -->
<defs>
  <clipPath id="sq"><rect x="{X0}" y="{Y0}" width="{SIZE}" height="{SIZE}" rx="{R}"/></clipPath>
  <clipPath id="gl"><path d="{glyph_d}"/></clipPath>
</defs>
<rect x="{X0}" y="{Y0}" width="{SIZE}" height="{SIZE}" rx="{R}" fill="{PAPER}"/>
<g clip-path="url(#sq)" fill="none" stroke="{GRIDLINE}" stroke-linecap="round">
{chr(10).join(lines)}
</g>
<g clip-path="url(#sq)">{ink_edge}</g>
<g clip-path="url(#sq)"><path d="{glyph_d}" transform="translate(12 14)" fill="{INK}" opacity=".2"/></g>
<path d="{glyph_d}" fill="{FILL}"/>
<g clip-path="url(#gl)">{ink_glyph}</g>
</svg>
'''
open(out, "w").write(svg)
print("ok", len(svg) // 1024, "KB")
