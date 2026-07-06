# AniBeam Visual Overhaul: "Quiet Console"

Date: 2026-07-06. Branch: `feat/visual-overhaul`.

## North star

AniBeam is a calm instrument panel for a personal anime archive. The posters are
the only loud thing on screen; every piece of chrome is flat, dark, hairline-bordered,
and set in tabular mono like a studio timing sheet. The interface feels alive under
the cursor: everything clickable reacts on hover, presses down on click, and moves
with exponential smoothing. Nothing decorates; every mark encodes state.

User philosophy (verbatim constraints):

- Responsive AND interactive: if the mouse touches anything clickable, something
  visibly happens. Hover feedback on every interactive element, no exceptions.
- No visual clutter.
- All existing functionality intact: trackers, series view, tags, metadata, graph,
  tabs, language switch. It may look and animate differently, but must work the same
  and feel good.
- Consistent above all. Personality, not generic.

## Identity

- Voice: quiet console. Flat surfaces (no gradients on chrome), hairline borders,
  big radii (the existing 10/14/20/28/pill scale), generous dark space.
- One living accent: brand teal. Teal means "now / yours / alive": the current
  episode, your progress, live countdowns, the active tab, the viewing node.
- Display face: Zen Maru Gothic (rounded Japanese gothic, bundled locally).
  Used sparingly: brand word, page titles, series hero title, empty-state headings,
  modal titles. Its rounded terminals echo the big radii, and it renders native
  Japanese titles first-class (the app has a JP/EN/romaji language switch).
- Everything else: JetBrains Mono (bundled locally). Counts, dates, codes, and
  countdowns always get `font-variant-numeric: tabular-nums`.
- Signature elements (spend the boldness here, keep the rest disciplined):
  1. The ambient cursor halo + universal press physics. The cursor is a spotlight;
     the UI is an instrument that responds to touch.
  2. Timing-sheet data typography: episode codes (E07), fractions (08/12), and
     countdowns (3d 18h 19m) rendered as first-class typographic objects in
     tabular mono chips, consistent everywhere.

## Design tokens (index.css `:root`, full replacement)

Surfaces (flat, barely-teal neutral):

```css
--bg-primary:   #0b0d0e;   /* page */
--bg-secondary: #121517;
--bg-tertiary:  #191d20;
--bg-card:      #14181a;
--bg-hover:     #1c2124;

--border-color: rgba(255, 255, 255, 0.08);
--border-hover: rgba(255, 255, 255, 0.16);

--text-primary:   #eef3f2;
--text-secondary: #9aa7a4;
--text-muted:     #5f6b68;

--accent-primary:   #3ecfb2;                 /* brand teal */
--accent-secondary: #8ae7d4;                 /* lighter teal: hover text, focus ring */
--accent-glow:      rgba(62, 207, 178, 0.16);
--accent-amber:     #e0b568;                 /* pending, warnings, extras */
--accent-rose:      #e8647e;                 /* danger, failures, dropped */
--accent-blue:      #6cb0f0;                 /* completed, informational */
--accent-violet:    #b39ce8;                 /* rewatching, OVA-ish */

--status-watching:   var(--accent-primary);
--status-completed:  var(--accent-blue);
--status-paused:     var(--accent-amber);
--status-dropped:    var(--accent-rose);
--status-planning:   #8b9895;
--status-rewatching: var(--accent-violet);

--scrim: rgba(10, 12, 12, 0.78);             /* THE dark glass over posters */
--scrim-blur: 6px;

--shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.4);
--shadow-md: 0 4px 16px rgba(0, 0, 0, 0.4);
--shadow-lg: 0 12px 40px rgba(0, 0, 0, 0.5);

--radius-sm: 10px;  --radius-md: 14px;  --radius-lg: 20px;
--radius-xl: 28px;  --radius-pill: 999px;

--transition-fast:   120ms cubic-bezier(0.4, 0, 0.2, 1);
--transition-normal: 200ms cubic-bezier(0.4, 0, 0.2, 1);
--transition-slow:   320ms cubic-bezier(0.4, 0, 0.2, 1);

--font-display: 'Zen Maru Gothic', 'Noto Sans CJK JP', sans-serif;
--font-mono: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, monospace;

--s1: 4px; --s2: 8px; --s3: 12px; --s4: 16px; --s5: 20px;
--s6: 24px; --s8: 32px; --s10: 40px; --s12: 48px; --s16: 64px;

--motion-lift-speed: 12; --motion-halo-speed: 9; --motion-lift-amount: 4px;
```

