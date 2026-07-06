/**
 * Liquid glass: physically-derived refraction for floating surfaces.
 *
 * Technique after kube.io/blog/liquid-glass-css-svg: a rounded-rectangle
 * "lens" with a convex-squircle bezel profile refracts the backdrop through
 * Snell's law. The per-pixel refraction offsets are baked into an SVG
 * displacement map (R = x offset, G = y offset, 128 = neutral) and applied
 * with backdrop-filter: url(#...), which Chromium (Electron) supports.
 *
 * Usage: put `data-liquid-glass` on any element whose backdrop should
 * refract. Optional attributes:
 *   data-lg-bezel    bezel width in px (default: min(radius, 18))
 *   data-lg-strength refraction strength multiplier (default 1)
 *   data-lg-blur     blur inside the chain in px (default 3)
 * The manager (initLiquidGlass) watches the DOM, sizes a map per element,
 * and re-bakes on resize. Elements keep their CSS backdrop-filter as a
 * fallback until the manager assigns the inline filter.
 */

const REFRACTIVE_INDEX = 1.5; // crown glass
const SAMPLES = 127;          // one radius of magnitude samples (8-bit map)
const NEUTRAL = 128;

/** Convex squircle height profile: y = (1 - (1-x)^4)^(1/4), x in [0,1]. */
function surfaceHeight(x: number): number {
  const t = 1 - x;
  return Math.pow(1 - t * t * t * t, 0.25);
}

/**
 * Precompute refraction displacement magnitudes (in px) across the bezel.
 * x = 0 at the outer edge, 1 at the inner end of the bezel. Incident rays
 * are vertical; the surface normal tilts by atan(slope); Snell gives the
 * transmitted angle; the ray then travels roughly the glass thickness and
 * lands offset by tan(bend) * depth.
 */
function refractionProfile(bezel: number, strength: number): { magnitudes: Float32Array; maxPx: number } {
  const magnitudes = new Float32Array(SAMPLES);
  const delta = 0.001;
  let maxPx = 0;
  for (let i = 0; i < SAMPLES; i++) {
    const x = i / (SAMPLES - 1);
    const slope =
      (surfaceHeight(Math.min(1, x + delta)) - surfaceHeight(Math.max(0, x - delta))) /
      (Math.min(1, x + delta) - Math.max(0, x - delta));
    // Steep edges have huge slopes; the normal angle stays bounded.
    const theta1 = Math.atan(slope);
    const theta2 = Math.asin(Math.min(1, Math.sin(theta1) / REFRACTIVE_INDEX));
    const bend = theta1 - theta2;
    // Depth the ray travels inside the lens before hitting the backdrop:
    // proportional to the remaining glass thickness under this point.
    const depth = surfaceHeight(x) * bezel;
    const px = Math.tan(bend) * depth * strength;
    magnitudes[i] = px;
    if (px > maxPx) maxPx = px;
  }
  return { magnitudes, maxPx };
}

/** Signed distance of point p (relative to center) to a rounded-rect edge.
 *  Negative inside. Standard sdRoundedBox. */
function sdRoundedBox(px: number, py: number, hw: number, hh: number, r: number): number {
  const qx = Math.abs(px) - hw + r;
  const qy = Math.abs(py) - hh + r;
  const ox = Math.max(qx, 0);
  const oy = Math.max(qy, 0);
  return Math.hypot(ox, oy) + Math.min(Math.max(qx, qy), 0) - r;
}

export interface LiquidMaps {
  displacement: string;   // data URL
  specular: string;       // data URL
  scale: number;          // feDisplacementMap scale attribute
}

/**
 * Bake the displacement + specular maps for a w x h element with the given
 * corner radius and bezel. Direction of displacement is the outward SDF
 * gradient; magnitude follows the refraction profile within the bezel and
 * is zero (neutral 128) in the flat interior.
 */
