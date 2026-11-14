"""Writes the iPhone app's sources, <site>/altstore/stable.json and nightly.json, for AltStore,
SideStore and LiveContainer to subscribe to: a JSON feed of the app's versions, after which the app
sits in their library and they offer each new build as an update. releases.py calls it with the
GitHub releases it already read, so the feeds and the download buttons see the same data.

The format is AltStore's source format, which the other two read (AltStore Classic's Source,
StoreApp and AppVersion models, and LiveContainer's LCAltStoreSourcesView). Top level: name,
identifier (AltStore Classic refuses a source without one), apps. App: name, bundleIdentifier,
developerName, localizedDescription, iconURL, versions, appPermissions. Version: version, date,
downloadURL, size, plus buildVersion, minOSVersion and localizedDescription. The versions array is
ordered, newest first, because AltStore takes the first entry as the latest.

  stable.json   the phone's releases (android-v2.4.3) that carry an IPA
  nightly.json  the phone's nightlies and its releases together, so a nightly subscriber is never
                left behind a newer stable release; the newest build, whichever it is, comes first

An IPA's short version is the base of the release's version and its build number the run number
of a nightly, 1 for a release (apps/android/scripts/build-ios.sh); AltStore checks a downloaded
IPA against them, so they are derived the same way here. A source with no IPA yet has an empty
apps list, which every app accepts.
"""
import json
import plistlib
import re
from pathlib import Path

BASE = "https://anibeam.app"
BUNDLE_ID = "com.marcusrosado.anibeam"
IOS = Path(__file__).resolve().parent.parent.parent / "apps" / "android" / "ios"
CHANNELS = {
    "stable": {
        "name": "AniBeam",
        "identifier": "com.marcusrosado.anibeam.source",
        "subtitle": "AniBeam for iPhone",
        "description": "Releases of AniBeam for iPhone. Add this source and AniBeam shows up in your library with each new release as an update.",
        "app_subtitle": "Browse and play your anime library",
    },
    "nightly": {
        "name": "AniBeam Nightly",
        "identifier": "com.marcusrosado.anibeam.source.nightly",
        "subtitle": "The newest AniBeam for iPhone, built from main",
        "description": "The newest AniBeam builds for iPhone, built from main and less polished than a release. Newer releases appear here too.",
        "app_subtitle": "Nightly build of AniBeam",
    },
}
APP_DESCRIPTION = ("AniBeam browses and plays your anime library, keeps your AniList and MyAnimeList "
                   "progress in step, and talks to AniBeam on your computer.")


def deployment_target():
    m = re.search(r'deploymentTarget:\s*"?([\d.]+)"?', (IOS / "project.yml").read_text())
    return m.group(1) if m else None


def privacy():
    """Every UsageDescription in the app's Info.plist: AltStore refuses an IPA that has one the
    source does not declare."""
    plist = plistlib.loads((IOS / "Info.plist").read_bytes())
    return {k: v for k, v in sorted(plist.items()) if k.endswith("UsageDescription")}


def version_of(tag):
    """(short version, build number) as build-ios.sh stamps them, from android-v2.4.3 or
    android-v2.4.3-nightly.20261113.41."""
    m = re.fullmatch(r"android-v(\d+\.\d+\.\d+)(?:-nightly\.\d+\.(\d+))?", tag)
    if not m:
        return None
    build = m.group(2) if m.group(2) and int(m.group(2)) > 0 else "1"
    return m.group(1), build


def entries(releases, kinds, min_os):
    out = []
    for r in releases:
        if r.get("draft"):
            continue
        v = version_of(r["tag_name"])
        if not v:
            continue
        nightly = "-nightly." in r["tag_name"]
        if ("nightly" if nightly else "stable") not in kinds:
            continue
        ipa = next((a for a in r["assets"] if a["name"].endswith(".ipa")), None)
        if not ipa:
            continue
        entry = {
            "version": v[0],
            "buildVersion": v[1],
            "marketingVersion": re.sub(r"^android-v", "", r["tag_name"]),
            "date": r["published_at"],
            "localizedDescription": f"{'Nightly build' if nightly else 'Release'} {re.sub('^android-v', '', r['tag_name'])}",
            "downloadURL": ipa["browser_download_url"],
            "size": ipa["size"],
        }
        if min_os:
            entry["minOSVersion"] = min_os
        out.append(entry)
    out.sort(key=lambda e: e["date"], reverse=True)
    return out


def feed(channel, versions):
    c = CHANNELS[channel]
    apps = []
    if versions:
        apps.append({
            "name": c["name"],
            "bundleIdentifier": BUNDLE_ID,
            "developerName": "marcusbandit",
            "subtitle": c["app_subtitle"],
            "localizedDescription": APP_DESCRIPTION,
            "iconURL": f"{BASE}/img/icon.png",
            "category": "entertainment",
            "versions": versions,
            "appPermissions": {"entitlements": [], "privacy": privacy()},
        })
    return {
        "name": c["name"],
        "identifier": c["identifier"],
        "sourceURL": f"{BASE}/altstore/{channel}.json",
        "subtitle": c["subtitle"],
        "description": c["description"],
        "iconURL": f"{BASE}/img/icon.png",
        "website": BASE,
        "nsfw": False,
        "apps": apps,
        "news": [],
    }


def write(site, releases):
    out = Path(site, "altstore")
    out.mkdir(parents=True, exist_ok=True)
    min_os = deployment_target()
    counts = {}
    for channel, kinds in (("stable", {"stable"}), ("nightly", {"stable", "nightly"})):
        versions = entries(releases, kinds, min_os)
        counts[channel] = len(versions)
        (out / f"{channel}.json").write_text(json.dumps(feed(channel, versions), indent=2) + "\n")
    print(f"altstore: stable {counts['stable']} versions, nightly {counts['nightly']} versions")
