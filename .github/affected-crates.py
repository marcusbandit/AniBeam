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
  --output FILE     also write crates=, alone=, core=, tests=, doc= and lock= lines to FILE
  --github-output   the same into $GITHUB_OUTPUT

Prints one crate per line on stdout, dependencies first, and why each was picked (and what
was skipped) on stderr. `alone` is the picked crates another crate builds on: clippy runs on
each of those by itself too, since one cargo call over several crates merges their features
and hides a use the crate does not turn on itself. `doc` is the picked crates whose library has a
doctest, a Rust block in a doc comment, for `cargo test --doc`: that compiles every library it
names even to run nothing, and naming only binary-only crates fails.

How a changed file maps to crates:
  1. Workspace-wide files (WORKSPACE_WIDE below): every crate.
  2. A file under a crate's folder: that crate. A file outside it that a crate's build or
     tests read is declared in that crate's manifest, relative to its folder:
         [package.metadata.ci]
         reads = ["../i18n"]
  3. A file outside the crate that only one of its test targets reads picks that target and
     nothing else: no clippy, no other test, no crate above it. `tests=` lists them as
     crate:target. The core's patch note test is one (#1086):
         [package.metadata.ci]
         test-reads = { changelog = ["../apps/linux/CHANGELOG.md"] }
  4. Cargo.lock, when the lock it is compared with is at hand (--base): the workspace crates
     that use a package whose entry changed, directly or not. Without it, every crate.
  5. A file no crate owns or reads that sits under NO_CRATE below (docs, the site, the phone's
     own workspace, other workflows): none.
  6. Anything else: every crate. An unknown file is never taken to be harmless.
"""

import argparse
import fnmatch
import json
import os
import re
import subprocess
import sys
import tomllib

# Files every crate builds with, so a change to one checks everything. The CI files are here so
# a change to the selection itself is checked against the whole workspace. Cargo.lock is too,
# unless select() is handed what its change reaches (lock_users below).
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
    ".config/nextest.toml",
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
    "apps/phone/",
    # Build output a pull leaves behind in a checkout from before the move (#1309).
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
        #                 "dev": {names only its tests use},
        #                 "test_reads": {"changelog": ["apps/linux/CHANGELOG.md"]}}}
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
            ci = (p.get("metadata") or {}).get("ci") or {}

            def from_root(paths):
                paths = [os.path.relpath(os.path.normpath(os.path.join(root, d, r)), root)
                         for r in paths]
                return [r + "/" if os.path.isdir(os.path.join(root, r)) else r for r in paths]

            crates[p["name"]] = {
                "dir": d,
                "reads": from_root(ci.get("reads") or []),
                "test_reads": {t: from_root(rs) for t, rs in (ci.get("test-reads") or {}).items()},
                "lib_roots": [t["src_path"] for t in p["targets"]
                              if set(t["kind"]) & {"lib", "rlib", "dylib", "proc-macro"}],
                "deps": p["dependencies"],
            }
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

    def test_readers(self, path):
        """(crate, test target) for each test target that declares it reads the file."""
        return sorted((n, t) for n, c in self.crates.items()
                      for t, rs in c.get("test_reads", {}).items()
                      if any(matches(path, r) for r in rs))

    def below(self, names, dev=False):
        """The crates these build on, directly or not; with dev, their tests' too."""
        out, todo = set(), list(names)
        while todo:
            n = todo.pop()
            for d in self.crates[n]["uses"] | (self.crates[n].get("dev", set()) if dev else set()):
                if d not in out:
                    out.add(d)
                    todo.append(d)
        return out

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