Per-format hues: keep ten `--format-*` tokens but recalibrate to the palette's
saturation family and remove the brand collision (`--format-tv` must NOT equal
brand teal). Suggested: tv `#5fc4de`, movie `#e0b568`, ova `#b39ce8`,
ona `#7dd4a8`, special `#e8647e`, music `#62c99a`, manga `#e89a62`,
novel `#e88ab0`, oneshot `#d4c060`, vn `#cf8ae0`. All chosen so no two adjacent
formats are confusable and none equals `--accent-primary`.

New tokens must be used to eliminate every hardcoded literal these were standing in
for: `--accent-emerald`, `--accent-red`, `--bg-elev` are currently used but defined
NOWHERE, so their fallback literals always win. Define real replacements:
release-order edges use `--accent-rose`; parent-of-root edges use `--accent-blue`;
`--bg-elev: rgba(255, 255, 255, 0.04)`.

Deleted: `--font-ui` (it was a duplicate of `--font-mono`; replace usages),
`--font-sans`/Outfit (replaced by `--font-display`).

## Typography roles

- Page h1: `--font-display` 700, ~1.9rem, tight leading.
- Series hero title: `--font-display` 700, clamp(1.6rem, 3vw, 2.2rem).
- Section headers: eyebrow style. Mono 600, 0.72rem, uppercase, letter-spacing
  0.08em, `--text-secondary`, count in a neutral chip, hairline rule filling the
  remaining row width. Structure is information: a section header is a sheet label.
