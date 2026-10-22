#!/usr/bin/env python3
"""
mock-host.py: a dev-only fake of the AniBeam desktop's read-only JSON API.

It serves the routes described in host-api/src/lib.rs (hello, snapshot and
posters) from host-api/fixtures/snapshot.json, so the Android app can be built
and run against something without the desktop app being up. It never reads or
writes a real library. Standard library only.

Usage:
    python3 scripts/mock-host.py [--port 8790] [--bind 127.0.0.1] [--demo] [--aired] [--host-name banditbox]

--demo       adds six currently airing AniList shows (ids 100 to 105), their
             subscriptions and torrents. Needs the network once; the AniList
             reply is cached in apps/android/build/mock/demo.json.
--aired      makes Frieren (id 12) episode 8 air two hours ago, as if the
             snapshot was taken just before it aired.

The Android emulator reaches the host at http://10.0.2.2:<port>.
"""

import argparse
import hashlib
import json
import os
import re
import struct
import sys
import tempfile
import threading
import time
import urllib.parse
import urllib.request
import zlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.normpath(os.path.join(SCRIPT_DIR, "..", "..", ".."))
FIXTURE = os.path.join(REPO_ROOT, "host-api", "fixtures", "snapshot.json")
BUILD_DIR = os.path.normpath(os.path.join(SCRIPT_DIR, "..", "build", "mock"))
DEMO_CACHE = os.path.join(BUILD_DIR, "demo.json")
POSTER_DIR = os.path.join(BUILD_DIR, "posters")

ANILIST_URL = "https://graphql.anilist.co"
USER_AGENT = "anibeam-mock/0.1"
API_VERSION = 1
APP = "anibeam"
MOCK_VERSION = "2.1.0-mock"
HELLO_PATH = "/api/v1/hello"
SNAPSHOT_PATH = "/api/v1/snapshot"
POSTER_RE = re.compile(r"^/api/v1/posters/(\d+)$")

DEMO_FIRST_ID = 100
DEMO_QUERY = (
    "query { Page(perPage: 6) { media(type: ANIME, status: RELEASING, "
    "sort: POPULARITY_DESC, isAdult: false) { id idMal title { romaji english native } "
    "format season seasonYear episodes nextAiringEpisode { episode airingAt } "
    "airingSchedule(notYetAired: true, perPage: 12) { nodes { episode airingAt } } "
    "coverImage { large } } } }"
)
FIXTURE_COVER_QUERY = "query ($ids: [Int]) { Page { media(id_in: $ids) { id coverImage { large } } } }"

AIRED_SERIES_ID = 12
AIRED_EPISODE = 8
DAY = 86400
HOUR = 3600

CONTENT_TYPES = {"jpg": "image/jpeg", "png": "image/png", "webp": "image/webp"}
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"

# Filled in by main() from the command line and the demo fetch.
OPTS = {"host_name": "banditbox", "aired": False}
DEMO = {"media": None, "covers": {}}
POSTER_LOCK = threading.Lock()


def now_s():
    return int(time.time())


def load_fixture():
    with open(FIXTURE, encoding="utf-8") as fh:
        return json.load(fh)


def anilist(query, variables=None):
    body = json.dumps({"query": query, "variables": variables or {}}).encode("utf-8")
    req = urllib.request.Request(
        ANILIST_URL,
        data=body,
        method="POST",
        headers={
            "Content-Type": "application/json",
            "Accept": "application/json",
            "User-Agent": USER_AGENT,
        },
    )
    with urllib.request.urlopen(req, timeout=20) as resp:
        return json.loads(resp.read().decode("utf-8"))


def fetch_bytes(url):
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(req, timeout=20) as resp:
        return resp.read()


def write_atomic(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=os.path.dirname(path))
    with os.fdopen(fd, "wb") as fh:
        fh.write(data)
    os.replace(tmp, path)


def ext_for(url):
    ext = os.path.splitext(urllib.parse.urlsplit(url).path)[1].lower().lstrip(".")
    if ext == "jpeg":
        ext = "jpg"
    return ext if ext in CONTENT_TYPES else "jpg"


def media_from(data):
    try:
        media = data["data"]["Page"]["media"]
    except (KeyError, TypeError):
        return None
    return media if isinstance(media, list) and media else None


