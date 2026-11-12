// Switching language without reloading the page. Every language is a real page of its own
// (site/tools/i18n.py), with the same elements in the same places and only the words and paths
// changed. So instead of loading it, this fetches it, lays its text, alt text and links over the
// page that is already running, and moves the address bar to it. Animations keep going and
// nothing blinks. A link to a different kind of page, or any failure, falls back to a normal load.
(() => {
  const cache = new Map();
  let shown = location.href; // the page whose words are on screen
  const page = (url) => {
    if (!cache.has(url)) {
      cache.set(url, fetch(url).then((r) => (r.ok ? r.text() : Promise.reject(r.status)))
        .then((text) => new DOMParser().parseFromString(text, "text/html")));
    }
    return cache.get(url);
  };

  // An element's place in the page. Tiles carry their system (data-os) because the download
  // script moves the visitor's own to the front; the rest are counted among their siblings.
  const key = (el) => {
    if (el.dataset && el.dataset.os) return `${el.tagName}[${el.dataset.os}]`;
    const sibs = [...el.parentElement.children].filter((c) => !(c.dataset && c.dataset.os));
    return `${el.tagName}:${sibs.indexOf(el)}`;
  };
  const index = (doc) => {
    const map = new Map();
    for (const el of doc.body.querySelectorAll("*")) {
      if (el.closest("script, style, svg, .nav-lang")) continue;
      const path = [];
      for (let e = el; e && e !== doc.body; e = e.parentElement) path.unshift(key(e));
      map.set(path.join(">"), el);
    }
    return map;
  };
  const texts = (el) => [...el.childNodes].filter((n) => n.nodeType === 3);
  const ATTRS = ["href", "src", "alt", "aria-label", "title", "placeholder"];

  // The page's own kind, without the language: "" for home, "download/" for downloads.
  const kind = (url) => new URL(url, location.href).pathname.replace(/^\/(?:[a-z]{2}(?:-[a-z]{2})?\/)?/, "");

  async function go(url, push) {
    const target = new URL(url, location.href).href;
    const [now, next] = await Promise.all([page(shown), page(target)]);
    // The browser asks for the icon again the moment the address changes, so its links get their
    // full address first; a relative one would point into the wrong folder.
    document.head.querySelectorAll('link[rel="icon"], link[rel="apple-touch-icon"]').forEach((l) => l.setAttribute("href", l.href));
    if (push) history.pushState({ lang: true }, "", target);
    const cur = index(now), tgt = index(next), live = index(document);

    for (const [path, t] of tgt) {
      const l = live.get(path), c = cur.get(path);
      if (!l || !c) continue;
      // Text the scripts have since changed is left to them; they hear "langchange" below.
      const [lt, ct, tt] = [texts(l), texts(c), texts(t)];
      if (lt.length === ct.length && ct.length === tt.length) {
        lt.forEach((n, i) => { if (n.nodeValue === ct[i].nodeValue && n.nodeValue !== tt[i].nodeValue) n.nodeValue = tt[i].nodeValue; });
      }
      for (const a of ATTRS) {
        const lv = l.getAttribute(a), cv = c.getAttribute(a), tv = t.getAttribute(a);
        if (lv !== null && lv === cv && tv !== null && tv !== lv) l.setAttribute(a, tv);
      }
    }

    // The head, the menu and the strings the scripts use.
    document.documentElement.lang = next.documentElement.lang;
    document.title = next.title;
    for (const sel of ['meta[name="description"]', 'meta[property="og:title"]', 'meta[property="og:description"]', 'link[rel="canonical"]']) {
      const a = document.head.querySelector(sel), b = next.head.querySelector(sel);
      if (a && b) a.replaceWith(b.cloneNode(true));
    }
    const menu = document.querySelector(".nav-lang"), fresh = next.querySelector(".nav-lang");
    if (menu && fresh) menu.replaceWith(document.importNode(fresh, true));
    const strings = [...next.scripts].map((s) => s.textContent).find((s) => s.includes("window.I18N"));
    window.I18N = strings ? JSON.parse(strings.slice(strings.indexOf("{"), strings.lastIndexOf("}") + 1)) : undefined;
    shown = target;
    document.dispatchEvent(new Event("langchange"));
  }

  document.addEventListener("click", (e) => {
    const a = e.target.closest(".lang-menu a");
    if (!a || e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return;
    const menu = a.closest("details");
    if (kind(a.href) !== kind(location.href)) return; // another kind of page: a normal load
    e.preventDefault();
    if (menu) menu.open = false;
    if (a.getAttribute("aria-current") === "true") return;
    go(a.href, true).catch(() => { location.href = a.href; });
  });
  // Opening the menu fetches the other languages ahead, so a pick lands at once.
  document.addEventListener("toggle", (e) => {
    if (e.target.matches && e.target.matches(".nav-lang") && e.target.open) {
      e.target.querySelectorAll(".lang-menu a").forEach((a) => { if (kind(a.href) === kind(location.href)) page(a.href).catch(() => {}); });
    }
  }, true);
  addEventListener("popstate", () => { go(location.href, false).catch(() => location.reload()); });
})();
