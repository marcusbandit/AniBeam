# AniBeam — Gestalt Overhaul Design

**Date:** 2026-05-21
**Author:** Liam (marcusbandit) + Claude
**Status:** Approved for implementation planning

---

## Goal

The app feels visually "off" because three system-level primitives never existed: a spacing scale, a card-grouping language, and a motion contract. Padding, gaps, and hover effects are decided per-component, so they conflict. This spec defines those primitives, plus a small React component kit that consumes them, and a page-by-page rollout sequence.

## Non-goals

- No router, state, or backend changes.
- No re-theming (dark only, by user rule).
- No changes to: VideoPlayer/JASSUB internals, LangSwitch, ActivityLogDrawer behavior.
- No page-to-page route transitions (continuity / shared-element transitions deferred — nice-to-have only).
- No `<Heading>` / `<Text>` typography wrappers — existing fonts + token-driven CSS classes are enough.

## Concrete bugs this must fix

1. **Series Detail — episode text reads off-center.** `.bare-episode-row` has asymmetric padding (`0.7rem 1rem 0.7rem 0.95rem`) overridden to `padding-bottom: 0.95rem` to reserve room for the hover seekbar. The seekbar reservation is baked into the resting padding, so the text rides high whether the seekbar is visible or not.
2. **Series Detail — Related is pixel-brothers with the last episode row.** The `.bare-episode-head` selector is reused for the Related header without a top margin, inheriting only the `.bare-episode-list` 0.3rem row gap as its visual separation from the previous section.
3. **Metadata page — entire page shifts horizontally when the list shortens enough to not scroll.** The scrollbar's appearance/disappearance changes the content area's width. Needs `scrollbar-gutter: stable`.
4. **Relation cards — left-edge stripe doesn't follow the card's rounded corner.** A 3px-wide `::before` cannot carry a 20px `border-radius`; the browser clamps the radius to half the element's width.
5. **System-wide — hover feedback is jittery and weak.** translateY is applied as a discrete CSS transition; nothing reacts to cursor position outside of color crossfades.

## System

### Layer 1 — Foundation (CSS + utility)

**Spacing scale** (added to `styles/index.css`, 4px base):

```
--s1:  4px
--s2:  8px
--s3:  12px
--s4:  16px
--s5:  20px
--s6:  24px
--s8:  32px
--s10: 40px
--s12: 48px
--s16: 64px
```

**Proximity rule** (informs every margin/gap decision):
> Gaps inside a unit must be smaller than gaps between units. Row gap inside a list ≤ s2. Header → list ≤ s4. List → next section ≥ s10.

**Motion tokens:**

```
--motion-lift-speed:   12     /* exponential speed, frame-rate independent */
--motion-halo-speed:   9
--motion-lift-amount:  4px
```

**Motion engine** (`utils/motion.ts`, new):

- One module-level `requestAnimationFrame` loop, started on first subscription, stopped when the last subscriber unmounts.
- Subscribers register a target + current pair; the loop applies `current += (target − current) × (1 − e^(−speed·dt))` each tick with `dt` clamped to `0.05s`.
- Snap to target when `|current − target| < 0.02` to avoid sub-pixel jitter and keep idle cards with no `transform`.
- API sketch:
  ```ts
  type SmoothHandle = { setTarget(t: number): void; release(): void };
  function smoothScalar(
    initial: number,
    speed: number,
    onChange: (v: number) => void,
  ): SmoothHandle;
  ```

**`<AmbientCursor>`** (new component, mounted once in `App.tsx`):

- Renders a fixed-position pointer-events:none div behind `.main-content`, with a radial-gradient halo whose center is driven by smoothed pointer x/y (mouse-relative to the page).
- Exposes a `useAmbient()` hook returning the smoothed normalized coordinates for any surface that wants to opt into the same engine (none planned for v1; provided for future use).
- The halo fades in on `mousemove` and fades out after ~600ms of inactivity. No halo while the player is full-screen.

**`scrollbar-gutter: stable`** added to `.main-content`. Eliminates Metadata's horizontal shift and any other "list got short → page jumped" symptoms.

### Layer 2 — Layout primitives

**`<Page>`**
- Replaces the `.page` div used at the top of every page component.
- Provides max-width, default vertical padding (`--s8` top, `--s16` bottom). The scrollbar-gutter rule lives on `.main-content` (the actual scrolling element); `<Page>` doesn't restate it.
- Slot props: `head` (rendered with `--s6` bottom margin), `children`.

**`<Section title count? action?>`**
- The pattern that's currently `.bare-episode-head` + a sibling content block.
- Owns the section-gap rule: `--s12` top margin (except the first section in a `<Page>`), `--s4` between the head and the children.
- The "Related" pixel-brothers bug becomes impossible because section-gap is structural, not decorated by hand on each usage.

