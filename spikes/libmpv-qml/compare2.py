#!/usr/bin/env python3
import os, re, subprocess
RUNS = os.path.expanduser("~/spike-libmpv/qruns")
CROP = "crop=1600:900:160:150"   # inside the video on every geometry: no cursor, no window edge

def metrics(a, b):
    lav = f"[0]{CROP}[a];[1]{CROP}[b];[a][b]psnr=stats_file=-"
    p = subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-i", a, "-i", b,
                        "-lavfi", lav, "-f", "null", "-"], capture_output=True, text=True)
    m = re.search(r"psnr_avg:(\S+)", p.stdout)
    psnr = m.group(1) if m else "?"
    lav = f"[0]{CROP}[a];[1]{CROP}[b];[a][b]blend=all_mode=difference,format=gray,signalstats,metadata=print:file=-"
    d = subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-i", a, "-i", b,
                        "-lavfi", lav, "-f", "null", "-"], capture_output=True, text=True)
    ymax = re.search(r"signalstats\.YMAX=(\S+)", d.stdout)
    yavg = re.search(r"signalstats\.YAVG=(\S+)", d.stdout)
    return psnr, (ymax.group(1) if ymax else "?"), (yavg.group(1) if yavg else "?")

ORDER = ["base2", "hq", "scale-ewa", "cscale-ewa", "dscale-mit", "dither8", "deband", "interp"]
for block in ("fhd", "uhd", "win"):
    base = os.path.join(RUNS, f"{block}-base")
    if not os.path.isdir(base):
        continue
    print(f"\n=== {block}, cropped to the video interior: psnr dB / max delta / mean delta (0-255)")
    for c in ORDER:
        d = os.path.join(RUNS, f"{block}-{c}")
        if not os.path.isdir(d):
            continue
        row = []
        for i in range(3):
            a, b = os.path.join(base, f"still-{i}.png"), os.path.join(d, f"still-{i}.png")
            if os.path.exists(a) and os.path.exists(b):
                psnr, ymax, yavg = metrics(a, b)
                row.append(f"s{i}: {psnr:>7} /{ymax:>4} /{yavg:>9}")
        print(f"  {c:<12} " + "  ".join(row))
