#!/usr/bin/env python3
"""Writes site/shows.json: what the card shows when a poster in the home page's wall is clicked.

Each poster in img/posters/ is named for a show; SHOWS gives its AniList id. This asks AniList
once for every show and keeps the title, format, year, episodes, main studio, score, genres, the
MyAnimeList id and a short description. The site reads the file as it is, so the card opens with
no request to AniList. Run it again after adding a poster:

  python3 site/tools/shows.py
"""
import html
import json
import re
import urllib.request
from pathlib import Path

SITE = Path(__file__).resolve().parent.parent
SHOWS = {
    "akira": 47, "aot": 16498, "bebop": 1, "bocchi": 130003, "champloo": 205, "csm": 127230,
    "dandadan": 171018, "dbz": 813, "deathnote": 1535, "demonslayer": 101922, "edgerunners": 120377,
    "eva": 30, "fmab": 5114, "frieren": 154587, "geass": 1575, "gits": 43, "gurren": 2001,
    "haikyuu": 20464, "hxh": 11061, "jjk": 113415, "kaguya": 101921, "madeinabyss": 97986,
    "mha": 21459, "mob": 21507, "mononoke": 164, "naruto": 20, "onepiece": 21, "opm": 21087,
    "oshinoko": 150672, "perfectblue": 437, "sailormoon": 530, "sololeveling": 151807,
    "spirited": 199, "spyfamily": 140960, "steinsgate": 9253, "vinland": 101348, "violet": 21827,
    "yourname": 21519,
}
QUERY = """query($ids: [Int]) { Page(perPage: 50) { media(id_in: $ids, type: ANIME) {
  id idMal title { english romaji } format episodes seasonYear averageScore genres
  studios(isMain: true) { nodes { name } } description(asHtml: false)
} } }"""
# Long enough for the premise, short enough to read in the card without scrolling much.
MAX = 560


def blurb(text):
    """AniList's description as plain text: its first paragraphs, without the source credit."""
    text = re.sub(r"<br\s*/?>", "\n", text or "", flags=re.I)
    text = html.unescape(re.sub(r"<[^>]+>", "", text))
    paras = []
    for p in re.split(r"\n\s*\n|\n", text):
        p = re.sub(r"\s+", " ", p).strip()
        if not p or re.match(r"^[(\[]?(source|written by|note)", p, re.I):
            continue
        p = re.sub(r"\s*[(\[](source|written by)[^)\]]*[)\]]\s*$", "", p, flags=re.I)
        paras.append(p)
    out = ""
    for p in paras:
        if out and len(out) + len(p) > MAX:
            break
        out = f"{out}\n\n{p}" if out else p
    if len(out) > MAX:
        cut = out[:MAX]
        end = max(cut.rfind(". "), cut.rfind("! "), cut.rfind("? "))
        out = cut[: end + 1] if end > MAX // 2 else cut.rsplit(" ", 1)[0] + "..."
    return out


def main():
    body = json.dumps({"query": QUERY, "variables": {"ids": list(SHOWS.values())}}).encode()
    req = urllib.request.Request("https://graphql.anilist.co", body, {
        "Content-Type": "application/json", "Accept": "application/json", "User-Agent": "anibeam-site"})
    media = {m["id"]: m for m in json.load(urllib.request.urlopen(req, timeout=30))["data"]["Page"]["media"]}
    out = {}
    for key, aid in sorted(SHOWS.items()):
        m = media[aid]
        studios = m["studios"]["nodes"]
        out[key] = {
            "anilist": aid,
            "mal": m["idMal"],
            "title": (m["title"]["english"] or m["title"]["romaji"]).replace("’", "'"),
            "romaji": m["title"]["romaji"],
            "format": m["format"],
            "year": m["seasonYear"],
            "episodes": m["episodes"],
            "studio": studios[0]["name"] if studios else None,
            "score": m["averageScore"],
            "genres": m["genres"][:3],
            "description": blurb(m["description"]),
        }
    (SITE / "shows.json").write_text(json.dumps(out, ensure_ascii=False, indent=1) + "\n")
    print(f"wrote {len(out)} shows to {SITE / 'shows.json'}")


if __name__ == "__main__":
    main()
