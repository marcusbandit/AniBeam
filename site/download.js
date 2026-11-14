// The download page: every file of the picked channel, Stable or Nightly, from the release links
// the site was built with (common.js, releases.json). Without them the cards lead to the releases
// page, so nothing here is required. #nightly in the address opens on Nightly.
(() => {
  const page = document.querySelector(".dl-page");
  if (!page) return;
  const { t, locale, ROOT } = AB;
  const meta = document.getElementById("dl-meta");
  const toggle = document.querySelector(".channel");
  let rel = null;

  function show(channel) {
    page.dataset.channel = channel;
    if (toggle) toggle.querySelectorAll("button").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.channel === channel)));
    if (!rel) return;
    // No nightly newer than the release means Nightly offers the release's files, and says so.
    // The phone has a nightly of its own, under its own number.
    const nightly = channel === "nightly";
    const android = AB.os === "android";
    const behind = nightly && !(android ? rel.phone_nightly : rel.nightly);
    const desktop = nightly && rel.nightly ? rel.nightly : rel.stable;
    const phone = nightly && rel.phone_nightly ? rel.phone_nightly : rel.phone;
    for (const card of document.querySelectorAll("#dl-grid [data-asset]")) {
      const from = card.dataset.asset === "apk" ? phone : desktop;
      if (from) card.href = from.assets[card.dataset.asset] || from.url;
    }
    // The iPhone's tile follows the channel like the others: Stable links the stable phone
    // release's IPA, Nightly the phone nightly's. It is there only when that release holds one,
    // so nothing shows before the first IPA, and only the nightly's is called a test build.
    const ipa = (nightly ? rel.phone_nightly : rel.phone)?.assets?.ipa;
    const tile = document.getElementById("dl-iphone");
    if (tile) {
      tile.hidden = !ipa;
      if (ipa) {
        for (const a of tile.querySelectorAll("[data-install]")) a.href = AB.installLink(a.dataset.install, ipa);
        tile.querySelector("[data-ipa]").href = ipa;
      }
    }
    // An Android phone sees only the phone's tile, so the line under the title is the phone
    // release's. An iPhone sees the Mac's tile until there is an iPhone app.
    const shown = android ? phone : desktop;
    if (!meta || !shown) return;
    const when = new Date(shown.published).toLocaleDateString(locale(), { day: "numeric", month: "long", year: "numeric" });
    meta.textContent = behind
      ? t("No nightly newer than {version} yet.", { version: shown.version }) + " "
      : `${shown.version}, ${when}. `;
    // whats-new.py writes each release's notes page into releases.json, or the list when it has none.
    const notes = document.createElement("a");
    notes.href = ROOT + (shown.notes || "whats-new/");
    notes.textContent = t("What's new");
    meta.append(notes);
  }

  if (toggle) toggle.querySelectorAll("button").forEach((b) => b.addEventListener("click", () => {
    show(b.dataset.channel);
    history.replaceState(history.state, "", b.dataset.channel === "nightly" ? "#nightly" : location.pathname);
  }));
  // #nightly or ?channel=nightly opens on Nightly; a phone has no toggle, so this is how an iPhone gets there.
  if ((location.hash === "#nightly" || new URLSearchParams(location.search).get("channel") === "nightly") && toggle) show("nightly");
  // A new language redraws the date and the What's new link in its words.
  document.addEventListener("langchange", () => show(page.dataset.channel));
  AB.releases().then((r) => { rel = r; show(page.dataset.channel); });

  // The visitor's own system gets the wide tile on top and the brand-coloured button. A phone gets
  // the phone tile and the page shows nothing else (style.css, data-device); an iPhone gets the
  // Mac's, under a note that the iPhone app is coming. A system we build nothing for gets no tile
  // picked, and neither does a Mac the browser says is Intel.
  function feature(want) {
    const mine = want && document.querySelector(`.dl-tile[data-os="${want}"]`);
    if (!mine) return;
    mine.classList.add("is-featured");
    mine.parentElement.prepend(mine);
    const main = mine.querySelector(".dl-main");
    if (AB.os !== "ios") { main.classList.remove("btn-ghost"); main.classList.add("btn-primary"); }
  }
  if (AB.os === "mac") AB.macChip.then((chip) => feature(chip === false ? null : "mac"));
  else feature(AB.os === "ios" ? "mac" : AB.os);
})();