def select(ws, files, everything=False, skip=(), lock=None):
    """Returns (crates, alone, tests, why). tests is crate:target for a test target to run by
    itself; why is a list of lines for a log reader. lock is the set of crates the Cargo.lock
    change reaches (lock_users), or None when it is not known."""
    why = []
    picked = {}  # crate: the first file that picked it
    tests = {}  # (crate, target): the first file that picked it
    if everything:
        why.append("every crate: asked for all")
        picked = {n: None for n in ws.crates}
    for f in files:
        if everything:
            break
        if f == "Cargo.lock" and lock is not None:
            if not lock:
                why.append("no crate: Cargo.lock changes no package a crate uses")
            for n in sorted(lock):
                picked.setdefault(n, "Cargo.lock")
            continue
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
        # A file can be a crate's own and read by another's test too: a patch note is the
        # shell's, and the core's changelog test reads it.
        targets = ws.test_readers(f)
        for t in targets:
            tests.setdefault(t, f)
        for n in sorted(owners):
            picked.setdefault(n, f)
        if owners or targets:
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
    # A crate checked in full runs every test target anyway.
    alone_tests = []
    for (n, t), f in sorted(tests.items()):
        if n in picked:
            continue
        if n in skip:
            why.append(f"left out: {n} --test {t}, skipped on this machine")
            continue
        why.append(f"{n} --test {t} only: {f} changed")
        alone_tests.append(f"{n}:{t}")
    # A crate that runs one test target only is still not checked: no clippy, no other test.
    unpicked = sorted(set(ws.crates) - set(picked))
    if unpicked:
        why.append("not checked: " + ", ".join(unpicked))
    crates = ws.order(names)
    alone = [n for n in crates if ws.users[n] or ws.dev_users[n]]
    return crates, alone, alone_tests, why


def lock_users(old_text, new_text, members):
    """The workspace members a Cargo.lock change reaches: each one whose own entry changed, or
    that uses a package whose entry changed, directly or not, in the old lock or the new one.
    A package counts as changed when it came, went, or its checksum or what it uses changed.
    Returns None when either lock cannot be read or something besides the packages changed,
    so the caller checks every crate. Only the members' own crates are named; select() adds
    the crates above them."""
    try:
        old, new = tomllib.loads(old_text), tomllib.loads(new_text)
    except (tomllib.TOMLDecodeError, TypeError):
        return None
    if {k: v for k, v in old.items() if k != "package"} != \
            {k: v for k, v in new.items() if k != "package"}:
        return None

    def graph(lock):
        pkgs = lock.get("package") or []
        ids = [(p["name"], p.get("version", ""), p.get("source", "")) for p in pkgs]
        by_name = {}
        for i in ids:
            by_name.setdefault(i[0], []).append(i)

        def resolve(spec):
            # "name", "name version" or "name version (source)", as Cargo.lock spells a use.
            m = re.fullmatch(r"(\S+)(?: (\S+))?(?: \((.+)\))?", spec)
            if not m:
                raise ValueError(spec)
            name, version, source = m.groups()
            found = [i for i in by_name.get(name, [])
                     if (version is None or i[1] == version) and (source is None or i[2] == source)]
            # A use that names no package, or several, would drop an edge and miss its users.
            if len(found) != 1:
                raise ValueError(spec)
            return found[0]

        uses = {}
        sign = {}
        for i, p in zip(ids, pkgs):
            uses[i] = {resolve(d) for d in p.get("dependencies") or []}
            sign[i] = (p.get("checksum"), tuple(sorted(uses[i])))
        return uses, sign

    try:
        old_uses, old_sign = graph(old)
        new_uses, new_sign = graph(new)
    except (KeyError, TypeError, AttributeError, ValueError):
        return None
    changed = {i for i in set(old_sign) | set(new_sign) if old_sign.get(i) != new_sign.get(i)}
    reached = set()
    for uses in (old_uses, new_uses):
        users = {}
        for i, ds in uses.items():
            for d in ds:
                users.setdefault(d, set()).add(i)
        todo = [i for i in changed if i in uses]
        seen = set(todo)
        while todo:
            i = todo.pop()
            for u in users.get(i, ()):
                if u not in seen:
                    seen.add(u)
                    todo.append(u)
        reached |= seen
    # A member is a package with no source; a path dependency outside the workspace has none
    # either, and its users are reached through it.
    return {i[0] for i in reached if not i[2] and i[0] in members}


# A doc comment line that opens or closes a fenced block, and the info strings rustdoc compiles
# and runs as a doctest.
FENCE = re.compile(r"^\s*//[/!]\s?\s*(```+|~~~+)\s*([^\s`]*)")
NOT_RUST = {"text", "ignore", "sh", "bash", "console", "json", "toml", "yaml", "md", "markdown",
            "html", "css", "sql", "diff", "ini", "xml", "qml", "kotlin", "swift", "c", "cpp",
            "python", "js", "ts"}


def doc_blocks(text):
    """The info string of each fenced block opened in a doc comment."""
    out, open_fence = [], None
    for line in text.splitlines():
        m = FENCE.match(line)
        if not m:
            continue
        if open_fence is None:
            open_fence = m.group(1)
            out.append(m.group(2))
        elif m.group(1).startswith(open_fence[0] * len(open_fence)) and not m.group(2):
            open_fence = None
    return out


