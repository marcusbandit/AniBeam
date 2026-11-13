// What every page shares, loaded before its own script: the translated strings, the site's root,
// the visitor's system, the release links and the copy buttons. One copy of each, so the home
// page and the download page cannot drift apart.
document.documentElement.classList.add("js");

const AB = (() => {
  // The site root is where this file lives, so it is right from any folder, translated pages too.
  const ROOT = new URL("./", document.currentScript.src).href;

  // A translated page carries its strings in window.I18N (site/tools/i18n.py), swapped by lang.js
  // when the language changes; English is the fallback.
  const t = (s, vars = {}) => ((window.I18N || {})[s] || s).replace(/\{(\w+)\}/g, (_, k) => vars[k] ?? `{${k}}`);
  const locale = () => (window.I18N && window.I18N._locale) || document.documentElement.lang || "en";
  const names = { linux: "Linux", windows: "Windows", mac: "Mac", android: "Android", ios: "iPhone" };

  // The visitor's system: linux, mac, windows, android, ios, or null when it is none we build for
  // (ARM Linux, an Intel Mac Chromium can name). A Mac whose chip the browser will not say is
  // "mac" with mac.sure false: Safari and Firefox never tell, so the dmg is offered with its chip.
  function detect() {
    const ua = navigator.userAgent;
    const platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
    if (/android/i.test(ua)) return "android";
    if (/iphone|ipad|ipod/i.test(ua)) return "ios";
    if (/win/i.test(platform) || /windows/i.test(ua)) return "windows";
    if (/mac/i.test(platform) || /mac os x/i.test(ua)) return navigator.maxTouchPoints > 1 ? "ios" : "mac"; // an iPad asks for the desktop site
    if (/aarch64|arm/i.test(platform)) return null;
    if (/linux|x11/i.test(platform) || /linux/i.test(ua)) return "linux";
    return null;
  }
  const os = detect();
  const phone = os === "android" || os === "ios";
  document.documentElement.dataset.os = os || "other";
  if (phone) document.documentElement.dataset.device = "phone";

  // Whether the Mac is Apple silicon: yes, no, or unknown (null) where the browser will not say.
  const macChip = (os !== "mac" || !(navigator.userAgentData && navigator.userAgentData.getHighEntropyValues))
    ? Promise.resolve(null)
    : navigator.userAgentData.getHighEntropyValues(["architecture"]).then((v) => v.architecture !== "x86").catch(() => null);

  // The release links, written when the site deploys (tools/releases.py). When the file is missing
  // the links stay on the releases page.
  let releases;
  const loadReleases = () => (releases ??= fetch(ROOT + "releases.json").then((r) => (r.ok ? r.json() : null)).catch(() => null));

  // Copy buttons: copy the command, say so out loud for screen readers, and when the clipboard is
  // refused, select the command so a keyboard copy works.
  const live = document.createElement("span");
  live.className = "visually-hidden";
  live.setAttribute("aria-live", "polite");
  document.addEventListener("DOMContentLoaded", () => document.body.append(live));
  document.addEventListener("click", async (e) => {
    const b = e.target.closest(".copy");
    if (!b) return;
    try {
      await navigator.clipboard.writeText(b.dataset.copy);
      b.textContent = t("Copied");
      b.classList.add("done");
      live.textContent = t("Copied");
      setTimeout(() => { b.textContent = t("Copy"); b.classList.remove("done"); }, 1600);
    } catch {
      const code = b.parentElement.querySelector("code");
      if (code) getSelection().selectAllChildren(code);
      live.textContent = t("Selected. Press Ctrl+C to copy.");
    }
  });

  return { ROOT, t, locale, names, os, phone, macChip, releases: loadReleases };
})();
