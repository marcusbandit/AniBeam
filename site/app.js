document.documentElement.classList.add("js");

// Points every download button at the file in the newest release and puts the visitor's own
// system first. Without JavaScript, or when GitHub does not answer, every link still goes to
// the releases page, so nothing here is required.
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

  // Only builds that exist: the dmg is Apple silicon, the AppImage x86_64. A system we
  // cannot place gets the plain releases link rather than a file that will not run.
  function detect() {
    const ua = navigator.userAgent;
    const platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
    if (/android/i.test(ua)) return "android";
    if (/iphone|ipad|ipod/i.test(ua)) return null;
    if (/win/i.test(platform) || /windows/i.test(ua)) return "windows";
    if (/mac/i.test(platform) || /mac os x/i.test(ua)) {
      return navigator.maxTouchPoints > 1 ? null : "mac"; // an iPad asks for the desktop site as a Mac
    }
    if (/aarch64|arm/i.test(platform)) return null;
    if (/linux|x11/i.test(platform) || /linux/i.test(ua)) return "linux";
    return null;
  }

  let os = detect();
  const mains = document.querySelectorAll(".dl-auto");
  const phoneReleases = `https://github.com/${repo}/releases?q=android&expanded=true`;

  // The CPU check and the release lookup can finish in either order; whichever comes last
  // points the main button at the file.
  const urls = {};
  function aim() {
    if (os && urls[mainAsset[os]]) mains.forEach((b) => { b.href = urls[mainAsset[os]]; });
  }

  function pick(chosen) {
    os = chosen;
    if (!os) return;
    for (const b of mains) {
      b.textContent = os === "android" ? "Get the Android app" : `Download for ${names[os]}`;
      if (os === "android") b.href = phoneReleases;
    }
    aim();
    const row = document.querySelector(`.platform[data-os="${os}"]`);
    if (row) {
      row.classList.add("yours");
      row.parentElement.prepend(row);
    }
  }

  // Chromium can say the CPU; an Intel Mac has no build, so it gets no "Download for Mac".
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
      for (const link of document.querySelectorAll(`[data-asset="${key}"]`)) {
        link.href = asset.browser_download_url;
      }
      urls[key] = asset.browser_download_url;
    }
  }

  async function get(url) {
    const res = await fetch(url, { headers: { Accept: "application/vnd.github+json" } });
    if (!res.ok) throw new Error(`${res.status}`);
    return res.json();
  }

  // One call for both: the release list holds the desktop stable (v*, not a pre-release) and
  // the phone's (android-v*). Unauthenticated calls are limited to 60 an hour per address.
  get(`${api}?per_page=100`)
    .then((list) => {
      const live = list.filter((r) => !r.draft);
      const stable = live.find((r) => /^v\d/.test(r.tag_name) && !r.prerelease);
      const phone = live.find((r) => r.tag_name.startsWith("android-v"));
      wire(stable, ["appimage", "setup", "zip", "dmg"]);
      wire(phone, ["apk"]);
      aim();
      if (stable) {
        const version = stable.tag_name.replace(/^v/, "");
        const when = new Date(stable.published_at).toLocaleDateString("en-GB", { day: "numeric", month: "long", year: "numeric" });
        const note = document.getElementById("install-version");
        note.textContent = `AniBeam ${version}, released ${when}. `;
        const all = document.createElement("a");
        all.href = stable.html_url;
        all.textContent = "What's in it";
        note.append(all);
      }
      const shown = os === "android" ? phone : stable;
      if (shown) {
        const version = shown.tag_name.replace(/^(android-)?v/, "");
        document.getElementById("dl-note").textContent = `Version ${version}. Free and open source.`;
      }
    })
    .catch(() => {});
})();

// The tour: the rail beside the screenshot works like the app's own, one screen per icon.
(() => {
  const img = document.getElementById("tour-img");
  const caption = document.getElementById("tour-caption");
  if (!img) return;
  const screens = {
    home: ["Home picks up where you left off and shows what just landed in your folder.", "Home: recently downloaded shows and shows not started yet, as poster cards with episode progress"],
    library: ["The library holds every show in the folder, matched and with its poster.", "The Library grid with every show as a poster"],
    airing: ["Airing shows what is out this week and when each next episode lands.", "The Airing page: shows out in the last two days, each with time until its next episode"],
    series: ["A show's page has its episodes, your progress and everything it connects to.", "A series page with its banner, episode list and watch progress"],
    player: ["The player runs right in the window, with subtitles and audio a click away.", "An episode playing in the window with the subtitle menu open"],
  };
  const buttons = [...document.querySelectorAll(".rail button")];
  // Fetch the other screens once the page is idle, so a click swaps without a wait.
  const warm = () => Object.keys(screens).forEach((k) => { const i = new Image(); i.src = `img/tour/${k}.webp`; });
  if ("requestIdleCallback" in window) requestIdleCallback(warm, { timeout: 3000 });
  else setTimeout(warm, 1500);

  function show(key, focus) {
    for (const b of buttons) b.setAttribute("aria-pressed", String(b.dataset.screen === key));
    img.classList.add("swap");
    const next = new Image();
    next.onload = () => { img.src = next.src; img.alt = screens[key][1]; img.classList.remove("swap"); };
    next.src = `img/tour/${key}.webp`;
    caption.textContent = screens[key][0];
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

// The theme row: each button shows Home in that theme, captured from the app itself.
(() => {
  const img = document.getElementById("theme-img");
  const picks = document.getElementById("theme-picks");
  if (!img || !picks) return;
  // name, file, the theme's ground and accent for the swatch
  const themes = [
    ["AniBeam Dark", "anibeam-dark", "#07090f", "#46e0c4"],
    ["Catppuccin Mocha", "catppuccin-mocha", "#1e1e2e", "#89b4fa"],
    ["Gruvbox", "gruvbox-dark-medium", "#282828", "#83a598"],
    ["Everforest", "everforest-dark-medium", "#2d353b", "#7fbbb3"],
    ["Nord", "nord", "#2e3440", "#81a1c1"],
    ["Catppuccin Latte", "catppuccin-latte", "#eff1f5", "#1e66f5"],
  ];
  let warmed = false;
  function apply(i) {
    const [name, file] = themes[i];
    img.classList.add("swap");
    const next = new Image();
    next.onload = () => { img.src = next.src; img.alt = `Home in the ${name} theme`; img.classList.remove("swap"); };
    next.src = `img/themes/${file}.webp`;
    picks.querySelectorAll("button").forEach((b, n) => b.setAttribute("aria-pressed", String(n === i)));
  }
  themes.forEach(([name, file, ground, accent], i) => {
    const b = document.createElement("button");
    b.type = "button";
    b.setAttribute("aria-pressed", String(i === 0));
    b.innerHTML = `<span class="sw" style="background:linear-gradient(135deg, ${ground} 55%, ${accent} 55%)"></span>${name}`;
    b.addEventListener("click", () => apply(i));
    b.addEventListener("pointerenter", () => {
      if (warmed) return;
      warmed = true;
      themes.forEach(([, f]) => { new Image().src = `img/themes/${f}.webp`; });
    });
    picks.append(b);
  });
})();

// Each feature's small UI plays its one action the first time it is mostly on screen.
(() => {
  const vigs = document.querySelectorAll(".vig");
  if (!("IntersectionObserver" in window)) { vigs.forEach((v) => v.classList.add("in")); return; }
  const io = new IntersectionObserver((entries) => {
    for (const e of entries) if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); }
  }, { threshold: 0.55 });
  vigs.forEach((v) => io.observe(v));
})();
