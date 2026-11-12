document.documentElement.classList.add("js");
const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;

// Points every download button at the file in the newest release and opens the install tab for
// the visitor's own system. Without JavaScript, or when GitHub does not answer, every link still
// goes to the releases page, so nothing here is required.
(() => {
  const repo = "marcusbandit/AniBeam";
  const api = `https://api.github.com/repos/${repo}/releases`;
  const names = { linux: "Linux", windows: "Windows", mac: "Mac", android: "Android" };
  const mainAsset = { linux: "appimage", windows: "setup", mac: "dmg", android: "apk" };
  const match = {
    appimage: (n) => n.endsWith(".AppImage"),
    setup: (n) => n.endsWith("-setup.exe"),
    zip: (n) => /windows.*\.zip$/.test(n),
    dmg: (n) => n.endsWith(".dmg"),
    apk: (n) => n.endsWith(".apk"),
  };

  // Only builds that exist: the dmg is Apple silicon, the AppImage x86_64. A system we cannot
  // place gets the plain releases link rather than a file that will not run.
  function detect() {
    const ua = navigator.userAgent;
    const platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
    if (/android/i.test(ua)) return "android";
    if (/iphone|ipad|ipod/i.test(ua)) return null;
    if (/win/i.test(platform) || /windows/i.test(ua)) return "windows";
    if (/mac/i.test(platform) || /mac os x/i.test(ua)) return navigator.maxTouchPoints > 1 ? null : "mac";
    if (/aarch64|arm/i.test(platform)) return null;
    if (/linux|x11/i.test(platform) || /linux/i.test(ua)) return "linux";
    return null;
  }

  let os = detect();
  const mains = [...document.querySelectorAll(".dl-auto")];
  const phoneReleases = `https://github.com/${repo}/releases?q=android&expanded=true`;
  const urls = {};

  // The CPU check and the release lookup can finish in either order; whichever comes last points
  // the buttons at the file.
  function aim() {
    if (os && urls[mainAsset[os]]) mains.forEach((b) => { if (!b.closest(".nav")) b.href = urls[mainAsset[os]]; });
  }
  function pick(chosen) {
    os = chosen;
    if (!os) return;
    for (const b of mains) {
      if (b.closest(".nav")) continue;
      const label = b.querySelector("span") || b;
      label.textContent = os === "android" ? "Get the Android app" : `Download for ${names[os]}`;
      if (os === "android") b.href = phoneReleases;
    }
    aim();
    document.documentElement.dataset.os = os;
    document.dispatchEvent(new CustomEvent("os", { detail: os }));
  }
  if (os === "mac" && navigator.userAgentData && navigator.userAgentData.getHighEntropyValues) {
    navigator.userAgentData.getHighEntropyValues(["architecture"])
      .then((v) => pick(v.architecture === "x86" ? null : "mac"))
      .catch(() => pick("mac"));
    os = null;
  } else {
    pick(os);
  }

  function wire(release, keys) {
    if (!release) return;
    for (const key of keys) {
      const asset = release.assets.find((a) => match[key](a.name));
      if (!asset) continue;
      document.querySelectorAll(`[data-asset="${key}"]`).forEach((l) => { l.href = asset.browser_download_url; });
      urls[key] = asset.browser_download_url;
    }
  }
  // One call for both: the release list holds the desktop stable (v*, not a pre-release) and the
  // phone's (android-v*). Unauthenticated calls are limited to 60 an hour per address.
  fetch(`${api}?per_page=100`, { headers: { Accept: "application/vnd.github+json" } })
    .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
    .then((list) => {
      const live = list.filter((r) => !r.draft);
      wire(live.find((r) => /^v\d/.test(r.tag_name) && !r.prerelease), ["appimage", "setup", "zip", "dmg"]);
      wire(live.find((r) => r.tag_name.startsWith("android-v")), ["apk"]);
      aim();
    })
    .catch(() => {});
})();

// Pauses infinite motion (the marquee, the cast pulse) while it is off screen.
const motionGate = (el) => {
  if (!("IntersectionObserver" in window)) return;
  new IntersectionObserver(([e]) => el.style.setProperty("--motion", e.isIntersecting ? "running" : "paused")).observe(el);
};

// The tour: the rail beside the screenshot works like the app's own, one screen per icon.
(() => {
  const img = document.getElementById("tour-img");
  if (!img) return;
  const alts = {
    home: "AniBeam's Home: shows to continue and shows not started yet, as poster cards",
    library: "The Library grid with every show as a poster",
    series: "A show's page with its banner, episodes and characters",
    player: "An episode playing in AniBeam's window",
  };
  const buttons = [...document.querySelectorAll(".rail button")];
  const warm = () => Object.keys(alts).forEach((k) => { new Image().src = `img/tour/${k}.webp`; });
  if ("requestIdleCallback" in window) requestIdleCallback(warm, { timeout: 3000 });
  else setTimeout(warm, 1500);

  function show(key, focus) {
    buttons.forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.screen === key)));
    img.classList.add("swap");
    const next = new Image();
    next.onload = () => { img.src = next.src; img.alt = alts[key]; img.classList.remove("swap"); };
    next.src = `img/tour/${key}.webp`;
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

