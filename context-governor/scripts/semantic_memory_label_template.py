#!/usr/bin/env python3
"""Create a manual semantic-memory label CSV from exported facts."""

from __future__ import annotations

import argparse
import csv
import json
import re
from pathlib import Path
from typing import Any

FIELDS = [
    "fact_id",
    "namespace",
    "source_receipt",
    "item_id",
    "content_blake3",
    "label",
    "reason",
    "should_promote_to_projects",
    "preview",
]


def _facts(data: Any) -> list[dict[str, Any]]:
    if isinstance(data, list):
        return data
    if isinstance(data, dict):
        for key in ("facts", "results", "items"):
            if isinstance(data.get(key), list):
                return data[key]
    return []


def _extract(pattern: str, content: str) -> str:
    match = re.search(pattern, content, flags=re.MULTILINE)
    return match.group(1).strip() if match else ""


def _row(fact: dict[str, Any]) -> dict[str, str]:
    content = str(fact.get("content") or "")
    receipt = _extract(r"^receipt_id:\s*(.+)$", content)
    item_id = _extract(r"^item_id:\s*(.+)$", content)
    content_blake3 = _extract(r"^content_blake3:\s*(.+)$", content)
    if not receipt:
        source = str(fact.get("source") or "")
        match = re.search(r"receipt\s+(\S+)\s+item\s+(\S+)", source)
        if match:
            receipt, item_id = match.group(1), item_id or match.group(2)
    preview = content.split("Archived content:", 1)[-1].strip() if "Archived content:" in content else content
    return {
        "fact_id": str(fact.get("fact_id") or fact.get("id") or fact.get("result_id") or ""),
        "namespace": str(fact.get("namespace") or ""),
        "source_receipt": receipt,
        "item_id": item_id,
        "content_blake3": content_blake3,
        "label": "",
        "reason": "",
        "should_promote_to_projects": "",
        "preview": preview.replace("\n", " ")[:240],
    }


def write_label_template(facts_json: Path, out_csv: Path) -> int:
    rows = [_row(fact) for fact in _facts(json.loads(facts_json.read_text()))]
    out_csv.parent.mkdir(parents=True, exist_ok=True)
    with out_csv.open("w", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=FIELDS)
        writer.writeheader()
        writer.writerows(rows)
    return len(rows)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--facts", required=True, help="JSON file from sm_list_facts/search export")
    parser.add_argument("--out", required=True, help="Output CSV label template")
    args = parser.parse_args()
    count = write_label_template(Path(args.facts), Path(args.out))
    print(f"wrote {count} label rows to {args.out}")


if __name__ == "__main__":
    main()
