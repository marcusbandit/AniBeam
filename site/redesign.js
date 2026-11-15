// The redesign mockup's own behaviour, beside app.js: the bar that turns to glass, the hero's
// beams following the pointer, and the tour's tabs projecting their screen.
(() => {
  const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;

  // The bar turns to glass once the page has scrolled under it.
  const nav = document.querySelector(".nav");
  const stick = () => nav.classList.toggle("stuck", scrollY > 8);
  stick();
  addEventListener("scroll", stick, { passive: true });

  // The beams lean toward the pointer, the near ones more than the far ones.
  const hero = document.querySelector(".bhero");
  const lanes = document.getElementById("marquee");
  if (hero && lanes && !reduced && matchMedia("(pointer: fine)").matches) {
    let frame = 0;
    hero.addEventListener("pointermove", (e) => {
      if (frame) return;
      frame = requestAnimationFrame(() => {
        frame = 0;
        const r = hero.getBoundingClientRect();
        lanes.style.setProperty("--px", `${(((e.clientX - r.left) / r.width - 0.5) * 60).toFixed(1)}px`);
        lanes.style.setProperty("--py", `${(((e.clientY - r.top) / r.height - 0.5) * 44).toFixed(1)}px`);
      });
    });
    hero.addEventListener("pointerleave", () => { lanes.style.setProperty("--px", "0px"); lanes.style.setProperty("--py", "0px"); });
  }

  // The tour: a tab shows its screen, and the cone of light runs from that tab to the window.
  const stage = document.querySelector(".tour-stage");
  const img = document.getElementById("tour-shot");
  if (!stage || !img) return;
  const tabs = [...stage.querySelectorAll(".tour-tabs button")];
  const win = stage.querySelector(".tour-window");
  const cone = document.getElementById("cone");
  const railIcons = [...stage.querySelectorAll(".tw-rail i[data-for]")];
  const alts = {
    home: "Anibeam's Home: shows to continue and shows not started yet, as poster cards",
    library: "The Library grid with every show as a poster",
    series: "A show's page with its banner, episodes and characters",
  };
  const wide = matchMedia("(min-width: 900px)").matches;
  const src = (k) => `${AB.ROOT}img/tour/${k}${wide ? "" : "-1134"}.webp`;
  if (wide) setTimeout(() => Object.keys(alts).forEach((k) => { new Image().src = src(k); }), 1500);

  function aim() {
    const active = tabs.find((t) => t.getAttribute("aria-selected") === "true");
    if (!active || !cone) return;
    const box = cone.getBoundingClientRect();
    const t = active.getBoundingClientRect();
    const w = win.getBoundingClientRect();
    const x0 = t.right + 14 - box.left;
    const y0 = t.top + t.height / 2 - box.top;
    const x1 = w.left + 30 - box.left;
    cone.style.clipPath = `polygon(${x0}px ${y0 - 3}px, ${x1}px ${w.top - box.top}px, ${x1}px ${w.bottom - box.top}px, ${x0}px ${y0 + 3}px)`;
  }
  function show(key, focus) {
    tabs.forEach((b) => {
      const on = b.dataset.screen === key;
      b.setAttribute("aria-selected", String(on));
      b.tabIndex = on ? 0 : -1;
      if (on && focus) b.focus();
    });
    railIcons.forEach((i) => i.classList.toggle("on", i.dataset.for === (key === "series" ? "library" : key)));
    aim();
    img.classList.add("swap");
    const next = new Image();
    next.onload = () => { img.removeAttribute("srcset"); img.src = next.src; img.alt = AB.t(alts[key]); img.classList.remove("swap"); };
    next.onerror = () => img.classList.remove("swap");
    next.src = src(key);
  }
  tabs.forEach((b, i) => {
    b.tabIndex = i === 0 ? 0 : -1;
    b.addEventListener("click", () => show(b.dataset.screen));
    b.addEventListener("keydown", (e) => {
      const step = { ArrowDown: 1, ArrowRight: 1, ArrowUp: -1, ArrowLeft: -1 }[e.key];
      if (!step) return;
      e.preventDefault();
      show(tabs[(i + step + tabs.length) % tabs.length].dataset.screen, true);
    });
  });
  railIcons.forEach((i) => i.classList.toggle("on", i.dataset.for === "home"));
  aim();
  addEventListener("resize", aim);
  if ("ResizeObserver" in window) new ResizeObserver(aim).observe(stage);
  document.fonts && document.fonts.ready.then(aim);
})();