def is_doctest(info):
    words = [w for w in re.split(r"[,\s]+", info) if w]
    return not words or not any(w in NOT_RUST for w in words)


# Doc forms rustdoc can run code from that doc_blocks does not read: a block comment, docs from
# an attribute (#[doc = include_str!("../README.md")]), and an indented code block, a doc line
# indented four spaces after a blank doc line. Any of them counts as a doctest, so a mistake
# here runs cargo test --doc for nothing rather than leaving a doctest unrun.
OTHER_DOCS = re.compile(r"^\s*(/\*\*[^*/]|/\*!|#!?\[doc\s*=)", re.M)
INDENTED = re.compile(r"^\s*//[/!][ \t]*\n\s*//[/!] {5,}\S", re.M)


def has_doctest(lib_root):
    """Whether any .rs file in the library's folder may hold a doctest. Over-counts a file the
    library never includes, and some prose, which only runs cargo test --doc for nothing. A
    library that cannot be read counts as having one."""
    if not os.path.isfile(lib_root):
        return True
    top = os.path.dirname(lib_root)
    for here, subdirs, files in os.walk(top):
        subdirs[:] = [d for d in subdirs if d != "target"]
        for f in files:
            if f.endswith(".rs"):
                with open(os.path.join(here, f), errors="replace") as fh:
                    text = fh.read()
                if any(is_doctest(i) for i in doc_blocks(text)):
                    return True
                if OTHER_DOCS.search(text) or INDENTED.search(text):
                    return True
    return False


def git(root, *args):
    return subprocess.run(["git", "-C", root, *args], check=True, capture_output=True,
                          text=True).stdout


def changed_files(root, base):
    mb = git(root, "merge-base", "HEAD", base).strip()
    # NUL-separated, so a path with any character in it comes back as it is, unquoted. Renames
    # show as a delete and an add, so the crate a file left is checked too.
    files = git(root, "diff", "-z", "--name-only", "--no-renames", "--relative", mb).split("\0")
    files += git(root, "ls-files", "-z", "--others", "--exclude-standard").split("\0")
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
    lock = None
    if a.base:
        files, mb = changed_files(root, a.base)
        print(f"affected: {len(files)} files changed since {mb[:10]}", file=sys.stderr)
        if "Cargo.lock" in files:
            try:
                old = git(root, "show", f"{mb}:Cargo.lock")
                with open(os.path.join(root, "Cargo.lock")) as f:
                    lock = lock_users(old, f.read(), set(ws.crates))
            except (subprocess.CalledProcessError, OSError):
                lock = None
    elif a.files is not None:
        files = a.files
    crates, alone, tests, why = select(ws, files, everything=a.all, skip=a.skip, lock=lock)
    for line in why:
        print(f"affected: {line}", file=sys.stderr)
    if not crates and not tests:
        print("affected: nothing to clippy or test", file=sys.stderr)
    for n in crates:
        print(n)
    for t in tests:
        print(f"{t} (that test only)")
    out = os.environ["GITHUB_OUTPUT"] if a.github_output else a.output
    if out:
        with open(out, "a") as f:
            f.write(f"crates={' '.join(crates)}\n")
            f.write(f"alone={' '.join(alone)}\n")
            f.write(f"core={'true' if 'anibeam-core' in crates else 'false'}\n")
            f.write(f"tests={' '.join(tests)}\n")
            # A lock change is checked against the manifests even when it reaches no crate.
            f.write(f"lock={'true' if 'Cargo.lock' in files or a.all else 'false'}\n")
            # Worked out for the picked crates only, since it reads their sources.
            doc = [n for n in crates
                   if any(has_doctest(r) for r in ws.crates[n].get("lib_roots", ["?"]))]
            f.write(f"doc={' '.join(doc)}\n")
    return 0


# Paths spelled out in a crate's Rust: include_str!("..."), include_bytes!("..."), include!("...")
# relative to the file, and env!("CARGO_MANIFEST_DIR")).join("...") relative to the crate. A path
# built at run time cannot be seen here; declare those by hand.
INCLUDE = re.compile(r'include(?:_str|_bytes)?!\(\s*"([^"]+)"\s*\)')
MANIFEST_JOIN = re.compile(r'env!\("CARGO_MANIFEST_DIR"\)\s*\)\s*\.join\(\s*"([^"]+)"\s*\)')