def load_demo_media():
    """Return the AniList media list for --demo, or None to serve the plain fixture."""
    if os.path.exists(DEMO_CACHE):
        try:
            with open(DEMO_CACHE, encoding="utf-8") as fh:
                media = media_from(json.load(fh))
            if media:
                return media
        except (OSError, ValueError):
            pass  # unreadable cache: fetch again below
    try:
        data = anilist(DEMO_QUERY)
    except Exception as exc:
        print(f"warning: AniList unreachable for --demo ({exc}); serving the plain fixture", file=sys.stderr)
        return None
    media = media_from(data)
    if media is None:
        print("warning: AniList answered without media for --demo; serving the plain fixture", file=sys.stderr)
        return None
    write_atomic(DEMO_CACHE, json.dumps(data, indent=2).encode("utf-8"))
    return media


def airing(node):
    return {"episode": node["episode"], "season": None, "at": node["airingAt"]}


def demo_covers(media):
    covers = {}
    for i, m in enumerate(media):
        url = (m.get("coverImage") or {}).get("large")
        if url:
            covers[DEMO_FIRST_ID + i] = url
    return covers


def torrent(name, state, progress, size, speed, eta, added_on, completed_on, series_id):
    return {
        "hash": hashlib.sha1(f"{series_id}:{name}".encode("utf-8")).hexdigest(),
        "name": name,
        "state": state,
        "progress": progress,
        "size": size,
        "download_speed": speed,
        "eta": eta,
        "added_on": added_on,
        "completed_on": completed_on,
        "series_id": series_id,
    }


def demo_parts(media, now):
    series, subs, torrents = [], [], []
    for i, m in enumerate(media):
        sid = DEMO_FIRST_ID + i
        titles = m.get("title") or {}
        romaji = titles.get("romaji") or titles.get("english") or titles.get("native") or "Untitled"
        year = m.get("seasonYear")
        season = m.get("season")
        season_text = f"{season.capitalize()} {year}" if season and year else None
        nae = m.get("nextAiringEpisode")
        total = m.get("episodes")

        next_ep = nae["episode"] if nae else (total or 0) + 1
        # Media index 2 is the one with an aired episode missing from disk.
        on_disk = max(0, next_ep - 1 - (1 if i == 2 else 0))
        missing = [next_ep - 1] if i == 2 and next_ep - 1 >= 1 else []
        progress = max(0, on_disk - 1)
        next_airing = airing(nae) if nae else None

        nodes = (m.get("airingSchedule") or {}).get("nodes") or []
        upcoming = sorted((airing(n) for n in nodes if n), key=lambda a: a["at"])

        cover = (m.get("coverImage") or {}).get("large")
        poster = hashlib.sha1(cover.encode("utf-8")).hexdigest()[:10] if cover else None

        episodes = [
            {
                "key": str(n),
                "number": float(n),
                "season": None,
                "code": f"E{n:02d}",
                "title": None,
                "watched": n <= progress,
                "aired_at": None,
            }
            for n in range(1, on_disk + 1)
        ]

        series.append({
            "id": sid,
            "kind": "show",
            "title": romaji,
            "titles": {
                "romaji": titles.get("romaji"),
                "english": titles.get("english"),
                "native": titles.get("native"),
            },
            "status": "Releasing",
            "format": m.get("format"),
            "year": year,
            "season": season_text,
            "anilist_id": m["id"],
            "mal_id": m.get("idMal"),
            "tvmaze_id": None,
            "total_episodes": total,
            "episodes_on_disk": on_disk,
            "progress": progress,
            "next_airing": next_airing,
            "upcoming": upcoming,
            "poster": poster,
            "episodes": episodes,
            "added_at": now - 40 * DAY,
            "latest_activity_at": now - (i + 1) * 3 * HOUR,
        })

        if i < 3:
            query = f"{romaji} 1080p"
            subs.append({
                "name": romaji,
                "title": romaji,
                "feed_url": "https://nyaa.si/?page=rss&q=" + urllib.parse.quote(query) + "&c=1_2&f=0",
                "query": query,
                "enabled": True,
                "torrent_count": on_disk,
                "file_count": on_disk,
                "series_id": sid,
                "next_airing": dict(next_airing) if next_airing else None,
                "complete": False,
                "missing": missing,
            })

        if i == 0:
            torrents.append(torrent(
                f"[SubsPlease] {romaji} - {next_ep:02d} (1080p) [DEADBEEF].mkv",
                "downloading", 0.42, 1_400_000_000, 3_400_000, 240, now - 300, None, sid))
        elif i == 1:
            torrents.append(torrent(
                f"[SubsPlease] {romaji} - {next_ep:02d} (1080p) [DEADBEEF].mkv",
                "stalledDL", 0.07, 1_400_000_000, 0, None, now - 2 * HOUR, None, sid))
        elif i == 2:
            torrents.append(torrent(
                f"[SubsPlease] {romaji} - {on_disk:02d} (1080p) [DEADBEEF].mkv",
                "pausedUP", 1.0, 1_300_000_000, 0, None, now - DAY - 600, now - DAY, sid))

    name = "Kusuriya no Hitorigoto Batch"
    subs.append({
        "name": name,
        "title": name,
        "feed_url": "https://nyaa.si/?page=rss&q=" + urllib.parse.quote(name) + "&c=1_2&f=0",
        "query": name,
        "enabled": True,
        "torrent_count": 0,
        "file_count": 0,
        "series_id": None,
        "next_airing": None,
        "complete": False,
        "missing": [],
    })
    return series, subs, torrents


