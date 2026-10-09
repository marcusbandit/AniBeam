# The lobby — the Linux shell's design direction

Named for where the app puts you: a dark room with one warm light source, and the
posters are the colour. The direction was set with the frontend-design skill's process
(plan, review against the generated-design tells, build, critique on captures); this file
is the memory of what was chosen and why, for future passes.

## Tokens

Palette (dark flagship, `themes/lobby-dark.yaml`; light companion `lobby-light.yaml`):

| Role | Hex | Use |
|---|---|---|
| Ink `base00` | `#1a1511` | the ground — warm brown-black, not neutral |
| Shade `base01–02` | `#221b15`–`#2c241c` | raised surfaces, lines |
| Vellum `base05` | `#efe6d8` | text |
| Dim `base04` | `#9a8b77` | secondary text |
| Tungsten `base09` | `#e8a33d` | the one accent: active, playing, airing-live, watched fill |
| Gold `base0A` | `#f2c14e` | behind/paused states — deliberately distinct from the accent |

Type: **Fraunces** (variable, qrc) is the voice — page titles, section headers, series
titles, its optical sizing settling as point size climbs. **Schibsted Grotesk** (variable,
qrc) is the worker — controls, meta, times, counts. There is no monospace face anywhere;
the fixed-face readouts of earlier designs were a generated-design tell. The faces load
through `FontLoader` in `Tokens.qml` from the qrc, so the packaged binary is
self-contained; `scripts/fetch-fonts.sh` refreshes the committed artifacts.

## What was deliberately killed (and must not creep back)

- Numbered nav markers ("01 / LIBRARY") — numbering implies a sequence; nav isn't one.
- Letter-spaced ALL-CAPS eyebrow labels, everywhere.
- Monospace for data labels and timecodes.
- Middle-dot meta joins ("A · B · C") — plain sentences instead.
- Appended arrows on controls.

## The one bold spend

The card's lightbox treatment (`Card.qml`): on hover the poster lifts and two warm halos
fade in behind it, a lobby lightbox lit from behind. Everything else on the card stays
quiet. If a future pass adds a second bold effect, remove one of the two instead.

## Defaults

A fresh install (no `theme.toml`) lands on the lobby pair with `source = "theme"` —
`ThemeSettings::default` in `config.rs`, guarded by the
`a_fresh_install_without_theme_toml_lands_on_the_lobby_pair` test. Existing configs that
pin `source = "system"` keep reading the desktop's scheme and accent.

## Reviewing the work

`scripts/shoot.sh <name> --page <page>` renders any page offscreen into `captures/`;
settings tabs ride `--props '{"tab":"appearance"}'` (the `:<action>` suffix is the
drawer's, not tabs). The player needs a real file id, not a path:
`bench.sh lobby-player <ws> x --page player --props '{"file": 44}'`. Read the PNGs at
native resolution — thumbnail scaling invented a light search field that was never there.
