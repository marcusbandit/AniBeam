document.documentElement.classList.add("js");
// A translated page carries its strings in window.I18N (site/tools/i18n.py); English is the
// fallback, so a page without it reads as written here.
const t = (s, vars = {}) => ((window.I18N || {})[s] || s).replace(/\{(\w+)\}/g, (_, k) => vars[k] ?? `{${k}}`);
const locale = () => (window.I18N && window.I18N._locale) || "en-GB";
// Paths the script builds start from the site root; a translated page sits one folder down.
// Read each time, since switching language changes it (lang.js).
const root = () => (window.I18N && window.I18N._root) || "";
const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;

// Pauses infinite motion (the marquee, the beams) while it is off screen.
const motionGate = (el) => {
  if (!el || !("IntersectionObserver" in window)) return;
  new IntersectionObserver(([e]) => el.style.setProperty("--motion", e.isIntersecting ? "running" : "paused")).observe(el);
};

// Adds `in` the first time an element is mostly on screen, which plays its one action.
const onceInView = (els, threshold = 0.45) => {
  if (!("IntersectionObserver" in window)) { els.forEach((el) => el.classList.add("in")); return; }
  const io = new IntersectionObserver((entries) => {
    for (const e of entries) if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); }
  }, { threshold });
  els.forEach((el) => io.observe(el));
};

// Points every download button at the file in the newest release and puts the visitor's own
// system first. Without JavaScript, or when GitHub does not answer, every link still goes to the
// releases page, so nothing here is required.
(() => {
  const repo = "marcusbandit/AniBeam";
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
  // the hero's download button and the one under Get AniBeam
  const mains = [...document.querySelectorAll(".dl-auto")];
  const urls = {};
  const aim = () => { if (os && urls[mainAsset[os]]) mains.forEach((m) => { m.href = urls[mainAsset[os]]; }); };

  function label() {
    if (!os) return;
    for (const m of mains) m.querySelector("span").textContent = os === "android" ? t("Get the Android app") : t("Download for {os}", { os: names[os] });
  }
  document.addEventListener("langchange", label);
  function pick(chosen) {
    os = chosen;
    if (!os) return;
    label();
    for (const m of mains) if (os === "android") m.href = `https://github.com/${repo}/releases?q=android&expanded=true`;
    aim();
    // The visitor's card lights up, and its download for their system gets the yellow button.
    const card = document.querySelector(`.plat[data-os~="${os}"], .phone[data-os="${os}"]`);
    if (card) {
      card.classList.add("is-yours");
      const half = card.querySelector(`.os-half[data-for="${os}"]`);
      if (half) half.classList.add("is-yours");
      const btn = card.querySelector(`.btn-ghost[data-for="${os}"]`) || card.querySelector(".btn-ghost");
      if (btn) { btn.classList.remove("btn-ghost"); btn.classList.add("btn-primary"); }
    }
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
  fetch(`https://api.github.com/repos/${repo}/releases?per_page=100`, { headers: { Accept: "application/vnd.github+json" } })
    .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
    .then((list) => {
      const live = list.filter((r) => !r.draft);
      wire(live.find((r) => /^v\d/.test(r.tag_name) && !r.prerelease), ["appimage", "setup", "zip", "dmg"]);
      wire(live.find((r) => r.tag_name.startsWith("android-v")), ["apk"]);
      aim();
    })
    .catch(() => {});
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
  const warm = () => Object.keys(alts).forEach((k) => { new Image().src = `${root()}img/tour/${k}.webp`; });
  if ("requestIdleCallback" in window) requestIdleCallback(warm, { timeout: 3000 });
  else setTimeout(warm, 1500);

  function show(key, focus) {
    buttons.forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.screen === key)));
    img.classList.add("swap");
    const next = new Image();
    next.onload = () => { img.src = next.src; img.alt = alts[key]; img.classList.remove("swap"); };
    next.src = `${root()}img/tour/${key}.webp`;
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

// The shelf: two rows of posters drifting in opposite directions, old and new together. Each
// row holds its set twice so a move of half its width loops without a seam.
(() => {
  const box = document.getElementById("marquee");
  if (!box) return;
  const rows = {
    a: ["frieren", "bebop", "aot", "spirited", "jjk", "eva", "csm", "akira", "spyfamily", "gits", "demonslayer", "mononoke", "oshinoko", "fmab", "dandadan", "perfectblue"],
    b: ["deathnote", "violet", "steinsgate", "sololeveling", "hxh", "bocchi", "naruto", "kaguya", "onepiece", "madeinabyss", "dbz", "edgerunners", "sailormoon", "haikyuu", "mha", "opm", "yourname", "mob", "vinland", "champloo", "geass", "gurren"],
  };
  for (const track of box.querySelectorAll(".marquee-track")) {
    const html = rows[track.dataset.row].map((s) => `<span class="poster"><img src="${root()}img/posters/${s}.webp" alt="" loading="lazy" width="160" height="240"></span>`).join("");
    track.innerHTML = html + html;
  }
  motionGate(box);
})();

// Each section plays its one action the first time it is mostly on screen.
onceInView([...document.querySelectorAll("#match, .feat")], 0.4);

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

// Copy buttons on the install commands.
document.querySelectorAll(".copy").forEach((b) => {
  b.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(b.dataset.copy);
      b.textContent = t("Copied");
      b.classList.add("done");
      setTimeout(() => { b.textContent = t("Copy"); b.classList.remove("done"); }, 1600);
    } catch {}
  });
});
