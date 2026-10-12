# Tesserae: the Linux shell's design direction

AniBeam is one person's anime library on their own desktop: a 5120x1440 ultrawide and a
portrait panel, a local folder of series, mpv in the window, AniList on the side. Its first job
is getting from launch to the next episode in one click. Its second is keeping the library
tidy.

The direction is a mosaic. Every surface in the app is a tile, set into one grid with one
constant joint (the grout). Two things carry the meaning:

- **A tile's size says how much it matters right now.** What you are mid-way through and what
  airs this week sit as large tiles at the head of the library. The rest of the shelf is the
  small module. A series page is a set of tiles cut to fit its facts.
- **The colour comes from the art.** Each poster's own average colour, darkened into a legible
  ground, becomes the plate its title sits on. The grid reads as a field of coloured stones
  cut from the posters. The chrome around it stays neutral slate.

This supersedes "the lobby" (warm ink, Fraunces, the lightbox halo). Its rules on killed
idioms (below) still hold.

## Tokens

Palette (dark flagship `themes/tesserae-dark.yaml`, light companion `tesserae-light.yaml`).
The slate ramp is borrowed from the owner's own desktop shell (bandit shell, theme slate), so
the app sits on that desktop as if it came with it.

| Role | Hex | Use |
|---|---|---|
| Grout `base00` | `#181b1f` | the ground and every joint between tiles |
| Tile `base01` | `#1d2126` | the resting tile |
| Raised `base02` | `#262b31` | hovered and selected tiles, menus |
| Line `base03` | `#3a4149` | the rare hairline (inputs, menus) |
| Text `base05` | `#eef2f5` | text |
| Dim `base04` | `#a5aeb8` | secondary text |
| Smalt `base0D` | `#7fb3d4` | the accent: active, playing, caught up, focus |
| Gilt `base0A` | `#e0b25a` | behind, paused, waiting: deliberately not the accent |
| Alarm `base08` | `#ff7a4d` | errors and destructive actions only |

Poster tone: `Fmt.posterTone(path)` returns the poster's average colour (decoded at 8x12 in
C++, cached per path). The plate is that colour moved toward the grout until the text on it
holds 4.5:1; hover lifts it one step toward the tone itself.

Type: **Archivo**, one variable family (`wdth` 62 to 125, `wght` 100 to 900) cut to the tile it
sits in, the way one stone is cut to different shapes:

