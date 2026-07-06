/**
 * Liquid glass: refraction for floating surfaces, after
 * kube.io/blog/liquid-glass-css-svg, adapted to a hard Electron constraint
 * discovered empirically: feImage NEVER loads inside a backdrop-filter
 * context in Chromium (the blog's data-URL displacement map silently
 * produces an empty result, leaving blur-only glass). feTurbulence-driven
 * displacement DOES work, which proves feDisplacementMap itself is fine in
 * backdrop filters; only image loading is dead.
 *
 * So the lens map is built PROCEDURALLY inside the filter graph: feFlood
 * strips cropped to each edge, Gaussian-blurred into smooth ramps (an erf
 * profile, close cousin of the blog's squircle lens falloff), recombined
 * into R (x offset) / G (y offset) displacement channels around the 128
 * neutral. Corners get diagonal vectors for free where the ramps overlap.
 *
 * LIGHTING is physical, not painted: the same squircle profile and rounded
 * rect SDF give the true 3D surface normal at every bezel pixel, which is
 * shaded with Blinn-Phong against ONE global light vector and baked into a
 * per-surface overlay map (a plain canvas PNG on a ::before, NOT part of
 * the backdrop filter, so the feImage limitation doesn't apply). Glints
 * appear only where the bezel actually faces the light; the flat interior
 * shades itself to nothing.
 *
 * Usage: put `data-liquid-glass` on any element whose backdrop should
 * refract. Optional attributes:
 *   data-lg-bezel    refracting rim width in px (default: min(radius, 18))
 *   data-lg-strength refraction strength multiplier (default 1)
 *   data-lg-blur     blur inside the chain in px (default 2.5)
 *   data-lg-light    specular intensity multiplier (default 1)
 * The manager (initLiquidGlass) watches the DOM, builds a filter + light
 * map per element, and rebuilds on resize. Elements keep their CSS
 * backdrop-filter as a fallback until the manager attaches the inline one.
 */

const SVG_NS = 'http://www.w3.org/2000/svg';
let defsHost: SVGSVGElement | null = null;
let nextId = 0;

function ensureHost(): SVGSVGElement {
  if (defsHost && defsHost.isConnected) return defsHost;
  const svg = document.createElementNS(SVG_NS, 'svg');
  svg.setAttribute('width', '0');
  svg.setAttribute('height', '0');
  svg.style.position = 'fixed';
  svg.style.inset = '0';
  svg.style.pointerEvents = 'none';
  svg.setAttribute('aria-hidden', 'true');
  document.body.appendChild(svg);
  defsHost = svg;
  return svg;
}

function el(name: string, attrs: Record<string, string | number>): SVGElement {
  const node = document.createElementNS(SVG_NS, name);
  for (const [k, v] of Object.entries(attrs)) node.setAttribute(k, String(v));
  return node;
}

// ─── Physical specular ───────────────────────────────────────────────────────

/** The app's ONE light source: up-left and GRAZING (low z), so the mirror
 *  reflection toward the viewer peaks on steep bezel, i.e. near the rim. */
const LIGHT = (() => {
  const x = -0.6, y = -0.74, z = 0.3;
  const l = Math.hypot(x, y, z);
  return { x: x / l, y: y / l, z: z / l };
})();
const SPEC_EXP = 22;          // primary glint lobe width
const COUNTER_EXP = 12;       // far-rim exit glint, broader and dimmer
const COUNTER_GAIN = 0.55;
const LIGHT_GAIN = 2.6;       // overall specular energy

/** Signed distance of point p (relative to center) to a rounded-rect edge.
 *  Negative inside. Standard sdRoundedBox. */
function sdRoundedBox(px: number, py: number, hw: number, hh: number, r: number): number {
  const qx = Math.abs(px) - hw + r;
  const qy = Math.abs(py) - hh + r;
  const ox = Math.max(qx, 0);
  const oy = Math.max(qy, 0);
  return Math.hypot(ox, oy) + Math.min(Math.max(qx, qy), 0) - r;
}

