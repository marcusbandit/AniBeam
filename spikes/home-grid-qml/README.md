# Home grid prototype in QML: throwaway

Wayfinder ticket #17 on map #2. A thing to react to, not the shell: the home grid inside the
cxx-qt spike window with every theme knob live. Build with `cargo build --release` (needs `rust`,
`lld`, `qt6-base`, `qt6-declarative`) and run `target/release/anibeam-proto`.

It reads the real AniBeam config directory (metadata, the tracker progress cache, view history,
the image cache) and the kitty config strictly read-only; nothing is written anywhere.

## What is where

- `src/bridge.rs` is the `Proto` singleton: two JSON strings, `libraryJson` and `palettesJson`,
  and a `reload()` invokable. `src/library.rs`, `src/kitty.rs` and `src/palettes.rs` build them.
  `--dump` prints both without opening a window.
- `qml/Theme.qml` turns a palette and the knobs into tokens: the mix ratios from the theme ticket,
  the base16 slot mapping, the density factor, the radius ladder and the three type sizes.
- `qml/Corner.qml` is the one rounded-shape primitive, a G2 squircle with reach semantics; a
  poster is an `Image` painted through it with `ShapePath.fillItem`.
- `qml/Card.qml`, `qml/Chip.qml`, `qml/Seg.qml`, `qml/Rail.qml` are the pieces; `qml/Main.qml`
  is the window with the page switch; `qml/KnobBar.qml` is the floating prototype bar. H hides
  the bar, Ctrl+K and / focus the search, Ctrl+R re-reads the files, Ctrl+Q quits.
- Second round: `qml/SettingsPage.qml` (Ctrl+, opens it; four tabs, Library, Appearance, Playback
  and Data, each a page of panels in two columns that fill the width, capped at
  `theme.space(560)` and centred past that; the Appearance tab drives the same knobs as the bar,
  the rest is fake state), `qml/StatusStrip.qml` and `qml/ActivityDrawer.qml` at the foot of every
  page (click the strip or Ctrl+L, Escape closes), and the primitives they are built from:
  `Switch`, `Button`, `Field`, `Dropdown`, `Swatches`, `SliderRow`, `SettingRow`, `Panel`. Feed,
  Watching and Metadata are a title only.
- Fifth round: a settings tab fits the viewport and scrolls only when forced. Every column is a
  `ColumnLayout` as tall as the viewport while its content fits; one panel per column marked
  `grows: true` takes the spare height (`Layout.fillHeight`) and a panel's minimum is its content
  height, so when the natural height passes the viewport (the portrait monitor) the Flickable
  scrolls and nothing is squished. `Panel` gained `stretch` (one item that takes what is left) and
  `foot` (rows pinned to the bottom) slots beside its default rows; `SettingRow` drops its control
  under the words when they would be narrower than `theme.space(60)`. The Playback tab holds
  Playback, Tracks and Subtitle defaults on the left and the preview alone on the right,
  letterboxed; Library and Storage open with stat tiles, Storage has a usage bar, Trackers have
  avatars and counts, and the Data tab ends with an About panel across both columns.
- Third round: `qml/LookPreview.qml` beside the Appearance controls draws one `qml/LookPane.qml`
  per mode, a small Library page from the header to the status strip. A pane holds its own
  `Theme`, forced to that mode and following every other knob, under the id `theme`, so the
  Cards, Chips, Seg, Switch, Buttons and Icons inside render as they would with the app in that
  mode; `Theme.tokensFor(mode)` hands out either token set for anything else.
- Fourth round: `qml/Icon.qml` draws a Lucide glyph (`assets/icons/`, the SVGs with their stroke
  set to black so QtSvg reads them, tinted by `IconImage.color`; needs `qt6-svg`), and `Seg`,
  `Button` and `Chip` take an `icon`. A Seg option can also carry a `delegate` Component, which
  is how the Corners switch shows a smooth and a plain corner instead of words.
  `qml/SubtitlePreview.qml` at the top of the subtitle defaults draws the text style over a
  still, sized off mpv's 720 line reference, and follows the fields as you type.
- `themes/` holds the built-ins as verbatim tinted-theming base16 YAML plus the two AniBeam files.

## Presets and screenshots

Every knob can be set at launch, so a state can be captured unattended:

    target/release/anibeam-proto --preset mode=light,source=theme,light=catppuccin-latte,knobs=0

Keys: `mode` (dark, light, system), `source` (system, theme), `dark` and `light` (theme slugs),
`density` (compact, normal, comfortable), `poster` (px), `smoothing` (0 to 1), `base` (radius
base px), `accent` (terminal slot 1 to 6), `lang` (jp, en), `knobs` (0 hides the bar), `sort`
(0 to 4), `tab` (0 to 2), `page` (library, feed, watching, metadata, settings; `settings:appearance` (or
`settings:look`), `settings:playback` and `settings:data` open the other settings tabs), `drawer` (open), `job`
(1 fakes a running scan on the strip), `scroll` (px down the settings tab shown), `confirm`
(1 opens the first source's Remove question).

`scripts/shoot.sh <name> <preset> [keep]` launches a preset, moves the window to DP-1's workspace
6 on this desktop, captures it with grim into `captures/` (or `$OUT`) and closes it unless `keep`
is given. `scripts/shoot-main.sh <name> <preset> <workspace> [keep]` does the same on the main
monitor: it moves the window to that workspace, shows the workspace, and captures the window's
own rectangle, so a landscape window and nothing else lands in the picture. The captures the
ticket was judged on are under `docs/prototypes/home-grid-qml/`.
