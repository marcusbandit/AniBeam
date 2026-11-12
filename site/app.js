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

  function detect() {
    const ua = navigator.userAgent;
    const platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
    if (/android/i.test(ua)) return "android";
    if (/iphone|ipad|ipod/i.test(ua)) return null;
    if (/win/i.test(platform) || /windows/i.test(ua)) return "windows";
    if (/mac/i.test(platform) || /mac os x/i.test(ua)) return "mac";
    if (/linux|x11/i.test(platform) || /linux/i.test(ua)) return "linux";
    return null;
  }

  const os = detect();
  const main = document.getElementById("dl-main");

  if (os) {
    main.textContent = os === "android" ? "Get the Android app" : `Download for ${names[os]}`;
    const row = document.querySelector(`.platform[data-os="${os}"]`);
    if (row) {
      row.classList.add("yours");
      row.parentElement.prepend(row);
    }
  }

  function wire(release, keys) {
    if (!release) return;
    for (const key of keys) {
      const asset = release.assets.find((a) => match[key](a.name));
      if (!asset) continue;
      for (const link of document.querySelectorAll(`[data-asset="${key}"]`)) {
        link.href = asset.browser_download_url;
      }
      if (os && mainAsset[os] === key) main.href = asset.browser_download_url;
    }
  }

  async function get(url) {
    const res = await fetch(url, { headers: { Accept: "application/vnd.github+json" } });
    if (!res.ok) throw new Error(`${res.status}`);
    return res.json();
  }

  get(`${api}/latest`)
    .then((stable) => {
      wire(stable, ["appimage", "setup", "zip", "dmg"]);
      const version = stable.tag_name.replace(/^v/, "");
      const when = new Date(stable.published_at).toLocaleDateString("en-GB", { day: "numeric", month: "long", year: "numeric" });
      const note = document.getElementById("install-version");
      note.textContent = `AniBeam ${version}, released ${when}. `;
      const all = document.createElement("a");
      all.href = stable.html_url;
      all.textContent = "What's in it";
      note.append(all);
      if (os !== "android") {
        document.getElementById("dl-note").textContent = `Version ${version}. Free and open source.`;
      }
    })
    .catch(() => {});

  get(`${api}?per_page=30`)
    .then((list) => {
      const phone = list.find((r) => r.tag_name.startsWith("android-v") && !r.draft);
      wire(phone, ["apk"]);
      if (phone && os === "android") {
        document.getElementById("dl-note").textContent = `Version ${phone.tag_name.replace(/^android-v/, "")}. Free and open source.`;
      }
    })
    .catch(() => {});
})();
