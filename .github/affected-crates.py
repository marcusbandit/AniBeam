#!/usr/bin/env python3
"""Which workspace crates a change has to clippy and test (#1075).

A pull request checks the crates whose files it changed, plus every crate that depends on one
of them, directly or not. A dev-dependency counts for its direct user only, whose tests build
it; cargo never builds a dev-dependency for the user's own users. The crates, their folders and their dependencies come from
`cargo metadata`, so a crate split out later needs no edit here.

  affected-crates.py --base REV        files changed since the merge base of HEAD and REV,
                                       plus uncommitted and untracked ones
  affected-crates.py --files A B ...   these files (paths from the workspace root)
  affected-crates.py --all             every crate
  affected-crates.py --self-test       the cases at the bottom of this file

  --skip NAME       leave a crate out of the answer, e.g. one the machine cannot build
  --output FILE     also write crates=, alone= and core= lines to FILE
  --github-output   the same into $GITHUB_OUTPUT

Prints one crate per line on stdout, dependencies first, and why each was picked (and what
was skipped) on stderr. `alone` is the picked crates another crate builds on: clippy runs on
each of those by itself too, since one cargo call over several crates merges their features
and hides a use the crate does not turn on itself.

How a changed file maps to crates:
  1. Workspace-wide files (WORKSPACE_WIDE below): every crate.
  2. A file under a crate's folder: that crate. A file outside it that a crate's build or
     tests read is declared in that crate's manifest, relative to its folder:
         [package.metadata.ci]
         reads = ["../i18n"]
  3. A file no crate owns or reads that sits under NO_CRATE below (docs, the site, the phone's
     own workspace, other workflows): none.
  4. Anything else: every crate. An unknown file is never taken to be harmless.
"""

import argparse
import fnmatch
import json
import os
import re
import subprocess
import sys

# Files every crate builds with, so a change to one checks everything. The CI files are here so
# a change to the selection itself is checked against the whole workspace.
WORKSPACE_WIDE = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain",
    "rust-toolchain.toml",
    ".cargo/",
    "clippy.toml",
    ".clippy.toml",
    ".github/affected-crates.py",
    ".github/workflows/ci.yml",
    ".github/local-ci.sh",
]

# Paths no workspace crate builds from or reads. A crate that starts reading one declares it in
# its manifest (see above), and then rule 2 wins over this list.
NO_CRATE = [
    "*.md",
    "docs/",
    "site/",
    "mockups/",
    "spikes/",
    "assets/",
    "Casks/",
    "relay/",
    "apps/android/",
    ".github/",
    "LICENSE",
    ".gitignore",
    ".gitattributes",
    ".env.example",
    "build.sh",
]


def matches(path, pattern):
    """A pattern ending in / is a folder; one with * or ? is a glob; anything else is a file or
    a folder by its exact name."""
    if pattern.endswith("/"):
        return path.startswith(pattern) or path == pattern[:-1]
    if any(c in pattern for c in "*?["):
        return fnmatch.fnmatch(path, pattern) or fnmatch.fnmatch(os.path.basename(path), pattern)
    return path == pattern or path.startswith(pattern + "/")


