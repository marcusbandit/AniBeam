#!/usr/bin/env python3
import os, re, sys
RUNS = os.path.expanduser("~/spike-libmpv/qruns")
ORDER = ["base", "hq", "scale-ewa", "cscale-ewa", "dscale-mit", "dither8", "deband", "interp", "base2"]

def parse(path):
    d = {}
    for line in open(path, errors="replace"):
        if line.startswith("play:"):
            m = re.search(r"delta (-?\d+)", line)
            d["drops"] = m.group(1) if m else "?"
            m = re.search(r"\(([\d.]+) s of video\)", line)
            d["secs"] = m.group(1) if m else "?"
        elif line.startswith("gpu busy:"):
            d["busy"] = re.search(r"([\d.]+) mean", line).group(1)
            d["busymax"] = re.search(r"([\d.]+) max", line).group(1)
        elif line.startswith("gpu power:"):
            d["pw"] = re.search(r"([\d.]+) mean", line).group(1)
            d["pwmax"] = re.search(r"([\d.]+) max", line).group(1)
        elif line.startswith("gpu clock:"):
            d["clk"] = re.search(r"([\d.]+) mean", line).group(1)
        elif line.startswith("gpu temp:"):
            d["temp"] = re.search(r"([\d.]+) max", line).group(1)
        elif line.startswith("at end:"):
            m = re.search(r"vf-fps=([\d.]+)", line)
            d["vffps"] = ("%.3f" % float(m.group(1))) if m else "?"
            m = re.search(r"decdrops=(\S+)", line)
            d["dec"] = m.group(1)
            m = re.search(r"delayed=(\S+)", line)
            d["delayed"] = m.group(1)
        elif line.startswith("resolved:"):
            d["resolved"] = line.strip()
    return d

for block, label in (("fhd", "1080p fullscreen (video drawn 1920x1080, 1:1)"),
                     ("uhd", "2160p fullscreen (video drawn 1920x1080, 0.5x)"),
                     ("win", "1080p tiled (video drawn 1824x1026, 0.95x)")):
    rows = [(c, os.path.join(RUNS, f"{block}-{c}", "summary.txt")) for c in ORDER]
    rows = [(c, p) for c, p in rows if os.path.exists(p)]
    if not rows:
        continue
    print(f"\n## {block}: {label}")
    print(f"{'config':<12} {'drops':>6} {'vf-fps':>8} {'dec':>4} {'dly':>4} {'busy%':>7} {'max%':>6} {'watt':>6} {'wmax':>6} {'MHz':>6} {'Cmax':>5}")
    for c, p in rows:
        d = parse(p)
        print(f"{c:<12} {d.get('drops','?'):>6} {d.get('vffps','?'):>8} {d.get('dec','?'):>4} "
              f"{d.get('delayed','?'):>4} {d.get('busy','?'):>7} {d.get('busymax','?'):>6} "
              f"{d.get('pw','?'):>6} {d.get('pwmax','?'):>6} {d.get('clk','?'):>6} {d.get('temp','?'):>5}")