export function bakeMaps(w: number, h: number, radius: number, bezel: number, strength: number): LiquidMaps {
  const width = Math.max(2, Math.round(w));
  const height = Math.max(2, Math.round(h));
  const r = Math.min(radius, width / 2, height / 2);
  const bz = Math.max(2, Math.min(bezel, width / 2, height / 2));
  const { magnitudes, maxPx } = refractionProfile(bz, strength);
  const scale = Math.max(1, 2 * maxPx); // feDisplacementMap: offset = scale * (C - 0.5)

  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext('2d')!;
  const img = ctx.createImageData(width, height);
  const data = img.data;

  const spec = document.createElement('canvas');
  spec.width = width;
  spec.height = height;
  const sctx = spec.getContext('2d')!;
  const simg = sctx.createImageData(width, height);
  const sdata = simg.data;

  const cx = width / 2;
  const cy = height / 2;
  const hw = width / 2;
  const hh = height / 2;
  // Light from the top-left for the specular rim.
  const lx = -0.45;
  const ly = -0.89;
  const eps = 0.75;

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const px = x + 0.5 - cx;
      const py = y + 0.5 - cy;
      const sd = sdRoundedBox(px, py, hw, hh, r);
      const inside = -sd; // distance from edge, positive inside
      const o = (y * width + x) * 4;
      let rr = NEUTRAL;
      let gg = NEUTRAL;
      let sAlpha = 0;
      if (inside >= 0 && inside < bz) {
        // Outward direction = SDF gradient (numerical, robust at corners).
        const gx =
          sdRoundedBox(px + eps, py, hw, hh, r) - sdRoundedBox(px - eps, py, hw, hh, r);
        const gy =
          sdRoundedBox(px, py + eps, hw, hh, r) - sdRoundedBox(px, py - eps, hw, hh, r);
        const gl = Math.hypot(gx, gy) || 1;
        const nx = gx / gl;
        const ny = gy / gl;
        const t = inside / bz; // 0 edge .. 1 interior
        const idx = Math.min(SAMPLES - 1, Math.round(t * (SAMPLES - 1)));
        const m = maxPx > 0 ? magnitudes[idx] / maxPx : 0;
        // A convex lens magnifies: the backdrop is sampled from further
        // OUT toward the edges, i.e. displacement points outward.
        rr = Math.round(NEUTRAL + nx * m * 127);
        gg = Math.round(NEUTRAL + ny * m * 127);
        // Specular rim: normal-vs-light, strongest near the edge.
        const facing = Math.max(0, nx * lx + ny * ly);
        const edgeFall = 1 - t;
        sAlpha = Math.round(255 * 0.55 * facing * facing * edgeFall * edgeFall);
      }
      data[o] = rr;
      data[o + 1] = gg;
      data[o + 2] = NEUTRAL;
      data[o + 3] = 255;
      sdata[o] = 255;
      sdata[o + 1] = 255;
      sdata[o + 2] = 255;
      sdata[o + 3] = sAlpha;
    }
  }
  ctx.putImageData(img, 0, 0);
  sctx.putImageData(simg, 0, 0);
  return { displacement: canvas.toDataURL('image/png'), specular: spec.toDataURL('image/png'), scale };
}

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

interface Managed {
  el: HTMLElement;
  filter: SVGFilterElement;
  id: string;
  ro: ResizeObserver;
  timer: ReturnType<typeof setTimeout> | null;
  lastKey: string;
}

const managed = new Map<HTMLElement, Managed>();

function buildFilter(id: string): SVGFilterElement {
  const f = document.createElementNS(SVG_NS, 'filter');
  f.setAttribute('id', id);
  f.setAttribute('color-interpolation-filters', 'sRGB');
  f.setAttribute('filterUnits', 'userSpaceOnUse');
  return f;
}