class Workspace:
    """The workspace's crates: name, folder from the root, what they read, what they use."""

    def __init__(self, crates):
        # crates: {name: {"dir": "core", "reads": ["apps/linux/packaging/"], "uses": {names},
        #                 "dev": {names only its tests use}}}
        self.crates = crates
        self.users = {name: set() for name in crates}
        self.dev_users = {name: set() for name in crates}
        for name, c in crates.items():
            for dep in c["uses"]:
                self.users[dep].add(name)
            for dep in c.get("dev", set()) - c["uses"]:
                self.dev_users[dep].add(name)

    @classmethod
    def from_metadata(cls, meta):
        root = meta["workspace_root"]
        members = set(meta["workspace_members"])
        by_dir = {}
        crates = {}
        for p in meta["packages"]:
            if p["id"] not in members:
                continue
            d = os.path.relpath(os.path.dirname(p["manifest_path"]), root)
            by_dir[os.path.normpath(os.path.dirname(p["manifest_path"]))] = p["name"]
            reads = ((p.get("metadata") or {}).get("ci") or {}).get("reads") or []
            reads = [os.path.relpath(os.path.normpath(os.path.join(root, d, r)), root) for r in reads]
            reads = [r + "/" if os.path.isdir(os.path.join(root, r)) else r for r in reads]
            crates[p["name"]] = {"dir": d, "reads": reads, "deps": p["dependencies"]}
        for c in crates.values():
            local = [(by_dir[os.path.normpath(dep["path"])], dep.get("kind"))
                     for dep in c.pop("deps")
                     if dep.get("path") and os.path.normpath(dep["path"]) in by_dir]
            c["uses"] = {n for n, kind in local if kind != "dev"}
            c["dev"] = {n for n, kind in local if kind == "dev"}
        return cls(crates)

    def order(self, names):
        """Dependencies first, then by name, so a log reads bottom up."""
        out, seen = [], set()

        def visit(n):
            if n in seen:
                return
            seen.add(n)
            for d in sorted(self.crates[n]["uses"] | self.crates[n].get("dev", set())):
                visit(d)
            out.append(n)

        for n in sorted(names):
            visit(n)
        return [n for n in out if n in names]

    def owner(self, path):
        """The crate whose folder holds the file: the deepest one, for nested crates."""
        best = None
        for name, c in self.crates.items():
            d = c["dir"]
            if d == "." or path.startswith(d + "/"):
                if best is None or len(d) > len(self.crates[best]["dir"]):
                    best = name
        return best

    def readers(self, path):
        return sorted(n for n, c in self.crates.items() if any(matches(path, r) for r in c["reads"]))

    def users_of(self, names):
        """The crates that build on any of these, directly or not, plus the direct users whose
        tests use one of them, or a crate built on one, as a dev-dependency."""
        out, todo = set(), list(names)
        while todo:
            for u in self.users[todo.pop()]:
                if u not in out:
                    out.add(u)
                    todo.append(u)
        built = out | set(names)
        out |= {u for n in built for u in self.dev_users[n]}
        return out - set(names)


def select(ws, files, everything=False, skip=()):
    """Returns (crates, alone, why). why is a list of lines for a log reader."""
    why = []
    picked = {}  # crate: the first file that picked it
    if everything:
        why.append("every crate: asked for all")
        picked = {n: None for n in ws.crates}
    for f in files:
        if everything:
            break
        wide = next((p for p in WORKSPACE_WIDE if matches(f, p)), None)
        if wide:
            why.append(f"every crate: {f} is workspace-wide")
            everything = True
            picked = {n: None for n in ws.crates}
            break
        owners = set(ws.readers(f))
        o = ws.owner(f)
        if o:
            owners.add(o)
        if owners:
            for n in sorted(owners):
                picked.setdefault(n, f)
            continue
        if any(matches(f, p) for p in NO_CRATE):
            why.append(f"no crate: {f}")
            continue
        why.append(f"every crate: {f} is outside every crate and not known to be harmless")
        everything = True
        picked = {n: None for n in ws.crates}
        break
    if not everything:
        changed = sorted(picked)
        for n in changed:
            why.append(f"{n}: {picked[n]} changed")
        through = {}
        for n in changed:
            for u in ws.users_of([n]):
                through.setdefault(u, n)
        for u in sorted(set(through) - set(changed)):
            why.append(f"{u}: builds on {through[u]}")
            picked[u] = None
    for n in sorted(set(skip) & set(picked)):
        why.append(f"left out: {n}, skipped on this machine")
    names = set(picked) - set(skip)
    unpicked = sorted(set(ws.crates) - set(picked))
    if unpicked:
        why.append("not checked: " + ", ".join(unpicked))
    crates = ws.order(names)
    alone = [n for n in crates if ws.users[n] or ws.dev_users[n]]
    return crates, alone, why


def changed_files(root, base):
    def git(*args):
        return subprocess.run(["git", "-C", root, *args], check=True, capture_output=True,
                              text=True).stdout
    mb = git("merge-base", "HEAD", base).strip()
    # NUL-separated, so a path with any character in it comes back as it is, unquoted. Renames
    # show as a delete and an add, so the crate a file left is checked too.
    files = git("diff", "-z", "--name-only", "--no-renames", "--relative", mb).split("\0")
    files += git("ls-files", "-z", "--others", "--exclude-standard").split("\0")
    return sorted(set(f for f in files if f)), mb


def metadata(root):
    out = subprocess.run(["cargo", "metadata", "--format-version", "1", "--no-deps", "--offline"],
                         cwd=root, check=True, capture_output=True, text=True).stdout
    return json.loads(out)


