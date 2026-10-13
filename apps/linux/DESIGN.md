# The Linux shell's design direction: flat, home first

AniBeam is one person's anime library on their own desktop: a 5120x1440 ultrawide and a
portrait panel, a local folder of series, mpv in the window, AniList on the side. Its first job
is getting from launch to the next episode in one click. Its second is keeping the library
tidy.

The direction is a glass mosaic, aired out: futuristic, quiet, with room between the panes.
Every surface in the app is a rounded pane, set into one grid with one constant joint (the
grout, 12 px at normal density) that is also the page's margin. Three things carry it:

- **The posters are the colour, and the titles live on them.** A series is its poster,
  full-bleed and cropped to the tile, with the title and one ledger line written on a dark
  scrim that rises from the poster's foot. Nothing sits under a poster and no surface takes a
  poster's colour.
- **Everything else is glass.** The chrome is translucent panes over a near-black slate
  ground: a faint white fill, a 1 px edge and a 1 px top highlight. No colour of its own.
- **A tile's size says how much it matters right now.** What you are mid-way through and what
  airs this week sit as large tiles at the head of the library; the rest of the shelf is the
  small module.

**Flat home pass, 2026-10-13.** The owner found the glass half-shaded and the poster wall hard
to digest, and asked for flat, modern and clean. What changed, and it outranks the glass text
below where they disagree:

- **Flat surfaces.** Panes are one solid step up from the ground. No top highlight, no sheen,
  hairline edges only where a control needs one. The library header's controls sit on the
  ground in a two-row toolbar, not in boxes.
- **Home first.** The app opens on Home: the series to go on with (an episode left half-way, or
  the next one on disk, never a caught-up show), then Airing now, Catch up, Start something
  new, and the genres as a way into the library. Airing comes first because airing is what the
  app is mostly used for.
- **A slim sidebar.** Home and Library are the two places; Feed, Watching, Metadata and
  Settings are quiet round glyphs at the foot.
- **A width cap.** Pages sit in a centred column no wider than `theme.contentMax`, so an
  ultrawide window shows a readable page instead of an endless wall.
- **The series' own colour.** The home band takes the tone of its poster (`PosterTone.qml`,
  the most saturated common hue, read off a thumbnail) as its wash, its Resume button and a
  glow into the page's head. The one place the chrome takes a colour, and it changes with
  what you are watching.
- **The week.** Airing series sit in seven day columns from today, on the day their next
  episode lands, with a Later column for the rest. A card with an episode out and unseen
  carries a "New" tag in the accent.
