#!/usr/bin/env python3
"""Writes <site>/releases.json: what the download buttons, the download page and install.sh link
to, read once from GitHub when the site deploys. The site deploys after every stable release
and every nightly, so the file is current without each visitor asking GitHub's API (which allows
60 unauthenticated calls an hour per address).

  {
    "stable":  {"tag", "version", "published", "url", "assets": {"appimage", "deb", "rpm", "dmg", "setup", "zip"}},
    "nightly": the same, or null when no nightly is newer than the stable release,
    "phone":   {"tag", "version", "published", "url", "assets": {"apk"}},
    "phone_nightly": the same, or null when no phone nightly is newer than the phone's release,
    "tags":    every published tag, for What's new to link only releases that exist
  }

usage: releases.py <site-dir>    GH_TOKEN, when set, authenticates the call
Without network, or when GitHub fails, nothing is written and the pages keep their plain links
to the releases page.
"""
import json
import os
import re
import sys
import urllib.request
from pathlib import Path

REPO = "marcusbandit/AniBeam"
MATCH = {
    "appimage": lambda n: n.endswith(".AppImage"),
    "deb": lambda n: n.endswith("_amd64.deb"),
    "rpm": lambda n: n.endswith(".x86_64.rpm"),
    "dmg": lambda n: n.endswith(".dmg"),
    "setup": lambda n: n.endswith("-setup.exe"),
    "zip": lambda n: bool(re.search(r"windows.*\.zip$", n)),
    "apk": lambda n: n.endswith(".apk"),
}


def fetch():
    headers = {"Accept": "application/vnd.github+json", "User-Agent": "anibeam-site-build"}
    if os.environ.get("GH_TOKEN"):
        headers["Authorization"] = f"Bearer {os.environ['GH_TOKEN']}"
    req = urllib.request.Request(f"https://api.github.com/repos/{REPO}/releases?per_page=100", headers=headers)
    return json.load(urllib.request.urlopen(req, timeout=30))


def shape(r, keys):
    if not r:
        return None
    assets = {}
    for k in keys:
        a = next((a for a in r["assets"] if MATCH[k](a["name"])), None)
        if a:
            assets[k] = a["browser_download_url"]
    return {"tag": r["tag_name"], "version": re.sub(r"^(android-)?v", "", r["tag_name"]),
            "published": r["published_at"], "url": r["html_url"], "assets": assets}


def semver(tag):
    return tuple(int(x) for x in re.findall(r"\d+", tag)[:3])


def main(site):
    try:
        releases = [r for r in fetch() if not r["draft"]]
    except Exception as e:  # no network, rate limit: keep the plain links
        print(f"releases: skipped ({e})")
        return
    stables = [r for r in releases if re.fullmatch(r"v\d+\.\d+\.\d+", r["tag_name"]) and not r["prerelease"]]
    stable = max(stables, key=lambda r: semver(r["tag_name"]), default=None)
    nightlies = [r for r in releases if re.match(r"v[\d.]+-nightly\.", r["tag_name"])]
    nightly = max(nightlies, key=lambda r: r["published_at"], default=None)
    # A nightly older than the stable release is behind it, not ahead: there is no nightly then.
    if nightly and stable and nightly["published_at"] <= stable["published_at"]:
        nightly = None
    # The phone's releases are android-v2.4.3; the ones made by hand before the Release workflow
    # shipped the phone (#794) went out as pre-releases, so they count too. Its nightlies are
    # android-v2.4.3-nightly.*, the same rule as the desktop's.
    phones = [r for r in releases if re.fullmatch(r"android-v\d+\.\d+\.\d+", r["tag_name"])]
    phone = max(phones, key=lambda r: semver(r["tag_name"]), default=None)
    phone_nightlies = [r for r in releases if re.match(r"android-v[\d.]+-nightly\.", r["tag_name"])]
    phone_nightly = max(phone_nightlies, key=lambda r: r["published_at"], default=None)
    if phone_nightly and phone and phone_nightly["published_at"] <= phone["published_at"]:
        phone_nightly = None
    desktop = ["appimage", "deb", "rpm", "dmg", "setup", "zip"]
    data = {
        "stable": shape(stable, desktop),
        "nightly": shape(nightly, desktop),
        "phone": shape(phone, ["apk"]),
        "phone_nightly": shape(phone_nightly, ["apk"]),
        "tags": sorted(r["tag_name"] for r in releases),
    }
    Path(site, "releases.json").write_text(json.dumps(data, indent=1) + "\n")
    print(f"releases: stable {data['stable'] and data['stable']['version']}, "
          f"nightly {data['nightly'] and data['nightly']['version']}, phone {data['phone'] and data['phone']['version']}, "
          f"phone nightly {data['phone_nightly'] and data['phone_nightly']['version']}")


if __name__ == "__main__":
    main(sys.argv[1])
