#!/usr/bin/env python3
"""Translates the assembled website into every language in site/i18n/.

The English pages are the source. For each language file site/i18n/<code>.json this writes the
same pages under <site>/<code>/ (the home page and /download/), with every visible string and
alt or aria text swapped for its translation, asset paths pointing one level up, the What's new
links pointing at the English notes (the changelog is written in English), and the strings the
scripts write themselves (window.I18N). Every page, English included, gets a working language
menu and hreflang links to its translations.

  i18n.py extract <site-dir>   print the English strings the pages hold, as JSON
  i18n.py build <site-dir>     write the translated pages and the menus

A string with no translation stays English, so a page is never broken by a missing line.
"""
import html
import json
import re
import sys
from pathlib import Path

I18N = Path(__file__).resolve().parent.parent / "i18n"  # site/i18n
PAGES = ["index.html", "download/index.html"]
# Strings the scripts put on the page (app.js, download.js), so the page carries them translated.
JS_STRINGS = ["Download for {os}", "Get the Android app", "Copy", "Copied", "What's new", "Download AniBeam"]
SKIP = {"AniBeam", "GitHub", "Ko-fi", "Linux", "Mac", "Windows", "AUR", ".deb", ".rpm", "Homebrew",
        "$", "|", "English", "AniList", "marcusbandit/AniBeam", "GPL 3.0", "AniBeam, GPL 3.0",
        # show titles in the mock app screens are names, not text
        "City the Animation", "Frieren: Beyond Journey's End", "Sousou no Frieren", "Attack on Titan Season 2",
        "Attack on Titan Season 3", "Season 3 Part 2", "The Final Season", "Final Season Part 2", "The Final Chapters"}
PROTECT = re.compile(r"<(script|style|code|svg)\b.*?</\1>", re.S | re.I)
TEXT = re.compile(r"(>|\x00\d+\x00)([^<>\x00]+)(?=<|\x00)")
ATTR = re.compile(r'\b(alt|aria-label|title|placeholder)="([^"]*)"')
META = re.compile(r'(<meta (?:name="description"|property="og:(?:title|description)") content=")([^"]*)(")')
TITLE = re.compile(r"(<title>)([^<]*)(</title>)")


def languages():
    out = []
    for f in sorted(I18N.glob("*.json")):
        data = json.loads(f.read_text())
        out.append((f.stem, data))
    return out


def protected(page):
    """The page with script, style, code and svg blocks set aside, and the blocks."""
    blocks = []

    def keep(m):
        blocks.append(m.group(0))
        return f"\x00{len(blocks) - 1}\x00"

    return PROTECT.sub(keep, page), blocks


def restore(page, blocks):
    return re.sub(r"\x00(\d+)\x00", lambda m: blocks[int(m.group(1))], page)


def wanted(text):
    t = html.unescape(text).strip()
    return t and t not in SKIP and re.search(r"[A-Za-z]{2}", t) and not t.startswith("http")


def extract(site):
    found = []
    for name in PAGES:
        page, _ = protected((site / name).read_text())
        for m in TITLE.finditer(page):
            found.append(html.unescape(m.group(2)))
        for m in META.finditer(page):
            found.append(html.unescape(m.group(2)))
        for m in ATTR.finditer(page):
            if wanted(m.group(2)):
                found.append(html.unescape(m.group(2)))
        for m in TEXT.finditer(page):
            if wanted(m.group(2)):
                found.append(html.unescape(m.group(2)).strip())
    seen, out = set(), []
    for s in found + JS_STRINGS:
        if s not in seen:
            seen.add(s)
            out.append(s)
    return out


def translate(page, table):
    page, blocks = protected(page)
    tr = lambda s: html.escape(table.get(s, s), quote=False)

    def text(m):
        start, raw = m.group(1), m.group(2)
        key = html.unescape(raw).strip()
        if key not in table:
            return m.group(0)
        lead = raw[: len(raw) - len(raw.lstrip())]
        tail = raw[len(raw.rstrip()):]
        return f"{start}{lead}{tr(key)}{tail}"

    page = TEXT.sub(text, page)
    page = ATTR.sub(lambda m: f'{m.group(1)}="{html.escape(table.get(html.unescape(m.group(2)), html.unescape(m.group(2))))}"', page)
    page = META.sub(lambda m: m.group(1) + html.escape(table.get(html.unescape(m.group(2)), html.unescape(m.group(2)))) + m.group(3), page)
    page = TITLE.sub(lambda m: m.group(1) + tr(html.unescape(m.group(2))) + m.group(3), page)
    return restore(page, blocks)


