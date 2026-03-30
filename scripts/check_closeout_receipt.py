#!/usr/bin/env python3
from __future__ import annotations

import json
import sys

from generate_closeout_receipt import RECEIPT_PATH, build_receipt


def main() -> int:
    expected = build_receipt()
    actual = json.loads(RECEIPT_PATH.read_text(encoding="utf-8"))

    if actual != expected:
        print(
            "closeout receipt check failed: release/closeout_receipt_v1.json is stale",
            file=sys.stderr,
        )
        return 1

    print("closeout receipt check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
