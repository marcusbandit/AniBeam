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

```
rail  header row of tiles                                      shelf (the mosaic)
┌──┐ ┌──────────────────────────┐┌──────────────┐┌─────┐┌─────┐
│▣ │ │ Library                  ││ Search       ││ 56  ││  3  │
│▣ │ └──────────────────────────┘└──────────────┘└─────┘└─────┘
│▣ │ ┌ All  Series  Movies ┐ ┌ Sort ┐ ┌ Group ┐
│▣ │ ┌───────────┐┌───────────┐┌────┐┌────┐┌────┐┌────┐
│▣ │ │ hero 2x2  ││ hero 2x2  ││ 1  ││ 1  ││ 1  ││ 1  │
│  │ │ poster    ││ poster    │├────┤├────┤├────┤├────┤
│  │ │ plate+Play││ plate+Play││ 1  ││ 1  ││ 1  ││ 1  │
│  │ └───────────┘└───────────┘└────┘└────┘└────┘└────┘
└──┘ ┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐
```

- Content is left-aligned; numbers are right-aligned inside their tile.
- Pages fill the viewport (the owner's standing rule): tiles stretch to the width, columns are
  computed from it, and only the shelf scrolls.
- The rail is a column of square tiles, icon over label; the active one is raised, its glyph in
  smalt. No accent bar.
- Hero tiles (2x2) go to series that are behind or in progress and were viewed recently, or air
  within seven days, capped at a third of the columns. Each carries the next episode and a Play
  button that starts it directly.
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
