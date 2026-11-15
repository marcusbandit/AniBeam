#!/usr/bin/env python3
"""The layout check (#792): every page in every language, measured against English.

Runs the shell offscreen once per language and window size against one sandbox root, with a
drive that opens each page and Settings tab, grabs it and prints its words with their line
counts (the Driver's "texts" step). A piece of text that takes more lines than the same piece
in English, or that is cut off where English is not, is a finding. Exits 1 when there is one.

    apps/linux/scripts/layout-check.py --root /tmp/sandbox --bin target/release/anibeam \\
        --out /tmp/layout [--langs da,de] [--sizes 1600x1000,1000x760]

The root needs a library for the pages to show anything; the captures land in
<out>/<size>/<lang>/<page>.png and the findings in <out>/report.json. Runs are silent: audio
goes nowhere and mpv gets ao=null.
"""

import argparse
import json
import os
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
APP = HERE.parent
LANGS = ["en", "da", "de", "es", "fr", "pt-BR", "it", "nl", "pl", "ru", "ja", "ko", "zh-CN"]
PAGES = ["home", "library", "airing", "feed", "watching", "stats", "metadata", "downloads",
         "recommended"]
TABS = ["library", "accounts", "appearance", "playback", "subtitles", "keys", "downloads",
        "phone", "data", "help", "about"]


def drive_script():
    pages = [{"page": p, "shot": p, "texts": p} for p in PAGES]
    pages += [{"page": "settings", "props": {"tab": t}, "shot": "settings-" + t,
               "texts": "settings-" + t} for t in TABS]
    pages.append({"page": "series", "props": {"id": 1}, "shot": "series", "texts": "series"})
    return [{"pages": pages}]


def run(binary, root, out, lang, size):
    out.mkdir(parents=True, exist_ok=True)
    script = out / "drive.json"
    script.write_text(json.dumps(drive_script()))
    conf = out / "mpv.conf"
    conf.write_text("ao=null\n")
    w, h = size.split("x")
    env = dict(os.environ)
    env.update({
        "ANIBEAM_LANG": lang,
        "QT_QPA_PLATFORM": "offscreen",
        "QT_FORCE_STDERR_LOGGING": "1",
        "ANIBEAM_THEMES_DIR": str(APP / "themes"),
        "ANIBEAM_MPV_CONF": str(conf),
        "PULSE_SERVER": "unix:/nonexistent",
        "PIPEWIRE_REMOTE": "/nonexistent",
    })
    if env.get("DISPLAY"):
        env.update({"QT_QUICK_BACKEND": "rhi", "QSG_RHI_BACKEND": "opengl"})
    proc = subprocess.run(
        [binary, "--root", root, "--drive", str(script), "--out", str(out), "--width", w,
         "--height", h],
        env=env, capture_output=True, text=True, timeout=600)
    (out / "run.log").write_text(proc.stderr + proc.stdout)
    texts = {}
    for line in (proc.stderr + proc.stdout).splitlines():
        if line.startswith("DRIVE TEXTS "):
            _, _, rest = line.partition("DRIVE TEXTS ")
            name, _, payload = rest.partition(" ")
            texts[name] = json.loads(payload)
    # A run that failed, or measured fewer pages than the drive opens, is no result: it must
    # never read as a clean one
    pages = len(drive_script()[0]["pages"])
    if proc.returncode != 0 or len(texts) < pages:
        raise SystemExit(f"{lang} {size}: the drive exited {proc.returncode} and measured "
                         f"{len(texts)} of {pages} pages; see {out / 'run.log'}")
    return texts


def compare(english, other):
    """Findings for one language at one size: (page, path, kind, english, theirs)."""
    found = []
    for page, items in other.items():
        base = {i["p"]: i for i in english.get(page, [])}
        for item in items:
            en = base.get(item["p"])
            # Text that reads the same in both (a title, a path) moved only because longer
            # words beside it took its room; fixing those words fixes it
            if en is None or en["t"] == item["t"]:
                continue
            if item["l"] > en["l"]:
                found.append({"page": page, "path": item["p"], "kind": "wraps",
                              "english": en["t"], "text": item["t"],
                              "lines": [en["l"], item["l"]], "width": item["w"]})
            elif item["cut"] and not en["cut"]:
                found.append({"page": page, "path": item["p"], "kind": "cut",
                              "english": en["t"], "text": item["t"], "width": item["w"]})
    return found


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0],
                                 epilog="Exits 1 on a finding, and with a message when a run fails.")
    ap.add_argument("--root", required=True)
    ap.add_argument("--bin", default=str(APP.parent.parent / "target/release/anibeam"))
    ap.add_argument("--out", required=True)
    ap.add_argument("--langs", default=",".join(LANGS[1:]))
    ap.add_argument("--sizes", default="1600x1000,1000x760")
    a = ap.parse_args()
    out = pathlib.Path(a.out)
    langs = [l for l in a.langs.split(",") if l and l != "en"]
    report = {}
    total = 0
    for size in a.sizes.split(","):
        print(f"{size}: en")
        english = run(a.bin, a.root, out / size / "en", "en", size)
        for lang in langs:
            print(f"{size}: {lang}")
            found = compare(english, run(a.bin, a.root, out / size / lang, lang, size))
            report.setdefault(lang, {})[size] = found
            total += len(found)
            print(f"  {len(found)} findings")
    (out / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=1))
    print(f"{total} findings; {out / 'report.json'}")
    return 1 if total else 0


if __name__ == "__main__":
    sys.exit(main())
