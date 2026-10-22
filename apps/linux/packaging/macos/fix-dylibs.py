#!/usr/bin/env python3
"""usage: fix-dylibs.py <AniBeam.app> [--check]

Makes an app bundle self-contained: every Mach-O inside it may load only system libraries
and files inside the bundle. Run after macdeployqt, which deploys Qt's own frameworks and
plugins; this catches what it leaves behind (libmpv and the FFmpeg tree under it, MpvQt).

Whatever a Mach-O loads from outside the bundle is copied into Contents/Frameworks and the
load command is pointed at @executable_path/../Frameworks/<name>, which resolves the same
from the executable, a framework and a dlopen'd plugin alike. Rpaths that lead outside the
bundle are deleted. --check changes nothing and exits 1 when anything still leaks out.

The signatures this invalidates are redone by bundle.sh afterwards.

Two kinds of Mach-O live outside the main executable's reach. qbittorrent-nox sits beside
anibeam in Contents/MacOS and gets the same treatment as anibeam does. The Python tree under
Contents/Resources/anirss/python is an island: python-build-standalone ships it with
@rpath/libpython3.13.dylib and an rpath of @executable_path/../lib, which resolves from its
own bin/python3.13 and not from anibeam. Its references are judged from that executable, left
alone when they land inside the tree, and an error when they do not, since copying a stray
library into Contents/Frameworks would not help a Python that looks for it in its own lib.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

SYSTEM = ("/usr/lib/", "/System/Library/")
PYTHON_HOME = Path("Contents/Resources/anirss/python")
MAGICS = {
    b"\xcf\xfa\xed\xfe",  # 64-bit, little endian
    b"\xce\xfa\xed\xfe",
    b"\xca\xfe\xba\xbe",  # universal
    b"\xbe\xba\xfe\xca",
}


def run(*args: str) -> str:
    return subprocess.run(args, check=True, capture_output=True, text=True).stdout


def is_macho(path: Path) -> bool:
    if path.is_symlink() or not path.is_file():
        return False
    with path.open("rb") as f:
        return f.read(4) in MAGICS


def machos(app: Path) -> list[Path]:
    return sorted(p for p in (app / "Contents").rglob("*") if is_macho(p))


def install_id(path: Path) -> str | None:
    lines = run("otool", "-D", str(path)).splitlines()
    return lines[1].strip() if len(lines) > 1 else None


def loads(path: Path) -> list[str]:
    own = install_id(path)
    out = []
    for line in run("otool", "-L", str(path)).splitlines()[1:]:
        name = line.strip().split(" (compatibility")[0]
        if name and name != own:
            out.append(name)
    return out


def rpaths(path: Path) -> list[str]:
    found, lines = [], run("otool", "-l", str(path)).splitlines()
    for i, line in enumerate(lines):
        if line.strip() == "cmd LC_RPATH":
            for follow in lines[i + 1 : i + 4]:
                follow = follow.strip()
                if follow.startswith("path "):
                    found.append(follow[5:].rsplit(" (offset", 1)[0])
    return found


def expand(ref: str, binary: Path, exe_dir: Path) -> str:
    return ref.replace("@executable_path", str(exe_dir)).replace(
        "@loader_path", str(binary.parent)
    )


def resolve(ref: str, binary: Path, exe: Path, exe_rpaths: list[str]) -> Path | None:
    exe_dir = exe.parent
    if ref.startswith("@rpath/"):
        rest = ref[len("@rpath/") :]
        # The image's own rpaths first, then the executable's, which every image in the
        # process inherits. The executable's are the ones it had before fix() started
        # deleting, so a library found through /opt/homebrew/lib is still found.
        for rp in rpaths(binary) + exe_rpaths:
            candidate = Path(expand(rp, binary, exe_dir)) / rest
            if candidate.exists():
                return candidate
        return None
    candidate = Path(expand(ref, binary, exe_dir))
    return candidate if candidate.exists() else None


def main_exe(app: Path) -> Path:
    return app / "Contents" / "MacOS" / "anibeam"


def exe_for(binary: Path, app: Path) -> Path:
    """The executable whose @executable_path the binary's references are read against."""
    home = app / PYTHON_HOME
    if inside(binary, home):
        return home / "bin" / "python3.13"
    return main_exe(app)


def inside(path: Path, app: Path) -> bool:
    try:
        path.resolve().relative_to(app.resolve())
        return True
    except ValueError:
        return False


def fix(app: Path) -> None:
    frameworks = app / "Contents" / "Frameworks"
    frameworks.mkdir(parents=True, exist_ok=True)
    # The executables' rpaths as they were before fix() started deleting.
    exe_rpaths = {exe: rpaths(exe) for exe in {exe_for(b, app) for b in machos(app)}}
    todo = machos(app)
    seen: set[Path] = set()
    while todo:
        binary = todo.pop()
        if binary in seen:
            continue
        seen.add(binary)
        for ref in loads(binary):
            if ref.startswith(SYSTEM):
                continue
            exe = exe_for(binary, app)
            target = resolve(ref, binary, exe, exe_rpaths[exe])
            if target is not None and inside(target, app):
                continue
            if inside(binary, app / PYTHON_HOME):
                sys.exit(f"fix-dylibs: {binary.relative_to(app)} loads {ref}, which is outside "
                         "the Python tree it ships in")
            if target is None:
                sys.exit(f"fix-dylibs: {binary.relative_to(app)} loads {ref}, which is nowhere")
            if ".framework/" in str(target):
                sys.exit(f"fix-dylibs: {binary.relative_to(app)} loads the framework {ref}; "
                         "macdeployqt should have deployed it")
            name = Path(ref).name
            copy = frameworks / name
            if not copy.exists():
                shutil.copy2(target.resolve(), copy)
                os.chmod(copy, 0o755)
                run("install_name_tool", "-id", f"@executable_path/../Frameworks/{name}", str(copy))
                todo.append(copy)
                print(f"copied {name}")
            run("install_name_tool", "-change", ref, f"@executable_path/../Frameworks/{name}",
                str(binary))
        for rp in rpaths(binary):
            if rp.startswith("@") or inside(Path(rp), app):
                continue
            run("install_name_tool", "-delete_rpath", rp, str(binary))


def leaks(app: Path) -> list[str]:
    exe_rpaths = {exe: rpaths(exe) for exe in {exe_for(b, app) for b in machos(app)}}
    found = []
    for binary in machos(app):
        rel = binary.relative_to(app)
        exe = exe_for(binary, app)
        for ref in loads(binary):
            if ref.startswith(SYSTEM):
                continue
            target = resolve(ref, binary, exe, exe_rpaths[exe])
            if target is None or not inside(target, app):
                found.append(f"{rel}: loads {ref} -> {target}")
        for rp in rpaths(binary):
            if not rp.startswith("@") and not inside(Path(rp), app):
                found.append(f"{rel}: rpath {rp}")
    return found


def main() -> None:
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    app = Path(sys.argv[1])
    if "--check" not in sys.argv[2:]:
        fix(app)
    left = leaks(app)
    for line in left:
        print(f"leak: {line}")
    if left:
        sys.exit(f"fix-dylibs: {len(left)} references still leave {app.name}")
    print(f"fix-dylibs: {app.name} loads nothing from outside itself")


if __name__ == "__main__":
    main()