def main():
    ap = argparse.ArgumentParser(description="Which workspace crates a change has to check.")
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--base")
    g.add_argument("--files", nargs="*")
    g.add_argument("--all", action="store_true")
    g.add_argument("--self-test", action="store_true")
    ap.add_argument("--skip", action="append", default=[])
    ap.add_argument("--output")
    ap.add_argument("--github-output", action="store_true")
    a = ap.parse_args()

    if a.self_test:
        return self_test()

    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    ws = Workspace.from_metadata(metadata(root))
    files = []
    if a.base:
        files, mb = changed_files(root, a.base)
        print(f"affected: {len(files)} files changed since {mb[:10]}", file=sys.stderr)
    elif a.files is not None:
        files = a.files
    crates, alone, why = select(ws, files, everything=a.all, skip=a.skip)
    for line in why:
        print(f"affected: {line}", file=sys.stderr)
    if not crates:
        print("affected: nothing to clippy or test", file=sys.stderr)
    for n in crates:
        print(n)
    out = os.environ["GITHUB_OUTPUT"] if a.github_output else a.output
    if out:
        with open(out, "a") as f:
            f.write(f"crates={' '.join(crates)}\n")
            f.write(f"alone={' '.join(alone)}\n")
            f.write(f"core={'true' if 'anibeam-core' in crates else 'false'}\n")
    return 0


# Paths spelled out in a crate's Rust: include_str!("..."), include_bytes!("..."), include!("...")
# relative to the file, and env!("CARGO_MANIFEST_DIR")).join("...") relative to the crate. A path
# built at run time cannot be seen here; declare those by hand.
INCLUDE = re.compile(r'include(?:_str|_bytes)?!\(\s*"([^"]+)"\s*\)')
MANIFEST_JOIN = re.compile(r'env!\("CARGO_MANIFEST_DIR"\)\s*\)\s*\.join\(\s*"([^"]+)"\s*\)')


def undeclared_reads(ws, root):
    """Every file a crate's Rust names outside its own folder has to sit in a crate it builds on
    or in its reads, or a change to it would check nothing."""
    problems = []
    for name, c in sorted(ws.crates.items()):
        below, todo = set(), [name]
        while todo:
            n = todo.pop()
            for d in ws.crates[n]["uses"] | ws.crates[n].get("dev", set()):
                if d not in below:
                    below.add(d)
                    todo.append(d)
        dirs = [ws.crates[n]["dir"] for n in below | {name}]
        top = os.path.join(root, c["dir"])
        for here, subdirs, files in os.walk(top):
            subdirs[:] = [d for d in subdirs if d != "target"
                          and not os.path.exists(os.path.join(here, d, "Cargo.toml"))]
            for f in files:
                if not f.endswith(".rs"):
                    continue
                text = open(os.path.join(here, f), errors="replace").read()
                found = [(os.path.join(here, m), m) for m in INCLUDE.findall(text)]
                found += [(os.path.join(top, m), m) for m in MANIFEST_JOIN.findall(text)]
                for target, spelled in found:
                    rel = os.path.relpath(os.path.normpath(target), root)
                    if rel.startswith("..") or not os.path.exists(os.path.join(root, rel)):
                        continue
                    # A folder above the crate, such as the root, is only a base for a path
                    # made at run time.
                    if rel == "." or c["dir"] == rel or c["dir"].startswith(rel + "/"):
                        continue
                    if any(d == "." or rel == d or rel.startswith(d + "/") for d in dirs):
                        continue
                    if any(matches(rel, r) for r in c["reads"]):
                        continue
                    src = os.path.relpath(os.path.join(here, f), root)
                    problems.append(f"{src} reads {spelled} ({rel}), which {name} does not "
                                    "declare under [package.metadata.ci] reads")
    return problems


# The self-test: a small workspace shaped like the real one, and change lists with the crates
# they must pick. Run by ci.yml and local-ci.sh --quick.

