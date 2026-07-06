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
 * The specular rim is CSS inset light (see [data-liquid-glass] in base.css)
 * since the feImage specular is equally impossible.
 *
 * Usage: put `data-liquid-glass` on any element whose backdrop should
 * refract. Optional attributes:
 *   data-lg-bezel    refracting rim width in px (default: min(radius, 18))
 *   data-lg-strength refraction strength multiplier (default 1)
 *   data-lg-blur     blur inside the chain in px (default 2.5)
 * The manager (initLiquidGlass) watches the DOM, builds a filter per
 * element, and rebuilds on resize. Elements keep their CSS backdrop-filter
 * as a fallback until the manager attaches the inline one.
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
  const key = [w, h, bezel, strength, blur].join(':');
  if (key === m.lastKey) return;
  m.lastKey = key;
  renderFilterContents(m.filter, w, h, bezel, strength, blur);
  m.el.style.backdropFilter = `url(#${m.id})`;
}

function adopt(elm: HTMLElement): void {
  if (managed.has(elm)) return;
  const id = `liquid-glass-${nextId++}`;
  const filter = document.createElementNS(SVG_NS, 'filter') as SVGFilterElement;
  filter.setAttribute('id', id);
  filter.setAttribute('color-interpolation-filters', 'sRGB');
  filter.setAttribute('filterUnits', 'userSpaceOnUse');
  ensureHost().appendChild(filter);
  const m: Managed = { el: elm, filter, id, ro: null as unknown as ResizeObserver, timer: null, lastKey: '' };
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
