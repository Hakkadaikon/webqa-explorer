#!/usr/bin/env python3
"""Decide which CI jobs a pull request needs, from the paths it changes.

Reads one changed path per line on stdin and prints GitHub outputs:
    code=true   when the check job (lint, tests, coverage) must run
    rust=true   when the mutation jobs must run
"""

import fnmatch
import sys

# Files that the tests or the check job read, also outside crates/. A change
# to the workflows runs everything too, so that a CI change tests itself.
CODE = [
    "crates/*",
    "skills/*",
    ".claude/*",
    ".agents/*",
    ".cargo/*",
    ".config/*",
    "scripts/*",
    "bridge/*",
    "share/*",
    ".github/workflows/*",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "justfile",
    "docs/arch/config.md",
    "tasks/loopeng/*.feature",
]


def decide(paths):
    # No paths means the diff could not be read, so run the check to be safe.
    return {
        "code": not paths
        or any(fnmatch.fnmatchcase(p, g) for p in paths for g in CODE),
        # cargo mutants --in-diff mutates only changed .rs lines.
        "rust": any(p.endswith(".rs") for p in paths),
    }


def main():
    paths = [line.strip() for line in sys.stdin if line.strip()]
    for name, value in decide(paths).items():
        print(f"{name}={str(value).lower()}")


if __name__ == "__main__":
    main()