FAKE = {
    "contract": {"dir": "contract", "reads": ["i18n/"], "uses": set()},
    "host-api": {"dir": "host-api", "reads": [], "uses": set()},
    "room": {"dir": "room", "reads": [], "uses": set()},
    "foundation": {"dir": "foundation", "reads": [], "uses": {"contract", "host-api", "room"}},
    "net": {"dir": "net", "reads": [], "uses": {"contract", "foundation"}},
    "library": {"dir": "library", "reads": [], "uses": {"foundation", "net"}},
    "metadata": {"dir": "metadata", "reads": [], "uses": {"library"}},
    "trackers": {"dir": "trackers", "reads": [], "uses": {"metadata"}},
    "relay": {"dir": "apps/relay", "reads": [], "uses": {"room"}},
    "core": {"dir": "core", "reads": ["apps/linux/packaging/", "apps/*/changes/*"],
             "uses": {"trackers", "room"}, "dev": {"relay"}},
    "cli": {"dir": "apps/cli", "reads": [], "uses": {"core"}},
    "shell": {"dir": "apps/linux", "reads": [], "uses": {"core"}},
}
ALL = set(FAKE)
ABOVE_FOUNDATION = {"foundation", "net", "library", "metadata", "trackers", "core", "cli", "shell"}
CASES = [
    # A change low down checks everything above it, and nothing beside it.
    (["foundation/src/store.rs"], ABOVE_FOUNDATION),
    (["trackers/src/anilist.rs"], {"trackers", "core", "cli", "shell"}),
    (["apps/cli/src/main.rs"], {"cli"}),
    # The relay builds on the room, and the core's tests use the relay.
    (["room/src/lib.rs"], {"room", "foundation", "net", "library", "metadata", "trackers",
                           "relay", "core", "cli", "shell"}),
    # A dev-dependency reaches its direct user and stops: cli never builds the relay.
    (["apps/relay/src/main.rs"], {"relay", "core"}),
    # Two changes pick the union.
    (["net/src/a.rs", "apps/cli/src/main.rs"], {"net", "library", "metadata", "trackers", "core",
                                                "cli", "shell"}),
    # A crate's own manifest is the crate's, not the workspace's.
    (["library/Cargo.toml"], {"library", "metadata", "trackers", "core", "cli", "shell"}),
    # Files outside a crate that it declares it reads.
    (["i18n/en/core.ftl"], ALL - {"host-api", "room", "relay"}),
    (["apps/linux/packaging/linux/appimage.sh"], {"shell", "core", "cli"}),
    (["apps/android/changes/12-x.md"], {"core", "cli", "shell"}),
    # Touches no crate.
    (["docs/agents/triage.md"], set()),
    (["README.md", "site/index.html", ".github/workflows/site.yml"], set()),
    (["apps/android/src/main.rs"], set()),
    ([], set()),
    # Workspace-wide.
    (["Cargo.lock"], ALL),
    (["Cargo.toml"], ALL),
    (["rust-toolchain.toml"], ALL),
    ([".github/affected-crates.py"], ALL),
    ([".github/workflows/ci.yml", "docs/x.md"], ALL),
    # Unknown files outside every crate are never taken as harmless.
    (["tools/new-thing.rs"], ALL),
]


def self_test():
    ws = Workspace(FAKE)
    bad = 0
    for files, want in CASES:
        got, alone, _ = select(ws, files)
        if set(got) != want:
            bad += 1
            print(f"self-test: {files}: got {sorted(got)}, want {sorted(want)}", file=sys.stderr)
        if any(not (ws.users[n] or ws.dev_users[n]) for n in alone) or any(n not in got for n in alone):
            bad += 1
            print(f"self-test: {files}: alone {alone} has a crate nothing builds on", file=sys.stderr)
    # Skipping leaves the rest, and dependencies come first.
    got, _, _ = select(ws, ["foundation/x.rs"], skip=["shell"])
    if "shell" in got or got.index("foundation") > got.index("core"):
        bad += 1
        print(f"self-test: skip or order wrong: {got}", file=sys.stderr)
    got, _, _ = select(ws, [], everything=True)
    if set(got) != ALL:
        bad += 1
        print("self-test: --all does not pick every crate", file=sys.stderr)
    total = len(CASES) + 2
    # The real workspace: a declared read that names no file would silently check nothing, and
    # a change at the bottom has to reach the crates the shells build on.
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    real = Workspace.from_metadata(metadata(root))
    for name, c in sorted(real.crates.items()):
        for r in c["reads"]:
            total += 1
            if not any(ch in r for ch in "*?[") and not os.path.exists(os.path.join(root, r)):
                bad += 1
                print(f"self-test: {name} reads {r}, which does not exist", file=sys.stderr)
    total += 1
    problems = undeclared_reads(real, root)
    for problem in problems:
        print(f"self-test: {problem}", file=sys.stderr)
    bad += 1 if problems else 0
    total += 1
    got, _, _ = select(real, ["foundation/src/x.rs"])
    want = {"anibeam-foundation", "anibeam-net", "anibeam-library", "anibeam-trackers",
            "anibeam-core", "anibeam-cli", "anibeam"}
    if not want <= set(got) or "anibeam-contract" in got:
        bad += 1
        print(f"self-test: a change in foundation/ picks {got}", file=sys.stderr)
    print(f"self-test: {total - bad} of {total} passed", file=sys.stderr)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
