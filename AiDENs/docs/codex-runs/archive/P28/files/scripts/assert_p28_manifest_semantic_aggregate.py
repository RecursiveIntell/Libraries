#!/usr/bin/env python3
"""Assert degraded validation results downgrade aggregate manifest semantics."""

from pathlib import Path
import argparse
import json
import sys


DEGRADED_MARKERS = ("degraded", "partial", "failed", "blocked")
EXACT_STATUSES = {"exact_check", "pass_exact_check"}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", nargs="?", default="P27_STATUS_EVIDENCE_MANIFEST.json")
    args = parser.parse_args()

    path = Path(args.manifest)
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except Exception as error:
        print(f"FAIL: unable to parse {path}: {error}", file=sys.stderr)
        return 2

    aggregate = str(manifest.get("semantic_status", ""))
    validation_results = manifest.get("validation_results", [])
    has_degraded_result = any(
        any(marker in str(result.get("semantic_status", "")) for marker in DEGRADED_MARKERS)
        for result in validation_results
    )
    if has_degraded_result and aggregate in EXACT_STATUSES:
        print(
            f"FAIL: aggregate semantic_status={aggregate} despite degraded validation result",
            file=sys.stderr,
        )
        return 2

    print("PASS: manifest aggregate semantic status is consistent")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
