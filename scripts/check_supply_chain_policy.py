#!/usr/bin/env python3
"""Validate the cargo-deny hard gates and dated advisory exception ledger."""

from __future__ import annotations

import re
import sys
import tomllib
from datetime import date
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
POLICY = ROOT / "deny.toml"
LEDGER_RE = re.compile(
    r'"(RUSTSEC-\d{4}-\d{4})",?\s*#\s*owner:\s*([^;]+);\s*expires:\s*(\d{4}-\d{2}-\d{2})'
)


def main() -> int:
    text = POLICY.read_text(encoding="utf-8")
    with POLICY.open("rb") as handle:
        policy = tomllib.load(handle)

    errors: list[str] = []
    expected = {
        "advisories.yanked": policy.get("advisories", {}).get("yanked"),
        "bans.wildcards": policy.get("bans", {}).get("wildcards"),
        "sources.unknown-registry": policy.get("sources", {}).get("unknown-registry"),
        "sources.unknown-git": policy.get("sources", {}).get("unknown-git"),
    }
    for field, value in expected.items():
        if value != "deny":
            errors.append(f"{field} must be 'deny', found {value!r}")
    if policy.get("graph", {}).get("all-features") is not True:
        errors.append("graph.all-features must be true for the release supply-chain graph")

    ignored = policy.get("advisories", {}).get("ignore", [])
    ledger = {advisory: (owner.strip(), expiry) for advisory, owner, expiry in LEDGER_RE.findall(text)}
    for advisory in ignored:
        if advisory not in ledger:
            errors.append(f"ignored advisory {advisory} lacks an owner/expiry ledger entry")
            continue
        owner, expiry_text = ledger[advisory]
        try:
            expiry = date.fromisoformat(expiry_text)
        except ValueError:
            errors.append(f"ignored advisory {advisory} has invalid expiry {expiry_text!r}")
            continue
        if not owner:
            errors.append(f"ignored advisory {advisory} has an empty owner")
        if expiry < date.today():
            errors.append(f"ignored advisory {advisory} exception expired on {expiry_text}")

    undeclared = sorted(set(ledger) - set(ignored))
    if undeclared:
        errors.append("ledger entries are not present in advisories.ignore: " + ", ".join(undeclared))

    if errors:
        for error in errors:
            print(f"supply-chain policy error: {error}", file=sys.stderr)
        return 1
    print(f"supply-chain policy ok: {len(ignored)} owned, unexpired advisory exception(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