def named_paths(root, crate_dir):
    """(source file, path as spelled, path from the root) for every existing file or folder a
    crate's Rust names, other than its own folder and the folders above it, which are only a
    base for a path made at run time. Nested crates and target/ are left out."""
    top = os.path.join(root, crate_dir)
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
                if rel == "." or crate_dir == rel or crate_dir.startswith(rel + "/"):
                    continue
                yield os.path.relpath(os.path.join(here, f), root), spelled, rel


def undeclared_reads(ws, root):
    """Every file a crate's Rust names outside its own folder has to sit in a crate it builds on
    or in its reads, or a change to it would check nothing."""
    problems = []
    for name, c in sorted(ws.crates.items()):
        dirs = [ws.crates[n]["dir"] for n in ws.below([name], dev=True) | {name}]
        tests_dir = os.path.join(c["dir"], "tests")
        for src, spelled, rel in named_paths(root, c["dir"]):
            if any(d == "." or rel == d or rel.startswith(d + "/") for d in dirs):
                continue
            if any(matches(rel, r) for r in c["reads"]):
                continue
            # A test target's declared reads cover its own file or folder in tests/.
            if any(src in (os.path.join(tests_dir, t + ".rs"),)
                   or src.startswith(os.path.join(tests_dir, t) + "/")
                   for t, rs in c.get("test_reads", {}).items()
                   if any(matches(rel, r) for r in rs)):
                continue
            problems.append(f"{src} reads {spelled} ({rel}), which {name} does not "
                            "declare under [package.metadata.ci] reads (or test-reads, "
                            "for the test target that reads it)")
    return problems


# The lists outside ci.yml that say which paths an app is built from (#974): nightly.yml's change
# checks and android.yml's path filters. Unlike the crates above they are written out by hand, so
# the self-test holds them to the workspace: each takes in every file of every crate the app
# builds, at any depth, and every file outside those crates that one of them reads, or a change
# that touches only that crate (room/, recs/) skips the build though the binary differs. The
# desktop apps are workspace members; the phone is its own workspace, read with its own
# `cargo metadata`. ios.yml leaves the shared crates out on purpose and is not checked.
DESKTOP_APPS = ["apps/linux", "apps/cli"]
PHONE_APP = "apps/phone"
# A crate's declared read that only its own tests make, which no app build compiles in.
TEST_ONLY_READS = {
    ("anibeam-downloads", "apps/linux/packaging/"): "downloads/src/anirss.rs checks the "
                                                    "packaging scripts in a test",
}
# Root files each app's build reads besides the crates. The phone has its own Cargo.lock.
ROOT_FILES = {"desktop": ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"],
              "phone": ["Cargo.toml", "rust-toolchain.toml"]}
APP_LISTS = [
    # (workflow, list name, app, how the list is matched)
    (".github/workflows/nightly.yml", "desktop_paths", "desktop", "pathspec"),
    (".github/workflows/nightly.yml", "phone_paths", "phone", "pathspec"),
    (".github/workflows/android.yml", "push", "phone", "glob"),
    (".github/workflows/android.yml", "pull_request", "phone", "glob"),
]


def phone_crates(root):
    """The phone workspace's crates: their folders and declared reads from the root, and the
    root workspace folders they name by path, any kind of dependency, inherited or not."""
    meta = metadata(os.path.join(root, PHONE_APP))
    members = set(meta["workspace_members"])
    out = []
    for p in meta["packages"]:
        if p["id"] not in members:
            continue
        here = os.path.dirname(p["manifest_path"])
        reads = ((p.get("metadata") or {}).get("ci") or {}).get("reads") or []
        out.append({
            "dir": os.path.relpath(here, root),
            "reads": [os.path.relpath(os.path.normpath(os.path.join(here, r)), root) for r in reads],
            "paths": [os.path.relpath(os.path.normpath(d["path"]), root)
                      for d in p["dependencies"] if d.get("path")],
        })
    return out


