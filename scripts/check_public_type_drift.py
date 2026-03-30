#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
ALLOWLIST_PATH = ROOT / "scripts" / "public_type_drift_allowlist.json"
LANE_MANIFEST = ROOT / "scripts" / "lane_manifest.json"

def _load_crate_list() -> list[str]:
    """Load crate list from lane_manifest.json if available, else fall back to hardcoded list."""
    if LANE_MANIFEST.is_file():
        manifest = json.loads(LANE_MANIFEST.read_text(encoding="utf-8"))
        return manifest.get("supported_lane", []) + manifest.get("governance_lane", [])
    return [
        "stack-ids", "forge-memory-bridge", "forge-pilot", "knowledge-runtime",
        "recursive-kernel-core", "kernel-execution", "kernel-oracles",
        "semantic-memory", "semantic-memory-forge", "living-memory/living-memory",
        "effect-runtime", "profile-runtime", "authority-delegation", "mechanism-runtime",
        "federated-settlement", "discovery-portfolio", "constitutional-memory",
        "remote-oracle-admission", "spec-execution", "verification-adjudication",
        "verification-control", "verification-policy",
    ]

CRATES = _load_crate_list()
PUBLIC_TYPE_RE = re.compile(r"^\s*pub\s+(?:struct|enum|type)\s+([A-Za-z_][A-Za-z0-9_]*)\b")


def crate_for_path(path: Path) -> str:
    rel = path.relative_to(ROOT)
    parts = rel.parts
    if len(parts) >= 3 and parts[0] == "living-memory" and parts[1] == "living-memory":
        return "living-memory/living-memory"
    return parts[0]


def load_allowlist() -> dict[str, set[str]]:
    raw = json.loads(ALLOWLIST_PATH.read_text(encoding="utf-8"))
    allow = {}
    for entry in raw.get("allowlist", []):
        name = entry["name"]
        owners = set(entry.get("owners", []))
        allow[name] = owners
    return allow


def main() -> int:
    allowlist = load_allowlist()
    found: dict[str, list[tuple[str, str]]] = defaultdict(list)

    for crate in CRATES:
        src = ROOT / crate / "src"
        if not src.exists():
            continue
        for path in sorted(src.rglob("*.rs")):
            owner = crate_for_path(path)
            for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
                match = PUBLIC_TYPE_RE.match(line)
                if match:
                    found[match.group(1)].append(
                        (owner, f"{path.relative_to(ROOT)}:{lineno}")
                    )

    duplicates = {
        name: entries for name, entries in found.items() if len({owner for owner, _ in entries}) > 1
    }

    disallowed = []
    print("public type drift report")
    for name in sorted(duplicates):
        owners = {owner for owner, _ in duplicates[name]}
        print(f"{name}: {', '.join(sorted(owners))}")
        if name not in allowlist or owners != allowlist[name]:
            disallowed.append((name, owners))

    if disallowed:
        print("unallowlisted duplicate public semantic types remain:", file=sys.stderr)
        for name, owners in disallowed:
            print(f"  {name}: {', '.join(sorted(owners))}", file=sys.stderr)
        return 1

    print(
        f"public type drift check passed with {len(duplicates)} allowlisted duplicate name(s)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