def apply_aired(snap, now):
    """Make Frieren's episode 8 air two hours ago, everywhere it is scheduled."""
    at = now - 2 * HOUR
    for s in snap["series"]:
        if s.get("id") == AIRED_SERIES_ID:
            for a in [s.get("next_airing")] + list(s.get("upcoming") or []):
                if a and a.get("episode") == AIRED_EPISODE:
                    a["at"] = at
            break
    for sub in snap["subscriptions"]:
        if sub.get("series_id") == AIRED_SERIES_ID:
            na = sub.get("next_airing")
            if na and na.get("episode") == AIRED_EPISODE:
                na["at"] = at


def build_snapshot(now):
    snap = load_fixture()
    snap["generated_at"] = now
    snap["host"] = OPTS["host_name"]
    if OPTS["aired"]:
        apply_aired(snap, now)
    if DEMO["media"] is not None:
        series, subs, torrents = demo_parts(DEMO["media"], now)
        snap["series"].extend(series)
        snap["subscriptions"].extend(subs)
        snap["torrents"] = list(snap.get("torrents") or []) + torrents
    return snap


def cached_poster(sid):
    for ext in CONTENT_TYPES:
        path = os.path.join(POSTER_DIR, f"{sid}.{ext}")
        if os.path.exists(path):
            return path
    return None


def fetch_fixture_covers(anilist_ids):
    """One AniList query for all the ids. Returns {anilist_id: cover url}, or {} on failure."""
    try:
        data = anilist(FIXTURE_COVER_QUERY, {"ids": anilist_ids})
        media = data["data"]["Page"]["media"]
    except Exception as exc:
        print(f"warning: AniList cover query failed ({exc}); using placeholder posters", file=sys.stderr)
        return {}
    covers = {}
    for m in media or []:
        url = ((m or {}).get("coverImage") or {}).get("large")
        if url:
            covers[m["id"]] = url
    return covers


def png_chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)


