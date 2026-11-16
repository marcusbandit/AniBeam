#!/usr/bin/env python3
"""Builds the website's wiki (#1007): short task pages the app can link to.

  <out>/wiki/index.html            every page, as a grid of cards
  <out>/wiki/<slug>/index.html     one page per entry in PAGES

Each page's text is a fragment in site/wiki/<slug>.html: sections, each an <h2> with an id, so
a link such as /wiki/sources/#add lands on its step. This wraps the fragments in the site's nav
and footer, a list of the other pages and a pager. The slugs are addresses the app links to,
so a slug never changes once it ships; a page that moves keeps its old slug.

The wiki is English only, like What's new, so its language menu leads to each language's home.

usage: wiki.py <repo-root> <out-dir>
"""
import html
import re
import shutil
import sys
from pathlib import Path

root, out = Path(sys.argv[1]), Path(sys.argv[2])
SRC = root / "site/wiki"

# slug, title, the description search engines show, and the card's icon (Lucide paths)
PAGES = [
    ("getting-started", "Getting started", "Install Anibeam and get through the first setup: your folder, your list and downloads.",
     '<path d="M6 3l14 9-14 9z"/>'),
    ("library", "Your library", "Add the folder your anime lives in, name files so Anibeam reads them, and fix a show it got wrong.",
     '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>'),
    ("sources", "Sources", "What a source list is, how to add one in Anibeam, and how to make and host your own.",
     '<path d="M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7-7l-1.7 1.7"/><path d="M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7 7l1.7-1.7"/>'),
    ("downloads", "Downloads", "The built-in torrent client or your own qBittorrent, where downloads go, and following a show as it airs.",
     '<path d="M12 3v12"/><path d="M7 10l5 5 5-5"/><path d="M4 21h16"/>'),
    ("tracking", "AniList and MyAnimeList", "Connect AniList or MyAnimeList so every episode you finish is marked on your list.",
     '<path d="M20 6L9 17l-5-5"/>'),
    ("watch-together", "Watch together", "Start a room, send its code, and watch an episode in step with friends.",
     '<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>'),
    ("troubleshooting", "Troubleshooting", "What to do when a show is not recognised, a search finds nothing, a download never arrives or a room will not open.",
     '<path d="M14.7 6.3a4 4 0 0 0-5.4 5.4L3 18l3 3 6.3-6.3a4 4 0 0 0 5.4-5.4l-2.5 2.5-2.5-.5-.5-2.5z"/>'),
]

# The nav and the footer come from the download page, as on What's new: it sits at the same
# depth as the wiki's index, and a page one level down climbs one more.
_dl = (out / "download/index.html").read_text()
NAV = re.search(r'<header class="nav">.*?</header>', _dl, re.S).group(0)
FOOT = re.search(r'<footer class="credits">.*?</footer>', _dl, re.S).group(0)
# The language menu leads to each language's home page, since the wiki is English only.
# English, the language the visitor is in, stays on the page (page() points it at its own
# address): lang.js sees the same page and the current language and does nothing, rather
# than leaving for the home page.
NAV = re.sub(r'(<a href=")\.\./download/(" hreflang="en"[^>]*aria-current="true")', r'\1../wiki/\2', NAV)
NAV = re.sub(r'(<a href=")\.\./(?:([\w-]+)/)?download/(" hreflang=)', lambda m: f'{m.group(1)}../{m.group(2) + "/" if m.group(2) else ""}{m.group(3)}', NAV)
# The nav's own Wiki link marks where the visitor is. A change to its markup fails the build
# here rather than shipping a nav with no current page.
WIKI_LINK = '<a class="nav-wiki" href="../wiki/">'
if WIKI_LINK not in NAV:
    sys.exit("wiki: the download page's nav has no Wiki link to mark as current")
NAV = NAV.replace(WIKI_LINK, '<a class="nav-wiki" href="../wiki/" aria-current="page">', 1)


def chrome(fragment, prefix):
    if prefix == "../":
        return fragment
    return re.sub(r'(href|src)="\.\./', lambda m: f'{m.group(1)}="{prefix}', fragment)


