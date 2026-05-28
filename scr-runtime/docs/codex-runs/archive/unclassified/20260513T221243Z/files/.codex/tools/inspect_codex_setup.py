#!/usr/bin/env python3
"""Simple codex setup sanity helper."""
from __future__ import annotations

from pathlib import Path


def load_manifest(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
