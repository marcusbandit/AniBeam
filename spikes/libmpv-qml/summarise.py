#!/usr/bin/env python3
import json, sys, os, statistics as st

out = sys.argv[1]
ev = []
for line in open(os.path.join(out, "stdout.log"), errors="replace"):
    if not line.startswith("SPIKE "):
        continue
    _, tag, rest = line.split(" ", 2)
    try:
        ev.append((tag, json.loads(rest)))
    except Exception:
        pass

def first(tag):
    for t, o in ev:
        if t == tag:
            return o
    return None

def all_of(tag):
    return [o for t, o in ev if t == tag]

sets = [f"{o['key']}={o['value']}" for o in all_of("set")] or ["(defaults)"]
print("set:        " + " ".join(sets))

opts = first("options") or {}
keys = ["scale", "cscale", "dscale", "tscale", "dither-depth", "deband", "deband-iterations",
        "deband-threshold", "deband-range", "deband-grain", "interpolation", "video-sync",
        "hdr-peak-percentile", "hdr-contrast-recovery"]
print("resolved:   " + "  ".join(f"{k}={json.dumps(opts.get(k))}" for k in keys if k in opts))
print("video:      %sx%s -> osd %s  hwdec=%s interop=%s pixfmt=%s hwfmt=%s vo=%s" % (
    opts.get("video-params/w"), opts.get("video-params/h"), opts.get("osd-dimensions"),
    opts.get("hwdec-current"), opts.get("hwdec-interop"), opts.get("video-params/pixelformat"),
    opts.get("video-params/hw-pixelformat"), opts.get("current-vo")))

b, e = first("measure-begin"), first("measure-end")
if b and e:
    print("play:       t %.2f -> %.2f (%.1f s of video), drops %d -> %d  (delta %d)" % (
        b["time-pos"], e["time-pos"], e["time-pos"] - b["time-pos"], b["drops"], e["drops"],
        e["drops"] - b["drops"]))

pf = first("play-final") or {}
print("at end:     drops=%s decdrops=%s delayed=%s mistimed=%s vf-fps=%s display-fps=%s avsync=%s video-sync=%s" % (
    pf.get("frame-drop-count"), pf.get("decoder-frame-drop-count"), pf.get("vo-delayed-frame-count"),
    pf.get("mistimed-frame-count"), pf.get("estimated-vf-fps"), pf.get("display-fps"),
    pf.get("avsync"), pf.get("video-sync")))

# GPU over the measurement window
marks = {}
mp = os.path.join(out, "marks.log")
if os.path.exists(mp):
    for line in open(mp):
        ts, name = line.split()
        marks[name] = int(ts)
samples = []
for line in open(os.path.join(out, "gpu.log"), errors="replace"):
    p = line.split()
    if len(p) == 5:
        try:
            samples.append(tuple(int(x) for x in p))
        except ValueError:
            pass
if "measure-begin" in marks and "measure-end" in marks:
    lo, hi = marks["measure-begin"], marks["measure-end"]
    w = [s for s in samples if lo <= s[0] <= hi]
    if w:
        busy = [s[1] for s in w]
        pw = [s[2] / 1e6 for s in w if s[2] > 0]
        sclk = [s[3] / 1e6 for s in w if s[3] > 0]
        temp = [s[4] / 1000 for s in w if s[4] > 0]
        def stat(v, unit, fmt="%.1f"):
            if not v:
                return "n/a"
            return (fmt + " mean / " + fmt + " p95 / " + fmt + " max %s") % (
                st.mean(v), sorted(v)[int(len(v) * 0.95) - 1], max(v), unit)
        print("gpu busy:   " + stat(busy, "%"))
        print("gpu power:  " + stat(pw, "W", "%.2f"))
        print("gpu clock:  " + stat(sclk, "MHz", "%.0f"))
        print("gpu temp:   " + stat(temp, "C"))
        print("samples:    %d over %.1f s" % (len(w), (hi - lo) / 1000))
stills = sorted(f for f in os.listdir(out) if f.startswith("still-") and f.endswith(".png"))
print("stills:     " + (", ".join(stills) or "none"))
for o in all_of("still-hold"):
    print("  still-%d target %.1f actual %.3f" % (o["n"], o["target"], o.get("time-pos") or -1))
err = os.path.join(out, "stderr.log")
bad = [l.rstrip() for l in open(err, errors="replace") if l.strip() and "locale" not in l.lower()]
if bad:
    print("stderr:     " + " | ".join(bad[:6]))
