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
