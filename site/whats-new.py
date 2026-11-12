#!/usr/bin/env python3
"""Builds the website's What's new pages from the apps' own patch notes (#777):

  <out>/whats-new/index.html            every version, newest first
  <out>/whats-new/<version>/index.html  one page per version

The desktop's apps/linux/CHANGELOG.md and the phone's apps/android/CHANGELOG.md hold the shipped
versions. The change files in apps/*/changes/ hold what has merged since the last stable release
and ships in the next nightly; they show as "Coming next". The page for a version that ships
appears with the site deploy the release starts, and the Coming next block with the one each
nightly starts, so nothing here is written by hand.

usage: whats-new.py <repo-root> <out-dir>
"""
import html
import os
import re
import sys
from pathlib import Path

root, out = Path(sys.argv[1]), Path(sys.argv[2])
APPS = [("desktop", "Desktop", root / "apps/linux"), ("phone", "Android", root / "apps/android")]
SECTIONS = ["New", "Changed", "Fixed", "Removed"]
HEAD = re.compile(r"^## (\d+\.\d+\.\d+)(?: · (.+?))? · (\d{4}-\d{2}-\d{2})\s*$")


def parse(text):
    """[(version, title, date, {section: [lines]})], newest first, as the file has them."""
    text = re.sub(r"<!--.*?-->", "", text, flags=re.S)
    versions, cur, section = [], None, None
    for line in text.splitlines():
        m = HEAD.match(line)
        if m:
            cur = {"version": m[1], "title": m[2] or "", "date": m[3], "sections": {}}
            versions.append(cur)
            section = None
            continue
        if cur is None:
            continue
        if line.startswith("### "):
            section = line[4:].strip()
            cur["sections"].setdefault(section, [])
        elif line.startswith("- ") and section:
            cur["sections"][section].append(line[2:].strip())
    return versions


def pending(changes_dir):
    """The lines in the change files, merged by section."""
    sections = {}
    for f in sorted(changes_dir.glob("*.md")):
        if f.name.lower() == "readme.md":
            continue
        section = None
        for line in f.read_text().splitlines():
            if line.startswith("### "):
                section = line[4:].strip()
            elif line.startswith("- ") and section:
                sections.setdefault(section, []).append(line[2:].strip())
    return sections


def inline(text):
    text = html.escape(text)
    return re.sub(r"`([^`]+)`", r"<code>\1</code>", text)


def nice_date(iso):
    y, m, d = iso.split("-")
    months = ["January", "February", "March", "April", "May", "June", "July", "August",
              "September", "October", "November", "December"]
    return f"{int(d)} {months[int(m) - 1]} {y}"


# Collect: one entry per version number, holding each app's notes for it.
by_version = {}
for key, name, path in APPS:
    for v in parse((path / "CHANGELOG.md").read_text()):
        if not any(v["sections"].values()):
            continue
        e = by_version.setdefault(v["version"], {"version": v["version"], "title": "", "date": v["date"], "apps": {}})
        e["title"] = e["title"] or (v["title"] if key == "desktop" else "")
        e["date"] = max(e["date"], v["date"])
        e["apps"][key] = v["sections"]


def vkey(v):
    return tuple(int(x) for x in v.split("."))


versions = sorted(by_version.values(), key=lambda e: (vkey(e["version"]), e["date"]), reverse=True)
coming = {key: pending(path / "changes") for key, _, path in APPS}
coming = {k: v for k, v in coming.items() if any(v.values())}
# site.yml passes the newest nightly when nightly.yml starts the deploy.
nightly_version = os.environ.get("ANIBEAM_NIGHTLY", "")


def notes_html(apps):
    parts = []
    for key, name, _ in APPS:
        sections = apps.get(key)
        if not sections:
            continue
        parts.append(f'<div class="wn-app"><h3>{name}</h3>')
        for s in SECTIONS + [x for x in sections if x not in SECTIONS]:
            lines = sections.get(s)
            if not lines:
                continue
            items = "".join(f"<li>{inline(l)}</li>" for l in lines)
            parts.append(f'<div class="wn-sec wn-{s.lower()}"><span class="wn-tag">{html.escape(s)}</span><ul>{items}</ul></div>')
        parts.append("</div>")
    return "".join(parts)


