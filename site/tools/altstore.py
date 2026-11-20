"""Writes the iPhone app's source, <site>/altstore/source.json, for AltStore, SideStore and
LiveContainer to subscribe to: a JSON feed of the app's versions, after which the app sits in their
library and they offer each new build as an update. releases.py calls it with the GitHub releases it
already read, so the feed and the download buttons see the same data.

The format is AltStore's source format, which the other two read (AltStore Classic's Source,
StoreApp and AppVersion models, and LiveContainer's LCAltStoreSourcesView). Top level: name,
identifier (AltStore Classic refuses a source without one), apps. App: name, bundleIdentifier,
developerName, localizedDescription, iconURL, versions, appPermissions. Version: version, date,
downloadURL, size, plus buildVersion, minOSVersion and localizedDescription. The versions array is
ordered, newest first, because AltStore takes the first entry as the latest.

One source, two apps, each with a bundle id of its own so both install side by side:

  Anibeam          com.marcusrosado.anibeam          the phone's releases (android-v2.4.3)
  Anibeam Nightly  com.marcusrosado.anibeam.nightly  the phone's nightlies (android-v2.4.3-nightly.N.M)

An app with no IPA yet is left out, so the source is valid with two apps, one or none.

Nightlies published before the nightly bundle id (#944) carry the stable id inside their IPA, and
AltStore refuses an IPA whose bundle id differs from its app's. They cannot be told apart by tag,
so nightly.yml names the IPA of a nightly with the nightly id Anibeam-Nightly-<version>.ipa, and
only nightlies holding such an asset are listed; the old ones are skipped.

An IPA's short version is the base of the release's version and its build number the run number
of a nightly, 1 for a release (apps/phone/scripts/build-ios.sh); AltStore checks a downloaded
IPA against them, so they are derived the same way here.
"""
import json
import plistlib
import re
from pathlib import Path

BASE = "https://anibeam.app"
IOS = Path(__file__).resolve().parent.parent.parent / "apps" / "phone" / "ios"
SOURCE = {
    "name": "Anibeam",
    "identifier": "com.marcusrosado.anibeam.source",
    "subtitle": "Anibeam for iPhone",
    "description": "Anibeam for iPhone, and Anibeam Nightly, the newest build from main, side by side. Add this source and both show up in your library, each new build as an update.",
    "icon": "img/icon.png",
}
APPS = {
    "stable": {
        "name": "Anibeam",
        "bundle_id": "com.marcusrosado.anibeam",
        "app_subtitle": "Browse and play your anime library",
        "icon": "img/icon.png",
    },
    "nightly": {
        "name": "Anibeam Nightly",
        "bundle_id": "com.marcusrosado.anibeam.nightly",
        "app_subtitle": "Nightly build of Anibeam",
        # The nightly mark, assets/logo/logo-nightly.png, which site/build.sh copies here.
        "icon": "img/icon-nightly.png",
    },
}
# nightly.yml names a nightly's IPA with this prefix once it is built with the nightly bundle id.
# Builds before 2.5 spelled it with a capital B, so it matches in any case.
NIGHTLY_IPA = "Anibeam-Nightly-"
APP_DESCRIPTION = ("Anibeam browses and plays your anime library, keeps your AniList and MyAnimeList "
                   "progress in step, and talks to Anibeam on your computer.")


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


def entries(releases, channel, min_os):
    out = []
    for r in releases:
        if r.get("draft"):
            continue
        v = version_of(r["tag_name"])
        if not v:
            continue
        nightly = "-nightly." in r["tag_name"]
        if nightly != (channel == "nightly"):
            continue
        ipa = next((a for a in r["assets"] if a["name"].endswith(".ipa")
                    and (not nightly or a["name"].lower().startswith(NIGHTLY_IPA.lower()))), None)
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


def feed(versions):
    apps = []
    for channel in ("stable", "nightly"):
        c = APPS[channel]
        if not versions[channel]:
            continue
        apps.append({
            "name": c["name"],
            "bundleIdentifier": c["bundle_id"],
            "developerName": "marcusbandit",
            "subtitle": c["app_subtitle"],
            "localizedDescription": APP_DESCRIPTION,
            "iconURL": f"{BASE}/{c['icon']}",
            "category": "entertainment",
            "versions": versions[channel],
            "appPermissions": {"entitlements": [], "privacy": privacy()},
        })
    return {
        "name": SOURCE["name"],
        "identifier": SOURCE["identifier"],
        "sourceURL": f"{BASE}/altstore/source.json",
        "subtitle": SOURCE["subtitle"],
        "description": SOURCE["description"],
        "iconURL": f"{BASE}/{SOURCE['icon']}",
        "website": BASE,
        "nsfw": False,
        "apps": apps,
        "news": [],
    }


def write(site, releases):
    out = Path(site, "altstore")
    out.mkdir(parents=True, exist_ok=True)
    min_os = deployment_target()
    versions = {c: entries(releases, c, min_os) for c in APPS}
    (out / "source.json").write_text(json.dumps(feed(versions), indent=2) + "\n")
    print(f"altstore: stable {len(versions['stable'])} versions, nightly {len(versions['nightly'])} versions")
