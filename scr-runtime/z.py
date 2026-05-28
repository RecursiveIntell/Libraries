#!/usr/bin/env python3
"""Repo-agnostic launcher for zip_source_certifier.py."""

from __future__ import annotations

import os
import sys
from pathlib import Path


def _find_certifier() -> Path:
    explicit = os.environ.get("ZIP_SOURCE_CERTIFIER_PATH")
    if explicit:
        candidate = Path(explicit).expanduser().resolve()
        if candidate.is_file():
            return candidate
        raise FileNotFoundError(f"ZIP_SOURCE_CERTIFIER_PATH does not point to a file: {candidate}")

    script_dir = Path(__file__).resolve().parent
    cwd = Path.cwd()
    search_roots = [cwd]
    search_roots.extend(cwd.parents)
    search_roots.extend([script_dir])
    search_roots.extend(script_dir.parents)

    candidates = []
    for root in search_roots:
        candidates.append(root / "scripts" / "zip_source_certifier.py")
        candidates.append(root / "zip_source_certifier.py")

    for candidate in candidates:
        if candidate.is_file():
            return candidate

    fallback = Path("/home/sikmindz/Coding/Libraries/scr-runtime/scripts/zip_source_certifier.py")
    if fallback.is_file():
        return fallback

    raise FileNotFoundError(
        "Could not locate zip_source_certifier.py. Place z.py next to a scripts/zip_source_certifier.py "
        "or set ZIP_SOURCE_CERTIFIER_PATH."
    )


def main() -> int:
    certifier = _find_certifier()
    os.execv(sys.executable, [sys.executable, str(certifier), *sys.argv[1:]])


if __name__ == "__main__":
    raise SystemExit(main())