/** Convex squircle height h(x) = (1 - (1-x)^4)^(1/4) and its slope. The slope
 *  diverges at the outer edge (x -> 0): the surface stands vertical there. */
function squircleSlope(x: number): number {
  const u = 1 - Math.min(1, Math.max(1e-4, x));
  const base = 1 - u * u * u * u;
  return (u * u * u) / Math.pow(Math.max(base, 1e-6), 0.75);
}

/**
 * Bake the Blinn-Phong lit rim for a w x h rounded-rect pane. Per bezel
 * pixel: outward direction from the SDF gradient, tilt from the squircle
 * slope, 3D normal N = (g * sin t, cos t), then
 *   primary  = max(0, N . H)^SPEC_EXP
 *   counter  = max(0, N' . H)^COUNTER_EXP  with N' the mirrored (exit) normal
 * weighted by a Fresnel-ish rim factor so grazing bezel catches more light.
 * The interior needs no special casing: slope -> 0 makes N -> (0,0,1) and
 * the specular self-extinguishes.
 */
function bakeSpecular(w: number, h: number, radius: number, bezel: number, gain: number): string {
  const width = Math.max(2, Math.round(w));
  const height = Math.max(2, Math.round(h));
  const r = Math.min(radius, width / 2, height / 2);
  const bz = Math.max(2, Math.min(bezel, width / 2, height / 2));
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext('2d')!;
  const img = ctx.createImageData(width, height);
  const data = img.data;
  const cx = width / 2;
  const cy = height / 2;
  const hw = width / 2;
  const hh = height / 2;
  const eps = 0.75;

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const px = x + 0.5 - cx;
      const py = y + 0.5 - cy;
      const sd = sdRoundedBox(px, py, hw, hh, r);
      const inside = -sd;
      const o = (y * width + x) * 4;
      let alpha = 0;
      if (inside >= -1 && inside <= bz) {
        const t = Math.min(1, Math.max(0, inside / bz));
        const gx =
          sdRoundedBox(px + eps, py, hw, hh, r) - sdRoundedBox(px - eps, py, hw, hh, r);
        const gy =
          sdRoundedBox(px, py + eps, hw, hh, r) - sdRoundedBox(px, py - eps, hw, hh, r);
        const gl = Math.hypot(gx, gy) || 1;
        const nx2 = gx / gl;
        const ny2 = gy / gl;
        const slope = squircleSlope(t);
        const denom = Math.sqrt(1 + slope * slope);
        const sinT = slope / denom;
        const cosT = 1 / denom;
        const nX = nx2 * sinT;
        const nY = ny2 * sinT;
        const nZ = cosT;
        // Mirror (Phong) response toward the viewer V = (0,0,1):
        // spec = max(0, R.z)^n with R = 2(N.L)N - L. Unlike the Blinn half
        // vector, this peaks on STEEP bezel with a grazing light, so the
        // glint hugs the rim instead of floating mid-bezel.
        const nDotL = nX * LIGHT.x + nY * LIGHT.y + nZ * LIGHT.z;
        const rz = 2 * nDotL * nZ - LIGHT.z;
        const primary = nDotL > 0 ? Math.pow(Math.max(0, rz), SPEC_EXP) : 0;
        // Exit glint on the far rim: same response with the mirrored normal
        // (light leaving the pane catches the opposite inner bezel).
        const mDotL = -nX * LIGHT.x + -nY * LIGHT.y + nZ * LIGHT.z;
        const mrz = 2 * mDotL * nZ - LIGHT.z;
        const counter = mDotL > 0 ? Math.pow(Math.max(0, mrz), COUNTER_EXP) * COUNTER_GAIN : 0;
        // Grazing bezel reflects more (Schlick-flavoured rim term).
        const rim = Math.pow(1 - cosT, 1.35);
        // Tight anti-alias only: light must begin AT the silhouette.
        const edgeAA = Math.min(1, Math.max(0, (inside + 1) / 1.1));
        alpha = Math.min(1, (primary + counter) * (0.2 + 0.8 * rim) * gain * LIGHT_GAIN) * edgeAA;
      }
      data[o] = 255;
      data[o + 1] = 255;
      data[o + 2] = 255;
      data[o + 3] = Math.round(alpha * 255);
    }
  }
  ctx.putImageData(img, 0, 0);
  return canvas.toDataURL('image/png');
}

