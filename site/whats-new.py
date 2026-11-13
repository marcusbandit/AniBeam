#!/usr/bin/env python3
"""Builds the website's What's new pages from the apps' own patch notes (#777):

  <out>/whats-new/index.html                  every release, newest first
  <out>/whats-new/<version>/index.html        one page per desktop release
  <out>/whats-new/android-<version>/index.html  one page per phone release

The desktop's apps/linux/CHANGELOG.md and the phone's apps/android/CHANGELOG.md hold the shipped
versions. The change files in apps/*/changes/ hold what has merged since the last stable release
and ships in the next nightly; they show as "Coming next". The page for a version that ships
appears with the site deploy the release starts, and the Coming next block with the one each
nightly starts, so nothing here is written by hand.

usage: whats-new.py <repo-root> <out-dir>
"""
import html
import json
import re
import subprocess
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


def git(*args):
    return subprocess.run(["git", "-C", str(root), *args], capture_output=True, text=True, check=True).stdout


def change_files(changes_dir, ref):
    """The change files' texts, as main has them, or as the commit `ref` had them."""
    if ref:
        rel = changes_dir.relative_to(root).as_posix()
        names = git("ls-tree", "--name-only", f"{ref}:{rel}").split()
        return [(n, git("show", f"{ref}:{rel}/{n}")) for n in sorted(names) if n.endswith(".md")]
    return [(f.name, f.read_text()) for f in sorted(changes_dir.glob("*.md"))]


def pending(changes_dir, ref=None):
    """The lines in the change files, merged by section."""
    sections = {}
    for name, text in change_files(changes_dir, ref):
        if name.lower() == "readme.md":
            continue
        section = None
        for line in text.splitlines():
            if line.startswith("### "):
                section = line[4:].strip()
            elif line.startswith("- ") and section:
                sections.setdefault(section, []).append(line[2:].strip())
    return sections


def respell(text):
    """The site spells the name Anibeam; the notes are shared with the apps, which still say
    AniBeam, so prose is respelled here. Code spans, paths and file names (Pictures/AniBeam,
    %APPDATA%\\AniBeam, AniBeam.AppImage, AniBeam-2.4.0) keep their spelling."""
    parts = text.split("`")
    parts[::2] = [re.sub(r"(?<![/\\.\w])AniBeam(?!\.\w|[\w-])", "Anibeam", p) for p in parts[::2]]
    return "`".join(parts)


def inline(text):
    text = html.escape(respell(text))
    return re.sub(r"`([^`]+)`", r"<code>\1</code>", text)


def nice_date(iso):
    y, m, d = iso.split("-")
    months = ["January", "February", "March", "April", "May", "June", "July", "August",
              "September", "October", "November", "December"]
    return f"{int(d)} {months[int(m) - 1]} {y}"


# Collect: one entry per release. The desktop and the phone count their versions apart (the
# first number is shared, the rest each app's own), so desktop 2.0.0 and Android 2.0.0 are two
# releases with two entries.
versions = []
for key, name, path in APPS:
    for v in parse((path / "CHANGELOG.md").read_text()):
        if not any(v["sections"].values()):
            continue
        slug = v["version"] if key == "desktop" else f"android-{v['version']}"
        tag = f"v{v['version']}" if key == "desktop" else f"android-v{v['version']}"
        versions.append({"version": v["version"], "slug": slug, "tag": tag, "app": key, "app_name": name,
                         "title": v["title"] if key == "desktop" else "", "date": v["date"], "apps": {key: v["sections"]}})


def vkey(v):
    return tuple(int(x) for x in v.split("."))


versions.sort(key=lambda e: (e["date"], e["app"] == "desktop", vkey(e["version"])), reverse=True)
# The tags that exist on GitHub (tools/releases.py, run just before). A version with notes but
# no published release gets no download link; without the list every version keeps its link.
try:
    RELEASES = json.loads((out / "releases.json").read_text())
    TAGS = set(RELEASES["tags"])
except (OSError, ValueError, KeyError):
    RELEASES, TAGS = {}, None
# Coming next is what the newest nightly brings: the change files at its own tag, so a merge
# after it never shows as in it. With no nightly newer than the stable release, or without
# git or the network, it is what main holds, under a plain "Coming next".
nightly = RELEASES.get("nightly") or {}
nightly_version, ref = "", None
if nightly.get("tag"):
    try:
        git("fetch", "--quiet", "--depth=1", "origin", "tag", nightly["tag"])
        ref, nightly_version = nightly["tag"], nightly["version"]
    except (OSError, subprocess.CalledProcessError) as e:
        print(f"whats-new: {nightly['tag']} not fetched, Coming next is main's ({e})")
coming = {key: pending(path / "changes", ref) for key, _, path in APPS}
coming = {k: v for k, v in coming.items() if any(v.values())}


def notes_html(apps, named=True):
    parts = []
    for key, name, _ in APPS:
        sections = apps.get(key)
        if not sections:
            continue
        parts.append(f'<div class="wn-app"><h3>{name}</h3>' if named else '<div class="wn-app">')
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


# The notes are English only, so the language menu here leads to each language's home page.
# The folder comes from the download page's own link (../da/download/), since a hreflang such
# as zh-Hans is not a folder name.
NAV = re.sub(r'(<a href=")\.\./(?:([\w-]+)/)?download/(" hreflang=)', lambda m: f'{m.group(1)}../{m.group(2) + "/" if m.group(2) else ""}{m.group(3)}', NAV)


def chrome(fragment, prefix):
    if prefix == "../":
        return fragment
    return re.sub(r'(href|src)="\.\./', lambda m: f'{m.group(1)}="{prefix}', fragment)


def page(title, prefix, body, canonical, description):
    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{html.escape(title)}</title>