def app_needs(ws, root, app, phone=None):
    """What a change to which changes the app: {path: (why, whole)}. whole says every file under
    the path has to be taken in; the app's own folder, which may leave parts out on purpose (the
    iPhone's files in android.yml), needs only some of it."""
    if app == "desktop":
        tops = {n for n, c in ws.crates.items() if c["dir"] in DESKTOP_APPS}
        need = {p: ("the app itself", False) for p in DESKTOP_APPS}
    else:
        by_dir = {c["dir"]: n for n, c in ws.crates.items()}
        phone = phone_crates(root) if phone is None else phone
        tops = {by_dir[d] for c in phone for d in c["paths"] if d in by_dir}
        need = {PHONE_APP: ("the app itself", False)}
        for c in phone:
            outside = c["reads"] + [rel for _, _, rel in named_paths(root, c["dir"])]
            for r in outside:
                if r != PHONE_APP and not r.startswith(PHONE_APP + "/") and r != "." \
                        and not c["dir"].startswith(r + "/"):
                    need.setdefault(r, (f"{c['dir']} reads it", True))
    for n in sorted(ws.below(tops) | tops):
        need.setdefault(ws.crates[n]["dir"], (f"{n} builds into it", True))
        for r in ws.crates[n]["reads"]:
            if (n, r) not in TEST_ONLY_READS:
                need.setdefault(r.rstrip("/"), (f"{n} reads it", True))
    for f in ROOT_FILES[app]:
        need.setdefault(f, ("every build reads it", True))
    # A read inside a crate the app builds is checked with that crate.
    return {p: v for p, v in need.items()
            if not any(p.startswith(q + "/") and w for q, (_, w) in need.items())}


def workflow_list(text, name):
    """The entries of one list: a bash array `name=(a b c)`, or the `paths:` under the trigger
    `name:` in a workflow. None when the list is not there."""
    m = re.search(rf"^\s*{re.escape(name)}=\(([^)]*)\)", text, re.M)
    if m:
        return m.group(1).split()
    lines = text.splitlines()
    for i, line in enumerate(lines):
        if not re.fullmatch(rf"  {re.escape(name)}:\s*", line):
            continue
        out, inside = [], False
        for line in lines[i + 1:]:
            s = line.strip()
            if not inside:
                if s == "paths:":
                    inside = True
                elif s and not s.startswith("#") and not line.startswith("    "):
                    break
                continue
            if not s or s.startswith("#"):
                continue
            m = re.fullmatch(r"-\s*(?:'([^']*)'|\"([^\"]*)\"|([^\s#'\"]+))\s*(?:#.*)?", s)
            if not m:
                break
            out.append(next(g for g in m.groups() if g is not None))
        return out if inside else None
    return None


def glob_re(pattern):
    """A GitHub paths filter pattern as a regex: * stops at a slash, ** does not."""
    out, i = "", 0
    while i < len(pattern):
        if pattern.startswith("**", i):
            out, i = out + ".*", i + 2
        elif pattern[i] == "*":
            out, i = out + "[^/]*", i + 1
        elif pattern[i] == "?":
            out, i = out + "[^/]", i + 1
        else:
            out, i = out + re.escape(pattern[i]), i + 1
    return re.compile(out)


def covers(entry, path, how):
    """Whether one list entry takes in a file. A pathspec (`git diff -- room`) is a file or a
    folder by name; a glob is GitHub's, matched against the whole path."""
    if how == "pathspec":
        return path == entry or path.startswith(entry.rstrip("/") + "/")
    return glob_re(entry).fullmatch(path) is not None


def taken(entries, path, how):
    """Whether a list takes in a file. GitHub reads the patterns in order and the last one that
    matches wins; a pathspec list has no exclusions."""
    state = False
    for e in entries:
        if covers(e.removeprefix("!"), path, how):
            state = not e.startswith("!")
    return state