ASSET = re.compile(r'\b(href|src)="(?!https?:|#|/|mailto:|data:)([^"]+)"')


def deeper(page):
    """Paths for a copy one folder down: assets and What's new go up a level; links to the
    home page and the download page stay inside the language."""
    def fix(m):
        attr, url = m.groups()
        bare = url.lstrip("./")
        is_page = url.endswith("/") or url in (".", "./")
        if is_page and "whats-new" not in url:
            return m.group(0)
        return f'{attr}="../{url}"'
    return ASSET.sub(fix, page)


def menu(langs, here_code, page_name, depth_to_root):
    """The language menu for one page: each entry is the same page in that language."""
    sub = "" if page_name == "index.html" else page_name.replace("index.html", "")
    items = []
    for code, data in langs:
        target = depth_to_root + ("" if code == "en" else f"{code}/") + sub
        if not target:
            target = "./"
        current = ' aria-current="true"' if code == here_code else ""
        items.append(f'<a href="{target}" hreflang="{code}" lang="{code}"{current}>{html.escape(data["_name"])}</a>')
    name = dict(langs)[here_code]["_name"]
    globe = ('<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="10"/>'
             '<path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>')
    return (f'<details class="nav-lang"><summary>{globe}<span>{html.escape(name)}</span></summary>'
            f'<div class="lang-menu" role="menu">{"".join(items)}</div></details>')


NAV_LANG = re.compile(r'<(span|details) class="nav-lang".*?</\1>', re.S)


def alternates(langs, page_name):
    sub = "" if page_name == "index.html" else page_name.replace("index.html", "")
    links = [f'<link rel="alternate" hreflang="{"en" if c == "en" else c}" href="https://anibeam.app/{"" if c == "en" else c + "/"}{sub}">'
             for c, _ in langs]
    links.append(f'<link rel="alternate" hreflang="x-default" href="https://anibeam.app/{sub}">')
    return "\n".join(links)


def build(site):
    langs = [("en", {"_name": "English"})] + [(c, d) for c, d in languages() if c != "en"]
    for page_name in PAGES:
        src = (site / page_name).read_text()
        src = re.sub(r'<link rel="alternate" hreflang=[^>]*>\n?', "", src)
        depth = "../" * page_name.count("/")
        for code, data in langs:
            if code == "en":
                page = src
                to_root = depth
            else:
                page = deeper(translate(src, data))
                page = page.replace('<html lang="en">', f'<html lang="{code}">')
                sub = "" if page_name == "index.html" else page_name.replace("index.html", "")
                page = re.sub(r'<link rel="canonical" href="[^"]*">',
                              f'<link rel="canonical" href="https://anibeam.app/{code}/{sub}">', page)
                strings = {k: data[k] for k in JS_STRINGS if k in data}
                strings["_locale"] = data.get("_locale", code)
                strings["_root"] = "../"
                page = page.replace("</head>", f"<script>window.I18N = {json.dumps(strings, ensure_ascii=False)};</script>\n</head>", 1)
                to_root = "../" + depth
            page = NAV_LANG.sub(lambda m: menu(langs, code, page_name, to_root), page, count=1)
            page = page.replace("</head>", alternates(langs, page_name) + "\n</head>", 1)
            out = site / page_name if code == "en" else site / code / page_name
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(page)
    # The sitemap lists every page in every language, with its translations alongside.
    urls = []
    for page_name in PAGES:
        sub = "" if page_name == "index.html" else page_name.replace("index.html", "")
        alts = "".join(f'<xhtml:link rel="alternate" hreflang="{c}" href="https://anibeam.app/{"" if c == "en" else c + "/"}{sub}"/>' for c, _ in langs)
        for code, _ in langs:
            urls.append(f'  <url><loc>https://anibeam.app/{"" if code == "en" else code + "/"}{sub}</loc>{alts}</url>')
    urls.append("  <url><loc>https://anibeam.app/whats-new/</loc></url>")
    (site / "sitemap.xml").write_text('<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">\n' + "\n".join(urls) + "\n</urlset>\n")
    print(f"i18n: {len(langs)} languages, {len(PAGES)} pages each")


if __name__ == "__main__":
    mode, site = sys.argv[1], Path(sys.argv[2])
    if mode == "extract":
        print(json.dumps(extract(site), ensure_ascii=False, indent=1))
    else:
        build(site)