def icon(paths):
    return f'<svg viewBox="0 0 24 24" aria-hidden="true">{paths}</svg>'


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
<meta property="og:type" content="article">
<meta property="og:title" content="{html.escape(title)}">
<meta property="og:description" content="{html.escape(description)}">
<meta property="og:url" content="https://anibeam.app/{canonical}">
<meta property="og:image" content="https://anibeam.app/img/og.jpg">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta name="twitter:card" content="summary_large_image">
<link rel="stylesheet" href="{prefix}style.css">
<link rel="stylesheet" href="{prefix}redesign.css">
<script src="{prefix}lang.js" defer></script>
<script src="{prefix}redesign.js" defer></script>
</head>
<body class="rd">
{chrome(NAV, prefix).replace(f'href="{prefix}wiki/" hreflang="en"', f'href="{prefix}{canonical}" hreflang="en"', 1)}
<main class="wiki">
{body}
</main>
{chrome(FOOT, prefix)}
</body>
</html>
"""


def check(slug, text):
    """A fragment's sections each need an id of their own, since the app links to them."""
    seen = set()
    for m in re.finditer(r"<h2\b([^>]*)>", text):
        found = re.search(r'(?:^|\s)id="([\w-]+)"', m.group(1))
        if not found:
            sys.exit(f"wiki: site/wiki/{slug}.html has an <h2> without an id")
        if found.group(1) in seen:
            sys.exit(f"wiki: site/wiki/{slug}.html has two sections with the id {found.group(1)}")
        seen.add(found.group(1))
    if re.search("[\\u2013\\u2014]", text):
        sys.exit(f"wiki: site/wiki/{slug}.html has an en or em dash")


fragments = {}
for slug, *_ in PAGES:
    text = (SRC / f"{slug}.html").read_text()
    check(slug, text)
    fragments[slug] = text
extra = sorted(p.stem for p in SRC.glob("*.html") if p.stem not in fragments)
if extra:
    sys.exit(f"wiki: site/wiki/ has pages PAGES does not list: {', '.join(extra)}")

# The fragments were copied in with the rest of site/; the built pages replace them.
shutil.rmtree(out / "wiki", ignore_errors=True)
(out / "wiki").mkdir(parents=True)

# The index: a card per page, its title alone.
cards = "\n".join(f'  <a class="wiki-card" href="{slug}/">{icon(paths)}<span>{html.escape(title)}</span></a>'
                  for slug, title, _, paths in PAGES)
index = f"""<section class="wiki-hero"><h1>Wiki</h1></section>
<nav class="wiki-cards" aria-label="Wiki pages">
{cards}
</nav>"""
(out / "wiki/index.html").write_text(page("Anibeam wiki", "../", index, "wiki/",
                                          "How to set up and use Anibeam: your library, sources, downloads, AniList and MyAnimeList, rooms and fixes."))

for i, (slug, title, description, _) in enumerate(PAGES):
    here = ' aria-current="page"'
    toc = "".join(f'<a href="../{s}/"{here if s == slug else ""}>{html.escape(t)}</a>' for s, t, *_ in PAGES)
    older = PAGES[i - 1] if i > 0 else None
    newer = PAGES[i + 1] if i + 1 < len(PAGES) else None
    pager = '<nav class="wn-pager wiki-pager" aria-label="Next and previous">'
    pager += f'<a class="wiki-prev" href="../{older[0]}/">{html.escape(older[1])}</a>' if older else "<span></span>"
    pager += f'<a class="wiki-next" href="../{newer[0]}/">{html.escape(newer[1])}</a>' if newer else "<span></span>"
    pager += "</nav>"
    body = f"""<section class="wiki-hero"><a class="wn-back" href="../">Wiki</a><h1>{html.escape(title)}</h1></section>
<div class="wiki-body">
<article class="wiki-text">
{fragments[slug].strip()}
</article>
<nav class="wiki-toc" aria-label="Wiki pages">{toc}</nav>
</div>
{pager}"""
    (out / "wiki" / slug).mkdir(parents=True, exist_ok=True)
    (out / "wiki" / slug / "index.html").write_text(page(f"{title}: Anibeam wiki", "../../", body, f"wiki/{slug}/", description))

# The wiki goes in the sitemap next to the pages i18n.py listed.
sitemap = out / "sitemap.xml"
if sitemap.exists():
    urls = "".join(f"  <url><loc>https://anibeam.app/wiki/{s}</loc></url>\n" for s in [""] + [f"{p[0]}/" for p in PAGES])
    sitemap.write_text(sitemap.read_text().replace("</urlset>", urls + "</urlset>"))

print(f"wiki: {len(PAGES)} pages")
