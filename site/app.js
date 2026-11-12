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
  const main = document.getElementById("dl-main");
  const phoneReleases = `https://github.com/${repo}/releases?q=android&expanded=true`;

  // The CPU check and the release lookup can finish in either order; whichever comes last
  // points the main button at the file.
  const urls = {};
  function aim() {
    if (os && urls[mainAsset[os]]) main.href = urls[mainAsset[os]];
  }

  function pick(chosen) {
    os = chosen;
    if (!os) return;
    main.textContent = os === "android" ? "Get the Android app" : `Download for ${names[os]}`;
    if (os === "android") main.href = phoneReleases;
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
