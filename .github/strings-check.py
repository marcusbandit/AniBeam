#!/usr/bin/env python3
"""Every word a person reads comes from the catalog in i18n/ (#792).

Two checks:

1. Raw strings. A string literal in a shell's UI files that reads as words for a person
   ("Skip intro", " episodes") is a hit. The words belong in i18n/en/<file>.ftl under an ID,
   and the UI asks for the ID. The files that held raw strings before the catalog existed are
   listed with their count in .github/strings-baseline.txt; a file may hold at most that many
   and the count only goes down. `--update` rewrites the baseline to today's counts, which a
   PR does once it moved strings out.
2. Missing IDs. An ID the code asks for by name (`tr("player-skip-intro"`, `msg:
   "player-skip-intro"`) has to exist in the English catalog.

A line that really holds no words for a person, such as a format string or a brand name the
check does not know, ends in `// i18n: none`.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / ".github" / "strings-baseline.txt"

# The UI files checked for raw strings. One glob a line; a shell adds its own.
UI = [
    "apps/linux/qml/**/*.qml",
    "apps/android/ui/**/*.slint",
]

# The files whose ID uses are checked against the English catalog.
CODE = UI + [
    "core/src/**/*.rs",
    "apps/linux/src/**/*.rs",
    "apps/cli/src/**/*.rs",
    "apps/android/src/**/*.rs",
]

# Properties that never hold words for a person, so a literal on their line is a code.
NOT_TEXT = re.compile(
    r"^\s*(objectName|role|icon|actionIcon|glyph|color|[a-zA-Z]*Color|source|font\.[a-zA-Z]+"
    r"|border\.[a-zA-Z]+|state|name|id|kind|key|rkey|code|codes|progressCode|tag|category"
    r"|nameFilters|import|property\s+(?:color|url))\s*:"
)

# Literals made only of these words stay as they are in every language.
BRANDS = {
    "AniBeam", "AniList", "MyAnimeList", "MAL", "TVmaze", "Jikan", "AniSkip", "qBittorrent",
    "Discord", "Tailscale", "mpv", "libmpv", "Qt", "GitHub", "AUR", "Chromecast", "DLNA",
    "RSS", "torznab", "anirss", "MPRIS", "Linux", "Windows", "Mac", "macOS", "Android",
}

# A property or key whose value is words, on a line such as `text: "Save"` or
# `{ label: "Off" }`. Off these lines only a literal with a space counts, since a lone
# capitalised token there is more often a code from the core ("Desc", "Anilist").
TEXT_PROP = re.compile(
    r"(?<![\w.])(?:[a-z]+[A-Z]?)?(?:[tT]ext|[tT]itle|[lL]abel|[hH]elper|[bB]ody|[hH]eading|[lL]ead"
    r"|[pP]laceholder|[tT]ip|[nN]ote|[hH]int|[cC]aption|[mM]essage|[qQ]uestion|[fF]act"
    r"|[pP]refix|[sS]uffix|[lL]ine|[dD]escription|[sS]ummary|[dD]etail|Accessible\.name"
    r"|model|[oO]ptions|[lL]abels|[iI]tems)\s*:"
)

# A literal compared against, or used as a case label or a key, is a code from the core.
# A list of labels (`model: ["Watching", "Completed"]`) is words; `x in ["a", "b"]` is not.
COMPARED = re.compile(r"(?:[!=]==?|\bcase|\bin\s*\[[^\]]*)\s*$")
COMPARED_AFTER = re.compile(r"\s*(?:[!=]==?|\bin\b)")
# Slint has double quotes only. QML is JavaScript too: single quotes and template literals,
# whose ${...} parts are expressions, not words.
STRING = re.compile(r'"((?:[^"\\]|\\.)*)"')
STRING_QML = re.compile(r'"((?:[^"\\]|\\.)*)"|\'((?:[^\'\\]|\\.)*)\'|`((?:[^`\\]|\\.)*)`')
TEMPLATE_PART = re.compile(r"\$\{[^}]*\}")
ID = r"([a-z][a-z0-9]*(?:-[a-z0-9]+)*(?:\.[a-z][a-z0-9-]*)?)"
ID_USES = [
    re.compile(r"\b(?:tr|tr_in|t)\(\s*(?:[A-Za-z:]+,\s*)?\"" + ID + r"\""),
    re.compile(r"\bTr\.(?:s|s2|s3|n|n2|ns|ns2)\(\s*\"" + ID + r"\""),
    re.compile(r"\b[a-z]*[mM]sg\s*:\s*\"" + ID + r"\""),
]
DEFINED = re.compile(r"^([a-zA-Z][a-zA-Z0-9_-]*)\s*=", re.M)
ATTR = re.compile(r"^\s+\.([a-zA-Z][a-zA-Z0-9_-]*)\s*=", re.M)


def files(globs):
    seen = set()
    for pattern in globs:
        for path in sorted(ROOT.glob(pattern)):
            if path.is_file() and path not in seen:
                seen.add(path)
                yield path


def is_words(literal, text_line):
    """Whether a literal reads as words for a person rather than a code or a path.
    `text_line` says the line sets a text property, where a single word is words too."""
    text = literal.strip()
    if not re.search(r"[^\W\d_]", text):
        return False
    words = re.findall(r"[^\W_]+", text)
    if words and all(w in BRANDS for w in words) and not re.search(r"\s", literal.strip()):
        return False
    # A literal with a space, or an edge space that joins it to a number, is words.
    if re.search(r"\s", literal):
        return True
    # One token: a code (kebab, snake, camel, a path, a url, a hex colour) unless it starts
    # with a capital and goes on in lower case, as a word does ("Save", "Episodes").
    return (
        text_line
        and bool(re.fullmatch(r"[A-Z][a-z]+[.!?:…]*", text))
        and text not in BRANDS
    )


def code_lines(path, quotes):
    """Each line of a file with its comments blanked to spaces, so columns still line up.
    A `/* */` comment may open and close anywhere, on one line or across several; `//` and
    `/*` inside a string literal (one of `quotes`) are part of the string."""
    in_block = False
    for n, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        out, i, quote = [], 0, None
        while i < len(line):
            ch = line[i]
            if in_block:
                if line.startswith("*/", i):
                    in_block = False
                    out.append("  ")
                    i += 2
                else:
                    out.append(" ")
                    i += 1
                continue
            if quote:
                out.append(ch)
                if ch == "\\" and i + 1 < len(line):
                    out.append(line[i + 1])
                    i += 2
                    continue
                if ch == quote:
                    quote = None
                i += 1
                continue
            if ch in quotes:
                quote = ch
            elif line.startswith("//", i):
                break
            elif line.startswith("/*", i):
                in_block = True
                out.append("  ")
                i += 2
                continue
            out.append(ch)
            i += 1
        yield n, line, "".join(out)


# The phone's lookups: Tr.t("id"), Tr.s("id", "name", value), Tr.n("id", "count", n). The ID
# and every argument name are codes; the values between them are expressions.
SLINT_TR = re.compile(r"Tr\.(?:t|s|s2|s3|n|n2|ns|ns2)\(")
SLINT_CODE = re.compile(
    r'import\s*\{[^}]*\}\s*from\s*"[^"]*"|import\s*"[^"]*"|@image-url\(\s*"[^"]*"\s*\)'
)


def slint_codes(line):
    """The spans of a Slint line whose literals are codes: imports, images, Tr IDs and names."""
    spans = [m.span() for m in SLINT_CODE.finditer(line)]
    for m in SLINT_TR.finditer(line):
        depth, j, arg = 1, m.end(), 0
        while j < len(line) and depth:
            ch = line[j]
            if ch == '"':
                k = j + 1
                while k < len(line) and not (line[k] == '"' and line[k - 1] != "\\"):
                    k += 1
                if arg == 0 or arg % 2 == 1:
                    spans.append((j, k + 1))
                j = k
            elif ch in "([":
                depth += 1
            elif ch in ")]":
                depth -= 1
            elif ch == "," and depth == 1:
                arg += 1
            j += 1
    return spans


def raw_strings_slint(path):
    """Slint has no JS, so every literal with a letter is words unless it is a code: a font
    name, an import, an image path, or a Tr call's ID and argument names."""
    hits = []
    theme = path.name == "theme.slint"
    for n, line, code in code_lines(path, '"'):
        if line.rstrip().endswith("// i18n: none"):
            continue
        if re.search(r"font-family\s*:", code) and "Tr." not in code:
            continue
        if theme and re.search(r"property\s*<string>", code):
            continue
        spans = slint_codes(code)
        for m in STRING.finditer(code):
            if any(a <= m.start() and m.end() <= b for a, b in spans):
                continue
            if re.search(r"[^\W\d_]", m.group(1)):
                hits.append((n, m.group(1)))
    return hits