<meta name="description" content="{html.escape(description)}">
<link rel="canonical" href="https://anibeam.app/{canonical}">
<meta name="theme-color" content="#07080a">
<meta name="color-scheme" content="dark">
<link rel="icon" href="{prefix}img/logo.svg" type="image/svg+xml">
<meta property="og:site_name" content="Anibeam">
<meta property="og:type" content="website">
<meta property="og:title" content="{html.escape(title)}">
<meta property="og:description" content="{html.escape(description)}">
<meta property="og:url" content="https://anibeam.app/{canonical}">
<meta property="og:image" content="https://anibeam.app/img/og.jpg">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta name="twitter:card" content="summary_large_image">
<link rel="stylesheet" href="{prefix}style.css">
<script src="{prefix}lang.js" defer></script>
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
    v, slug = e["version"], e["slug"]
    title = f'<span class="wn-title">{html.escape(respell(e["title"]))}</span>' if e["title"] else ""
    name = f'<a href="{prefix}whats-new/{slug}/">{v}</a>' if link else v
    dl = (f'\n  <a class="wn-dl" href="https://github.com/marcusbandit/AniBeam/releases/tag/{e["tag"]}">Download {v}</a>'
          if TAGS is None or e["tag"] in TAGS else "")
    anchor = f"v{v}" if e["app"] == "desktop" else slug
    return f"""<article class="wn-entry" id="{anchor}">
  <header class="wn-head"><{level} class="wn-ver">{name}</{level}><span class="wn-app-tag">{e['app_name']}</span>{title}<time datetime="{e['date']}">{nice_date(e['date'])}</time></header>
  {notes_html(e['apps'], named=False)}{dl}
</article>"""


# The index
blocks = []
if coming:
    label = f"Coming next, in {nightly_version}" if nightly_version else "Coming next"
    blocks.append(f"""<article class="wn-entry wn-coming" id="next">
  <header class="wn-head"><h2 class="wn-ver">{html.escape(label)}</h2><a class="wn-try" href="../download/#nightly">Try the nightly</a></header>
  {notes_html(coming)}
</article>""")
# The newest few stay open; the rest fold under one line so the page does not run on for ever.
OPEN = 6
blocks += [entry(e, "../") for e in versions[:OPEN]]
if len(versions) > OPEN:
    blocks.append('<details class="wn-older"><summary>Older releases</summary>\n'
                  + "\n".join(entry(e, "../") for e in versions[OPEN:]) + "\n</details>")
index = f"""<section class="wn-hero"><h1>What's new</h1></section>
<div class="wn-list">
{chr(10).join(blocks)}
</div>"""
(out / "whats-new").mkdir(parents=True, exist_ok=True)
(out / "whats-new/index.html").write_text(page("What's new in Anibeam", "../", index, "whats-new/",
                                              "Every Anibeam release for desktop and Android, newest first."))

# One page per version
for i, e in enumerate(versions):
    v, slug = e["version"], e["slug"]
    label = lambda o: f'{o["app_name"]} {o["version"]}'
    newer = versions[i - 1] if i > 0 else None
    older = versions[i + 1] if i + 1 < len(versions) else None
    nav = '<nav class="wn-pager">'
    nav += f'<a href="../{older["slug"]}/">{label(older)}</a>' if older else "<span></span>"
    nav += '<a href="../">All releases</a>'
    nav += f'<a href="../{newer["slug"]}/">{label(newer)}</a>' if newer else "<span></span>"
    nav += "</nav>"
    body = f"""<section class="wn-hero"><a class="wn-back" href="../">What's new</a></section>
<div class="wn-list">
{entry(e, "../../", link=False, level="h1")}
</div>
{nav}"""
    title = f"Anibeam {'' if e['app'] == 'desktop' else 'for Android '}{v}" + (f": {respell(e['title'])}" if e["title"] else "")
    (out / "whats-new" / slug).mkdir(parents=True, exist_ok=True)
    (out / "whats-new" / slug / "index.html").write_text(
        page(title, "../../", body, f"whats-new/{slug}/", f"What changed in {e['app_name']} {v}, released {nice_date(e['date'])}."))

# The page GitHub Pages serves for an address that leads nowhere. It can be served at any depth,
# so its links start from the site root.
lost = """<section class="wn-hero"><h1>Nothing here.</h1></section>
<p class="lost"><a class="btn btn-primary btn-lg" href="/">Anibeam home</a></p>"""
(out / "404.html").write_text(page("Not found: Anibeam", "/", lost, "404.html", "This page does not exist.")
                              .replace("<head>", '<head>\n<meta name="robots" content="noindex">', 1))

# The download page links each release to its notes. A release with no page of its own (a
# hotfix, a version whose notes are empty) links the list instead.
if RELEASES:
    slugs = {e["slug"] for e in versions}
    for key, slug in (("stable", lambda v: v), ("nightly", lambda v: None), ("phone", lambda v: f"android-{v}"),
                      ("phone_nightly", lambda v: None)):
        r = RELEASES.get(key)
        if r:
            page_slug = slug(r["version"])
            r["notes"] = f"whats-new/{page_slug}/" if page_slug in slugs else ("whats-new/#next" if key == "nightly" else "whats-new/")
    (out / "releases.json").write_text(json.dumps(RELEASES, indent=1) + "\n")

# Every release page goes in the sitemap next to the pages i18n.py listed.
sitemap = out / "sitemap.xml"
if sitemap.exists():
    extra = "".join(f"  <url><loc>https://anibeam.app/whats-new/{e['slug']}/</loc></url>\n" for e in versions)
    sitemap.write_text(sitemap.read_text().replace("</urlset>", extra + "</urlset>"))

print(f"whats-new: {len(versions)} releases, coming next: {'yes' if coming else 'no'}")
