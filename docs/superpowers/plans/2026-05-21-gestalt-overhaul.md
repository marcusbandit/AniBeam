# Gestalt Overhaul Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace ad-hoc spacing/grouping/motion with a token-driven primitive kit, then migrate every page to consume the primitives — fixing the diagnosed bugs in Series Detail, Relation cards, Metadata, and the broader "feels flat / off-center / pixel-brothers" complaint.

**Architecture:** Token-first incremental. Foundation (CSS tokens + motion engine + `<AmbientCursor>`) ships first and changes nothing visible except the cursor halo and the Metadata scrollbar shift. Layout + content primitives ship next as unused exports. Pages then migrate one at a time, worst-offender (Series Detail) first.

**Tech Stack:** React 19, TypeScript, Electron, Vite. No new dependencies. Verification is `bun run typecheck` + `bun run lint` for code, `bun --bun scripts/verify-motion.mjs` for the smoothing math, and `bun run package` + open-and-look for visual sign-off (per project convention — no test framework in the renderer).

**Spec reference:** `docs/superpowers/specs/2026-05-21-gestalt-overhaul-design.md`

---

## File Structure

**New files:**
- `src/renderer/utils/motion.ts` — exponential smoothing engine (pure, no React).
- `src/renderer/components/AmbientCursor.tsx` — root-level cursor halo.
- `src/renderer/components/primitives/Page.tsx`
- `src/renderer/components/primitives/Section.tsx`
- `src/renderer/components/primitives/Stack.tsx` (exports `Stack` and `Inline`)
- `src/renderer/components/primitives/Pill.tsx`
- `src/renderer/components/primitives/Card.tsx`
- `src/renderer/components/primitives/EpisodeRow.tsx`
- `src/renderer/components/primitives/index.ts` — re-exports for ergonomic imports.
- `scripts/verify-motion.mjs` — runs the smoothing math against fixed dt sequences.

**Modified files:**
- `src/renderer/styles/index.css` — adds spacing scale + motion tokens.
- `src/renderer/styles/App.css` — adds `scrollbar-gutter: stable` to `.main-content`; older rules deleted per-page as their owners migrate.
- `src/renderer/App.tsx` — mounts `<AmbientCursor />`.
- `src/renderer/pages/SeriesDetailPage.tsx` — migrates to primitives.
- `src/renderer/pages/HomePage.tsx` — migrates.
- `src/renderer/pages/FeedPage.tsx` — migrates.
- `src/renderer/pages/SubscriptionsPage.tsx` — migrates.
- `src/renderer/pages/MetadataTab.tsx` — migrates.
- `src/renderer/components/SettingsTab.tsx` — migrates (spacing audit, exact scope determined during the task).
- `src/renderer/pages/VideoPlayer.tsx` — chrome only; JASSUB/mpv internals untouched per project memory.
- `package.json` — adds `verify:motion` script entry.

