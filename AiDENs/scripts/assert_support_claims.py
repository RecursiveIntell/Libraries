#!/usr/bin/env python3
"""Assert support claims in public-facing docs are explicit and not overstated."""
from pathlib import Path
import re
import sys

ROOT = Path.cwd()
DOCS = [
    ROOT / "README.md",
    ROOT / "STATUS.md",
    ROOT / "SUPPORT_PROFILE.md",
]

FORBIDDEN = [
    "complete autonomous platform",
    "fully verified v10",
    "fully verified v10 runtime",
    "federated/proof-governed complete",
    "complete autonomy",
]

REQUIRED_MARKERS = [
    "supported-local",
    "fixture-backed",
    "deferred",
    "not production-cloud-ready",
    "not ready for production cloud deployment",
    "not production cloud ready",
]


def _read(path: Path) -> str:
    if not path.exists():
        raise FileNotFoundError(f"missing file: {path}")
    return path.read_text(encoding="utf-8", errors="replace")


def main() -> int:
    failures = []

    texts = []
    combined = []
    for path in DOCS:
        try:
            text = _read(path).lower()
        except FileNotFoundError as e:
            failures.append(str(e))
            continue
        texts.append((path.name, text))
        combined.append(text)

    for name, text in texts:
        for term in FORBIDDEN:
            if term in text:
                failures.append(f"{name}: forbidden support claim marker '{term}' found")

    joined = "\n".join(combined)
    for term in REQUIRED_MARKERS:
        if term in joined:
            break
    else:
        failures.append("README/STATUS/SUPPORT_PROFILE do not contain required support-status wording")

    overclaim_patterns = [
        r"supports\s+autonomous\s+platform",
        r"supports\s+full\s+autonomy",
        r"supports\s+cloud[- ]?ready",
        r"complete\s+autonomous\s+runtime",
    ]
    for name, text in texts:
        for pat in overclaim_patterns:
            if re.search(pat, text):
                failures.append(f"{name}: overclaim pattern matched '{pat}'")

    if failures:
        print("FAIL: support claim checks")
        for item in failures:
            print(f"  {item}")
        return 1

    print("PASS: support claim checks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