/**
 * Build the procedural lens chain. Band = the flooded edge strip; blurring
 * it by sigma turns the hard step into a smooth displacement ramp reaching
 * roughly band + sigma into the surface.
 */
function renderFilterContents(
  f: SVGFilterElement,
  w: number,
  h: number,
  bezel: number,
  strength: number,
  blur: number,
): void {
  while (f.firstChild) f.removeChild(f.firstChild);
  f.setAttribute('x', '0');
  f.setAttribute('y', '0');
  f.setAttribute('width', String(w));
  f.setAttribute('height', String(h));

  // Short surfaces (the player island) must keep a calm center: cap the
  // refracting band to a third of the smaller dimension.
  const band = Math.max(6, Math.min(bezel * 2.2, w / 3, h / 3));
  const sigma = band * 0.6;
  const scale = Math.max(8, 42 * strength * Math.min(1, band / 36));

  // X displacement map: black strip left, white strip right, blurred
  // horizontally. R channel only; G/B forced to neutral 0.5 via offsets.
  f.appendChild(el('feFlood', { 'flood-color': 'rgb(128,128,128)', result: 'xbase' }));
  f.appendChild(el('feFlood', { 'flood-color': 'black', x: 0, y: 0, width: band, height: h, result: 'xl' }));
  f.appendChild(el('feFlood', { 'flood-color': 'white', x: w - band, y: 0, width: band, height: h, result: 'xr' }));
  const xm = el('feMerge', { result: 'xm' });
  xm.appendChild(el('feMergeNode', { in: 'xbase' }));
  xm.appendChild(el('feMergeNode', { in: 'xl' }));
  xm.appendChild(el('feMergeNode', { in: 'xr' }));
  f.appendChild(xm);
  f.appendChild(el('feGaussianBlur', { in: 'xm', stdDeviation: `${sigma} 0`, result: 'xmap' }));
  f.appendChild(el('feColorMatrix', {
    in: 'xmap',
    values: '1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 0 1',
    result: 'xonly',
  }));

  // Y displacement map: strips top/bottom, blurred vertically. G channel.
  f.appendChild(el('feFlood', { 'flood-color': 'rgb(128,128,128)', result: 'ybase' }));
  f.appendChild(el('feFlood', { 'flood-color': 'black', x: 0, y: 0, width: w, height: band, result: 'yt' }));
  f.appendChild(el('feFlood', { 'flood-color': 'white', x: 0, y: h - band, width: w, height: band, result: 'yb' }));
  const ym = el('feMerge', { result: 'ym' });
  ym.appendChild(el('feMergeNode', { in: 'ybase' }));
  ym.appendChild(el('feMergeNode', { in: 'yt' }));
  ym.appendChild(el('feMergeNode', { in: 'yb' }));
  f.appendChild(ym);
  f.appendChild(el('feGaussianBlur', { in: 'ym', stdDeviation: `0 ${sigma}`, result: 'ymapb' }));
  f.appendChild(el('feColorMatrix', {
    in: 'ymapb',
    values: '0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 0 0',
    result: 'yonly',
  }));

  // Combine: R from the x map, G from the y map (arithmetic add).
  f.appendChild(el('feComposite', {
    in: 'xonly', in2: 'yonly', operator: 'arithmetic', k1: 0, k2: 1, k3: 1, k4: 0, result: 'map',
  }));

  f.appendChild(el('feDisplacementMap', {
    in: 'SourceGraphic', in2: 'map', scale, xChannelSelector: 'R', yChannelSelector: 'G', result: 'disp',
  }));
  f.appendChild(el('feGaussianBlur', { in: 'disp', stdDeviation: blur, result: 'soft' }));
  f.appendChild(el('feColorMatrix', { in: 'soft', type: 'saturate', values: '1.18' }));
}

