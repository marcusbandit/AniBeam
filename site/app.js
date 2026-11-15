// The home page. Shared pieces (strings, the visitor's system, the release links, copy buttons)
// come from common.js, which loads first.
// Pauses infinite motion (the hero's lanes, the cast lines) while it is off screen.
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

motionGate(document.querySelector(".beams"));
motionGate(document.querySelector(".cast-scene"));

motionGate(document.getElementById("lanes"));

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