- Body/UI: mono 0.85 to 0.9rem. Meta/labels: mono 0.7 to 0.75rem.
- Chips: exactly two sizes, `--chip-font-sm: 0.64rem` and `--chip-font: 0.72rem`.
  (Kills today's 0.62/0.65/0.7/0.72/0.78/0.8/0.85 drift.)

## Interaction contract (applies to EVERY interactive element)

1. Hover: a visible paint delta (background, border, or color) transitioned at
   `--transition-fast`. No instant snaps: every hover-affected property has a
   transition. No hover-only affordances without a rest-state hint.
2. Press: compact controls (buttons, chips, tabs, icon buttons) get
   `:active { transform: scale(0.97); }` with transform in their transition list.
   Large surfaces (rows, cards with JS lift) deepen background or border instead;
   never CSS-transform an element whose inline transform is owned by motion.ts.
3. Focus: the global `:focus-visible` ring (2px `--accent-secondary`, offset 2px)
   stays. Never `outline: none` without a replacement ring on the same element.
   Remove the existing removal on `.segmented-switch__label` only if a segment-level
   ring is added; otherwise container ring is acceptable, leave it.
4. Motion: smooth position/scale tracking uses `utils/motion.ts` `smoothScalar`.
   Never add a new bespoke rAF loop. (LangSwitch/SegmentedSwitch keep theirs for
   now; do not add more copies.)
5. Hover reveals (episode seekbar, card overlays) must transition opacity, and the
   information they reveal must not be the ONLY place that state exists.
6. Tooltips: portal `Tooltip` primitive only. Never native `title=`.
7. `prefers-reduced-motion`: one global block in index.css neutralizes transitions
   and animations; remove per-component reduced-motion patches that become redundant.
8. Active vs hover must be distinguishable: an active nav tab or filter chip gets an
   accent marker (teal text + teal underline dot or tinted border), hover gets a
   neutral paint change. Hovering must never make an inactive item identical to the
   active one.

## Component vocabulary

### The chip (one badge system to rule them all)

New `.chip` family in `styles/primitives.css`, rendered by the reworked `Pill.tsx`
(same props API, extended tones). Replaces ALL of: `Pill` styles, `.hero-chip*`,
`.relation-card-format`, `.show-card-*-badge` dark-glass shells, `.source-pill`,
`.type-tag`, `.filter-pill`, `.franchise-node__status-tag`, transcode pill shell.

- Base: mono 600, `--chip-font`, uppercase, tabular-nums, pill radius, flat
  `--bg-tertiary` + hairline border, `--text-secondary`.
- Sizes: default, `.chip--sm`.
- Tones: `.chip--teal/amber/rose/blue/violet/muted` (text+border tint, transparent
  tinted bg via color-mix).
- `.chip--scrim`: the poster overlay variant: `background: var(--scrim)`,
  `backdrop-filter: blur(var(--scrim-blur))`, no border, `--text-primary`.
  This is the ONLY dark-glass recipe in the app.
- `.chip--toggle`: interactive chip (filter pills, tag toggles): hover paint,
  press scale, `.is-on` state = teal text + teal-tinted border + `--accent-glow` bg.
- Status/format coloring via `data-status` / `data-format` attributes.

### Ownership / externality code (app-wide)

- Owned/in-library: normal solid hairline border.
- External (AniList-only, opens browser): DASHED hairline border + the AniList
  glyph in a `.chip--sm chip--scrim`. Used by: Watching page unowned cards,
  franchise external nodes, recommendation cards. One rule, learned once.

### Poster card anatomy (ShowCard, shared by Home/Feed/Watching)

- Corner chips use `.chip--sm chip--scrim`: top-left episode code (`E07` or
  `MOVIE`), top-right watched fraction.
- Fraction color encodes state calmly: caught-up = teal text, behind = amber text,
  complete = teal check. Rose is RESERVED for failures/dropped; the library grid
  must not read as a wall of alarms (today every behind-show is rose).
- Bottom-left stack: encode chip (amber, pulsing dot while encoding) above rating
  chips. Bottom-right: Hidden chip (scrim variant like everything else).
- Progress bar on the poster bottom edge stays: watched = teal, behind = amber
  underlay, unknown = scrim cap.
- Hover: existing JS lift + border-hover + shadow + poster zoom 1.04 + title to
  `--accent-secondary`. Add `:focus-visible` parity (ring already global; ensure
  the card button shows it: no local suppression).

### Buttons

One `.btn` system: `.btn` (neutral), `.btn--accent` (teal), `.btn--danger` (rose),
`.btn--ghost` (borderless icon-adjacent), `.icon-btn` (square, radius-sm). All get
hover paint + press scale + transitions. Delete the unused `Button.tsx` component
and its phantom classes; plain `<button className="btn ...">` is the pattern.

## CSS architecture

`App.css` (5478 lines) is split into files under `src/renderer/styles/`:

```
fonts.css        generated @font-face (bundled woff2, no network)
index.css        tokens + element resets + focus ring + scrollbar + selection
                 + reduced-motion block (imports fonts.css first)
base.css         .app shell, page scaffold, .btn/.icon-btn, empty/loading,
                 ambient cursor
primitives.css   chip, section, tooltip, lang/segmented switch, score picker,
                 transcode bar, episode row, card primitive
nav.css          navbar, brand, tabs, navbar tail
cards.css        show grid, show card, badges, home head (tabs/sort), airing
                 pager, context menu
series.css       series hero, chips rows, tags panel, characters, relations,
                 match-adjacent bits living on the page
feed.css         feed page, watching page, subscriptions page
player.css       ALL player chrome
settings.css     settings, trackers, metadata tab, match modal, activity log
franchise.css    graph, nodes, edges, frames, filters, graph panels
App.css          @import manifest only, in the order above
```

Import order is the cascade order; later files may override earlier ones. Each
area file starts with a one-line comment naming its owner components.

## Dead code to remove (verified orphans)

- `components/Button.tsx` (never imported), `components/EpisodeCard.tsx` (never
  imported) and their CSS (`.episode-card`, `.episode-thumb`, `.status-badge`).
- Legacy show-card CSS: `.show-card-badge`, `.badge-have/-sep/-total`,
  `.show-card-score/-year/-genre/-ep/-rel/-watched` blocks.
- Old detail layout: `.detail-header`, `.detail-poster-col`, `.detail-eyebrow`
  blocks.
- `.hero-score-select`, `.hero-score-cancel`, `.player-rating-select`,
  `.source-key` (confirm no consumer), `.btn-link` stray class usage.
- index.css duplicate scrollbar + `::selection` (superseded).
- Google Fonts `@import` (replaced by bundled fonts.css).
- MetadataTab dead "Updated" column (always renders a dash): remove column.
- The 7 hardcoded `ui-monospace, ...` font stacks: `var(--font-mono)`.

## Per-area briefs

### Shell / nav (App.tsx, nav.css)

- Brand word in `--font-display`.
- Tabs: hover = neutral bg paint; active = teal text + small teal marker;
  press scale. Keep icon+label layout.
- Version tag, LangSwitch stay. LangSwitch/SegmentedSwitch just get token/palette
  updates, their rAF thumbs are already the house physics.
- Scrollbar/selection singular definitions in index.css.

### Activity log drawer (settings.css)

- Full retokenization: kill the private 40-hex palette, use tokens; stage colors
  become a small set derived from accent tokens.
- Filter chips become `.chip--toggle` (fixes missing hover).
- Rows: hover paint + transitions; group heads get press feedback.
- Keep: bottom-right placement, signal-only content, mount on settings/metadata.

### Home (cards.css, HomePage.tsx, ShowCard.tsx, ContextMenu.tsx)

- Section headers to the eyebrow treatment (Airing, and give the library section a
  proper header too: reuse the Section primitive instead of the hand-rolled
  `.library-tabs-head`).
- Poster card anatomy as specced above; recolor behind-fractions rose to amber.
- Sort control: keep SegmentedSwitch + direction toggle, ensure each icon segment
  has a Tooltip, active segment reads teal.
- Context menu: items get press + focus-visible; replace `alert()` feedback with
  the existing toast idiom if trivially reachable, else leave behavior.
- FLIP reorder stays untouched.
- Use shared `utils/relativeTime.ts` (see cross-cutting).

### Series detail (series.css, SeriesDetailPage.tsx, EpisodeRow.tsx, ScorePicker.tsx)

- Hero: title in display face; chip rows unify on `.chip`; `.hero-chip-hide` gets
  hover/transition (it has none today); AniList chip keeps brand blue but tokenized
  as `--anilist-blue: #02a9ff` in index.css.
- Add a "Continue" affordance in the hero when a next-up episode exists: a
  `.btn--accent` with the episode code, navigates straight to the player. Also
  auto-scrolls to the row if modifier-free UX prefers (button click = play; that is
  the affordance the page lacks).
- Synopsis: keep 5-line clamp, add an expand/collapse toggle.
- Tags: keep spoiler toggle, add "show all" when the mask clips.
- Episode rows: keep marker-cascade tracking wave exactly (it is beloved,
  load-bearing UX); ensure marker has a rest-state hover cue (cursor + subtle ring)
  when tracker is connected; watched rows go neutral-dimmed with teal check;
  next-up row = teal treatment (replaces indigo).
- Score popover: keep; z-index tension noted, portal it via Tooltip-style portal if
  cheap, else keep absolute positioning.
- Recommendations/characters rows: single-row compression is fine at library sizes;
  restyle only.

### Feed / Watching / Subscriptions (feed.css, the three pages)

- Feed upcoming cards: stop duplicating the same fact (left "in 2d" + right
  countdown chip); upcoming shows countdown chip only, recent shows aired/added
  time only.
- Feed upcoming/recent groups: add a slim eyebrow divider between "coming soon"
  and the recent backfill instead of silent concatenation.
- Watching: kill the white rim. Owned = standard bordered card; unowned = dashed
  border + AniList scrim chip (ownership code above). Sub-text explains once.
- Subscriptions: keep the list form but bring it into the idiom: rows get hover
  paint, status becomes `.chip`, spacing tokens; row is still non-clickable but its
  interactive children all react.
- All three use `utils/relativeTime.ts`.

### Franchise graph (franchise.css, FranchiseGraphView.tsx, FranchiseFilters.tsx)

- Preserve: category-reflow vs format-hide filter behaviors, center-once-per-series,
  invisible handles, ghost system, lane math. Do NOT touch layout algorithms.
- Node hover: immediate feedback on the node itself (border-hover + slight
  poster brighten) while the 280ms delayed neighbor-dim stays.
- Add a "center on current" control next to zoom/fit (small crosshair icon btn).
- Debug panel: render only when `localStorage.anibeam.graphDebug === '1'` or
  DEV_MODE; production users should not see node/edge telemetry.
- Edge palette: tokenized (rose for release-order, blue for parent-of-root); chip
  dot colors in filters must exactly match the edge colors they stand for.
- Format chip dot for Series must not be brand teal (collision fixed by new
  `--format-tv`).
- Current node ring = teal (brand), root ring = amber: unchanged semantics, new hues.
- Filters panel + chips: `.chip--toggle` idiom, keep two grouped rows.

### Player chrome (player.css, VideoPlayer.tsx render only)

- HARD GUARDRAIL: do not touch JASSUB/libass/transcode/prewarm logic, video
  element wiring, keyboard handler logic, or auto-hide timing logic. Chrome only:
  classNames, CSS, and JSX inside the controls/overlay markup.
- Retokenize the whole chrome island (the `#6090d0` blue accent dies, brand teal
  takes over; all literals to tokens; fonts to `--font-mono`).
- `.player-ctl-btn` and range thumbs get transitions (hover currently snaps).
- Sub-menu tabs get hover states; sub-style controls get hover/focus states.
- Seek bar: keep the inline gradient mechanism (it encodes intro/outro bands,
  played/unplayed) but move colors to tokens read via getComputedStyle or CSS vars
  inlined from tokens; add a hover time bubble above the scrubber (pure chrome:
  position from mouse X * duration, mono chip). No thumbnails.
- Add a small keyboard-shortcut legend: a `?` icon-btn in the right control group
  toggling a scrim overlay listing the existing shortcuts (no new bindings).
- Keep: toast, rating prompt, codec modals, skip/autonext pills, replay; restyle
  to tokens/chips.

### Settings / Trackers / Metadata (settings.css + the three components)

- Folder rows, source rows, tracker rows: row hover paint; icon buttons already
  ok, keep.
- The three non-persistent controls (metadata-source toggles, subtitle segment,
  auto-scan) keep working as-is but their description line gains a muted
  "not wired up yet" suffix so they stop masquerading as functional.
- Metadata table: remove Updated column; give the rematch affordance an explicit
  small icon-btn (link icon) next to the title in addition to the clickable title.
- Match modal: restyle onto chips/tokens; keep flows (Esc, backdrop, applying).
- Replace SettingsTab's inline non-animated `Toggle`/`Segment` with the
  SegmentedSwitch primitive ONLY if drop-in trivial; otherwise restyle them to
  match visually (thumb physics can be a follow-up).

## Cross-cutting new utilities

- `src/renderer/utils/relativeTime.ts`: single relative-time module:
  `fmtShort(ts)` ("3mo ago", "in 2d"), `fmtCountdown(minutes)` ("3d 18h 19m"),
  `fmtVerbose(ts)` ("2 days ago") for tooltips. Replaces the four divergent
  formatters (HomePage, FeedPage, WatchingPage, airingUtils stays for airing
  math but its display formatting delegates here).
- `scripts/fetch-fonts.mjs`: downloads Zen Maru Gothic (500/700) + JetBrains Mono
  (400/500/600/700) woff2 subsets from the Google Fonts CSS API, writes
  `src/renderer/assets/fonts/*.woff2` + `src/renderer/styles/fonts.css` with
  relative URLs. Run once; artifacts are committed; runtime never touches the
  network for fonts.

## Guardrails (do not violate)

- No em dashes anywhere, including UI copy.
- VideoPlayer JASSUB/libass workarounds and transcode/prewarm logic: untouchable.
- No polling/intervals; no compositor coupling; activity log stays signal-only.
- No native `title=` tooltips.
- Franchise deliberate behaviors (render-stage format hide, center-once) stay.
- `bun run typecheck` green before every commit; relevant `verify:*` green before
  merge. Bun only.
- Functionality parity: trackers, tags, graph, tabs, language switch, hidden shows,
  FLIP reorder, marker cascade, score popover, subscriptions all keep working.
