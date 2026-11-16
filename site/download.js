// The download page: every file of the picked channel, Stable or Nightly, from the release links
// the site was built with (common.js, releases.json). Without them the rows lead to the releases
// page, so nothing here is required. #nightly in the address opens on Nightly.
(() => {
  const page = document.querySelector(".dl-page");
  if (!page) return;
  const { t, locale, ROOT } = AB;
  const meta = document.getElementById("dl-meta");
  const toggle = document.querySelector(".channel");
  let rel = null;
  let mine = null; // the card of the visitor's own system, once picked

  const size = (bytes) => (bytes ? `${Math.max(1, Math.round(bytes / 1048576))} MB` : "");

  function show(channel) {
    page.dataset.channel = channel;
    if (toggle) toggle.querySelectorAll("button").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.channel === channel)));
    tabs.ensure();
    if (rel) fill(channel);
    hero();
  }

  function fill(channel) {
    // No nightly newer than the release means Nightly offers the release's files, and says so.
    // The phone has a nightly of its own, under its own number.
    const nightly = channel === "nightly";
    const android = AB.os === "android";
    const behind = nightly && !(android ? rel.phone_nightly : rel.nightly);
    const desktop = nightly && rel.nightly ? rel.nightly : rel.stable;
    const phone = nightly && rel.phone_nightly ? rel.phone_nightly : rel.phone;
    for (const row of document.querySelectorAll("#dl-grid [data-asset]")) {
      const from = row.dataset.asset === "apk" ? phone : desktop;
      if (from) row.href = from.assets[row.dataset.asset] || from.url;
    }
    for (const el of document.querySelectorAll("#dl-grid [data-size]")) {
      const from = ["apk", "ipa"].includes(el.dataset.size) ? phone : desktop;
      el.textContent = size(from?.sizes?.[el.dataset.size]);
    }
    // The iPhone's card: the one Anibeam source carries both channels, so its rows do not follow
    // the toggle. The IPA and the one-off installs do: they are the picked channel's build, shown
    // only when that release holds an IPA, and only the nightly's is called a test build. The card
    // itself is there when either channel has an IPA, so nothing shows before the first one.
    const ipa = (nightly ? rel.phone_nightly : rel.phone)?.assets?.ipa;
    const card = document.getElementById("dl-iphone");
    if (card) {
      card.hidden = !(rel.phone?.assets?.ipa || rel.phone_nightly?.assets?.ipa);
      for (const a of card.querySelectorAll("[data-source]")) a.href = AB.sourceLink(a.dataset.source);
      for (const el of card.querySelectorAll("[data-ipa], [data-build]")) el.hidden = !ipa;
      if (ipa) {
        for (const a of card.querySelectorAll("[data-install]")) a.href = AB.installLink(a.dataset.install, ipa);
        card.querySelector("[data-ipa]").href = ipa;
      }
    }
    // An Android phone sees only the phone's download, so the version line is the phone
    // release's.
    const shown = android ? phone : desktop;
    if (!meta || !shown) return;
    const ver = document.createElement("span");
    ver.className = "dl-ver";
    ver.textContent = shown.version;
    const when = document.createElement("span");
    when.textContent = behind
      ? t("No nightly newer than {version} yet.", { version: shown.version })
      : new Date(shown.published).toLocaleDateString(locale(), { day: "numeric", month: "long", year: "numeric" });
    // whats-new.py writes each release's notes page into releases.json, or the list when it has none.
    const notes = document.createElement("a");
    notes.href = ROOT + (shown.notes || "whats-new/");
    notes.textContent = t("What's new");
    meta.replaceChildren(ver, when, notes);
  }

  // The hero's button takes the visitor's card: its icon, its main file and size, and its link.
  // The card's other rows become chips under it. With no card picked it leads down to the cards.
  function hero() {
    if (!mine || mine.hidden) return;
    const button = document.getElementById("dl-mine");
    const main = mine.querySelector(".dl-file[data-main]");
    button.href = main.href;
    document.getElementById("dl-mine-os").replaceChildren(mine.querySelector(".dl-glyph svg").cloneNode(true));
    document.getElementById("dl-mine-name").textContent = t("Download for {os}", { os: mine.querySelector("h3").textContent });
    const file = document.getElementById("dl-mine-file");
    file.textContent = [main.querySelector(".dl-fname").textContent, main.querySelector(".dl-size")?.textContent].filter(Boolean).join(" · ");
    file.hidden = false;
    const alt = document.getElementById("dl-alt");
    alt.replaceChildren(...[...mine.querySelectorAll(".dl-files > li")]
      .filter((li) => !li.hidden && getComputedStyle(li).display !== "none")
      .map((li) => li.querySelector(".dl-file"))
      .filter((a) => a && !a.hasAttribute("data-main"))
      .map((a) => {
        const chip = a.cloneNode(true);
        chip.className = "";
        chip.querySelector(".dl-size")?.remove();
        return chip;
      }));
  }

  // The terminal's tabs: one way in per tab, arrow keys move between them. A link to #get-brew (or
  // the Homebrew row) opens that tab. Nightly has only the script, so it falls back to that.
  const tabs = (() => {
    const list = document.querySelector(".dl-tabs");
    const all = list ? [...list.querySelectorAll('[role="tab"]')] : [];
    const usable = () => all.filter((b) => getComputedStyle(b).display !== "none");
    function pick(tab, focus) {
      for (const b of all) {
        const on = b === tab;
        b.setAttribute("aria-selected", String(on));
        b.tabIndex = on ? 0 : -1;
        document.getElementById(b.getAttribute("aria-controls")).hidden = !on;
      }
      if (focus) tab.focus();
    }
    function ensure() {
      const on = all.find((b) => b.getAttribute("aria-selected") === "true");
      if (on && !usable().includes(on)) pick(usable()[0]);
    }
    function open(name) {
      const tab = all.find((b) => b.dataset.tab === name);
      if (!tab || !usable().includes(tab)) return;
      pick(tab);
      document.getElementById("terminal").scrollIntoView({ block: "start" });
    }
    if (list) {
      list.addEventListener("click", (e) => { const b = e.target.closest('[role="tab"]'); if (b) pick(b); });
      list.addEventListener("keydown", (e) => {
        const step = { ArrowRight: 1, ArrowLeft: -1 }[e.key];
        if (!step) return;
        const u = usable();
        const i = u.indexOf(document.activeElement);
        if (i < 0) return;
        e.preventDefault();
        pick(u[(i + step + u.length) % u.length], true);
      });
    }
    document.addEventListener("click", (e) => {
      const a = e.target.closest("a[data-tab]");
      if (!a) return;
      e.preventDefault();
      open(a.dataset.tab);
    });
    if (location.hash.startsWith("#get-")) open(location.hash.slice(5));
    return { ensure };
  })();

  if (toggle) toggle.querySelectorAll("button").forEach((b) => b.addEventListener("click", () => {
    show(b.dataset.channel);
    history.replaceState(history.state, "", b.dataset.channel === "nightly" ? "#nightly" : location.pathname);
  }));
  // #nightly or ?channel=nightly opens on Nightly; a phone has no toggle, so this is how an iPhone gets there.
  if ((location.hash === "#nightly" || new URLSearchParams(location.search).get("channel") === "nightly") && toggle) page.dataset.channel = "nightly";
  // A new language redraws the date, the What's new link and the button in its words.
  document.addEventListener("langchange", () => show(page.dataset.channel));

  // The visitor's own system: its card is marked and moves first, and the hero's button takes its
  // file. An iPhone gets its own card once there is a build, the Mac's until then. A system we
  // build nothing for gets no card picked, and neither does a Mac the browser says is Intel.
  function feature(want) {
    const card = want && document.querySelector(`.dl-card[data-os="${want}"]`);
    if (!card) return;
    for (const c of document.querySelectorAll(".dl-card.is-mine")) c.classList.remove("is-mine");
    card.classList.add("is-mine");
    card.parentElement.prepend(card);
    mine = card;
  }
  const picked = AB.os === "mac" ? AB.macChip.then((chip) => (chip === false ? null : "mac")) : Promise.resolve(AB.os);
  Promise.all([picked, AB.releases()]).then(([os, r]) => {
    rel = r;
    if (os === "ios") {
      const iphone = document.getElementById("dl-iphone");
      if (rel) fill(page.dataset.channel);
      os = iphone && !iphone.hidden ? "ios" : "mac";
    }
    feature(os);
    show(page.dataset.channel);
  });
  show(page.dataset.channel);
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