**`<Stack gap="s4">` and `<Inline gap="s2" align="center">`**
- Thin wrappers around flexbox column / row with a `gap` prop bound to the spacing scale.
- Replace the ad-hoc `margin-top`/`margin-bottom` decisions in component-local CSS.
- Implementation: `<div style={{ display: 'flex', flexDirection, gap: `var(--${gap})` }} />`. Zero runtime cost, no library.

### Layer 3 — Content primitives

**`<Card variant="default" | "internal" | "external">`**
- Replaces the shell of `.show-card`, `.episode-card`, `.relation-card`. Inner content is page-specific and stays in the page's component.
- `variant` drives the inset edge glow (Layer-1 box-shadow pattern, NOT a `::before` stripe). Default = no glow; internal = teal; external = indigo.
- Hover lift wired to the motion engine (smooth, frame-rate independent), `--motion-lift-amount` translated on Y. No `transform: scale` on the inner image; the image scales via a CSS transition with the same speed used by the engine for visual coherence.
- Click handling and routing remain in the page's component — `<Card>` is a button-or-div based on `onClick` presence.

**`<EpisodeRow>`** (specifically replaces `.bare-episode-row`):
- Three slot CSS regions: `[marker | code | title | trailing]` plus a reserved **bottom slot** for the hover-revealed seekbar.
- Padding is symmetric (`--s3` vertical, `--s4` horizontal). The seekbar lives **outside** the text's bounding box, anchored to the row's bottom edge in a reserved row of height `--s1`.
- Visual states: `default` / `hovered` / `next-up` / `watched` / `in-progress`. Each adjusts color/border via tokens, never repositions the text.

**`<Pill tone="muted | accent | teal | rose | amber">`**
- Single shape, mono font, consistent letter-spacing.
- Replaces `.bare-episode-pill`, `.bare-episode-flag`, `.relation-card-pill`, `.genre-pill`, `.show-card-badge` (visual style only — the badge math on ShowCard stays).

## Architecture

```
styles/index.css           ← tokens (additions only in foundation PR)
src/renderer/utils/
  motion.ts                ← engine
src/renderer/components/
  primitives/
    AmbientCursor.tsx
    Page.tsx
    Section.tsx
    Stack.tsx               (exports Stack, Inline)
    Card.tsx
    EpisodeRow.tsx
    Pill.tsx
  (existing components stay until their page migrates)
```

Outside of React/lucide imports, primitives import only from `utils/motion.ts` and consume `styles/index.css` tokens. Primitives don't import each other — composition happens in the page that uses them (`<Section>` does not include `<Page>`; `<Card>` does not include `<Pill>`).

## Rollout sequence

Each step is its own commit (and its own `bun run package` per the user's workflow).

1. **Foundation PR** — tokens, motion engine, `<AmbientCursor>` mounted in `App.tsx`, `scrollbar-gutter` on `.main-content`. No page migrations. Verifiable: open any page, move the cursor, halo follows; Metadata page no longer shifts when list shortens.
2. **Layout primitives PR** — `<Page>`, `<Section>`, `<Stack>`, `<Inline>` added. No usages yet. Verifiable: import + render in isolation works.
3. **Content primitives PR** — `<Card>`, `<EpisodeRow>`, `<Pill>`. Same — no usages.
4. **Series Detail migration** (worst offender first). Both diagnosed bugs disappear. Hover lift smooth. Related has breathing room.
5. **Home migration.**
6. **Feed migration.**
7. **Subscriptions migration.**
8. **Metadata migration.** (Already fixed for horizontal shift in step 1; this is the spacing pass.)
9. **Settings migration.** Spacing audit drives this step's scope (user defers to Claude's judgment per the brainstorming session). Out-of-scope changes get their own follow-up specs.
10. **VideoPlayer chrome migration.** Only player chrome — the JASSUB / mpv internals stay untouched per memory.
11. **Cleanup PR.** Delete now-unused CSS rules in `App.css` whose pages have all migrated.

A page is "migrated" when:
- Its component renders only primitives + page-specific content (no raw `<div className="page">`, `<div className="section-head">`, etc.).
- Its hover/lift behavior is driven by the motion engine, not local CSS.
- Its page-local CSS contains no magic spacing numbers — only token references or content-specific layout.

## Verification

For each migration step, the user `bun run package`s and opens the affected page from the .desktop launcher. Visual sign-off only — there is no automated visual regression in the project. The user explicitly opted out of approval gates between steps, so each migration ships when it's done; if a step is wrong, it's the next commit, not a rollback.

## Open questions

None at spec time. Settings-page scope will be defined in the corresponding plan step.

## Out of scope (deferred)

- Page-to-page route transitions (view-transitions API, shared-element). Discussed in brainstorming; user marked as nice-to-have.
- Re-theming, light mode, accent customization.
- Mobile / responsive layouts beyond what already exists.