- Display (page titles, the hero tile's title): `wdth` 125, weight 800, `typeDisplay`.
- Titles (tile plates, section names): `wdth` 112, weight 650, `typeLarge` or `typeNormal`.
- UI (controls, body): `wdth` 100, weight 450, `typeNormal`.
- Ledger (tables, episode lists, counts, times): `wdth` 88, weight 500, tabular figures,
  `typeSmall`.

Four sizes only (small 0.85, normal 1, large 1.5, display 2.2 of the system size); widths and
weights do the rest. No monospace anywhere.

Geometry: grout 6 px at density 1 (`theme.grout`); tile radius 12 (`theme.radiusTile`),
anything inside a tile is `radiusTile - padding` so corners nest; pills for chips, segmented
switches and buttons. Corners are the G2 squircle from `Corner.qml`. No shadows, no gradients,
no blur: menus and sheets are raised tiles on a scrim, not glass.

Motion: exponential smoothing at rate 14 for anything tracking a target (bandit shell's value).
Hover is a plate step and a 1 px inner edge; press is scale 0.98. One orchestrated moment: when
the library opens, the hero tiles settle first and the shelf follows in one sweep (no per-card
stagger). Reduced motion (`Theme.reducedMotion` or the setting) turns it off.

## Layout

One module for the whole window. The library computes its columns from the page width
(as many as fit at `posterWidth` plus a grout, the spare pixels handed out one per column
from the left so the right edge is exact) and a vertical unit `u` of half a column. A tile
spanning k units is `k*u + (k-1)*grout` tall, so every edge on the page lands on the
module. The grout is the only joint: between tiles both ways, and as the page's margin on
the top, right and bottom (on the left the rail's own grout is the joint).

```
rail  header: one unit tall, whole column spans, spare columns to the title
┌──┐ ┌───────────────┐┌ Show ─────────┐┌ Sort by ─────────────┐┌ Group by ─┐┌ Show ┐┌ Search ─────┐┌─────┐
│▣ │ │ Library       ││ All Series .. ││ ^ A to Z  Last .. .. ││ None ..   ││  o   ││ [         ] ││ 52  │
│▣ │ └───────────────┘└───────────────┘└──────────────────────┘└───────────┘└──────┘└─────────────┘└─────┘
│▣ │ ┌─────┐┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐      Airing: label stone, then small
│▣ │ │Airing││    ││    ││    ││    ││    ││    ││    │      tiles; its heroes were demoted
│▣ │ │     6││plate││plate││ .. ││    ││    ││    ││    │      because they left a hole
│  │ └─────┘└────┘└────┘└────┘└────┘└────┘└────┘└────┘
│  │ ┌─────┐┌───────────┐┌───────────┐┌────┐┌────┐┌────┐
│  │ │Series││ hero 2x2  ││ hero 2x2  ││ 1  ││ 1  ││ 1  │   label stone, heroes, then small
│  │ │   52││ poster    ││ poster    │└────┘└────┘└────┘   tiles first-fit around them
│  │ └─────┘│           ││           │┌────┐┌────┐┌────┐
│  │ ┌────┐ │ plate     ││ plate     ││ 1  ││ 1  ││ 1  │
│  │ │ 1  │ │ Play      ││ Play      ││    ││    ││    │
└──┘ └────┘ └───────────┘└───────────┘└────┘└────┘└────┘
```

- **The header is a row of tiles**, one unit tall: the title, the tabs, the sorts (the
  active one carries its direction as a chevron; picking it again flips it), the grouping,
  Show watched, the search and the count. Each tile spans the whole columns its content
  needs, the row's spare columns go to the title, and a narrow window wraps it onto a
  second unit row. It scrolls with the wall as its first rows.
- **A series is one stone**: the poster on top (2:3) and, in the same rounded shape, a
  plate in the poster's own tone carrying the title (two lines at most) and one ledger line
  (the airing countdown in the accent, else "3 of 12", else "12 files") with the score at
  its right. The episode code rides the poster's top left, the progress strip its bottom
  edge. The plate is as tall as it takes to land the tile on a whole number of units.
  Nothing is captioned outside a tile, on any page that shows series (`Card.qml`).
- **Size says what matters now.** A hero is 2x2 (two columns, two small tiles and the
  joint between tall) and goes to a series that is in progress and was viewed in the last
  14 days, or airs within seven, most recent first, capped at a third of the columns (one
  from four columns, none below). Its plate carries the title in the display cut, the next
  episode, and a Play button that starts it (or opens the series when nothing is next). A
  series is a hero once on the wall.
- **Section labels are stones**: each section opens with a tile the size of a small one,
  its name at the top left and its count in big figures at the bottom right; Airing's in
  the accent's plate, the rest raised slate. Sections follow each other in the same
  packing, each on a fresh unit row.
- **The packer** (`qml/mosaic.js`, tested by `node apps/linux/qml/test-mosaic.mjs`) is a
  first-fit skyline per section: the label stone at column 0, then the heroes, then the
  small tiles, each at the lowest row its span fits, left-most on ties. A section may not
  leave a hole: every cell above its final small row is filled and that row runs short only
  on the right. While it would, the least important hero is set small and the section is
  packed again.
- The wall is one Flickable; a tile is built only within a screen of the viewport.
- The rail is a column of square tiles, icon over label; the active one is raised, its
  glyph in smalt. No accent bar.
- The series page is a bento: banner across the top, poster, title tile, fact tiles (score,
  episodes, status, next airing), synopsis, tags, the episode ledger, relations.

## Killed idioms (still dead)

- Numbered nav markers; letter-spaced ALL-CAPS eyebrows; monospace data labels.
- Middle-dot meta joins ("A · B · C"): plain sentences instead.
- Appended arrows on controls.
- The lobby's lightbox halo and the frosted glass (a live blur of the whole page every frame).

## The one bold spend

Poster-toned plates. Every other surface is quiet slate. If a future pass wants a second loud
effect, take one away instead.

## Reviewing the work

`scripts/shoot.sh <name> --page <page>` renders a page offscreen into `captures/`;
`scripts/drive.sh <script.json>` clicks through the real app offscreen and captures each step
(see README). Read the PNGs at native resolution.
