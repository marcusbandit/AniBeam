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
  is the page; `qml/KnobBar.qml` is the floating prototype bar. H hides the bar, Ctrl+K and /
  focus the search, Ctrl+R re-reads the files, Ctrl+Q quits.
- `themes/` holds the built-ins as verbatim tinted-theming base16 YAML plus the two AniBeam files.

## Presets and screenshots

Every knob can be set at launch, so a state can be captured unattended:

    target/release/anibeam-proto --preset mode=light,source=theme,light=catppuccin-latte,knobs=0

Keys: `mode` (dark, light, system), `source` (system, theme), `dark` and `light` (theme slugs),
`density` (compact, normal, comfortable), `poster` (px), `smoothing` (0 to 1), `base` (radius
base px), `accent` (terminal slot 1 to 6), `lang` (jp, en), `knobs` (0 hides the bar), `sort`
(0 to 4), `tab` (0 to 2).

`scripts/shoot.sh <name> <preset> [keep]` launches a preset, moves the window to DP-1's workspace
6 on this desktop, captures it with grim into `captures/` (or `$OUT`) and closes it unless `keep`
is given. The captures the ticket was judged on are under `docs/prototypes/home-grid-qml/`.