def unlisted(ws, root, texts=None, files=None, phone=None):
    """Problems with the app lists: a path an app needs that a list leaves out or excludes."""
    if files is None:
        files = [f for f in git(root, "ls-files", "-z", "--cached", "--others",
                                "--exclude-standard").split("\0") if f]
    needs = {app: app_needs(ws, root, app, phone) for app in ROOT_FILES}
    problems = []
    for (n, r), why in sorted(TEST_ONLY_READS.items()):
        if n not in ws.crates or r not in ws.crates[n]["reads"]:
            problems.append(f"TEST_ONLY_READS names {r} for {n}, which {n} does not declare")
    for wf, name, app, how in APP_LISTS:
        if texts is not None:
            text = texts.get(wf, "")
        else:
            with open(os.path.join(root, wf)) as f:
                text = f.read()
        entries = workflow_list(text, name)
        if entries is None:
            problems.append(f"{wf}: no list {name}")
            continue
        for path, (why, whole) in sorted(needs[app].items()):
            inside = [f for f in files if f == path or f.startswith(path + "/")] or [path]
            out = [f for f in inside if not taken(entries, f, how)]
            if len(out) == len(inside):
                problems.append(f"{wf}: {name} leaves out {path} ({why})")
            elif out and whole:
                problems.append(f"{wf}: {name} excludes {out[0]} in {path} ({why})")
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
    "core": {"dir": "core", "reads": ["apps/linux/packaging/"],
             "test_reads": {"changelog": ["apps/linux/CHANGELOG.md", "apps/phone/CHANGELOG.md"]},
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
    # Patch notes: the shell builds its own in, the core only tests them (TEST_CASES).
    (["apps/linux/CHANGELOG.md"], {"shell"}),
    (["apps/phone/CHANGELOG.md"], set()),
    # Touches no crate.
    (["docs/agents/triage.md"], set()),
    (["README.md", "site/index.html", ".github/workflows/site.yml"], set()),
    (["apps/phone/src/main.rs"], set()),
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


# Files a test target reads by itself: the crates to check in full, and the targets to run.
TEST_CASES = [
    (["apps/phone/CHANGELOG.md"], set(), ["core:changelog"]),
    (["apps/linux/CHANGELOG.md"], {"shell"}, ["core:changelog"]),
    (["apps/linux/CHANGELOG.md", "apps/phone/CHANGELOG.md"], {"shell"}, ["core:changelog"]),
    # The core checked in full runs the target anyway.
    (["apps/phone/CHANGELOG.md", "trackers/src/a.rs"], {"trackers", "core", "cli", "shell"},
     []),
    (["docs/x.md"], set(), []),
]

# Cargo.lock: what each change reaches, from a lock shaped like the real one.
LOCK = """version = 4

[[package]]
name = "core"
version = "0.1.0"
dependencies = ["serde", "trackers"]

[[package]]
name = "trackers"
version = "0.1.0"
dependencies = ["serde", "url"]

[[package]]
name = "shell"
version = "0.1.0"
dependencies = ["core", "qt"]

[[package]]
name = "qt"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aa"

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bb"

[[package]]
name = "url"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cc"
dependencies = ["idna"]

[[package]]
name = "idna"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dd"
"""
LOCK_MEMBERS = {"core", "trackers", "shell"}
SHELL_DEP = LOCK.replace('dependencies = ["core", "qt"]', 'dependencies = ["core", "icns", "qt"]') + """
[[package]]
name = "icns"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ee"
"""
LOCK_CASES = [
    # A use that names no package in the lock: unknown, so every crate.
    (LOCK.replace('dependencies = ["core", "qt"]', 'dependencies = ["core", "gone"]'), None),
    # A dependency only the shell takes reaches the shell alone.
    (SHELL_DEP, {"shell"}),
    # A bump deep down reaches every member that uses it, through the packages between.
    (LOCK.replace('checksum = "dd"', 'checksum = "d2"'), {"trackers", "core", "shell"}),
    (LOCK.replace('checksum = "bb"', 'checksum = "b2"'), {"trackers", "core", "shell"}),
    (LOCK, set()),
    # A second version of a package renames the plain uses of the first, which still reach the
    # same package; nothing uses the new one yet.
    (LOCK.replace('dependencies = ["core", "qt"]', 'dependencies = ["core", "qt 1.0.0"]')
     + '\n[[package]]\nname = "qt"\nversion = "2.0.0"\nsource = "x"\nchecksum = "ff"\n',
     set()),
    # The lock's own version, or a lock that does not parse: unknown, so every crate.
    (LOCK.replace("version = 4", "version = 5"), None),
    ("not [ toml", None),
]


def self_test():
    global SCRATCH
    import tempfile
    SCRATCH = tempfile.mkdtemp(prefix="affected-crates-")
    ws = Workspace(FAKE)
    bad = 0
    for files, want in CASES:
        got, alone, _, _ = select(ws, files)
        if set(got) != want:
            bad += 1
            print(f"self-test: {files}: got {sorted(got)}, want {sorted(want)}", file=sys.stderr)
        if any(not (ws.users[n] or ws.dev_users[n]) for n in alone) or any(n not in got for n in alone):
            bad += 1
            print(f"self-test: {files}: alone {alone} has a crate nothing builds on", file=sys.stderr)
    for files, want, want_tests in TEST_CASES:
        got, _, tests, _ = select(ws, files)
        if set(got) != want or tests != want_tests:
            bad += 1
            print(f"self-test: {files}: got {sorted(got)} and {tests}, want {sorted(want)} and "
                  f"{want_tests}", file=sys.stderr)
    for new, want in LOCK_CASES:
        got = lock_users(LOCK, new, LOCK_MEMBERS)
        if got != want:
            bad += 1
            print(f"self-test: lock change: got {got}, want {want}", file=sys.stderr)
    # The lock's reach feeds select(): the crates above it come along, and none means none.
    got, _, _, _ = select(ws, ["Cargo.lock"], lock={"trackers"})
    if set(got) != {"trackers", "core", "cli", "shell"}:
        bad += 1
        print(f"self-test: Cargo.lock reaching trackers picks {got}", file=sys.stderr)
    got, _, _, _ = select(ws, ["Cargo.lock"], lock=set())
    if got:
        bad += 1
        print(f"self-test: Cargo.lock reaching nothing picks {got}", file=sys.stderr)
    # Skipping leaves the rest, and dependencies come first.
    got, _, _, _ = select(ws, ["foundation/x.rs"], skip=["shell"])
    if "shell" in got or got.index("foundation") > got.index("core"):
        bad += 1
        print(f"self-test: skip or order wrong: {got}", file=sys.stderr)
    got, _, _, _ = select(ws, [], everything=True)
    if set(got) != ALL:
        bad += 1
        print("self-test: --all does not pick every crate", file=sys.stderr)
    total = len(CASES) + len(TEST_CASES) + len(LOCK_CASES) + 4
    # The real workspace: a declared read that names no file would silently check nothing, and
    # a change at the bottom has to reach the crates the shells build on.
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    real = Workspace.from_metadata(metadata(root))
    for name, c in sorted(real.crates.items()):
        for r in c["reads"] + [r for rs in c["test_reads"].values() for r in rs]:
            total += 1
            if not any(ch in r for ch in "*?[") and not os.path.exists(os.path.join(root, r)):
                bad += 1
                print(f"self-test: {name} reads {r}, which does not exist", file=sys.stderr)
    total += 1
    problems = undeclared_reads(real, root)
    for problem in problems:
        print(f"self-test: {problem}", file=sys.stderr)
    bad += 1 if problems else 0
    # The app lists' parser, on lists shaped like the real ones.
    flow = ("on:\n  push:\n    branches: [main]\n    paths:\n      - 'a/**'\n      # note\n"
            "      - \"b/**\"  # b's\n      - c/**\n  pull_request:\n    paths:\n      - '**'\n"
            "      - '!c/**'\n  workflow_dispatch:\n")
    for text, name, want in [
        ("  x=(contract room Cargo.toml)\n", "x", ["contract", "room", "Cargo.toml"]),
        (flow, "push", ["a/**", "b/**", "c/**"]),
        (flow, "pull_request", ["**", "!c/**"]),
        (flow, "workflow_dispatch", None),
        (flow, "x", None),
    ]:
        total += 1
        if workflow_list(text, name) != want:
            bad += 1
            print(f"self-test: list {name} reads as {workflow_list(text, name)}, want {want}",
                  file=sys.stderr)
    for entry, path, how, want in [
        ("room/**", "room/src/lib.rs", "glob", True), ("room/**", "roomy/a.rs", "glob", False),
        ("apps/*", "apps/phone/src/main.rs", "glob", False), ("*", "core/a.rs", "glob", False),
        ("**", "x/y", "glob", True), ("**.md", "docs/a.md", "glob", True),
        ("Cargo.toml", "core/Cargo.toml", "glob", False),
        ("apps/phone/src/**/ios.rs", "apps/phone/src/a/ios.rs", "glob", True),
        ("core", "core/tests/a.json", "pathspec", True), ("room", "roomy/a", "pathspec", False),
    ]:
        total += 1
        if covers(entry, path, how) != want:
            bad += 1
            print(f"self-test: {entry} covering {path} should be {want}", file=sys.stderr)
    # A list that leaves a crate out, or excludes part of one again, is caught; one that
    # excludes and then takes it in again is not.
    files = [f for f in git(root, "ls-files", "-z").split("\0") if f]
    phone = phone_crates(root)
    lists = {wf: "" for wf, _, _, _ in APP_LISTS}
    total += 1
    problems = unlisted(real, root, lists, files, phone)
    if len(problems) != len(APP_LISTS):
        bad += 1
        print(f"self-test: missing lists read as {problems}", file=sys.stderr)
    with open(os.path.join(root, ".github/workflows/android.yml")) as f:
        lists[".github/workflows/android.yml"] = f.read().replace("      - 'room/**'\n", "", 1) \
            .replace("      - 'core/**'\n", "      - 'core/**'\n      - '!core/src/**'\n") \
            .replace("      - 'i18n/**'\n", "      - '!i18n/**'\n      - 'i18n/**'\n")
    total += 1
    problems = [p for p in unlisted(real, root, lists, files, phone) if "android.yml" in p]
    want = [".github/workflows/android.yml: push leaves out room (anibeam-room builds into it)",
            ".github/workflows/android.yml: push excludes core/src/",
            ".github/workflows/android.yml: pull_request excludes core/src/"]
    if len(problems) != 3 or not all(p.startswith(w) for p, w in zip(sorted(problems),
                                                                       sorted(want))):
        bad += 1
        print(f"self-test: a list without room/ reads as {problems}", file=sys.stderr)
    # A phone crate that reads a file outside the phone needs it listed.
    total += 1
    extra = phone + [{"dir": PHONE_APP, "reads": ["docs"], "paths": []}]
    if "docs" not in app_needs(real, root, "phone", extra):
        bad += 1
        print("self-test: a phone read outside apps/phone is not needed", file=sys.stderr)
    total += 1
    problems = unlisted(real, root, None, files, phone)
    for problem in problems:
        print(f"self-test: {problem}", file=sys.stderr)
    bad += 1 if problems else 0
    total += 1
    got, _, _, _ = select(real, ["foundation/src/x.rs"])
    want = {"anibeam-foundation", "anibeam-net", "anibeam-library", "anibeam-trackers",
            "anibeam-core", "anibeam-cli", "anibeam"}
    if not want <= set(got) or "anibeam-contract" in got:
        bad += 1
        print(f"self-test: a change in foundation/ picks {got}", file=sys.stderr)
    # Which fenced doc blocks rustdoc runs.
    blocks = doc_blocks("/// ```\n/// a\n/// ```\n//! ```text\n//! x\n//! ```\n"
                        "/// ```rust,no_run\n/// b\n/// ```\n/// ```ignore\n/// ```\n")
    total += 1
    if [is_doctest(b) for b in blocks] != [True, False, True, False]:
        bad += 1
        print(f"self-test: doc blocks read as {blocks}", file=sys.stderr)
    for text, want in [("/// ```\n/// x\n/// ```\n", True), ("/// ```text\n/// ```\n", False),
                       ('#![doc = include_str!("../README.md")]\n', True),
                       ("/** Docs\n */\n", True), ("///\n///     let a = 1;\n", True),
                       ("/// - a\n///   b\n", False), ("// plain\n", False)]:
        total += 1
        d = os.path.join(SCRATCH, "doc-probe")
        os.makedirs(d, exist_ok=True)
        with open(os.path.join(d, "lib.rs"), "w") as fh:
            fh.write(text)
        if has_doctest(os.path.join(d, "lib.rs")) != want:
            bad += 1
            print(f"self-test: has_doctest({text!r}) should be {want}", file=sys.stderr)
    total += 1
    if not has_doctest("/no/such/lib.rs"):
        bad += 1
        print("self-test: an unreadable library should count as having doctests", file=sys.stderr)
    total += 1
    if any(has_doctest(r) for r in real.crates["anibeam-cli"]["lib_roots"]):
        bad += 1
        print("self-test: the cli has no library, so no doctests", file=sys.stderr)
    got, _, tests, why = select(ws, ["apps/phone/CHANGELOG.md"], skip=["core"])
    total += 1
    if tests or not any("left out: core --test changelog" in w for w in why):
        bad += 1
        print(f"self-test: a skipped test target should say so: {why}", file=sys.stderr)
    import shutil
    shutil.rmtree(SCRATCH, ignore_errors=True)
    print(f"self-test: {total - bad} of {total} passed", file=sys.stderr)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