def placeholder_png(seed, width=230, height=345):
    """A vertical two-colour gradient, colour derived from the seed, as a valid RGB PNG."""
    digest = hashlib.sha1(str(seed).encode("utf-8")).digest()
    top = [96 + digest[k] % 160 for k in range(3)]
    bottom = [c * 35 // 100 for c in top]
    rows = bytearray()
    for y in range(height):
        t = y / (height - 1)
        px = bytes(round(top[k] + (bottom[k] - top[k]) * t) for k in range(3))
        rows += b"\x00" + px * width  # filter byte 0, then RGB pixels
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    return (
        PNG_SIGNATURE
        + png_chunk(b"IHDR", ihdr)
        + png_chunk(b"IDAT", zlib.compress(bytes(rows), 9))
        + png_chunk(b"IEND", b"")
    )


def ensure_fixture_posters(wanted):
    """Cache a real AniList cover, or a placeholder, for every fixture series with a poster tag."""
    pending = {sid: aid for sid, aid in wanted.items() if cached_poster(sid) is None}
    if not pending:
        return
    covers = fetch_fixture_covers(sorted(set(pending.values())))
    for sid, aid in sorted(pending.items()):
        url = covers.get(aid)
        if url:
            try:
                write_atomic(os.path.join(POSTER_DIR, f"{sid}.{ext_for(url)}"), fetch_bytes(url))
                continue
            except Exception as exc:
                print(f"warning: cover download failed for series {sid} ({exc}); using a placeholder", file=sys.stderr)
        write_atomic(os.path.join(POSTER_DIR, f"{sid}.png"), placeholder_png(sid))


def resolve_poster(sid):
    """Return the path of the cached poster for a series id, or None when it has none."""
    with POSTER_LOCK:
        if sid in DEMO["covers"]:
            url = DEMO["covers"][sid]
            path = os.path.join(POSTER_DIR, f"{sid}.{ext_for(url)}")
            if not os.path.exists(path):
                write_atomic(path, fetch_bytes(url))
            return path
        fixture = load_fixture()
        wanted = {
            s["id"]: s["anilist_id"]
            for s in fixture.get("series", [])
            if s.get("poster") and s.get("anilist_id")
        }
        if sid in wanted:
            ensure_fixture_posters(wanted)
            return cached_poster(sid)
        return None


class Handler(BaseHTTPRequestHandler):
    server_version = "anibeam-mock/0.1"

    def do_GET(self):
        path = urllib.parse.urlsplit(self.path).path
        if path == HELLO_PATH:
            return self._json(200, {
                "app": APP,
                "api": API_VERSION,
                "version": MOCK_VERSION,
                "host": OPTS["host_name"],
                "now": now_s(),
            })
        if path == SNAPSHOT_PATH:
            return self._json(200, build_snapshot(now_s()))
        match = POSTER_RE.match(path)
        if match:
            return self._poster(int(match.group(1)))
        return self._json(404, {"error": f"no route for {path}"})

    def send_error(self, code, message=None, explain=None):
        # Anything this server does not serve, including methods it does not
        # implement, is a JSON 404. Overridden so the stdlib 501 never leaks out.
        self._json(404, {"error": message or "not found"})

    def _poster(self, sid):
        try:
            path = resolve_poster(sid)
        except Exception as exc:
            return self._json(502, {"error": f"poster for series {sid} could not be fetched: {exc}"})
        if path is None:
            return self._json(404, {"error": f"series {sid} has no poster"})
        ext = os.path.splitext(path)[1].lstrip(".")
        with open(path, "rb") as fh:
            body = fh.read()
        self._send(200, CONTENT_TYPES[ext], body)

    def _json(self, status, obj):
        body = json.dumps(obj).encode("utf-8")
        self._send(status, "application/json; charset=utf-8", body)

    def _send(self, status, ctype, body):
        try:
            self.send_response(status)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            if self.command != "HEAD":
                self.wfile.write(body)
        except (BrokenPipeError, ConnectionResetError):
            pass  # the client went away; nothing to do

    def log_message(self, fmt, *args):
        # One line per request: send_response is the only place that logs.
        sys.stderr.write(f"{time.strftime('%H:%M:%S')} {self.address_string()} {fmt % args}\n")


def main(argv=None):
    ap = argparse.ArgumentParser(description="Dev-only fake of the AniBeam host API.")
    ap.add_argument("--port", type=int, default=8790)
    ap.add_argument("--bind", default="127.0.0.1")
    ap.add_argument("--demo", action="store_true", help="add six live AniList shows, subscriptions and torrents")
    ap.add_argument("--aired", action="store_true", help="make Frieren episode 8 air two hours ago")
    ap.add_argument("--host-name", default="banditbox", help="the machine name the host reports")
    args = ap.parse_args(argv)

    OPTS["host_name"] = args.host_name
    OPTS["aired"] = args.aired
    if args.demo:
        media = load_demo_media()
        if media is not None:
            DEMO["media"] = media
            DEMO["covers"] = demo_covers(media)

    server = ThreadingHTTPServer((args.bind, args.port), Handler)
    port = server.server_address[1]
    print(f"mock host listening on http://{args.bind}:{port}", flush=True)
    print(
        f"host {args.host_name}; demo {'on' if DEMO['media'] is not None else 'off'}; "
        f"aired {'on' if args.aired else 'off'}",
        flush=True,
    )
    print(f"the Android emulator reaches it at http://10.0.2.2:{port}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