# The nav and the footer come from the download page, which sits at the same depth as the
# What's new index, so the pages share them exactly; deeper pages climb one more level.
_dl = (out / "download/index.html").read_text()
NAV = re.search(r'<header class="nav">.*?</header>', _dl, re.S).group(0)
FOOT = re.search(r'<footer class="foot">.*?</footer>', _dl, re.S).group(0)


def chrome(fragment, prefix):
    if prefix == "../":
        return fragment
    return re.sub(r'(href|src)="\.\./', lambda m: f'{m.group(1)}="{prefix}', fragment)


def page(title, prefix, body, canonical):
    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{html.escape(title)}</title>
<link rel="canonical" href="https://anibeam.app/{canonical}">
<meta name="theme-color" content="#07080a">
<meta name="color-scheme" content="dark">
<link rel="icon" href="{prefix}img/logo.svg" type="image/svg+xml">
<meta property="og:site_name" content="AniBeam">
<meta property="og:title" content="{html.escape(title)}">
<meta property="og:image" content="https://anibeam.app/img/og.jpg">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Archivo:wdth,wght@62..125,100..900&family=JetBrains+Mono:wght@400;500&display=swap">
<link rel="stylesheet" href="{prefix}style.css">
</head>
<body>
{chrome(NAV, prefix)}
<main class="wn-page">
{body}
</main>
{chrome(FOOT, prefix)}
</body>
</html>
"""


def entry(e, prefix, link=True, level="h2"):
    v = e["version"]
    # A version only the phone shipped lives on its android-v release.
    tag = f"v{v}" if "desktop" in e["apps"] else f"android-v{v}"
    title = f'<span class="wn-title">{html.escape(e["title"])}</span>' if e["title"] else ""
    name = f'<a href="{prefix}whats-new/{v}/">{v}</a>' if link else v
    return f"""<article class="wn-entry" id="v{v}">
  <header class="wn-head"><{level} class="wn-ver">{name}</{level}>{title}<time datetime="{e['date']}">{nice_date(e['date'])}</time></header>
  {notes_html(e['apps'])}
  <a class="wn-dl" href="https://github.com/marcusbandit/AniBeam/releases/tag/{tag}">Download {v}</a>
</article>"""


# The index
blocks = []
if coming:
    label = f"Coming next, in {nightly_version}" if nightly_version else "Coming next"
    blocks.append(f"""<article class="wn-entry wn-coming" id="next">
  <header class="wn-head"><h2 class="wn-ver">{html.escape(label)}</h2><a class="wn-try" href="../download/#nightly">Try the nightly</a></header>
  {notes_html(coming)}
</article>""")
blocks += [entry(e, "../") for e in versions]
index = f"""<section class="wn-hero"><h1>What's new</h1></section>
<div class="wn-list">
{chr(10).join(blocks)}
</div>"""
(out / "whats-new").mkdir(parents=True, exist_ok=True)
(out / "whats-new/index.html").write_text(page("What's new in AniBeam", "../", index, "whats-new/"))

# One page per version
for i, e in enumerate(versions):
    v = e["version"]
    newer = versions[i - 1]["version"] if i > 0 else None
    older = versions[i + 1]["version"] if i + 1 < len(versions) else None
    nav = '<nav class="wn-pager">'
    nav += f'<a href="../{older}/">{older}</a>' if older else "<span></span>"
    nav += '<a href="../">All versions</a>'
    nav += f'<a href="../{newer}/">{newer}</a>' if newer else "<span></span>"
    nav += "</nav>"
    body = f"""<section class="wn-hero"><a class="wn-back" href="../">What's new</a></section>
<div class="wn-list">
{entry(e, "../../", link=False, level="h1")}
</div>
{nav}"""
    title = f"AniBeam {v}" + (f": {e['title']}" if e["title"] else "")
    (out / "whats-new" / v).mkdir(parents=True, exist_ok=True)
    (out / "whats-new" / v / "index.html").write_text(page(title, "../../", body, f"whats-new/{v}/"))

print(f"whats-new: {len(versions)} versions, coming next: {'yes' if coming else 'no'}")
