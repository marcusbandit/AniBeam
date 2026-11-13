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

  // The posters turn into buttons once the shows have loaded, which starts when the wall comes
  // near the screen. Each track holds its posters twice so the loop is seamless; only the first
  // copy is read out and reached with Tab. A poster with no show stays a picture.
  const keyOf = (p) => (p.querySelector("img").getAttribute("src").match(/([\w-]+)\.webp$/) || [])[1];
  function enable(data) {
    let any = false;
    for (const track of wall.querySelectorAll(".marquee-track")) {
      const posters = [...track.querySelectorAll(".poster")];
      posters.forEach((p, i) => {
        const key = keyOf(p);
        if (!data[key]) return;
        any = true;
        p.dataset.show = key;
        if (i >= posters.length / 2) { p.setAttribute("aria-hidden", "true"); return; }
        p.setAttribute("role", "button");
        p.setAttribute("aria-label", data[key].title);
        p.tabIndex = 0;
      });
    }
    if (any) wall.removeAttribute("aria-hidden");
  }
  if ("IntersectionObserver" in window) {
    const io = new IntersectionObserver(([e]) => {
      if (!e.isIntersecting) return;
      io.disconnect();
      load().then(enable);
    }, { rootMargin: "600px 0px" });
    io.observe(wall);
  } else load().then(enable);

  // Right after the wall, so Tab from the card goes on to the page after it.
  const card = document.createElement("div");
  card.className = "show-card";
  card.setAttribute("role", "dialog");
  card.setAttribute("aria-modal", "false");
  card.setAttribute("aria-labelledby", "show-card-title");
  card.hidden = true;
  wall.after(card);

  const narrow = matchMedia("(max-width: 640px)");
  const el = (tag, cls, text) => {
    const e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  };
  // The two trackers' own marks: AniList's A beside its name, MyAnimeList's MAL wordmark alone.
  const ANILIST = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M24 17.53v2.421c0 .71-.391 1.101-1.1 1.101h-5l-.057-.165L11.84 3.736c.106-.502.46-.788 1.053-.788h2.422c.71 0 1.1.391 1.1 1.1v12.38H22.9c.71 0 1.1.392 1.1 1.101zM11.034 2.947l6.337 18.104h-4.918l-1.052-3.131H6.019l-1.077 3.131H0L6.361 2.948h4.673zm-.66 10.96-1.69-5.014-1.541 5.015h3.23z"/></svg>';
  const MAL = '<svg viewBox="0 6.6 24 10.8" aria-hidden="true"><path d="M8.273 7.247v8.423l-2.103-.003v-5.216l-2.03 2.404-1.989-2.458-.02 5.285H.001L0 7.247h2.203l1.865 2.545 2.015-2.546 2.19.001zm8.628 2.069l.025 6.335h-2.365l-.008-2.871h-2.8c.07.499.21 1.266.417 1.779.155.381.298.751.583 1.128l-1.705 1.125c-.349-.636-.622-1.337-.878-2.082a9.296 9.296 0 0 1-.507-2.179c-.085-.75-.097-1.471.107-2.212a3.908 3.908 0 0 1 1.161-1.866c.313-.293.749-.5 1.1-.687.351-.187.743-.264 1.107-.359a7.405 7.405 0 0 1 1.191-.183c.398-.034 1.107-.066 2.39-.028l.545 1.749H14.51c-.593.008-.878.001-1.341.209a2.236 2.236 0 0 0-1.278 1.92l2.663.033.038-1.81h2.309zm3.992-2.099v6.627l3.107.032-.43 1.775h-4.807V7.187l2.13.03z"/></svg>';
  const link = (href, cls, mark, label) => {
    const a = el("a", `show-link ${cls}`);
    a.href = href;
    a.target = "_blank";
    a.rel = "noopener";
    a.innerHTML = mark;
    if (cls === "to-mal") a.setAttribute("aria-label", label);
    else a.append(el("span", null, label));
    return a;
  };

  function fill(s, poster) {
    const close = el("button", "show-x");
    close.type = "button";
    close.setAttribute("aria-label", t("Close"));
    close.innerHTML = '<svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg>';
    close.addEventListener("click", () => shut(true));

    const head = el("div", "show-head");
    const thumb = el("img");
    thumb.src = poster.querySelector("img").src;
    thumb.alt = "";
    const names = el("div");
    const title = el("h3", null, s.title);
    title.id = "show-card-title";
    names.append(title);
    if (s.romaji && s.romaji.toLowerCase() !== s.title.toLowerCase()) names.append(el("p", "show-romaji", s.romaji));
    head.append(thumb, names);

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
    links.append(link(`https://anilist.co/anime/${s.anilist}`, "to-anilist", ANILIST, "AniList"));
    if (s.mal) links.append(link(`https://myanimelist.net/anime/${s.mal}`, "to-mal", MAL, "MyAnimeList"));

    card.replaceChildren(close, head, meta, tags, text, links);
  }
  // Beside the poster on whichever side has room, level with it, kept on screen and clear of the
  // nav bar. The card sits in the page, not fixed, so it scrolls along with its poster.
  function place(poster) {
    card.style.left = card.style.top = "";
    card.classList.remove("from-left", "from-right");
    if (narrow.matches) return;
    const r = poster.getBoundingClientRect();
    const box = card.offsetParent ? card.offsetParent.getBoundingClientRect() : { left: -scrollX, top: -scrollY };
    const w = card.offsetWidth, h = card.offsetHeight, gap = 16, edge = 16, under = 84;
    const right = r.right + gap + w <= innerWidth - edge;
    const left = right ? r.right + gap : Math.max(edge, r.left - gap - w);
    const top = Math.max(under, Math.min(r.top + r.height / 2 - h / 2, innerHeight - edge - h));
    card.classList.add(right ? "from-left" : "from-right");
    card.style.left = `${left - box.left}px`;
    card.style.top = `${top - box.top}px`;
  }

  // On a phone the sheet covers the page, so it gets a history entry and back closes it. Leaving
  // that entry puts the scroll back where it was when the sheet opened, so restoring is off
  // while it goes.
  let pushed = false;
  function pushSheet() {
    if (pushed || !narrow.matches) return;
    history.pushState({ showCard: true }, "");
    pushed = true;
  }
  function popSheet(viaLink) {
    if (!pushed) return;
    pushed = false;
    // A link the visitor tapped adds its own entry; going back would undo it.
    if (viaLink) return;
    history.scrollRestoration = "manual";
    history.back();
  }
  addEventListener("popstate", () => {
    history.scrollRestoration = "auto";
    if (pushed) { pushed = false; shut(false, true); }
  });

  let open = null;
  let wanted = null;
  async function show(poster, byKey) {
    wanted = poster;
    const s = (await load())[poster.dataset.show];
    // Closed, or another poster asked for, while the shows were loading.
    if (wanted !== poster || !s) return;
    if (open) open.classList.remove("is-open");
    open = poster;
    poster.classList.add("is-open");
    wall.classList.add("holding");
    fill(s, poster);
    card.hidden = false;
    card.classList.remove("in");
    place(poster);
    void card.offsetWidth; // start the slide from the closed state, on a phone too
    requestAnimationFrame(() => card.classList.add("in"));
    pushSheet();
    if (byKey) card.querySelector(".show-x").focus({ preventScroll: true });
  }
  function shut(refocus, fromHistory, viaLink) {
    wanted = null;
    if (!open) return;
    const poster = open;
    open = null;
    poster.classList.remove("is-open");
    wall.classList.remove("holding");
    card.classList.remove("in");
    card.hidden = true;
    if (!fromHistory) popSheet(viaLink);
    if (refocus) poster.focus({ preventScroll: true });
  }

  wall.addEventListener("click", (e) => {
    const p = e.target.closest(".poster[data-show]");
    if (!p) return;
    if (p === open || p === wanted) shut(false);
    else show(p, false);
  });
  wall.addEventListener("keydown", (e) => {
    const p = e.target.closest(".poster[role=button]");
    if (!p || (e.key !== "Enter" && e.key !== " ")) return;
    e.preventDefault();
    show(p, true);
  });
  document.addEventListener("keydown", (e) => { if (e.key === "Escape" && (open || wanted)) shut(true); });
  document.addEventListener("pointerdown", (e) => {
    if ((open || wanted) && !card.contains(e.target) && !e.target.closest("#marquee .poster")) shut(false, false, !!e.target.closest("a[href]"));
  });
  addEventListener("resize", () => {
    if (!open) return;
    place(open);
    pushSheet();
  });
})();
