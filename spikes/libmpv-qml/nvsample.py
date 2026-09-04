#!/usr/bin/env python3
# nvsample.py OUT: samples the NVIDIA GPU into OUT/gpu.log in summarise.py's five-field format
# (epoch_ms busy% power_uW clock_Hz temp_mC), OUT/dec.log (epoch_ms decoder%) and OUT/pmon.log
# (epoch_ms plus nvidia-smi pmon's per-process row: pid type sm% mem% enc% dec% ... name).
import os, signal, subprocess, sys, threading, time
out = sys.argv[1]
g = open(os.path.join(out, "gpu.log"), "w", buffering=1)
d = open(os.path.join(out, "dec.log"), "w", buffering=1)
p = open(os.path.join(out, "pmon.log"), "w", buffering=1)
smi = subprocess.Popen(["nvidia-smi", "--query-gpu=utilization.gpu,utilization.decoder,power.draw,clocks.gr,temperature.gpu",
                        "--format=csv,noheader,nounits", "-lms", "250"], stdout=subprocess.PIPE, text=True)
pm = subprocess.Popen(["nvidia-smi", "pmon", "-s", "u", "-d", "1"], stdout=subprocess.PIPE, text=True)
def stop(*a):
    smi.terminate(); pm.terminate(); sys.exit(0)
signal.signal(signal.SIGTERM, stop); signal.signal(signal.SIGINT, stop)
def rd_smi():
    for line in smi.stdout:
        parts = [x.strip() for x in line.split(",")]
        if len(parts) != 5:
            continue
        try:
            busy = int(float(parts[0])); dec = int(float(parts[1])); pw = int(float(parts[2]) * 1e6)
            clk = int(float(parts[3]) * 1e6); temp = int(float(parts[4]) * 1000)
        except ValueError:
            continue
        now = int(time.time() * 1000)
        g.write(f"{now} {busy} {pw} {clk} {temp}\n"); d.write(f"{now} {dec}\n")
def rd_pm():
    for line in pm.stdout:
        if line.startswith("#"):
            continue
        p.write(f"{int(time.time() * 1000)} {line.rstrip()}\n")
threading.Thread(target=rd_smi, daemon=True).start()
threading.Thread(target=rd_pm, daemon=True).start()
while True:
    time.sleep(1)
