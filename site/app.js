// The home page. Shared pieces (strings, the visitor's system, the release links, copy buttons)
// come from common.js, which loads first.
const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
// Pauses infinite motion (the marquee, the beams) while it is off screen.
const motionGate = (el) => {
  if (!el || !("IntersectionObserver" in window)) return;
  // CSS reads --motion; an SVG's own animation (the cast arc's dot) is paused through its clock.
  const svgs = [...el.querySelectorAll("svg")].filter((svg) => svg.querySelector("animate, animateMotion, animateTransform"));
  new IntersectionObserver(([e]) => {
    el.style.setProperty("--motion", e.isIntersecting ? "running" : "paused");
    svgs.forEach((svg) => (e.isIntersecting ? svg.unpauseAnimations() : svg.pauseAnimations()));
  }).observe(el);
};

// Adds `in` the first time an element is mostly on screen, which plays its one action.
const onceInView = (els, threshold = 0.45) => {
  if (!("IntersectionObserver" in window)) { els.forEach((el) => el.classList.add("in")); return; }
  const io = new IntersectionObserver((entries) => {
    for (const e of entries) if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); }
  }, { threshold });
  els.forEach((el) => io.observe(el));
};

// The two download buttons lead to the visitor's own file in the newest release. Without
// JavaScript, or for a system we build nothing for, they lead to the download page. A Mac whose
// chip the browser keeps quiet goes there too, where the dmg says it is for Apple silicon; an
// Intel Mac gets no file that will not run.
(() => {
  const { t, names, os } = AB;
  const mains = [...document.querySelectorAll(".dl-auto")];
  const want = { linux: ["stable", "appimage"], windows: ["stable", "setup"], mac: ["stable", "dmg"], android: ["phone", "apk"] }[os];
  let file = null;
  let chip = os === "mac" ? undefined : null;

  // Rewritten whole each time, so a Mac found to be Intel goes back to the plain words, and a
  // new language gets them in its own.
  function label() {
    let text = t("Download Anibeam");
    if (os === "ios") text = t("Coming to iPhone soon");
    else if (os === "android") text = t("Get the Android app");
    else if (want && !(os === "mac" && chip === false)) text = t("Download for {os}", { os: names[os] });
    for (const m of mains) m.querySelector("span").textContent = text;
  }
  function aim() {
    const sure = os !== "mac" || chip === true;
    for (const m of mains) {
      if (os === "ios") {
        m.removeAttribute("href");
        m.setAttribute("aria-disabled", "true");
        m.classList.replace("btn-primary", "btn-ghost");
        m.classList.add("is-soon");
      }
      else if (file && sure) m.href = file;
    }
  }
  label();
  aim();
  document.addEventListener("langchange", label);
  if (!want) return;
  AB.macChip.then((c) => { chip = c; label(); aim(); });
  AB.releases().then((r) => {
    const from = r && r[want[0]];
    file = from ? from.assets[want[1]] || from.url : null;
    aim();
  });
})();

// The tour: the rail beside the screenshot works like the app's own, one screen per icon.
(() => {
  const img = document.getElementById("tour-img");
  if (!img) return;
  const alts = {
    home: "Anibeam's Home: shows to continue and shows not started yet, as poster cards",
    library: "The Library grid with every show as a poster",
    series: "A show's page with its banner, episodes and characters",
  };
  const buttons = [...document.querySelectorAll(".rail button")];
  // The other screens load ahead on a wide screen only; a phone fetches one when it is asked for.
  const wide = matchMedia("(min-width: 900px)").matches;
  const src = (k) => `${AB.ROOT}img/tour/${k}${wide ? "" : "-1134"}.webp`;
  const warm = () => Object.keys(alts).forEach((k) => { new Image().src = src(k); });
  if (wide) {
    if ("requestIdleCallback" in window) requestIdleCallback(warm, { timeout: 3000 });
    else setTimeout(warm, 1500);
  }

  function show(key, focus) {
    buttons.forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.screen === key)));
    img.classList.add("swap");
    const next = new Image();
    next.onload = () => { img.removeAttribute("srcset"); img.src = next.src; img.alt = AB.t(alts[key]); img.classList.remove("swap"); };
    // A screen that fails to load leaves the last one showing, not a dimmed frame.
    next.onerror = () => img.classList.remove("swap");
    next.src = src(key);
    if (focus) buttons.find((b) => b.dataset.screen === key).focus();
  }
  buttons.forEach((b, i) => {
    b.addEventListener("click", () => show(b.dataset.screen));
    b.addEventListener("keydown", (e) => {
      const step = { ArrowDown: 1, ArrowRight: 1, ArrowUp: -1, ArrowLeft: -1 }[e.key];
      if (!step) return;
      e.preventDefault();
      show(buttons[(i + step + buttons.length) % buttons.length].dataset.screen, true);
    });
  });
})();

