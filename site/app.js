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

  function label() {
    for (const m of mains) {
      const span = m.querySelector("span");
      if (os === "ios") span.textContent = t("Coming to iPhone soon");
      else if (os === "android") span.textContent = t("Get the Android app");
      else if (want && !(os === "mac" && chip === false)) span.textContent = t("Download for {os}", { os: names[os] });
    }
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
    home: "AniBeam's Home: shows to continue and shows not started yet, as poster cards",
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
    next.onload = () => { img.removeAttribute("srcset"); img.src = next.src; img.alt = alts[key]; img.classList.remove("swap"); };
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