function renderFilterContents(
  f: SVGFilterElement,
  maps: LiquidMaps,
  w: number,
  h: number,
  blur: number,
): void {
  while (f.firstChild) f.removeChild(f.firstChild);
  f.setAttribute('x', '0');
  f.setAttribute('y', '0');
  f.setAttribute('width', String(w));
  f.setAttribute('height', String(h));

  const map = document.createElementNS(SVG_NS, 'feImage');
  map.setAttribute('href', maps.displacement);
  map.setAttribute('x', '0');
  map.setAttribute('y', '0');
  map.setAttribute('width', String(w));
  map.setAttribute('height', String(h));
  map.setAttribute('preserveAspectRatio', 'none');
  map.setAttribute('result', 'map');
  f.appendChild(map);

  const disp = document.createElementNS(SVG_NS, 'feDisplacementMap');
  disp.setAttribute('in', 'SourceGraphic');
  disp.setAttribute('in2', 'map');
  disp.setAttribute('scale', String(maps.scale));
  disp.setAttribute('xChannelSelector', 'R');
  disp.setAttribute('yChannelSelector', 'G');
  disp.setAttribute('result', 'disp');
  f.appendChild(disp);

  const gb = document.createElementNS(SVG_NS, 'feGaussianBlur');
  gb.setAttribute('in', 'disp');
  gb.setAttribute('stdDeviation', String(blur));
  gb.setAttribute('result', 'soft');
  f.appendChild(gb);

  const sat = document.createElementNS(SVG_NS, 'feColorMatrix');
  sat.setAttribute('in', 'soft');
  sat.setAttribute('type', 'saturate');
  sat.setAttribute('values', '1.18');
  sat.setAttribute('result', 'sat');
  f.appendChild(sat);

  const specImg = document.createElementNS(SVG_NS, 'feImage');
  specImg.setAttribute('href', maps.specular);
  specImg.setAttribute('x', '0');
  specImg.setAttribute('y', '0');
  specImg.setAttribute('width', String(w));
  specImg.setAttribute('height', String(h));
  specImg.setAttribute('preserveAspectRatio', 'none');
  specImg.setAttribute('result', 'spec');
  f.appendChild(specImg);

  const blend = document.createElementNS(SVG_NS, 'feBlend');
  blend.setAttribute('in', 'sat');
  blend.setAttribute('in2', 'spec');
  blend.setAttribute('mode', 'screen');
  f.appendChild(blend);
}

function refresh(m: Managed): void {
  const rect = m.el.getBoundingClientRect();
  const w = Math.round(rect.width);
  const h = Math.round(rect.height);
  if (w < 8 || h < 8) return;
  const styles = getComputedStyle(m.el);
  const radius = parseFloat(styles.borderTopLeftRadius) || 0;
  const bezel = parseFloat(m.el.dataset.lgBezel ?? '') || Math.max(8, Math.min(radius, 18));
  const strength = parseFloat(m.el.dataset.lgStrength ?? '') || 1;
  const blur = parseFloat(m.el.dataset.lgBlur ?? '') || 3;
  const key = [w, h, radius, bezel, strength, blur].join(':');
  if (key === m.lastKey) return;
  m.lastKey = key;
  const maps = bakeMaps(w, h, radius, bezel, strength);
  renderFilterContents(m.filter, maps, w, h, blur);
  m.el.style.backdropFilter = `url(#${m.id})`;
}

function adopt(el: HTMLElement): void {
  if (managed.has(el)) return;
  const id = `liquid-glass-${nextId++}`;
  const filter = buildFilter(id);
  ensureHost().appendChild(filter);
  const m: Managed = { el, filter, id, ro: null as unknown as ResizeObserver, timer: null, lastKey: '' };
  m.ro = new ResizeObserver(() => {
    if (m.timer) clearTimeout(m.timer);
    // Debounce: baking a map per resize tick would churn; one bake after
    // the surface settles is plenty (the CSS blur fallback covers the gap).
    m.timer = setTimeout(() => refresh(m), 120);
  });
  m.ro.observe(el);
  managed.set(el, m);
  refresh(m);
}

function release(el: HTMLElement): void {
  const m = managed.get(el);
  if (!m) return;
  m.ro.disconnect();
  if (m.timer) clearTimeout(m.timer);
  m.filter.remove();
  managed.delete(el);
}

/** Watch the document for [data-liquid-glass] surfaces. Call once. */
export function initLiquidGlass(): void {
  const scan = (root: ParentNode) => {
    if (root instanceof HTMLElement && root.hasAttribute('data-liquid-glass')) adopt(root);
    root.querySelectorAll?.('[data-liquid-glass]').forEach((el) => adopt(el as HTMLElement));
  };
  const mo = new MutationObserver((entries) => {
    for (const e of entries) {
      e.addedNodes.forEach((n) => { if (n instanceof HTMLElement) scan(n); });
      e.removedNodes.forEach((n) => {
        if (!(n instanceof HTMLElement)) return;
        if (managed.has(n)) release(n);
        n.querySelectorAll?.('[data-liquid-glass]').forEach((el) => release(el as HTMLElement));
      });
    }
  });
  mo.observe(document.body, { childList: true, subtree: true });
  scan(document.body);
}