// The posters around the headline follow the pointer, each by its own depth. The entrance moves
// `transform`, the parallax moves `translate`, so the two never fight.
(() => {
  const hero = document.querySelector(".hero");
  const field = document.getElementById("floats");
  if (!hero || !field || reduced || !matchMedia("(pointer: fine)").matches) return;
  let frame = 0;
  hero.addEventListener("pointermove", (e) => {
    if (frame) return;
    frame = requestAnimationFrame(() => {
      frame = 0;
      const r = hero.getBoundingClientRect();
      field.style.setProperty("--px", `${(((e.clientX - r.left) / r.width - 0.5) * 36).toFixed(1)}px`);
      field.style.setProperty("--py", `${(((e.clientY - r.top) / r.height - 0.5) * 28).toFixed(1)}px`);
    });
  });
  hero.addEventListener("pointerleave", () => { field.style.setProperty("--px", "0px"); field.style.setProperty("--py", "0px"); });
})();
motionGate(document.querySelector(".beams"));
motionGate(document.querySelector(".cast-scene"));

motionGate(document.getElementById("marquee"));

// Each section plays its one action the first time it is mostly on screen.
onceInView([...document.querySelectorAll(".feat")], 0.4);

// The player card is drawn at the app's own 960 x 540 and scaled to fit, so every size matches.
(() => {
  const player = document.querySelector(".player");
  const stage = player && player.querySelector(".pl-stage");
  if (!stage) return;
  const fit = () => stage.style.setProperty("--k", (player.clientWidth / 960).toFixed(4));
  fit();
  if ("ResizeObserver" in window) new ResizeObserver(fit).observe(player);
  else addEventListener("resize", fit);
})();

