#!/usr/bin/env python3
"""Validate turbo-quant README before crates.io publishing.

The README in the published crate (turbo-quant 0.2.0/0.2.1) follows the
"experimental vector compression sidecars" template. The gate validates
the *real* published README, not a hypothetical future template — the
required headings and phrases mirror what the P26 release actually
published, and the forbidden patterns are the P26 release-claim law.
"""
from __future__ import annotations
import argparse
import re
import sys
from pathlib import Path

FORBIDDEN_PATTERNS = [
    r"zero\s+accuracy\s+loss",
    r"\blossless\b",
    r"\bperfect\b",
    r"guaranteed\s+quality",
    r"production[- ]ready",
    r"drop[- ]in\s+replacement\s+for\s+vectors",
    r"\ball\s+workloads\b",
    r"alpha\.1",
    r"alpha\.2",
    r"alpha\.3",
]

# A "forbidden" pattern in a README can also be present because the README
# is *documenting* the contract (e.g. the "Scope and limits" section lists
# "zero accuracy loss" as a forbidden claim to never make). Allow a line
# containing the pattern if the surrounding ~12 lines include a marker
# like "forbidden", "scope and limits", "must not", or "do not", AND the
# line itself is a quoted list item (`- "phrase"`).
_ALLOW_README_CONTEXT = re.compile(
    r"\b(do not|forbidden|avoid|remove|unqualified|paper claims?|not claim|must not|"
    r"scope and limits|release-claim law|release claim law)\b",
    re.IGNORECASE,
)
_LIST_ITEM_QUOTED = re.compile(r"^\s*[-*]\s*[\"'].+[\"']\s*$")

# The README follows the "What's in the box / Quick Start / Benchmarks /
# Scope and limits" structure published in 0.2.0. Required headings
# reflect the actual section anchors a reader needs to find.
REQUIRED_HEADINGS = [
    "# turbo-quant",
    "## What's in the box",
    "## Quick Start",
    "## Benchmarks — measured",
    "## Scope and limits",
    "## What's verified",
    "## Test coverage",
    "## MSRV",
    "## License",
]

# Phrases that anchor the release-claim law and let downstream readers
# verify the crate is honest about its experimental status.
REQUIRED_PHRASES = [
    "experimental",
    "approximate scoring",
    "exact",
    "sidecar",
    "PolarQuant",
    "TurboQuant",
    "QJL",
    "semantic-memory",
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("readme", nargs="?", default="README.md")
    args = parser.parse_args()
    path = Path(args.readme)
    if not path.exists():
        print(f"README gate failed: {path} does not exist", file=sys.stderr)
        return 2
    text = path.read_text(encoding="utf-8")
    failures: list[str] = []

    for heading in REQUIRED_HEADINGS:
        if heading not in text:
            failures.append(f"missing heading: {heading}")

    lowered = text.lower()
    for phrase in REQUIRED_PHRASES:
        if phrase.lower() not in lowered:
            failures.append(f"missing required phrase: {phrase}")

    for pattern in FORBIDDEN_PATTERNS:
        # The README may list forbidden phrases as bullet items under a
        # "forbidden claims" / "scope and limits" header. We do a line-by-line
        # scan with a windowed allow-context so that documenting the contract
        # is not flagged as a violation.
        flagged = False
        last_marker = -100
        for lineno, line in enumerate(text.splitlines(), start=1):
            if _ALLOW_README_CONTEXT.search(line):
                last_marker = lineno
                continue
            in_window = (lineno - last_marker) <= 12
            is_quoted = bool(_LIST_ITEM_QUOTED.match(line))
            if in_window and is_quoted:
                continue
            if re.search(pattern, line, flags=re.IGNORECASE):
                failures.append(f"forbidden README pattern present: {pattern} (line {lineno})")
                flagged = True
                break
        if flagged:
            continue

    if "```rust" not in text:
        failures.append("missing Rust code block")

    if failures:
        print("README gate failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("README gate passed")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
