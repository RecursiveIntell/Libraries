#!/usr/bin/env python3
"""Compatibility entrypoint for read-only closeout-receipt verification."""
from __future__ import annotations

from run_release_gates import main

if __name__ == "__main__":
    raise SystemExit(main())
