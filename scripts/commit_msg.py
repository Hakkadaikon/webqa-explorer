#!/usr/bin/env python3
"""Check a commit message against docs/commit-messages.md.

Usage:
    commit_msg.py <file>                     check the message in a file
    commit_msg.py --range <range> --repo <d>  check each commit in a range
"""

import argparse
import re
import subprocess
import sys
import unicodedata

TYPES = {
    "feat",
    "fix",
    "docs",
    "style",
    "refactor",
    "perf",
    "test",
    "build",
    "ci",
    "chore",
    "revert",
}
SKIP = re.compile(r'^(Merge |Revert "|fixup! |squash! |amend! )')
HEADER = re.compile(
    r"^(?P<type>[a-z]+)(?:\((?P<scope>[^)]*)\))?(?P<bang>!)?: (?P<summary>.*)$"
)
SCOPE = re.compile(r"^[a-z][a-z0-9-]*$")
JAPANESE = re.compile(r"[぀-ヿ㐀-鿿＀-￯]")
URL = re.compile(r"https?://\S+")


def is_scissors(line):
    return line.startswith("# ") and ">8" in line


def before_scissors(lines):
    marks = [i for i, line in enumerate(lines) if is_scissors(line)]
    return lines[: (marks + [len(lines)])[0]]


def kept_lines(text):
    """Lines of text up to a `# ---8<---` scissors line, `#` comments dropped."""
    return [
        line for line in before_scissors(text.split("\n")) if not line.startswith("#")
    ]


def type_errors(kind):
    return [] if kind in TYPES else [f"種類 {kind} は使えない"]


def scope_errors(scope):
    if scope is None or SCOPE.match(scope):
        return []
    return [f"範囲 {scope} は英小文字・数字・ハイフンで書く"]


# (broken, message) pairs for the summary.
SUMMARY_RULES = (
    (lambda s: not JAPANESE.search(s), lambda s: "要約は日本語で書く"),
    (lambda s: len(s) > 50, lambda s: f"要約は50字以内にする({len(s)}字)"),
    (
        lambda s: s.endswith(("。", ".")),
        lambda s: "要約の文末に句点やピリオドを付けない",
    ),
)


def summary_errors(summary):
    return [message(summary) for broken, message in SUMMARY_RULES if broken(summary)]


def header_errors(header):
    m = HEADER.match(header)
    if not m:
        return [f"ヘッダーの形が{header!r}は不正(`種類(範囲)!: 要約` にする)"]
    return (
        type_errors(m["type"]) + scope_errors(m["scope"]) + summary_errors(m["summary"])
    )


def width(line):
    """Display width: 2 per East-Asian-wide/fullwidth char, else 1."""
    return sum(2 if unicodedata.east_asian_width(c) in "WF" else 1 for c in line)


def too_wide(line):
    return not URL.search(line) and width(line) > 72


def blank_line_errors(lines):
    if len(lines) < 2 or lines[1] == "":
        return []
    return ["2行目は空行にする"]


def width_errors(lines):
    numbered = enumerate(lines[2:], start=3)
    return [
        f"{n}行目は72桁以内にする({width(line)}桁)"
        for n, line in numbered
        if too_wide(line)
    ]


def check(text):
    if SKIP.match(text):
        return []
    lines = kept_lines(text) or [""]
    return header_errors(lines[0]) + blank_line_errors(lines) + width_errors(lines)


def commits_in_range(repo, commit_range):
    out = subprocess.run(
        ["git", "-C", repo, "log", "--format=%H", commit_range],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return [h for h in out.split("\n") if h]


def message_of(repo, commit_hash):
    return subprocess.run(
        ["git", "-C", repo, "log", "-1", "--format=%B", commit_hash],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.rstrip("\n")


def check_range(repo, commit_range):
    lines = []
    for commit_hash in commits_in_range(repo, commit_range):
        for e in check(message_of(repo, commit_hash)):
            lines.append(f"{commit_hash[:7]}: {e}")
    return lines


def file_errors(path):
    with open(path, encoding="utf-8") as f:
        return [f"{path}: {e}" for e in check(f.read().rstrip("\n"))]


def report(lines):
    """Print each error to stderr; 1 if there were any, else 0."""
    sys.stderr.writelines(line + "\n" for line in lines)
    return 1 if lines else 0


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("file", nargs="?")
    ap.add_argument("--range")
    ap.add_argument("--repo", default=".")
    args = ap.parse_args(argv)
    lines = check_range(args.repo, args.range) if args.range else file_errors(args.file)
    return report(lines)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
