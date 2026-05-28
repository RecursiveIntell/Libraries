#!/usr/bin/env python3
"""Assert issue matrix closure by status, not ID marker coverage."""

from pathlib import Path
import argparse
import csv
import re
import sys


FINAL_STATUSES = {
    "fixed",
    "quarantined",
    "deferred",
    "superseded",
    "unsupported",
    "gate-required-not-product-defect",
}


def phase_number(suggested_phase: str) -> int | None:
    match = re.search(r"Phase\s+(\d+)", suggested_phase)
    return int(match.group(1)) if match else None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--matrix", default="matrices/SUPER_PASS_BACKLOG_1020.csv")
    parser.add_argument(
        "--completed-through",
        type=int,
        help="Only require closure for phases up to and including this phase.",
    )
    args = parser.parse_args()

    matrix = Path(args.matrix)
    if not matrix.exists():
        print(f"missing issue matrix: {matrix}", file=sys.stderr)
        return 2

    rows = list(csv.DictReader(matrix.open(newline="")))
    blocking = []
    for row in rows:
        if args.completed_through is not None:
            number = phase_number(row.get("Suggested_Phase", ""))
            if number is None or number > args.completed_through:
                continue
        if row.get("Status") not in FINAL_STATUSES:
            blocking.append(row)

    if blocking:
        print(f"issue matrix has {len(blocking)} raw/unresolved rows", file=sys.stderr)
        for row in blocking[:25]:
            print(
                f"- {row.get('ID')}: status={row.get('Status')} phase={row.get('Suggested_Phase')}",
                file=sys.stderr,
            )
        return 1

    scope = (
        f"phases <= {args.completed_through}"
        if args.completed_through is not None
        else "all phases"
    )
    print(f"issue matrix closure OK for {scope}: {len(rows)} rows inspected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
