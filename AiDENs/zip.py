#!/usr/bin/env python3
"""Deprecated legacy packager.

This file is intentionally disabled so it cannot act as an alternate archive
builder. Use z.py, which emits manifests, findings, package roles, and strict
self-containment checks.
"""

from __future__ import annotations

import sys


def main() -> int:
    print(
        "zip.py is deprecated and disabled. Use z.py, for example: "
        "python3 z.py --root . --profile aidens --mode release-context --strict",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
