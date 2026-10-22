#!/usr/bin/env python3
"""usage: crash-report.py [process-name]

Prints the newest macOS crash report for the process (default anibeam) from
~/Library/Logs/DiagnosticReports: the exception, the termination reason and the
crashed thread's frames, symbolicated as far as the report itself goes. A CI run has
no Console.app, and a process that dies on SIGTRAP prints nothing of its own.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def newest(name: str) -> Path | None:
    reports = Path.home() / "Library" / "Logs" / "DiagnosticReports"
    found = sorted(reports.glob(f"{name}*.ips"), key=lambda p: p.stat().st_mtime)
    return found[-1] if found else None


def main() -> None:
    name = sys.argv[1] if len(sys.argv) > 1 else "anibeam"
    path = newest(name)
    if path is None:
        print(f"crash-report: no {name}*.ips report")
        return
    header, _, body = path.read_text().partition("\n")
    report = json.loads(body)
    print(f"crash-report: {path.name}")
    print("exception:", json.dumps(report.get("exception")))
    print("termination:", json.dumps(report.get("termination")))
    for key in ("asi", "lastExceptionBacktrace"):
        if key in report:
            print(f"{key}:", json.dumps(report[key])[:2000])
    images = report.get("usedImages", [])
    threads = report.get("threads", [])
    faulting = report.get("faultingThread", 0)
    if faulting >= len(threads):
        return
    thread = threads[faulting]
    print(f"thread {faulting} {thread.get('name', '')} {thread.get('queue', '')}")
    for i, frame in enumerate(thread.get("frames", [])[:40]):
        image = images[frame["imageIndex"]] if frame.get("imageIndex", -1) < len(images) else {}
        where = frame.get("symbol") or hex(frame.get("imageOffset", 0))
        print(f"  {i:2} {image.get('name', '?'):32} {where}")


if __name__ == "__main__":
    main()