def raw_strings(path):
    if path.suffix == ".slint":
        return raw_strings_slint(path)
    hits = []
    for n, line, code in code_lines(path, "\"'`"):
        if not code.strip() or code.lstrip().startswith("import "):
            continue
        if line.rstrip().endswith("// i18n: none") or NOT_TEXT.match(code):
            continue
        if re.search(r"\bconsole\.\w+\(|\bobjectName\b", code):
            continue
        for m in STRING_QML.finditer(code):
            if COMPARED.search(code[: m.start()]) or COMPARED_AFTER.match(code[m.end() :]):
                continue
            literal = next(g for g in m.groups() if g is not None)
            if m.group(3) is not None:
                literal = TEMPLATE_PART.sub(" ", literal)
            if is_words(literal, bool(TEXT_PROP.search(code))):
                hits.append((n, literal))
    return hits


def english_ids():
    ids = set()
    for ftl in sorted((ROOT / "i18n" / "en").glob("*.ftl")):
        text = ftl.read_text(encoding="utf-8")
        current = None
        for line in text.splitlines():
            m = DEFINED.match(line)
            if m:
                current = m.group(1)
                ids.add(current)
                continue
            a = ATTR.match(line)
            if a and current:
                ids.add(f"{current}.{a.group(1)}")
    return ids