**Untouched (project memory rules):**
- `src/renderer/components/LangSwitch.tsx` (switch recipe — reuse pattern, don't refactor).
- `src/renderer/components/ActivityLogDrawer.tsx` (signal-only feed; visual style migrates, behavior doesn't).
- The interior of `src/renderer/pages/VideoPlayer.tsx` related to JASSUB/mpv (load-bearing workarounds).

**Commits:** The repo policy in `CLAUDE.md` is "never commit without explicit user request." Each phase ends with a "Pause for user commit" step. The engineer assembles the diff, runs `bun run typecheck && bun run lint`, then waits for the user to say "commit" before staging and running `git commit`.

---

## Phase 1 — Foundation

### Task 1: Spacing & motion tokens

**Files:**
- Modify: `src/renderer/styles/index.css` (additions only)

- [ ] **Step 1: Read current token block**

Run: `head -60 src/renderer/styles/index.css`
Confirm the existing `:root { … }` block ends with the font tokens; the spacing/motion additions go inside the same `:root`.

- [ ] **Step 2: Append spacing scale + motion tokens to the `:root` block**

Add these lines to `src/renderer/styles/index.css` inside the existing `:root { … }` (right before its closing brace):

```css
  /* Spacing scale (4px base). Use these for every padding/margin/gap;
     gaps inside a unit must be smaller than gaps between units. */
  --s1:  4px;
  --s2:  8px;
  --s3:  12px;
  --s4:  16px;
  --s5:  20px;
  --s6:  24px;
  --s8:  32px;
  --s10: 40px;
  --s12: 48px;
  --s16: 64px;

  /* Motion engine constants — values here are read directly by motion.ts
     via getComputedStyle when needed, and by CSS for non-engine fades. */
  --motion-lift-speed:  12;
  --motion-halo-speed:  9;
  --motion-lift-amount: 4px;
```

- [ ] **Step 3: Typecheck**

Run: `bun run typecheck`
Expected: clean exit (no TS files changed; this guards against the build invariants).

- [ ] **Step 4: Visual confirmation**

Run: `bun run dev` and open the app once. Tokens are unused at this point — confirm nothing rendered differently. Close dev.

---

### Task 2: Motion engine

**Files:**
- Create: `src/renderer/utils/motion.ts`

- [ ] **Step 1: Write the engine module**

Create `src/renderer/utils/motion.ts`:

```ts
/**
 * Exponential smoothing engine.
 *
 * Subscribers register a target/current pair via smoothScalar(); a single
 * module-level requestAnimationFrame loop ticks all of them with
 * frame-rate-independent smoothing:
 *
 *   current += (target - current) * (1 - exp(-speed * dt))
 *
 * dt is clamped to 0.05s so that tab-defocused/long-paused frames don't
 * teleport. The loop starts on the first subscription and stops when the
 * last subscriber releases.
 */

export interface SmoothHandle {
  /** Move the target. Engine animates current toward it. */
  setTarget(value: number): void;
  /** Read the current smoothed value without recomputing. */
  current(): number;
  /** Unsubscribe. */
  release(): void;
}

interface Subscriber {
  target: number;
  current: number;
  speed: number;
  onChange: (v: number) => void;
}

const subscribers = new Set<Subscriber>();
let rafId: number | null = null;
let lastTime = 0;

function tick(now: number) {
  const dt = Math.min(0.05, lastTime ? (now - lastTime) / 1000 : 0.016);
  lastTime = now;

  subscribers.forEach((s) => {
    const k = 1 - Math.exp(-s.speed * dt);
    s.current += (s.target - s.current) * k;
    if (Math.abs(s.current - s.target) < 0.02) s.current = s.target;
    s.onChange(s.current);
  });

  if (subscribers.size > 0) {
    rafId = requestAnimationFrame(tick);
  } else {
    rafId = null;
    lastTime = 0;
  }
}

function ensureRunning() {
  if (rafId === null) {
    lastTime = 0;
    rafId = requestAnimationFrame(tick);
  }
}

/**
 * Subscribe a scalar to the engine.
 * onChange is called every frame with the new smoothed value.
 */
export function smoothScalar(
  initial: number,
  speed: number,
  onChange: (v: number) => void,
): SmoothHandle {
  const sub: Subscriber = { target: initial, current: initial, speed, onChange };
  subscribers.add(sub);
  ensureRunning();
  return {
    setTarget(value) { sub.target = value; },
    current() { return sub.current; },
    release() { subscribers.delete(sub); },
  };
}

/**
 * Pure smoothing step. Exposed for testing and for callers who manage their
 * own RAF loop (e.g., AmbientCursor coupling halo + per-card draws).
 */
export function smoothStep(current: number, target: number, speed: number, dt: number): number {
  const clampedDt = Math.min(0.05, Math.max(0, dt));
  const k = 1 - Math.exp(-speed * clampedDt);
  const next = current + (target - current) * k;
  return Math.abs(next - target) < 0.02 ? target : next;
}
```

- [ ] **Step 2: Typecheck**

Run: `bun run typecheck`
Expected: clean exit.

- [ ] **Step 3: Lint**

Run: `bun run lint`
Expected: clean exit.

---

### Task 3: Motion verify script

**Files:**
- Create: `scripts/verify-motion.mjs`
- Modify: `package.json` (add `verify:motion` script entry)

- [ ] **Step 1: Write the verify script**

Create `scripts/verify-motion.mjs`:

```js
import assert from 'node:assert/strict';

const { smoothStep } = await import('../src/renderer/utils/motion.ts');

// --- Identity: dt=0 does not move ---
assert.equal(smoothStep(0, 100, 10, 0), 0);
assert.equal(smoothStep(50, 50, 10, 0.016), 50);

// --- Monotonic progression: each step moves toward the target ---
let cur = 0;
for (let i = 0; i < 10; i++) {
  const next = smoothStep(cur, 100, 10, 0.016);
  assert.ok(next > cur, `step ${i}: expected monotonic increase, cur=${cur} next=${next}`);
  assert.ok(next <= 100, `step ${i}: overshot, next=${next}`);
  cur = next;
}

// --- Snap-to-target when very close ---
assert.equal(smoothStep(99.99, 100, 10, 0.016), 100);
assert.equal(smoothStep(-99.99, -100, 10, 0.016), -100);

// --- Frame-rate independence: same total dt → same result regardless of
//     how many sub-steps. Total dt is 0.04s, inside the engine's 0.05s
//     clamp window so neither path is truncated. Real-world dts are ~0.016s,
//     so this test exercises the regime the engine actually operates in.
//     Allow 0.5 unit tolerance for the iterative path's discretization error.
function manySteps(steps, totalDt, target, speed) {
  let v = 0;
  const dt = totalDt / steps;
  for (let i = 0; i < steps; i++) v = smoothStep(v, target, speed, dt);
  return v;
}
const oneShot    = smoothStep(0, 100, 10, 0.04);
const splitFour  = manySteps(4,  0.04, 100, 10);
const splitTwenty = manySteps(20, 0.04, 100, 10);
assert.ok(Math.abs(oneShot - splitFour)     < 0.5, `frame-rate sensitivity 1↔4: ${oneShot} vs ${splitFour}`);
assert.ok(Math.abs(splitFour - splitTwenty) < 0.5, `frame-rate sensitivity 4↔20: ${splitFour} vs ${splitTwenty}`);

// --- dt clamp prevents teleport from huge gaps ---
const after10sGap = smoothStep(0, 100, 10, 10); // 10 second gap; clamped to 0.05
const after005s   = smoothStep(0, 100, 10, 0.05);
assert.equal(after10sGap, after005s, 'dt should clamp to 0.05');

console.log('motion: ok');
```

- [ ] **Step 2: Add script entry to package.json**

In `package.json` under `scripts`, add the entry between `verify:probe` and `verify:folder` (alphabetic by trailing word doesn't matter — match the existing visual grouping):

```json
"verify:motion": "bun --bun scripts/verify-motion.mjs",
```

- [ ] **Step 3: Run the verify**

Run: `bun --bun scripts/verify-motion.mjs`
Expected stdout: `motion: ok`
Expected exit code: 0.

If anything fails, the assertion message names which property broke — fix and re-run.

---

### Task 4: AmbientCursor

**Files:**
- Create: `src/renderer/components/AmbientCursor.tsx`
- Modify: `src/renderer/styles/App.css` (add `.ambient-cursor` rules)

- [ ] **Step 1: Write the component**

Create `src/renderer/components/AmbientCursor.tsx`:

```tsx
import { useEffect, useRef } from 'react';
import { smoothScalar } from '../utils/motion';

const HALO_SPEED = 9;
const FADE_TIMEOUT_MS = 600;

/**
 * Root-level pointer halo. Renders a single fixed-position layer behind
 * .main-content that follows the cursor with exponential smoothing.
 * Fades out after FADE_TIMEOUT_MS of inactivity; fades back in on the next
 * mousemove.
 */
export default function AmbientCursor() {
  const layerRef = useRef<HTMLDivElement | null>(null);
  const fadeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const el = layerRef.current;
    if (!el) return;

    const writeX = (v: number) => { el.style.setProperty('--ambient-x', v.toFixed(2) + 'px'); };
    const writeY = (v: number) => { el.style.setProperty('--ambient-y', v.toFixed(2) + 'px'); };

    const x = smoothScalar(window.innerWidth / 2,  HALO_SPEED, writeX);
    const y = smoothScalar(window.innerHeight / 2, HALO_SPEED, writeY);

    const onMove = (ev: MouseEvent) => {
      x.setTarget(ev.clientX);
      y.setTarget(ev.clientY);
      el.classList.add('is-active');
      if (fadeTimerRef.current) clearTimeout(fadeTimerRef.current);
      fadeTimerRef.current = setTimeout(() => el.classList.remove('is-active'), FADE_TIMEOUT_MS);
    };

    window.addEventListener('mousemove', onMove, { passive: true });
    return () => {
      window.removeEventListener('mousemove', onMove);
      if (fadeTimerRef.current) clearTimeout(fadeTimerRef.current);
      x.release();
      y.release();
    };
  }, []);

  return <div ref={layerRef} className="ambient-cursor" aria-hidden="true" />;
}
```

- [ ] **Step 2: Add the CSS layer**

Append to `src/renderer/styles/App.css` (near the top, after the `.app` block):

```css
/* ============ AMBIENT CURSOR ============ */
/* Sits at the root, behind everything. The halo is driven by JS-set
   --ambient-x / --ambient-y custom props; opacity is the only thing CSS
   crossfades, via the .is-active class toggled by AmbientCursor.tsx. */
.ambient-cursor {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  opacity: 0;
  transition: opacity 280ms ease;
  background:
    radial-gradient(
      540px circle at var(--ambient-x, 50%) var(--ambient-y, 50%),
      color-mix(in srgb, var(--accent-primary) 16%, transparent),
      color-mix(in srgb, var(--accent-teal)    5%, transparent) 35%,
      transparent 65%
    );
}
.ambient-cursor.is-active { opacity: 1; }

/* The app shell already establishes a stacking context; nudge real content
   above the halo. */
.navbar, .main-content { position: relative; z-index: 1; }
```

- [ ] **Step 3: Typecheck + lint**

Run: `bun run typecheck && bun run lint`
Expected: clean exit.

---

### Task 5: Wire AmbientCursor + scrollbar-gutter

**Files:**
- Modify: `src/renderer/App.tsx`
- Modify: `src/renderer/styles/App.css` (`.main-content` rule)

- [ ] **Step 1: Mount AmbientCursor in App.tsx**

In `src/renderer/App.tsx`, add the import after the existing component imports:

```tsx
import AmbientCursor from "./components/AmbientCursor";
```

Inside `AppContent`, mount it as the first child of `<div className="app">`, but guard against player route (per spec — no halo while the player is full-screen):

```tsx
return (
  <div className="app">
    {!isPlayerRoute && <AmbientCursor />}
    {!isPlayerRoute && (
      <nav className="navbar">
        ...
```

- [ ] **Step 2: Add scrollbar-gutter to .main-content**

In `src/renderer/styles/App.css`, locate the existing `.main-content { … }` rule (around line 25) and add `scrollbar-gutter: stable;` to it:

```css
.main-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-gutter: stable;
}
```

- [ ] **Step 3: Typecheck + lint**

Run: `bun run typecheck && bun run lint`
Expected: clean exit.

---

### Task 6: Phase 1 visual sign-off

- [ ] **Step 1: Package the app**

Run: `bun run package`
Expected: build succeeds, `out/AniBeam-linux-x64/anibeam` exists.

- [ ] **Step 2: Open the app from the launcher** (per project memory; the .desktop file runs the packaged binary)

Verify:
1. **Halo follows cursor** on every page except `/player/*`. Move mouse — visible soft indigo/teal radial gradient trails the pointer. After ~600ms idle, halo fades out.
2. **Metadata page no longer shifts horizontally** when its list shortens past the scroll threshold. Navigate to Metadata, scrub the filter/search until the list fits without scroll — the page must not jump right.
3. **No other regressions** — Series Detail, Home, Feed, Subscriptions, Settings, Video Player all still render identically except for the halo behind them.

- [ ] **Step 3: PAUSE — request user commit**

Print a one-line diff summary and the message "Phase 1 ready to commit?". Wait for user to say "commit" before staging. Per `CLAUDE.md`, no auto-commits.

Suggested commit message:
```
feat(motion): ambient cursor halo + spacing tokens + scrollbar-gutter

- Introduce --s1..--s16 spacing scale and motion constants
- Add motion.ts smoothing engine (frame-rate independent) + verify script
- Mount AmbientCursor at root; halo follows pointer with exponential smoothing
- scrollbar-gutter: stable on .main-content fixes Metadata page horizontal shift
```

---

## Phase 2 — Layout primitives

### Task 7: Page primitive

**Files:**
- Create: `src/renderer/components/primitives/Page.tsx`
- Modify: `src/renderer/styles/App.css` (add `.page--primitive` rules; keep `.page` until pages migrate)

- [ ] **Step 1: Write Page.tsx**

Create `src/renderer/components/primitives/Page.tsx`:

```tsx
import type { ReactNode } from 'react';

interface PageProps {
  /** Optional page head — title, search, actions. Rendered with --s6 below. */
  head?: ReactNode;
  children: ReactNode;
  /** Override the default max-width for this page (e.g., metadata). */
  maxWidth?: number;
}

/**
 * Page shell. Replaces the bare <div className="page"> wrapping at the top
 * of every page component. Provides max-width, default vertical padding,
 * and a structural slot for the page head with consistent spacing.
 */
export default function Page({ head, children, maxWidth }: PageProps) {
  const style = maxWidth ? { maxWidth: `${maxWidth}px` } : undefined;
  return (
    <div className="page page--primitive" style={style}>
      {head && <div className="page__head">{head}</div>}
      {children}
    </div>
  );
}
```

- [ ] **Step 2: Add CSS for the primitive variant**

Append to `src/renderer/styles/App.css` (near the existing `.page` rule):

```css
/* Page primitive — used by <Page>. Existing pages still use plain .page;
   when a page migrates, its component wraps content in <Page> and the
   .page--primitive rules win via specificity. */
.page--primitive {
  padding: var(--s8) var(--s10) var(--s16);
}
.page--primitive .page__head {
  margin-bottom: var(--s6);
}
```

- [ ] **Step 3: Typecheck + lint**

Run: `bun run typecheck && bun run lint`
Expected: clean exit.

---

### Task 8: Section primitive

**Files:**
- Create: `src/renderer/components/primitives/Section.tsx`
- Modify: `src/renderer/styles/App.css`

- [ ] **Step 1: Write Section.tsx**

Create `src/renderer/components/primitives/Section.tsx`:

```tsx
import type { ReactNode } from 'react';

interface SectionProps {
  title: string;
  /** Optional small count badge next to the title (e.g., episode total). */
  count?: number | string;
  /** Optional right-aligned actions (e.g., "See all" link). */
  action?: ReactNode;
  children: ReactNode;
  /** Set true for the first section in a Page; suppresses the top gap. */
  first?: boolean;
}

/**
 * Section block. The Related-pixel-brother bug is impossible inside this
 * primitive because section-gap is structural, not decorated by hand.
 */
export default function Section({ title, count, action, children, first }: SectionProps) {
  return (
    <section className={`section--primitive${first ? ' section--first' : ''}`}>
      <header className="section__head">
        <div className="section__title-group">
          <h2 className="section__title">{title}</h2>
          {count !== undefined && <span className="section__count">{count}</span>}
        </div>
        {action && <div className="section__action">{action}</div>}
      </header>
      <div className="section__body">{children}</div>
    </section>
  );
}
```

- [ ] **Step 2: Add CSS**

Append to `src/renderer/styles/App.css`:

```css
/* Section primitive — every <Section> gets s12 top to break from the
   previous block. The 'first' modifier removes that for the first section
   inside a <Page>. */
.section--primitive { margin-top: var(--s12); }
.section--primitive.section--first { margin-top: 0; }
.section--primitive .section__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: var(--s4);
  padding-bottom: var(--s3);
  border-bottom: 1px solid var(--border-color);
}
.section--primitive .section__title-group {
  display: inline-flex;
  align-items: baseline;
  gap: var(--s2);
}
.section--primitive .section__title {
  font-family: var(--font-sans);
  font-weight: 700;
  font-size: 1.05rem;
  letter-spacing: -0.01em;
  margin: 0;
  color: var(--text-primary);
}
.section--primitive .section__count {
  font-family: var(--font-mono);
  font-size: 0.78rem;
  color: var(--text-muted);
}
.section--primitive .section__action {
  font-family: var(--font-mono);
  font-size: 0.78rem;
  color: var(--text-secondary);
}
.section--primitive .section__body { display: block; }
```

- [ ] **Step 3: Typecheck + lint**

Run: `bun run typecheck && bun run lint`
Expected: clean exit.

---

### Task 9: Stack + Inline primitives

**Files:**
- Create: `src/renderer/components/primitives/Stack.tsx`

- [ ] **Step 1: Write Stack.tsx (exports both Stack and Inline)**

Create `src/renderer/components/primitives/Stack.tsx`:

```tsx
import type { CSSProperties, HTMLAttributes, ReactNode } from 'react';

export type SpaceToken =
  | 's1' | 's2' | 's3' | 's4' | 's5' | 's6' | 's8' | 's10' | 's12' | 's16';

interface StackProps extends HTMLAttributes<HTMLDivElement> {
  gap?: SpaceToken;
  children: ReactNode;
}
interface InlineProps extends StackProps {
  /** flexbox align-items — defaults to 'center'. */
  align?: CSSProperties['alignItems'];
  /** flexbox justify-content. */
  justify?: CSSProperties['justifyContent'];
  /** wrap rows when content overflows. */
  wrap?: boolean;
}

/**
 * Vertical flex container with a token-driven gap. Replaces ad-hoc
 * margin-top/bottom decisions in page-local CSS.
 */
export function Stack({ gap = 's4', style, children, ...rest }: StackProps) {
  return (
    <div
      style={{ display: 'flex', flexDirection: 'column', gap: `var(--${gap})`, ...style }}
      {...rest}
    >
      {children}
    </div>
  );
}

/**
 * Horizontal flex container with a token-driven gap.
 */
export function Inline({
  gap = 's2',
  align = 'center',
  justify,
  wrap,
  style,
  children,
  ...rest
}: InlineProps) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: align,
        justifyContent: justify,
        flexWrap: wrap ? 'wrap' : 'nowrap',
        gap: `var(--${gap})`,
        ...style,
      }}
      {...rest}
    >
      {children}
    </div>
  );
}
```

- [ ] **Step 2: Typecheck + lint**

Run: `bun run typecheck && bun run lint`
Expected: clean exit.

- [ ] **Step 3: PAUSE — request user commit (end of Phase 2)**

Suggested commit message:
```
feat(primitives): add Page, Section, Stack/Inline layout primitives

Unused so far — page migrations consume them in later commits.
```

---

## Phase 3 — Content primitives

### Task 10: Pill primitive

**Files:**
- Create: `src/renderer/components/primitives/Pill.tsx`
- Modify: `src/renderer/styles/App.css`

- [ ] **Step 1: Write Pill.tsx**

Create `src/renderer/components/primitives/Pill.tsx`:

```tsx
import type { ReactNode } from 'react';

export type PillTone = 'muted' | 'accent' | 'teal' | 'rose' | 'amber';

interface PillProps {
  tone?: PillTone;
  children: ReactNode;
}

/**
 * Small status badge. Replaces .bare-episode-pill, .bare-episode-flag,
 * .relation-card-pill, .genre-pill (and the visual half of .show-card-badge).
 */
export default function Pill({ tone = 'muted', children }: PillProps) {
  return <span className={`pill pill--${tone}`}>{children}</span>;
}
```

- [ ] **Step 2: Add CSS**

Append to `src/renderer/styles/App.css`:

```css
/* Pill primitive — single shape, mono font, tone via modifier. */
.pill {
  display: inline-flex;
  align-items: center;
  gap: var(--s1);
  padding: 3px 10px;
  border-radius: var(--radius-pill);
  font-family: var(--font-mono);
  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  border: 1px solid var(--border-color);
  background: transparent;
  color: var(--text-muted);
}
.pill--accent {
  color: var(--accent-secondary);
  background: color-mix(in srgb, var(--accent-secondary) 14%, transparent);
  border-color: color-mix(in srgb, var(--accent-secondary) 35%, var(--border-color));
}
.pill--teal {
  color: var(--accent-teal);
  background: color-mix(in srgb, var(--accent-teal) 14%, transparent);
  border-color: color-mix(in srgb, var(--accent-teal) 35%, var(--border-color));
}
.pill--rose {
  color: var(--accent-rose);
  background: color-mix(in srgb, var(--accent-rose) 14%, transparent);
  border-color: color-mix(in srgb, var(--accent-rose) 35%, var(--border-color));
}
.pill--amber {
  color: var(--accent-amber);
  background: color-mix(in srgb, var(--accent-amber) 14%, transparent);
  border-color: color-mix(in srgb, var(--accent-amber) 35%, var(--border-color));
}
```

- [ ] **Step 3: Typecheck + lint**

Run: `bun run typecheck && bun run lint`
Expected: clean exit.

---

### Task 11: Card primitive

**Files:**
- Create: `src/renderer/components/primitives/Card.tsx`
- Modify: `src/renderer/styles/App.css`

- [ ] **Step 1: Write Card.tsx**

Create `src/renderer/components/primitives/Card.tsx`:

```tsx
import {
  forwardRef,
  useEffect,
  useRef,
  type ButtonHTMLAttributes,
  type HTMLAttributes,
  type ReactNode,
} from 'react';
import { smoothScalar, type SmoothHandle } from '../../utils/motion';

const LIFT_SPEED = 12;
const LIFT_AMOUNT_PX = 4;

export type CardVariant = 'default' | 'internal' | 'external';

interface CardBaseProps {
  variant?: CardVariant;
  children: ReactNode;
  /** Disable hover lift (e.g., for cards that are purely informational). */
  noLift?: boolean;
}
type CardAsButton = CardBaseProps & { onClick: () => void } & Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'onClick'>;
type CardAsDiv    = CardBaseProps & { onClick?: undefined } & HTMLAttributes<HTMLDivElement>;
type CardProps = CardAsButton | CardAsDiv;

/**
 * Card primitive. Owns:
 *  - inset edge-glow per variant (replaces the broken 3px ::before stripe).
 *  - hover lift via the shared smoothing engine.
 * Page-specific content (poster, body, badges) lives inside as children.
 */
const Card = forwardRef<HTMLElement, CardProps>(function Card(props, _ref) {
  const { variant = 'default', children, noLift } = props;
  const elRef = useRef<HTMLElement | null>(null);
  const liftRef = useRef<SmoothHandle | null>(null);

  useEffect(() => {
    if (noLift) return;
    const el = elRef.current;
    if (!el) return;
    const handle = smoothScalar(0, LIFT_SPEED, (v) => {
      // Skip the inline style entirely when essentially zero.
      el.style.transform = Math.abs(v) > 0.05 ? `translateY(${v.toFixed(2)}px)` : '';
    });
    liftRef.current = handle;
    return () => { handle.release(); el.style.transform = ''; };
  }, [noLift]);

  const onEnter = () => liftRef.current?.setTarget(-LIFT_AMOUNT_PX);
  const onLeave = () => liftRef.current?.setTarget(0);

  const className = `card card--${variant}`;

  if (props.onClick) {
    const { onClick, variant: _v, children: _c, noLift: _n, ...rest } = props as CardAsButton;
    return (
      <button
        ref={(node) => { elRef.current = node; }}
        type="button"
        className={className}
        onClick={onClick}
        onMouseEnter={onEnter}
        onMouseLeave={onLeave}
        {...rest}
      >
        {children}
      </button>
    );
  }
  const { onClick: _o, variant: _v, children: _c, noLift: _n, ...rest } = props as CardAsDiv;
  return (
    <div
      ref={(node) => { elRef.current = node; }}
      className={className}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
      {...rest}
    >
      {children}
    </div>
  );
});

export default Card;
```

- [ ] **Step 2: Add CSS**

Append to `src/renderer/styles/App.css`:

```css
/* Card primitive — shared shell. Inner content layout is each page's job. */
.card {
  position: relative;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 0;
  text-align: left;
  cursor: pointer;
  /* No transition on transform — motion engine writes it directly. */
  transition: border-color var(--transition-fast), box-shadow var(--transition-normal);
  /* Button reset for the button-mode of <Card>. */
  appearance: none;
  font: inherit;
  color: inherit;
}
.card:hover {
  border-color: var(--border-hover);
  box-shadow: var(--shadow-md);
}
/* Variant: in-library (teal inset glow). Replaces .relation-card.is-internal. */
.card--internal {
  box-shadow:
    inset 0 0 0 1.5px color-mix(in srgb, var(--accent-teal) 55%, transparent),
    inset 0 0 28px color-mix(in srgb, var(--accent-teal) 18%, transparent);
}
.card--internal:hover {
  box-shadow:
    inset 0 0 0 1.5px color-mix(in srgb, var(--accent-teal) 75%, transparent),
    inset 0 0 32px color-mix(in srgb, var(--accent-teal) 28%, transparent),
    var(--shadow-md);
}
/* Variant: external (indigo inset glow). Replaces .relation-card.is-external. */
.card--external {
  box-shadow:
    inset 0 0 0 1.5px color-mix(in srgb, var(--accent-secondary) 55%, transparent),
    inset 0 0 28px color-mix(in srgb, var(--accent-secondary) 18%, transparent);
}
.card--external:hover {
  box-shadow:
    inset 0 0 0 1.5px color-mix(in srgb, var(--accent-secondary) 75%, transparent),
    inset 0 0 32px color-mix(in srgb, var(--accent-secondary) 28%, transparent),
    var(--shadow-md);
}
```

- [ ] **Step 3: Typecheck + lint**

Run: `bun run typecheck && bun run lint`
Expected: clean exit.

---

### Task 12: EpisodeRow primitive

**Files:**
- Create: `src/renderer/components/primitives/EpisodeRow.tsx`
- Modify: `src/renderer/styles/App.css`

- [ ] **Step 1: Write EpisodeRow.tsx**

Create `src/renderer/components/primitives/EpisodeRow.tsx`:

```tsx
import { useEffect, useRef, type ReactNode } from 'react';
import { smoothScalar, type SmoothHandle } from '../../utils/motion';

const LIFT_SPEED = 12;
const LIFT_AMOUNT_PX = 2;

export type EpisodeRowState = 'default' | 'next-up' | 'watched' | 'in-progress';

interface EpisodeRowProps {
  marker: ReactNode;       // play / check / icon
  code: ReactNode;         // S01E03
  title: ReactNode;
  trailing?: ReactNode;    // pill, flag, "Next up", etc.
  /** 0..1 — hovered seekbar fill. */
  progress?: number;
  /** Whether the progress bar should be visible at rest (next-up / in-progress). */
  progressVisibleAtRest?: boolean;
  state?: EpisodeRowState;
  onClick?: () => void;
  disabled?: boolean;
}

/**
 * EpisodeRow primitive. Replaces .bare-episode-row.
 *
 * Layout: [marker | code | title | trailing] above a reserved seekbar row.
 * The seekbar slot is structural (its own row in a grid), so the text's
 * vertical centering is independent of the seekbar's presence — fixing the
 * off-center bug.
 */
export default function EpisodeRow({
  marker, code, title, trailing,
  progress = 0, progressVisibleAtRest = false,
  state = 'default', onClick, disabled,
}: EpisodeRowProps) {
  const elRef = useRef<HTMLButtonElement | null>(null);
  const liftRef = useRef<SmoothHandle | null>(null);

  useEffect(() => {
    const el = elRef.current;
    if (!el || disabled) return;
    const handle = smoothScalar(0, LIFT_SPEED, (v) => {
      el.style.transform = Math.abs(v) > 0.05 ? `translateY(${v.toFixed(2)}px)` : '';
    });
    liftRef.current = handle;
    return () => { handle.release(); el.style.transform = ''; };
  }, [disabled]);

  const onEnter = () => { if (!disabled) liftRef.current?.setTarget(-LIFT_AMOUNT_PX); };
  const onLeave = () => liftRef.current?.setTarget(0);

  const className = `episode-row episode-row--${state}` + (progressVisibleAtRest ? ' episode-row--has-rest-progress' : '');
  const pct = Math.max(0, Math.min(1, progress)) * 100;

  return (
    <button
      ref={elRef}
      type="button"
      className={className}
      onClick={onClick}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
      disabled={disabled}
    >
      <span className="episode-row__marker">{marker}</span>
      <span className="episode-row__code">{code}</span>
      <span className="episode-row__title">{title}</span>
      <span className="episode-row__trailing">{trailing}</span>
      <span className="episode-row__progress" aria-hidden="true">
        <span className="episode-row__progress-fill" style={{ width: `${pct}%` }} />
      </span>
    </button>
  );
}
```

- [ ] **Step 2: Add CSS**

Append to `src/renderer/styles/App.css`:

```css
/* EpisodeRow primitive. Reserves a structural row for the hover seekbar so
   the text's vertical centering does not depend on the seekbar's presence. */
.episode-row {
  position: relative;
  display: grid;
  grid-template-columns: 22px 78px 1fr auto;
  grid-template-rows: 1fr var(--s1);
  column-gap: var(--s3);
  row-gap: 0;
  align-items: center;
  width: 100%;
  text-align: left;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: var(--s3) var(--s4);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  cursor: pointer;
  transition:
    background var(--transition-fast),
    border-color var(--transition-fast),
    color var(--transition-fast),
    opacity var(--transition-fast);
}
.episode-row:hover { background: var(--bg-tertiary); border-color: var(--border-hover); }
.episode-row[disabled] { opacity: 0.55; cursor: not-allowed; }

.episode-row__marker {
  grid-column: 1; grid-row: 1;
  display: inline-flex; align-items: center; justify-content: center;
  width: 22px; height: 22px;
  border-radius: 50%;
  color: var(--text-muted);
  background: rgba(255,255,255,0.03);
  border: 1px solid var(--border-color);
  transition: color var(--transition-fast), background var(--transition-fast), border-color var(--transition-fast);
}
.episode-row__code  { grid-column: 2; grid-row: 1; font-size: 0.72rem; font-weight: 600; letter-spacing: 0.06em; text-transform: uppercase; color: var(--text-muted); }
.episode-row__title { grid-column: 3; grid-row: 1; min-width: 0; word-break: break-word; }
.episode-row__trailing { grid-column: 4; grid-row: 1; display: inline-flex; align-items: center; gap: var(--s2); }

/* Seekbar lives in the reserved second row, full-width, never overlapping text. */
.episode-row__progress {
  grid-column: 1 / -1;
  grid-row: 2;
  height: 2px;
  background: rgba(255,255,255,0.06);
  border-radius: var(--radius-pill);
  overflow: hidden;
  opacity: 0;
  transition: opacity var(--transition-fast);
}
.episode-row:hover .episode-row__progress,
.episode-row--has-rest-progress .episode-row__progress { opacity: 0.55; }
.episode-row:hover.episode-row--has-rest-progress .episode-row__progress { opacity: 1; }
.episode-row__progress-fill {
  display: block; height: 100%; width: 0;
  background: linear-gradient(90deg, var(--accent-primary), var(--accent-secondary));
  border-radius: inherit;
  transition: width var(--transition-slow);
}

/* States — only color/border. Never reposition the text. */
.episode-row--watched { background: transparent; color: var(--text-muted); border-color: rgba(255,255,255,0.04); }
.episode-row--watched .episode-row__marker { color: var(--bg-primary); background: var(--accent-teal); border-color: var(--accent-teal); }
.episode-row:hover.episode-row--watched { background: var(--bg-secondary); color: var(--text-secondary); }

.episode-row--next-up { border-color: color-mix(in srgb, var(--accent-secondary) 30%, var(--border-color)); background: color-mix(in srgb, var(--accent-secondary) 6%, var(--bg-secondary)); }
.episode-row--next-up .episode-row__marker { color: var(--accent-secondary); border-color: color-mix(in srgb, var(--accent-secondary) 50%, var(--border-color)); background: color-mix(in srgb, var(--accent-secondary) 12%, transparent); }
```

- [ ] **Step 3: Re-export through an index**

Create `src/renderer/components/primitives/index.ts`:

```ts
export { default as Page } from './Page';
export { default as Section } from './Section';
export { Stack, Inline, type SpaceToken } from './Stack';
export { default as Pill, type PillTone } from './Pill';
export { default as Card, type CardVariant } from './Card';
export { default as EpisodeRow, type EpisodeRowState } from './EpisodeRow';
```

- [ ] **Step 4: Typecheck + lint**

Run: `bun run typecheck && bun run lint`
Expected: clean exit.

- [ ] **Step 5: PAUSE — request user commit (end of Phase 3)**

Suggested commit message:
```
feat(primitives): add Card, EpisodeRow, Pill content primitives

- Card variants: default, internal (teal glow), external (indigo glow);
  inset box-shadow replaces the broken 3px ::before stripe.
- EpisodeRow reserves a structural row for the hover seekbar so text
  centering no longer depends on the seekbar's presence.
- Pill replaces the half-dozen pill/flag/badge variants.

Unused so far — Series Detail migration is next.
```

---

## Phase 4 — Series Detail migration (worst-offender first)

### Task 13: Migrate SeriesDetailPage

**Files:**
- Modify: `src/renderer/pages/SeriesDetailPage.tsx`
- Modify: `src/renderer/styles/App.css` (delete now-unused rules)

This task is the largest single edit in the plan because Series Detail has the most affected markup. Work in sub-steps, typecheck after each.

- [ ] **Step 1: Audit current markup**

Read `src/renderer/pages/SeriesDetailPage.tsx` end-to-end. List every:
- `<div className="page">` → maps to `<Page>`.
- `<div className="bare-episode-head">`, `<div className="detail-header">`, etc. → maps to `<Section>` where the header semantically opens a content group.
- `<button className="bare-episode-row">` → maps to `<EpisodeRow>`.
- `<button className="relation-card …">` → maps to `<Card variant>`.
- `<span className="bare-episode-pill | bare-episode-flag | relation-card-pill | genre-pill">` → maps to `<Pill>`.

Keep the audit as a scratch comment at the top of the file while editing.

- [ ] **Step 2: Add primitive imports**

At the top of `SeriesDetailPage.tsx`:

```tsx
import { Page, Section, Card, EpisodeRow, Pill } from "../components/primitives";
```

- [ ] **Step 3: Swap the page shell**

Replace the outer `<div className="series-detail">…</div>` wrapping with:

```tsx
return (
  <Page>
    {/* …existing content… */}
  </Page>
);
```

Run: `bun run typecheck`. Fix any type errors before moving on.

- [ ] **Step 4: Swap section headers to <Section>**

Where the file currently has paired markup like:

```tsx
<div className="bare-episode-head">
  <h2 className="section-h2">Episodes</h2>
  <span className="section-count">{count}</span>
</div>
<div className="bare-episode-list">{rows}</div>
```

Replace with:

```tsx
<Section title="Episodes" count={count}>
  <div className="episode-list">{rows}</div>
</Section>
```

For the **first** section inside the `<Page>`, pass `first` to suppress the top gap:

```tsx
<Section first title="…">…</Section>
```

The Related section becomes a normal `<Section title="Related" count={…}>` — its s12 top margin comes from the primitive automatically. This is the line that fixes the "pixel brothers" bug.

Run: `bun run typecheck`.

- [ ] **Step 5: Swap episode rows to <EpisodeRow>**

Find the existing `<button className="bare-episode-row …">…</button>` mapping (around lines 500–532 per spec audit) and replace it with:

```tsx
<EpisodeRow
  key={f.filePath}
  marker={isWatched ? <Check size={14} strokeWidth={2.5} /> : <Play size={14} />}
  code={code}
  title={f.title}
  trailing={
    isNext   ? <Pill tone="accent">Next up</Pill> :
    isWatched ? <Pill tone="muted">Watched</Pill> :
    null
  }
  progress={fraction}
  progressVisibleAtRest={hasResume || isNext}
  state={isNext ? "next-up" : isWatched ? "watched" : "default"}
  onClick={() => navigate(`/player/${encodeURIComponent(item.id)}/${f.episodeNumber}`)}
/>
```

Delete the now-unused inline computation of `bare-episode-progress` / `bare-episode-pill` markup — the primitive owns those.

Run: `bun run typecheck`.

- [ ] **Step 6: Swap relation cards to <Card>**

The relation grid (around line 540) becomes:

```tsx
{sortedRelations.map((rel) => {
  // …existing ownedId / relTitle / handleClick computation unchanged…
  const isInternal = ownedId != null;
  return (
    <Card
      key={`${rel.type ?? "x"}-${rel.anilistId}-${rel.relationType}`}
      variant={isInternal ? "internal" : "external"}
      onClick={handleClick}
      title={isInternal ? `Open ${relTitle} in your library` : `Open ${relTitle} on AniList`}
    >
      <div className="relation-card-poster">
        {/* existing img / placeholder markup unchanged */}
      </div>
      <div className="relation-card-body">
        {/* existing label / title markup unchanged, but replace .relation-card-pill with: */}
        <Pill tone={isInternal ? "teal" : "accent"}>
          {isInternal ? "In Library" : "AniList"}
        </Pill>
        {/* …rest of body… */}
      </div>
    </Card>
  );
})}
```

Wrap the `.map()` in a `<div className="relations-grid">` if not already, since `<Section>` only provides the header and outer spacing — the grid layout itself stays.

Run: `bun run typecheck`.

- [ ] **Step 7: Swap remaining pills**

Replace any remaining `<span className="genre-pill">…</span>`, `<span className="bare-episode-pill">…</span>`, `<span className="bare-episode-flag">…</span>` with `<Pill tone="…">…</Pill>`.

Run: `bun run typecheck && bun run lint`.

- [ ] **Step 8: Delete unused CSS**

In `src/renderer/styles/App.css`, delete the following rule blocks (they are now superseded by the primitives' CSS):
- `.bare-episode-head`, `.bare-episode-list`, `.bare-episode-row` (and all its `::before` / `:hover` / `.watched` / `.next-up` variants), `.bare-episode-marker`, `.bare-episode-code`, `.bare-episode-title`, `.bare-episode-pill`, `.bare-episode-flag`, `.bare-episode-progress` (and `-fill`).
- `.relation-card`, `.relation-card::before`, `.relation-card.is-internal`/`.is-external` (and their `:hover` variants), `.relation-card-pill` (visual styles only — the poster/empty rules stay since the page still uses `.relation-card-poster`).
- `.section-h2`, `.section-count`, `.section-head`, `.section-head-movies`, `.section-sub` — but only if no other page still references them. Use `rg 'className="section-h2"' src/renderer/pages` to verify before deleting. If other pages still depend on them, leave them and re-evaluate during their migration tasks.

Run: `bun run lint && bun run typecheck`.

- [ ] **Step 9: Package + visual sign-off**

Run: `bun run package`
Open the app via the launcher. Navigate to a series with episodes and a Related grid. Verify:

1. **Episode text is optically centered** in every row, whether the seekbar is visible or not. Hover a row — the seekbar appears in its reserved slot below the text; the text does not move.
2. **Related has breathing room** — there's clearly a gap (`--s12`) between the last episode row and the Related header. Not pixel-brothers.
3. **Relation card edges look curved** — the teal/indigo inset glow follows the card's rounded corner. No broken stripe at top-left.
4. **Hover lift on cards and rows is smooth** — no jitter when entering/leaving. Cards settle to rest.
5. **Pills (Next up / Watched / In Library / AniList) all look consistent** — same shape, same vertical rhythm.

- [ ] **Step 10: PAUSE — request user commit (end of Phase 4)**

Suggested commit message:
```
feat(series): migrate Series Detail to primitives

- Replace bare-episode-row with EpisodeRow primitive — text optically
  centered, hover seekbar in its own reserved row.
- Replace relation cards with <Card variant=internal|external> — the
  broken 3px ::before stripe is gone; inset box-shadow follows the
  rounded corner.
- Replace section headers with <Section> — the Related "pixel brothers"
  bug is now structurally impossible.
- Delete now-unused .bare-episode-* and .relation-card-* CSS.
```

---

## Phase 5 — Home migration

### Task 14: Migrate HomePage

**Files:**
- Modify: `src/renderer/pages/HomePage.tsx`
- Modify: `src/renderer/components/ShowCard.tsx` (delegate shell to `<Card>`)
- Modify: `src/renderer/styles/App.css`

- [ ] **Step 1: Audit HomePage**

List every `<div className="page">`, `.page-head`, `.section-head`, `.show-card …`, badge/pill markup.

- [ ] **Step 2: Swap page shell to <Page>**

```tsx
import { Page, Section, Pill } from "../components/primitives";

return (
  <Page head={<HomeHead /* existing title + library-search */ />}>
    {/* sections */}
  </Page>
);
```

Run: `bun run typecheck`.

- [ ] **Step 3: Swap each section to <Section>**

For "Airing", "Movies", and any other top-level section currently using `.section-head` / `.section-head-movies`, replace with `<Section first?={isFirst} title="…" count={n} action={…}>…</Section>`.

Run: `bun run typecheck`.

- [ ] **Step 4: Migrate ShowCard.tsx to wrap its shell in <Card>**

In `src/renderer/components/ShowCard.tsx`, replace the top-level `<button className="show-card …">` with:

```tsx
import Card from "./primitives/Card";

return (
  <Card variant="default" onClick={onClick} aria-label={…}>
    {/* existing poster + info markup — drop the outer .show-card class */}
  </Card>
);
```

Keep `.show-card-poster-wrap`, `.show-card-info`, etc. since those are inner layout, not the shell. Drop the duplicated `.show-card` hover/transition rules from App.css.

Run: `bun run typecheck && bun run lint`.

- [ ] **Step 5: Swap badges/pills**

`.show-card-badge` becomes a `<Pill tone="muted">` (or `tone="teal"` for finished series, per the existing color logic in `e43ed91`). The "watched/total" math stays; only the shell changes.

- [ ] **Step 6: Delete unused CSS**

Remove `.show-card` shell rules (hover, transform, transition) from App.css; keep `.show-card-poster-wrap`, `.show-card-info`, `.show-card-title`, etc. as inner content layout. Remove `.show-card-badge` shell — the inner badge text styling can stay if other places use it; verify with `rg 'show-card-badge' src/`.

Run: `bun run typecheck && bun run lint`.

- [ ] **Step 7: Package + visual sign-off**

Run: `bun run package`. Open Home. Verify:
- Airing carousel, Movies section, Library grid all have consistent vertical rhythm.
- Show cards lift smoothly on hover, no jitter.
- Library search bar in the page head has the s6 gap below it.
- No regressions vs. before.

- [ ] **Step 8: PAUSE — request user commit (end of Phase 5)**

Suggested commit message:
```
feat(home): migrate HomePage and ShowCard to primitives
```

---

## Phase 6 — Feed migration

### Task 15: Migrate FeedPage

**Files:**
- Modify: `src/renderer/pages/FeedPage.tsx`
- Modify: `src/renderer/styles/App.css`

Feed reuses `ShowCard`, which is already migrated in Phase 5. This phase only migrates FeedPage's own shell and section headers.

- [ ] **Step 1: Add primitive imports**

At the top of `src/renderer/pages/FeedPage.tsx`:

```tsx
import { Page, Section, Pill } from "../components/primitives";
```

- [ ] **Step 2: Swap the page shell**

Replace the outer `<div className="page">` with `<Page>`. If FeedPage has a `.page-head` block (title + filter / mode toggle), pass it via the `head` slot:

```tsx
return (
  <Page head={
    <Inline gap="s4" justify="space-between" align="baseline">
      <h1 className="page-title">Feed</h1>
      {/* existing right-side controls */}
    </Inline>
  }>
    {/* sections */}
  </Page>
);
```

Add `Inline` to the import line above.

- [ ] **Step 3: Swap each carousel section's head to <Section>**

For each existing block that pairs a `.feed-section` / `.section-head` with a `.feed-carousel`, replace with:

```tsx
<Section
  title="Recently Aired"
  count={items.length}
  action={<Link to="/feed/aired">See all</Link>}
  first={isFirst}
>
  <div className="feed-carousel">{/* existing carousel content */}</div>
</Section>
```

Existing carousel inner markup (`.feed-carousel`, `.feed-carousel-item`) stays — only the outer section wrapper and head change.

Run: `bun run typecheck && bun run lint`.

- [ ] **Step 4: Delete unused CSS**

For each of these selectors, run `rg 'className="<class>"' src/renderer` first. Delete only the ones with zero matches:
- `.feed-section`
- `.section-link`
- `.section-head-actions`

Keep `.feed-carousel` and `.feed-carousel-item` — they're inner layout still used by the page.

- [ ] **Step 5: Package + visual sign-off**

Run: `bun run package`. Open Feed. Verify:
- Section gaps are s12 between sections, consistent with Home/Series.
- "See all" link is in the `<Section action>` slot, right-aligned in the header.
- Cards lift smoothly (inherited from `ShowCard` migration).

- [ ] **Step 6: PAUSE — request user commit**

Suggested commit message: `feat(feed): migrate FeedPage to primitives`

---

## Phase 7 — Subscriptions migration

### Task 16: Migrate SubscriptionsPage

**Files:**
- Modify: `src/renderer/pages/SubscriptionsPage.tsx`
- Modify: `src/renderer/styles/App.css`

- [ ] **Step 1: Add primitive imports**

```tsx
import { Page, Section, Card, Pill, Stack, Inline } from "../components/primitives";
```

- [ ] **Step 2: Swap the page shell**

```tsx
return (
  <Page head={
    <Inline gap="s4" justify="space-between" align="baseline">
      <h1 className="page-title">Subscriptions</h1>
      {/* existing right-side controls (filter, add button, etc.) */}
    </Inline>
  }>
    {/* sections */}
  </Page>
);
```

- [ ] **Step 3: Swap each group's head to <Section>**

For every subscription group (e.g., "Active", "Completed", or however the page organizes them), wrap in `<Section title=… count=… first={isFirst}>`.

- [ ] **Step 4: Swap subscription cards to <Card>**

If subscription items render as cards with the same hover behavior as relation/show cards, wrap each in `<Card variant="default">`. Status indicators (e.g., "Updates daily", "Paused") → `<Pill tone="…">`.

If subscription items render as rows (table-like), use `<Stack gap="s2">` to wrap the list and either re-use `<EpisodeRow>` (if the shape fits) or keep the current row markup with token-driven padding (`var(--s3) var(--s4)`).

Run: `bun run typecheck && bun run lint`.

- [ ] **Step 5: Delete unused CSS**

Whatever subscription-only shell/card rules are dead now. Verify with `rg` before each deletion.

- [ ] **Step 6: Package + visual sign-off**

Run: `bun run package`. Open Subscriptions. Verify rhythm matches Home / Feed / Series.

- [ ] **Step 7: PAUSE — request user commit**

Suggested commit message: `feat(subs): migrate SubscriptionsPage to primitives`

---

## Phase 8 — Metadata migration

### Task 17: Migrate MetadataTab

**Files:**
- Modify: `src/renderer/pages/MetadataTab.tsx`
- Modify: `src/renderer/styles/App.css`

The horizontal-shift bug is already fixed in Phase 1 by `scrollbar-gutter: stable`. This task is the spacing pass.

- [ ] **Step 1: Add primitive imports**

```tsx
import { Page, Section, Stack, Inline, Pill } from "../components/primitives";
```

- [ ] **Step 2: Swap the page shell**

```tsx
return (
  <Page head={
    <Inline gap="s4" justify="space-between" align="baseline">
      <h1 className="page-title">Metadata</h1>
      {/* existing controls / filters */}
    </Inline>
  }>
    {/* content */}
  </Page>
);
```

- [ ] **Step 3: Migrate the candidate list**

The metadata candidate rows are not episode rows — they show match scores, source names, and confirm/skip controls. Don't force them into `<EpisodeRow>`. Instead:

```tsx
<Stack gap="s2">
  {candidates.map((c) => (
    <div key={c.id} className="metadata-row">
      {/* existing row layout — but its outer container uses var(--s3) var(--s4) padding and var(--radius-lg) */}
    </div>
  ))}
</Stack>
```

Update `.metadata-row` (or whatever the actual class is) in App.css to use spacing tokens instead of magic numbers. The grid layout itself stays.

- [ ] **Step 4: Swap badges**

Match-score badges, source labels → `<Pill tone="…">` (accent for high score, muted for low, teal for "in library", rose for "skip", per the existing visual logic).

Run: `bun run typecheck && bun run lint`.

- [ ] **Step 5: Delete unused CSS**

Metadata-only shell rules that are now dead. Verify with `rg` before deletion.

- [ ] **Step 6: Package + visual sign-off**

Run: `bun run package`. Open Metadata. Verify:
- Page no longer shifts horizontally as the list shortens (regression check on the Phase 1 fix — should still hold).
- Spacing rhythm matches Series / Home.
- Match-score badges, source labels are consistent with other Pills in the app.

- [ ] **Step 7: PAUSE — request user commit**

Suggested commit message: `feat(metadata): migrate MetadataTab to primitives`

---

## Phase 9 — Settings migration

### Task 18: Migrate SettingsTab (spacing audit)

**Files:**
- Modify: `src/renderer/components/SettingsTab.tsx`
- Modify: `src/renderer/styles/App.css`

Per the brainstorming session, the exact scope of Settings issues is decided during this task (user deferred to Claude's judgment). The brainstorming established that the spacing system fixes most of them.

- [ ] **Step 1: Audit SettingsTab**

Read `src/renderer/components/SettingsTab.tsx` and `src/renderer/components/TrackersSection.tsx` end-to-end. List visible spacing issues: section gaps, internal padding, form field rhythm, button placement.

- [ ] **Step 2: Add primitive imports**

```tsx
import { Page, Section, Stack, Inline, Pill } from "./primitives";
```

(Use relative path `./primitives` from `src/renderer/components/SettingsTab.tsx`.)

- [ ] **Step 3: Swap the page shell**

```tsx
return (
  <Page head={<h1 className="page-title">Settings</h1>}>
    {/* sections */}
  </Page>
);
```

- [ ] **Step 4: Swap each settings group to <Section>**

For every grouping (e.g., "Library", "Trackers", "Playback", "Advanced"), wrap in `<Section title="…" first={isFirst}>`. Inside, use `<Stack gap="s4">` for the vertical run of controls. For label-and-control rows, use `<Inline gap="s3" align="center" justify="space-between">`.

```tsx
<Section title="Library" first>
  <Stack gap="s4">
    <Inline gap="s3" align="center" justify="space-between">
      <label htmlFor="lib-root">Watch folder</label>
      <input id="lib-root" type="text" value={path} onChange={onPath} />
    </Inline>
    {/* more rows */}
  </Stack>
</Section>
```

Buttons keep their existing classes (`btn`, `btn-primary`, etc.) — they're not part of this overhaul.

- [ ] **Step 5: Migrate TrackersSection**

If `TrackersSection.tsx` is rendered inside SettingsTab, migrate its outer shell to `<Section title="Trackers">…</Section>` and its rows to `<Stack gap="s3">`. Internal tracker-status badges → `<Pill tone="teal | muted | rose">`.

Run: `bun run typecheck && bun run lint`.

- [ ] **Step 6: Delete unused CSS**

Settings-only rules that are now dead. Verify with `rg` before each deletion.

- [ ] **Step 7: Out-of-scope discoveries**

If during the audit you find spacing-adjacent issues that don't fit the primitive system (e.g., a settings control grouped under the wrong heading, a button that should be three buttons), do not expand scope. Add a one-line code comment in the source naming the issue clearly and stop. Surface the discovery in the user commit message ("found: X — out of scope, deferring").

- [ ] **Step 8: Package + visual sign-off**

Run: `bun run package`. Open Settings. Verify rhythm matches Series / Home / Metadata. Confirm tracker rows and Trackers section header look consistent with other sections.

- [ ] **Step 9: PAUSE — request user commit**

Suggested commit message: `feat(settings): migrate SettingsTab to primitives`

---

## Phase 10 — VideoPlayer chrome migration

### Task 19: Migrate VideoPlayer's chrome only

**Files:**
- Modify: `src/renderer/pages/VideoPlayer.tsx` (chrome regions only)
- Modify: `src/renderer/styles/App.css`

**Hard rule from project memory:** Do not touch JASSUB/mpv-related internals inside VideoPlayer.tsx. Those are load-bearing workarounds.

- [ ] **Step 1: Identify chrome regions**

The chrome = controls overlay, episode metadata strip, exit/back buttons, anything not the video surface or the subtitle canvas. Add a scratch comment at the top of each chrome region noting the migration intent so it's obvious during diff review.

JASSUB / mpv internals (subtitle canvas, video element, transcode wiring) are off-limits.

- [ ] **Step 2: Add primitive imports**

```tsx
import { Stack, Inline, Pill } from "../components/primitives";
```

- [ ] **Step 3: Migrate chrome markup**

For each chrome region:
- Vertical groups of controls → `<Stack gap="s3">`.
- Horizontal toolbars (play/pause/seek/volume) → `<Inline gap="s2" align="center">`.
- Status badges, chapter labels, "Stalled" / "Verifying" indicators → `<Pill tone="…">`.

Example — the bottom-bar layout becomes:

```tsx
<div className="player-controls">
  <Inline gap="s2" align="center" justify="space-between">
    {/* left cluster: prev/play/next */}
    <Inline gap="s2" align="center">{/* existing buttons */}</Inline>
    {/* center: time + seekbar */}
    <div className="player-seekbar">{/* unchanged */}</div>
    {/* right cluster: volume + subs + fullscreen */}
    <Inline gap="s2" align="center">{/* existing buttons */}</Inline>
  </Inline>
</div>
```

Do NOT add `<AmbientCursor>` to the player route (it's already guarded off by `!isPlayerRoute` in `App.tsx`).

Run: `bun run typecheck && bun run lint`.

- [ ] **Step 3: Package + visual sign-off**

Open a video. Verify:
- Subtitle rendering still works (JASSUB integration unaffected).
- Chrome looks consistent with the rest of the app.
- No regressions in playback controls, seek behavior, or chapter navigation.

- [ ] **Step 4: PAUSE — request user commit**

Suggested commit message: `feat(player): migrate VideoPlayer chrome to primitives (JASSUB internals untouched)`

---

## Phase 11 — Final cleanup

### Task 20: Sweep dead CSS

**Files:**
- Modify: `src/renderer/styles/App.css`

- [ ] **Step 1: Identify dead rules**

For each remaining class selector in App.css, run `rg 'className="<class>"' src/renderer` (or use the class name as a substring). If nothing matches, the rule is dead.

Use this list as your audit checklist:
- `.bare-episode-*` (should all be gone after Phase 4).
- `.relation-card` shell rules (Phase 4).
- `.show-card` shell rules (Phase 5).
- `.section-h2`, `.section-count`, `.section-head*`, `.page-head`, `.page-title`, `.page-sub` (replaced by Section's own classes).
- Any `.feed-*` or `.subs-*` page-specific layout (Phases 6, 7).
- Any `.metadata-*` (Phase 8).
- Any `.settings-*` (Phase 9).
- Any `.player-*` chrome rules (Phase 10).

- [ ] **Step 2: Delete dead rules**

Delete in batches by file region (carousel block, then episode block, then relation block, …). After each batch, run `bun run package` and click through every page to confirm nothing regressed visually. The diff for this PR will be large — that's the point.

- [ ] **Step 3: Final typecheck + lint**

Run: `bun run typecheck && bun run lint`
Expected: clean exit.

- [ ] **Step 4: Final visual pass**

Open every page (Library/Home, Feed, Subscriptions, Metadata, Settings, a Series Detail, a Video). Cursor halo follows everywhere except the player. Hovering any card / row produces a smooth lift. Spacing rhythm is uniform across pages.

- [ ] **Step 5: PAUSE — request user commit**

Suggested commit message: `chore(css): delete dead App.css rules superseded by primitives`

---

## Open follow-ups (not in this plan)

- Page-to-page route transitions (continuity gestalt). Discussed in brainstorming, marked nice-to-have. Would build on `<AmbientCursor>` + view-transitions API.
- Settings — any out-of-scope discoveries surfaced in Task 18 step 4.