// The streaks. Each switch of channel plays their entrance again, the same extension the page
// opens with. On Nightly they are redrawn as vector lines from the same numbers, each with a
// slight bend and dashes that crawl along it, like a line being traced in a draft.
(() => {
  const box = document.querySelector(".dl-beams");
  const toggle = document.querySelector(".channel");
  if (!box || !toggle) return;
  const page = document.querySelector(".dl-page");
  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const beams = [...box.querySelectorAll("i")];
  const NS = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(NS, "svg");
  svg.classList.add("beams-sketch");
  svg.setAttribute("aria-hidden", "true");
  box.append(svg);
  const sin = Math.sin(Math.PI / 6), cos = Math.cos(Math.PI / 6);
  let paths = [];

  const num = (el, name) => parseFloat(el.style.getPropertyValue(name));

  function draw() {
    const W = box.clientWidth, H = box.clientHeight;
    svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
    svg.replaceChildren();
    const defs = document.createElementNS(NS, "defs");
    svg.append(defs);
    paths = beams.map((el, i) => {
      const x = (num(el, "--x") / 100) * W, y = num(el, "--y"), len = (num(el, "--w") / 100) * W;
      const h = num(el, "--h"), a = num(el, "--a"), blur = num(el, "--b");
      const sx = x, sy = y + h / 2;
      const ex = sx + len * cos, ey = sy - len * sin;
      // a slight bow, a different one for every line: perpendicular to the beam, a little either way
      const bow = len * (0.026 + 0.014 * ((i * 37) % 5) / 4) * (i % 2 ? 1 : -1);
      // every other line bends back the other way near its end, a slight S
      const back = i % 3 === 0 ? -1.1 : 0.35;
      const nx = sin, ny = cos;
      const c1x = sx + (ex - sx) * 0.33 + nx * bow, c1y = sy + (ey - sy) * 0.33 + ny * bow;
      const c2x = sx + (ex - sx) * 0.7 + nx * bow * back, c2y = sy + (ey - sy) * 0.7 + ny * bow * back;
      const grad = document.createElementNS(NS, "linearGradient");
      grad.id = `bs-g${i}`;
      grad.setAttribute("gradientUnits", "userSpaceOnUse");
      Object.entries({ x1: sx, y1: sy, x2: ex, y2: ey }).forEach(([k, v]) => grad.setAttribute(k, v));
      for (const [off, op] of [[0, 0], [0.35, a], [0.7, a * 0.75], [1, 0]]) {
        const st = document.createElementNS(NS, "stop");
        st.setAttribute("offset", off);
        st.setAttribute("stop-color", "var(--paper, #f3e073)");
        st.setAttribute("stop-opacity", op);
        grad.append(st);
      }
      defs.append(grad);
      if (blur > 0.6) {
        const f = document.createElementNS(NS, "filter");
        f.id = `bs-f${i}`;
        f.setAttribute("x", "-20%"); f.setAttribute("y", "-20%"); f.setAttribute("width", "140%"); f.setAttribute("height", "140%");
        const g = document.createElementNS(NS, "feGaussianBlur");
        g.setAttribute("stdDeviation", blur * 0.8);
        f.append(g);
        defs.append(f);
      }
      const p = document.createElementNS(NS, "path");
      p.setAttribute("d", `M${sx} ${sy} C${c1x} ${c1y} ${c2x} ${c2y} ${ex} ${ey}`);
      p.setAttribute("stroke", `url(#bs-g${i})`);
      p.setAttribute("stroke-width", Math.max(h, 1.6));
      // Both dash patterns divide the 100 px the dashes crawl per loop (style.css, crawl), so the
      // loop never jumps: 34 + 16 is half of it, 15 + 10 a quarter.
      p.setAttribute("stroke-dasharray", h >= 8 ? "34 16" : "15 10");
      if (blur > 0.6) p.setAttribute("filter", `url(#bs-f${i})`);
      p.style.transformOrigin = `${sx}px ${sy}px`;
      svg.append(p);
      return p;
    });
  }

  function play(channel) {
    if (still) return;
    if (channel === "nightly") {
      paths.forEach((p, i) => p.animate(
        [{ transform: "scale(0.15)", opacity: 0 }, { transform: "scale(1)", opacity: 1 }],
        { duration: 1600, delay: i * 100, easing: "cubic-bezier(0.2, 0.7, 0.2, 1)", fill: "backwards" },
      ));
    } else {
      // the CSS entrance itself, rewound to just past its opening pause
      beams.forEach((el) => el.getAnimations().forEach((an) => {
        if (an.animationName === "bloom") { an.currentTime = 550; an.play(); }
      }));
    }
  }

  draw();
  addEventListener("resize", draw);
  let last = page.dataset.channel;
  // Opened on #nightly: the hand-drawn streaks play their entrance like the straight ones do.
  if (last === "nightly") play("nightly");
  toggle.querySelectorAll("button").forEach((b) => b.addEventListener("click", () => {
    // the click handler that switches the channel runs first; play for where it landed
    requestAnimationFrame(() => {
      if (page.dataset.channel !== last) play(page.dataset.channel);
      last = page.dataset.channel;
    });
  }));
})();