// A poster in the wall opens a card beside it: the show from shows.json (tools/shows.py), its
// description and its AniList and MyAnimeList pages. The wall holds still while the card is open,
// so the card stays by its poster; on a narrow screen the card comes up from the bottom instead.
// Back, Escape, a click outside and the close button all close it.
(() => {
  const wall = document.getElementById("marquee");
  if (!wall) return;
  const { t } = AB;
  let shows = null;
  const load = () => (shows ??= fetch(AB.ROOT + "shows.json").then((r) => (r.ok ? r.json() : {})).catch(() => ({})));

  // Each track holds its posters twice so the loop is seamless; only the first copy is read out
  // and reached with Tab.
  const keyOf = (p) => (p.querySelector("img").getAttribute("src").match(/([\w-]+)\.webp$/) || [])[1];
  for (const track of wall.querySelectorAll(".marquee-track")) {
    const posters = [...track.querySelectorAll(".poster")];
    posters.forEach((p, i) => {
      p.dataset.show = keyOf(p);
      if (i >= posters.length / 2) { p.setAttribute("aria-hidden", "true"); return; }
      p.setAttribute("role", "button");
      p.tabIndex = 0;
    });
  }
  wall.removeAttribute("aria-hidden");
  load().then((data) => {
    for (const p of wall.querySelectorAll(".poster[role=button]")) {
      const s = data[p.dataset.show];
      if (s) p.setAttribute("aria-label", s.title);
    }
  });

  const card = document.createElement("div");
  card.className = "show-card";
  card.setAttribute("role", "dialog");
  card.setAttribute("aria-modal", "false");
  card.setAttribute("aria-labelledby", "show-card-title");
  card.hidden = true;
  document.body.append(card);

  const narrow = matchMedia("(max-width: 640px)");
  const el = (tag, cls, text) => {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  };
  const link = (href, cls, label) => {
    const a = el("a", `btn btn-sm btn-ghost ${cls}`);
    a.href = href;
    a.target = "_blank";
    a.rel = "noopener";
    a.append(el("i"), el("span", null, label));
    return a;
  };

  function fill(s) {
    const close = el("button", "show-x");
    close.type = "button";
    close.setAttribute("aria-label", t("Close"));
    close.innerHTML = '<svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg>';
    close.addEventListener("click", () => shut(true));

    const head = el("div", "show-head");
    const title = el("h3", null, s.title);
    title.id = "show-card-title";
    head.append(title);
    if (s.romaji && s.romaji.toLowerCase() !== s.title.toLowerCase()) head.append(el("p", "show-romaji", s.romaji));

    const facts = [s.format === "MOVIE" ? t("Movie") : s.format === "TV" ? "TV" : s.format, s.year];
    if (s.format !== "MOVIE" && s.episodes > 1) facts.push(t("{n} episodes", { n: s.episodes }));
    if (s.studio) facts.push(s.studio);
    const meta = el("p", "show-meta");
    facts.filter(Boolean).forEach((f, i) => { if (i) meta.append(el("b", null, "·")); meta.append(el("span", null, String(f))); });

    const tags = el("div", "show-tags");
    if (s.score) tags.append(el("span", "show-score", `★ ${s.score}%`));
    for (const g of s.genres) tags.append(el("span", null, g));

    const text = el("div", "show-text");
    for (const para of s.description.split("\n\n")) text.append(el("p", null, para));

    const links = el("div", "show-links");
    links.append(link(`https://anilist.co/anime/${s.anilist}`, "to-anilist", "AniList"));
    if (s.mal) links.append(link(`https://myanimelist.net/anime/${s.mal}`, "to-mal", "MyAnimeList"));

    card.replaceChildren(close, head, meta, tags, text, links);
  }

  // Beside the poster on whichever side has room, top edge level with it, kept on screen. The
  // card sits in the page, not fixed, so it scrolls along with its poster.
  function place(poster) {
    card.style.left = card.style.top = "";
    card.classList.remove("from-left", "from-right");
    if (narrow.matches) return;
    const r = poster.getBoundingClientRect();
    const w = card.offsetWidth, h = card.offsetHeight, gap = 16, edge = 16, under = 84; // clear of the nav bar
    const right = r.right + gap + w <= innerWidth - edge;
    const left = right ? r.right + gap : Math.max(edge, r.left - gap - w);
    const top = Math.min(Math.max(under, r.top + r.height / 2 - h / 2), innerHeight - edge - h);
    card.classList.add(right ? "from-left" : "from-right");
    card.style.left = `${left + scrollX}px`;
    card.style.top = `${Math.max(edge, top) + scrollY}px`;
  }

  let open = null;
  let pushed = false;
  async function show(poster, byKey) {
    const s = (await load())[poster.dataset.show];
    if (!s) return;
    if (open) open.classList.remove("is-open");
    open = poster;
    poster.classList.add("is-open");
    wall.classList.add("holding");
    fill(s);
    card.hidden = false;
    card.classList.remove("in");
    place(poster);
    requestAnimationFrame(() => card.classList.add("in"));
    // On a phone the card covers the page, so back closes it rather than leaving the site.
    if (narrow.matches && !pushed) { history.pushState({ showCard: true }, ""); pushed = true; }
    if (byKey) card.querySelector(".show-x").focus({ preventScroll: true });
  }
  function shut(refocus) {
    if (!open) return;
    const poster = open;
    open = null;
    poster.classList.remove("is-open");
    wall.classList.remove("holding");
    card.classList.remove("in");
    card.hidden = true;
    if (pushed) { pushed = false; history.back(); }
    if (refocus) poster.focus({ preventScroll: true });
  }

  wall.addEventListener("click", (e) => {
    const p = e.target.closest(".poster");
    if (!p) return;
    if (p === open) shut(false);
    else show(p, false);
  });
  wall.addEventListener("keydown", (e) => {
    const p = e.target.closest(".poster[role=button]");
    if (!p || (e.key !== "Enter" && e.key !== " ")) return;
    e.preventDefault();
    show(p, true);
  });
  document.addEventListener("keydown", (e) => { if (e.key === "Escape" && open) shut(true); });
  document.addEventListener("pointerdown", (e) => {
    if (open && !card.contains(e.target) && !e.target.closest("#marquee .poster")) shut(false);
  });
  addEventListener("popstate", () => { if (pushed) { pushed = false; shut(false); } });
  addEventListener("resize", () => { if (open) place(open); });
})();