// Files resolve into shows the first time the panel is mostly on screen.
(() => {
  const panel = document.getElementById("match");
  if (!panel) return;
  if (!("IntersectionObserver" in window)) { panel.classList.add("in"); return; }
  const io = new IntersectionObserver(([e]) => { if (e.isIntersecting) { panel.classList.add("in"); io.disconnect(); } }, { threshold: 0.45 });
  io.observe(panel);
})();

// The shelf: two rows of posters drifting in opposite directions. Each row holds its set twice
// so a move of half its width loops without a seam.
(() => {
  const box = document.getElementById("marquee");
  if (!box) return;
  const rows = {
    a: ["gits", "bebop", "eva", "akira", "lain", "perfectblue", "frieren", "mononoke", "champloo", "steinsgate", "fmab", "mob", "trigun", "paprika"],
    b: ["sac", "mushishi", "monster", "hxh", "planetes", "spirited", "yourname", "deathnote", "geass", "gurren", "flcl", "ergoproxy", "oddtaxi", "vinland"],
  };
  for (const track of box.querySelectorAll(".marquee-track")) {
    const set = rows[track.dataset.row];
    const html = set.map((s) => `<span class="poster"><img src="img/posters/${s}.webp" alt="" loading="lazy" width="156" height="234"></span>`).join("");
    track.innerHTML = html + html;
  }
  motionGate(box);
})();

// The wall: a sentence of what it does. The picked phrase lights up and the stage shows it; it
// steps on by itself every 5 s while at least a fifth of it is on screen, and a click resets it.
(() => {
  const wall = document.getElementById("wall");
  const stage = document.getElementById("stage");
  const bar = document.getElementById("wall-bar");
  if (!wall || !stage) return;
  const phrases = [...wall.querySelectorAll(".ph")];
  const items = [...stage.querySelectorAll(".stage-item")];
  const STEP = 5000;
  let i = 0, timer = 0, visible = false, run = null;

  function select(n) {
    i = (n + phrases.length) % phrases.length;
    const key = phrases[i].dataset.stage;
    phrases.forEach((p, k) => p.setAttribute("aria-pressed", String(k === i)));
    items.forEach((it) => it.classList.toggle("is-on", it.dataset.stage === key));
    restart();
  }
  function restart() {
    clearTimeout(timer);
    if (run) run.cancel();
    if (!visible || reduced) return;
    run = bar.animate([{ transform: "scaleX(0)" }, { transform: "scaleX(1)" }], { duration: STEP, easing: "linear", fill: "forwards" });
    timer = setTimeout(() => select(i + 1), STEP);
  }
  phrases.forEach((p, k) => {
    p.addEventListener("click", () => select(k));
    p.addEventListener("keydown", (e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); select(k); } });
  });
  if ("IntersectionObserver" in window) {
    new IntersectionObserver(([e]) => { visible = e.isIntersecting; restart(); }, { threshold: 0.2 }).observe(wall.parentElement);
  }
  motionGate(stage);
  select(0);
})();

// Install: tabs with a pill that slides under the picked one. Opens on the visitor's system.
(() => {
  const tabs = document.getElementById("tabs");
  if (!tabs) return;
  const buttons = [...tabs.querySelectorAll("[role=tab]")];
  function place(b) {
    tabs.style.setProperty("--x", `${b.offsetLeft}px`);
    tabs.style.setProperty("--w", `${b.offsetWidth}px`);
  }
  function choose(b, focus) {
    buttons.forEach((t) => {
      const on = t === b;
      t.setAttribute("aria-selected", String(on));
      t.tabIndex = on ? 0 : -1;
      document.getElementById(t.getAttribute("aria-controls")).hidden = !on;
    });
    place(b);
    if (focus) b.focus();
  }
  buttons.forEach((b, k) => {
    b.addEventListener("click", () => choose(b));
    b.addEventListener("keydown", (e) => {
      const step = { ArrowRight: 1, ArrowLeft: -1 }[e.key];
      if (step) { e.preventDefault(); choose(buttons[(k + step + buttons.length) % buttons.length], true); }
    });
  });
  const start = () => place(buttons.find((b) => b.getAttribute("aria-selected") === "true"));
  document.fonts ? document.fonts.ready.then(start) : start();
  addEventListener("resize", start);
  const follow = (os) => { const b = buttons.find((t) => t.dataset.os === os); if (b) choose(b); };
  if (document.documentElement.dataset.os) follow(document.documentElement.dataset.os);
  document.addEventListener("os", (e) => follow(e.detail));
})();

// Copy buttons on the install commands.
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
