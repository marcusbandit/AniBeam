#!/usr/bin/env python3
"""Writes the iPhone app's icons into ios/Assets.xcassets from the images More's App icon
picker shows (ui/icons/app.png, app-dark.png, app-line.png), in the picker's order:
AppIcon (Default), AppIconDark and AppIconLine.

The images are used as they are, only scaled to the 1024 square iOS wants and flattened onto
an opaque ground, since iOS rejects an app icon with an alpha channel and masks the corners
itself. The first two are tiles with clear corners, so they are flattened onto their own tile
colour. The line art is a white outline for the picker to tint, so it is tinted with the
logo's yellow on its dark ink, which is how it looks on Android before Android 12.

Run it after the picker images change, then commit what it wrote. It needs ImageMagick's
magick on PATH.
"""

import json
import subprocess
from pathlib import Path

here = Path(__file__).resolve().parent
icons = here.parent / "ui/icons"
catalog = here / "Assets.xcassets"

YELLOW = "#FDC901"
INK = "#0D0E11"

# set name, picker image, ground to flatten onto, ink to tint it with (None: as it is)
SETS = [
    ("AppIcon", "app.png", YELLOW, None),
    ("AppIconDark", "app-dark.png", INK, None),
    ("AppIconLine", "app-line.png", INK, YELLOW),
]


def info() -> dict:
    return {"author": "xcode", "version": 1}


def write(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=2) + "\n")


catalog.mkdir(exist_ok=True)
write(catalog / "Contents.json", {"info": info()})
for name, source, ground, ink in SETS:
    d = catalog / f"{name}.appiconset"
    d.mkdir(parents=True, exist_ok=True)
    for stale in d.glob("*.png"):
        stale.unlink()
    cmd = ["magick", str(icons / source), "-filter", "Lanczos", "-resize", "1024x1024"]
    if ink:
        cmd += ["-fill", ink, "-colorize", "100"]
    cmd += ["-background", ground, "-alpha", "remove", "-alpha", "off",
            "-define", "png:color-type=2", str(d / "icon.png")]
    subprocess.run(cmd, check=True)
    write(
        d / "Contents.json",
        {
            "images": [
                {"filename": "icon.png", "idiom": "universal", "platform": "ios", "size": "1024x1024"}
            ],
            "info": info(),
        },
    )