- **The library is sections of one grid.** A title with its count, the search, one line of
  pill controls, then sections (franchises, or the chosen arrangement's groups) each a heading
  in the display cut with one fact and a count over uniform cards. No heroes, no framed boxes,
  no packing. The lone airing series lead under Airing; the rest close the page under
  Everything else.
- **The week is rows, not columns.** One row per day that has an episode landing, today always
  first and marked in the accent, every day's posters in one line so nothing hides below the
  fold; the series without a date yet close the list.
- **Scrolling** adds a fixed step per wheel notch to a target the content follows with
  exponential smoothing, so a spin of five notches goes five steps (`WheelStep.step`).

This supersedes Tesserae (rounded tiles on poster-toned plates), which superseded "the lobby".
The killed idioms (below) still hold.

## Tokens

Palette (dark flagship `themes/tesserae-dark.yaml`, light companion `tesserae-light.yaml`; the
file names stayed so existing `theme.toml` files keep resolving). The glass values are derived
in `qml/Tokens.qml` from the theme's ground and text, so any base16 theme takes the same glass.

| Role | Value | Use |
|---|---|---|
| Ground `base00` | `#0e1013` | the ground and every joint between panes |
| Glass `theme.glass` | white at 0.035 | the resting pane (`theme.tile`) |
| Raised `theme.glassRaised` | white at 0.06 | hovered and selected panes (`theme.raised`) |
| Pressed `theme.glassPressed` | white at 0.09 | pressed, a selected cell |
| Edge `theme.glassEdge` | white at 0.08 | the 1 px edge of every pane (`theme.line`) |
| Strong edge `theme.glassEdgeStrong` | white at 0.16 | hover edges, floating panes |
| Highlight `theme.glassHighlight` | white at 0.14 | the 1 px top line, the glassy cue |
| Float `theme.glassFloat` | ground lifted 7 percent, at 0.96 | menus, tips, sheets, popups |
| Video glass `theme.glassVideo` | near-black at 0.55 | the player's islands, both modes |
| Poster scrim `theme.posterScrim` | `rgba(8,9,11,0.92)` | the foot of every poster, clear by 55 percent |
| Text `base05` | `#eef2f5` | text |
| Dim `base04` | `#9aa3ad` | secondary text |
| Smalt `base0D` | `#7fb3d4` | the accent, for state only: focus, selection, playing, progress, the airing countdown |
| Gilt `base0A` | `#e0b25a` | warnings only (stray files, the skippable intro band) |
| Alarm `base08` | `#ff7a4d` | errors and destructive actions only |

`base01` and `base02` hold the glass composited on the ground, the opaque equivalents for code
that needs a solid colour. Light (`#e3e6ea` ground, `#111418` text, smalt `#28628a`): the glass
turns to the text colour at low alpha, dark-on-light, with a white top highlight; text on a
poster stays light because the scrim is dark in both modes. The player forces its own tokens
dark, since it is a dark room either way.

List statuses are quiet too: the accent for Watching and Rewatching, greys for the rest. The
other hues stay in the palette for the accent picker and for themes, not for surfaces.
`Fmt.posterTone` is kept but no surface uses it.

Type: **Archivo**, one variable family (`wdth` 62 to 125, `wght` 100 to 900):

- Display (page titles, the hero tile's title): `wdth` 125, weight 760, `typeDisplay`. The
  wide cut is the futuristic note.
- Titles (titles on posters, section names): `wdth` 112, weight 650, `typeLarge` or
  `typeNormal`.
- UI (controls, body): `wdth` 100, weight 450, `typeNormal`.
- Ledger (tables, episode lists, counts, times): `wdth` 88, weight 500, tabular figures,
  `typeSmall`.

Four sizes only (small 0.85, normal 1, large 1.5, display 2.2 of the system size); widths and
weights do the rest. No monospace, no ALL CAPS.

Geometry: rounded panes, plain circular corners, no squircle smoothing. Every tile, poster,
menu, tooltip, sheet, rail tile and player island takes `theme.radiusTile` (3.5 spacing units,
14 px at normal density); a control (button, chip, segmented switch, dropdown, the title
language square) is a pill at half its height; a row nested inside a tile takes
`theme.radiusInner(padding)`, so the two curves stay concentric. `Corner.qml` draws the radius
it is given (`smoothing` is accepted and ignored; `highlight` draws the top line, inset past
the curve) and stays a plain Rectangle unless it holds a fill item. Grout `theme.grout` (3
spacing units, 12 px at normal density), also the page's margin, so everything scales with
density together. No shadows and no live blur (the frosted glass cost too much): the glass is
translucency and hairlines. **Gradients are allowed for the poster scrim only.**

This aired-out pass (2026-10-13) supersedes the 2 px joint and the square geometry of the
first glass mosaic: the panes read as set apart now, and the one place that was straight-line
chrome, the accent bars along a pane's edge, became filled states (below).

Controls: a button is a raised glass pill (edge and highlight), primary is an accent fill with
ground-coloured ink. A segmented switch is cells in a glass pill, the selected cell carried
by a thumb of soft accent (`theme.accentSoft`) with an accent edge (`theme.accentEdge`).
Fields and dropdowns are glass whose edge turns to the accent on focus. The rail's active page
is a pane filled with the soft accent, its glyph and edge in the accent; a list's current row
(dropdown, score picker, track picker) takes the same soft fill, and a hovered row the raised
glass. No bars along an edge: a state is a fill, never a line.

The poster tile (`Card.qml`): the poster full-bleed (`PreserveAspectCrop`, sized by the
layout, never by the image, clipped to the tile's radius through a Corner fill), the scrim,
then bottom left the title (two lines at most) over one dim ledger line (progress, a page's
own line, or "Episode 3" with a clock and the countdown in accent text, or "12 files"); the
score bottom right; the episode code a small rounded tag top left; progress a rounded 3 px
accent line along the foot, inside the padding. Hover: a 1 px accent outline, a lift of 1.5
percent and a deeper scrim; press scales 0.98. The hero takes the display cut and a square
Play button. A ghost (Watching's AniList-only entries) shows its poster at 60 percent.

Motion: exponential smoothing at rate 14 for anything tracking a target (bandit shell's value).
Hover is a glass step and a 1 px edge; press is scale 0.98. One orchestrated moment: when the
library opens, the hero tiles settle first and the shelf follows in one sweep (no per-card
stagger). Reduced motion (`Theme.reducedMotion` or the setting) turns it off.

## Layout

One module for the whole window. The library computes its columns from the page width
(as many as fit at `posterWidth` plus a grout, the spare pixels handed out one per column
from the left so the right edge is exact) and a vertical unit `u` of half a column. A tile
spanning k units is `k*u + (k-1)*grout` tall, so every edge on the page lands on the
module. The grout is the only joint: between tiles both ways, and as the page's margin on
the top, right and bottom (on the left the rail's own grout is the joint).

```
rail  header: one unit tall, whole column spans, spare columns to the title (count at its right)
┌──┐ ┌──────────────────────────┐┌──────────────┐┌─────────────────────┐┌──────────────────────┐┌───────────┐┌──────────┐
│▣ │ │ Library        52 series ││ All Series ..││ ^ A to Z  Last .. ..││ Arrange Related .. ..││ Show  (o) ││ [Search] │
│▣ │ └──────────────────────────┘└──────────────┘└─────────────────────┘└──────────────────────┘│ watched   │└──────────┘
│▣ │ ╔═════════════════════════╤════╤════╗┌────┐╔══════════════╤════╤════╗┌────┐  boxes: a hairline frame,
│▣ │ ║ Airing this week        │ 1  │ 1  ║│ 1  │║ Youjo Senki  │ 1  │ 1  ║│ 1  │  the label stone at its top
│▣ │ ║ Next (1d 16h)         6 │    │    ║│    │║            2 │    │    ║│    │  left taking the slack, the
│▣ │ ╟────┬────┬────┬────┬────┼────┼────╢└────┘╚══════════════╧════╧════╝└────┘  members packed inside;
│  │ ║ 1  │ 1  │ 1  │ 1  │ 1  │ 1  │ 1  ║┌────┐┌────┐╔═════════╤══════════╗      lone series are plain
│  │ ║    │    │    │    │    │    │    ║│ 1  ││ 1  │║ Girls   │ hero 2x2 ║      tiles filling the gaps
│  │ ╚════╧════╧════╧════╧════╧════╧════╝└────┘└────┘║ und P. 3│          ║      around them
│  │ ┌────┐┌────┐┌────┐ ...  everything first-fit, the last row ragged on the right only
│▤ │ ┌────────────────────────────────────────────────────────────────────────────────┐
└──┘ │ status strip: one tile across the content, as tall as the rail's foot tile       │
     └────────────────────────────────────────────────────────────────────────────────┘
```

- **The header is a row of tiles**, one unit tall, content centred: the title with the
  count at its right, the tabs, the sorts (the active one carries its direction as a
  chevron; picking it again flips it), the arrangement, Show watched (the whole tile
  toggles) and the search. No eyebrow captions: a control says what it is, and only the
  arrangement keeps a word ("Arrange"), since a bare "None" would not. Each tile spans the whole columns
  its content needs, the row's spare columns go to the title, and a narrow window wraps it
  onto a second unit row. It scrolls with the wall as its first rows.
- **A series is one stone**: the poster on top (2:3) and, in the same rounded shape, a
  plate in the poster's own tone carrying the title (two lines at most) and one ledger line
  (the airing countdown, else "3 of 12", else "12 files") with the score at its right. The
  countdown is one treatment everywhere (library, heroes, feed): a small pill in the accent
  holding the time ("4d 18h"), its text the ground colour where that holds 4.5:1, and
  "Episode 2" beside it in plain plate text. The episode code rides the poster's top left, the progress strip its bottom
  edge. A small tile's plate is as tall as it takes to land the tile on a whole number of
  units; a hero's plate is exactly as tall as its lines and Play, and the poster takes the
  rest of the module.
  Nothing is captioned outside a tile, on any page that shows series (`Card.qml`).
- **Size says what matters now.** A hero is 2x2 (two columns, two small tiles and the
  joint between tall) and goes to a series that is in progress and was viewed in the last
  14 days, or airs within seven, most recent first: one per box, and among the lone tiles
  a third of the columns at most (one from four columns, none below). Its plate carries
  the title in the display cut, the next episode, and a Play button that starts it (or
  opens the series when nothing is next). A series is a hero once on the wall.
- **Arrange: the wall is boxes.** The owner's ask, after SpaceSniffer (its grouping, not
  its colours): related shows in one labelled box. The header's Arrange control picks
  Related (the default), Type (TV; Movie; OVA, ONA and specials; Music; Unknown), Genre
  (the first genre), Watch state (Watching, Not started, Caught up) or None, and it is a
  preference (`library_arrange`). Related groups by the core's franchise tag on each card
  (the connected franchise among library series); a group of one is a plain tile, no box.
  Type, Genre and Watch state box every group, one included, since there the label is the
  point. The sort orders the members inside a box (Related: release order when the cards
  carry a year), and the boxes and lone tiles go in the order of their best member.
- **A box** is a rectangle of whole slots (a slot: one column by one small tile's height)
  with a hairline frame (`lineStrong`) drawn over its members' outer pixel. Its header is
  the label stone at its top left: the group's name, its one fact line and its count. The
  stone is a slot tall and takes the box's slack (as wide as the members leave over in the
  first row), so every box is a full rectangle with no hole inside at any width, and every
  box edge stays on the same module as the lone tiles beside it. A thinner header strip
  would set each box a fraction of a slot out of step, and the wall could never close up.
  The width is the one whose pixel rectangle is nearest a square, each extra slot of
  header costing as much as a fair step off square, capped at the wall's columns.
- **Airing this week** is the first box in every arrangement: the airing rail, small tiles
  only, the accent countdown on its stone. Its series also stand in their own groups, the
  one deliberate exception to a series showing once, like a "now" shelf. It leaves while a
  search is up.
- **The packer** (`qml/mosaic.js`, tested by `node apps/linux/qml/test-mosaic.mjs`):
  `packGroups` packs boxes and lone tiles as one skyline. It always fills the lowest,
  left-most run of columns; the next item goes there when it fits, a box too wide for the
  run lets the next lone tile in first (lone tiles fill the gaps around boxes), and with
  none left the box takes the best shape the run allows. Every placement sits on a flat
  run, so nothing is left under a tile; only the bottom can come out uneven, and only the
  very last row may run short, on the right. When it would not, the greedy pass is tried
  again with lone tiles held back to the end (they level the bottom) and lone heroes set
  small, cheapest first, then a depth-first search goes back over the decisions, latest
  first, with a budget. The wall is also laid in **bands** (rows of boxes sharing one
  height, their widths summing exactly to the columns, chosen by dynamic programming; the
  lone tiles after them), which always closes; whichever of the two has the cheaper box
  shapes stands, the bands paying extra for moving lone tiles behind the boxes. A wall of
  boxes only (Type, Genre, Watch state) usually comes out in bands. `pack`, the section
  packer, is still the other walls'.
- The live window covers the boxes too: a box is its frame plus its stone and members as
  tiles in the same flat list, positioned absolutely; the frames are bare outlines, always
  drawn, and the rest is built only within a screen of the viewport.
- **The other walls** (Feed, Watching, Subscriptions) are `qml/MosaicWall.qml`: the same
  module and header tiles, and sections rather than boxes, packed by `pack`: each section
  opens with a label stone the size of a small tile (its name, fact line and count), then
  its heroes, then its small tiles, as one continuous skyline in which sections never
  interleave in reading order and heroes are demoted while the wall would leave a hole. The feed's heroes are the episodes out this week (Coming
  soon: airing this week); Watching's are owned series airing this week. A page with
  nothing to show is its header and one tile filling the rest of the viewport, saying what
  to do.
- The wall is one Flickable; a tile is built only within a screen of the viewport.
- The rail is a column of rounded tiles, icon over label; the active one is filled with
  the soft accent, its glyph in smalt. No accent bar.
- The series page is a bento: banner across the top, poster, title tile, fact tiles (score,
  episodes, status, next airing), synopsis, tags, the episode ledger, relations.

## Killed idioms (still dead)

- Numbered nav markers; letter-spaced ALL-CAPS eyebrows; monospace data labels.
- Middle-dot meta joins ("A · B · C"): plain sentences instead.
- Appended arrows on controls.
- The lobby's lightbox halo and the frosted glass (a live blur of the whole page every frame).
- Squircles (G2 corner smoothing): a corner is a plain arc.
- Accent bars along a pane's edge: a state is a fill.
- Poster-toned plates, and any surface tinted by a hue that is not saying something.
- Gradients, except the poster scrim.

## The one bold spend

The posters themselves. Every other surface is quiet glass. If a future pass wants a second
loud effect, take one away instead.

## Reviewing the work

`scripts/shoot.sh <name> --page <page>` renders a page offscreen into `captures/`;
`scripts/drive.sh <script.json>` clicks through the real app offscreen and captures each step
(see README). Read the PNGs at native resolution.