def read_baseline():
    base = {}
    if BASELINE.exists():
        for line in BASELINE.read_text(encoding="utf-8").splitlines():
            if line.strip() and not line.startswith("#"):
                count, path = line.split(None, 1)
                base[path] = int(count)
    return base


def main():
    update = "--update" in sys.argv[1:]
    counts = {}
    shown = {}
    for path in files(UI):
        hits = raw_strings(path)
        if hits:
            rel = path.relative_to(ROOT).as_posix()
            counts[rel] = len(hits)
            shown[rel] = hits

    if update:
        lines = [
            "# Raw strings each UI file still holds, from .github/strings-check.py --update.",
            "# A count only ever goes down; a file not listed may hold none (#792).",
        ]
        lines += [f"{counts[p]} {p}" for p in sorted(counts)]
        BASELINE.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"Baseline written: {sum(counts.values())} raw strings in {len(counts)} files.")
        return 0

    failed = False
    base = read_baseline()
    for path in sorted(counts):
        allowed = base.get(path, 0)
        if counts[path] > allowed:
            failed = True
            print(f"{path}: {counts[path]} raw strings, {allowed} allowed. Each one:")
            for n, s in shown[path]:
                print(f"  {path}:{n}: \"{s}\"")
    # The count has to come down in the PR that moved the strings, or a later PR could add
    # the same number back unseen.
    shrunk = sorted(p for p in base if counts.get(p, 0) < base[p])
    if shrunk:
        failed = True
        for p in shrunk:
            print(f"{p}: {counts.get(p, 0)} raw strings, the baseline still allows {base[p]}.")
        print("Run  .github/strings-check.py --update  and commit the lower baseline.")

    known = english_ids()
    for path in files(CODE):
        rel = path.relative_to(ROOT).as_posix()
        for n, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if line.rstrip().endswith("// i18n: none"):
                continue
            for pattern in ID_USES:
                for m in pattern.finditer(line):
                    if m.group(1) not in known:
                        failed = True
                        print(f"{rel}:{n}: {m.group(1)} is not in i18n/en/*.ftl")

    if failed:
        print()
        print("Words a person reads go in i18n/en/<file>.ftl under an ID, and the UI asks")
        print("for the ID. CONTRIBUTING.md has the rule under Words.")
        return 1
    print(f"Every string goes through the catalog ({sum(counts.values())} left to move).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