interface Managed {
  el: HTMLElement;
  filter: SVGFilterElement;
  /** Per-element stylesheet carrying the baked light map on ::before. */
  lightStyle: HTMLStyleElement;
  id: string;
  ro: ResizeObserver;
  timer: ReturnType<typeof setTimeout> | null;
  lastKey: string;
}

const managed = new Map<HTMLElement, Managed>();

function refresh(m: Managed): void {
  const rect = m.el.getBoundingClientRect();
  const w = Math.round(rect.width);
  const h = Math.round(rect.height);
  if (w < 8 || h < 8) return;
  const styles = getComputedStyle(m.el);
  const radius = parseFloat(styles.borderTopLeftRadius) || 0;
  const bezel = parseFloat(m.el.dataset.lgBezel ?? '') || Math.max(8, Math.min(radius, 18));
  const strength = parseFloat(m.el.dataset.lgStrength ?? '') || 1;
  const blur = parseFloat(m.el.dataset.lgBlur ?? '') || 2.5;
  const light = parseFloat(m.el.dataset.lgLight ?? '') || 1;
  const key = [w, h, radius, bezel, strength, blur, light].join(':');
  if (key === m.lastKey) return;
  m.lastKey = key;
  renderFilterContents(m.filter, w, h, bezel, strength, blur);
  m.el.style.backdropFilter = `url(#${m.id})`;
  // The lit rim rides a ::before selected by data-lg-id (static scaffold in
  // base.css); only the baked map itself lives in this per-element rule.
  const specular = bakeSpecular(w, h, radius, bezel, light);
  m.lightStyle.textContent =
    `[data-lg-id="${m.id}"]::before { background-image: url(${specular}); }`;
}

function adopt(elm: HTMLElement): void {
  if (managed.has(elm)) return;
  const id = `liquid-glass-${nextId++}`;
  const filter = document.createElementNS(SVG_NS, 'filter') as SVGFilterElement;
  filter.setAttribute('id', id);
  filter.setAttribute('color-interpolation-filters', 'sRGB');
  filter.setAttribute('filterUnits', 'userSpaceOnUse');
  ensureHost().appendChild(filter);
  const lightStyle = document.createElement('style');
  document.head.appendChild(lightStyle);
  elm.setAttribute('data-lg-id', id);
  const m: Managed = { el: elm, filter, lightStyle, id, ro: null as unknown as ResizeObserver, timer: null, lastKey: '' };
  m.ro = new ResizeObserver(() => {
    if (m.timer) clearTimeout(m.timer);
    // Debounce: rebuilding the chain per resize tick would churn; once per
    // settle is plenty (the CSS blur fallback covers the gap).
    m.timer = setTimeout(() => refresh(m), 120);
  });
  m.ro.observe(elm);
  managed.set(elm, m);
  refresh(m);
}

function release(elm: HTMLElement): void {
  const m = managed.get(elm);
  if (!m) return;
  m.ro.disconnect();
  if (m.timer) clearTimeout(m.timer);
  m.filter.remove();
  m.lightStyle.remove();
  elm.removeAttribute('data-lg-id');
  managed.delete(elm);
}

/** Watch the document for [data-liquid-glass] surfaces. Call once. */
export function initLiquidGlass(): void {
  const scan = (root: ParentNode) => {
    if (root instanceof HTMLElement && root.hasAttribute('data-liquid-glass')) adopt(root);
    root.querySelectorAll?.('[data-liquid-glass]').forEach((n) => adopt(n as HTMLElement));
  };
  const mo = new MutationObserver((entries) => {
    for (const e of entries) {
      e.addedNodes.forEach((n) => { if (n instanceof HTMLElement) scan(n); });
      e.removedNodes.forEach((n) => {
        if (!(n instanceof HTMLElement)) return;
        if (managed.has(n)) release(n);
        n.querySelectorAll?.('[data-liquid-glass]').forEach((c) => release(c as HTMLElement));
      });
    }
  });
  mo.observe(document.body, { childList: true, subtree: true });
  scan(document.body);
}
