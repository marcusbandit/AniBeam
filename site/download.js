// The download page: every file of the picked channel, Stable or Nightly, straight from GitHub's
// release list. Without JavaScript, or when GitHub does not answer, the cards lead to the
// releases page, so nothing here is required.
(() => {
  const repo = "marcusbandit/AniBeam";
  const page = document.querySelector(".dl-page");
  if (!page) return;
  const meta = document.getElementById("dl-meta");
  const toggle = document.querySelector(".channel"); // only on the download page
  const match = {
    appimage: (n) => n.endsWith(".AppImage"),
    deb: (n) => n.endsWith("_amd64.deb"),
    rpm: (n) => n.endsWith(".x86_64.rpm"),
    dmg: (n) => n.endsWith(".dmg"),
    setup: (n) => n.endsWith("-setup.exe"),
    zip: (n) => /windows.*\.zip$/.test(n),
    apk: (n) => n.endsWith(".apk"),
  };
  let releases = null;

  // "2.4.0" -> [2, 4, 0], for sorting stable versions by number rather than by date.
  const parts = (tag) => tag.replace(/^v/, "").split(/[.-]/).slice(0, 3).map(Number);
  const newer = (a, b) => { const x = parts(a), y = parts(b); for (let i = 0; i < 3; i++) if (x[i] !== y[i]) return x[i] > y[i]; return false; };

  function pick(channel) {
    if (!releases) return { desktop: null, phone: null };
    const live = releases.filter((r) => !r.draft);
    const stable = live.filter((r) => /^v\d+\.\d+\.\d+$/.test(r.tag_name) && !r.prerelease)
      .reduce((best, r) => (!best || newer(r.tag_name, best.tag_name) ? r : best), null);
    const nightly = live.find((r) => /^v[\d.]+-nightly\./.test(r.tag_name));
    return { desktop: channel === "nightly" ? nightly || stable : stable, phone: live.find((r) => r.tag_name.startsWith("android-v")) };
  }

  function show(channel) {
    page.dataset.channel = channel;
    if (toggle) toggle.querySelectorAll("button").forEach((b) => b.setAttribute("aria-selected", String(b.dataset.channel === channel)));
    const { desktop, phone } = pick(channel);
    for (const card of document.querySelectorAll("#dl-grid [data-asset]")) {
      const key = card.dataset.asset;
      const release = key === "apk" ? phone : desktop;
      const asset = release && release.assets.find((a) => match[key](a.name));
      card.href = asset ? asset.browser_download_url : (release ? release.html_url : `https://github.com/${repo}/releases`);
    }
    if (desktop && meta) {
      const version = desktop.tag_name.replace(/^v/, "");
      const when = new Date(desktop.published_at).toLocaleDateString("en-GB", { day: "numeric", month: "long", year: "numeric" });
      meta.textContent = `${version}, ${when}. `;
      const notes = document.createElement("a");
      notes.href = desktop.html_url;
      notes.textContent = "What's new";
      meta.append(notes);
    }
  }

  if (toggle) toggle.querySelectorAll("button").forEach((b) => b.addEventListener("click", () => show(b.dataset.channel)));
  fetch(`https://api.github.com/repos/${repo}/releases?per_page=100`, { headers: { Accept: "application/vnd.github+json" } })
    .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
    .then((list) => { releases = list; show(page.dataset.channel); })
    .catch(() => {});

  // The visitor's own system gets the wide tile on top and the brand-coloured button; one we
  // cannot place gets Linux there.
  const ua = navigator.userAgent;
  const os = /android/i.test(ua) ? "android" : /windows/i.test(ua) ? "windows" : /mac os x/i.test(ua) && navigator.maxTouchPoints < 2 ? "mac" : "linux";
  const mine = document.querySelector(`.dl-tile[data-os="${os}"]`);
  if (mine) {
    mine.classList.add("is-featured");
    mine.parentElement.prepend(mine);
    const main = mine.querySelector(".dl-main");
    main.classList.remove("btn-ghost");
    main.classList.add("btn-primary");
  }

  if (!toggle) return; // app.js wires the copy buttons on the home page
  document.querySelectorAll(".copy").forEach((b) => {
    b.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(b.dataset.copy);
        b.textContent = "Copied";
        b.classList.add("done");
        setTimeout(() => { b.textContent = "Copy"; b.classList.remove("done"); }, 1600);
      } catch {}
    });
  });
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
  toggle.querySelectorAll("button").forEach((b) => b.addEventListener("click", () => {
    // the click handler that switches the channel runs first; play for where it landed
    requestAnimationFrame(() => {
      if (page.dataset.channel !== last) play(page.dataset.channel);
      last = page.dataset.channel;
    });
  }));
})();
