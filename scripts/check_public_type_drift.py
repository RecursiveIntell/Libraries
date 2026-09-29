#!/usr/bin/env python3
"""Block undeclared public Rust type-name drift at owner-qualified paths.

This is a discovery/no-new-name-drift gate, not semantic equivalence proof.
Wire/schema compatibility remains with contract-schema-gen; owner behavior and
cross-language conformance require separate tests.
"""
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
    """Scan declared release lanes plus explicit semantic owners outside them."""
    if not LANE_MANIFEST.is_file():
        raise ValueError(f"missing current lane manifest: {LANE_MANIFEST}")
    manifest = json.loads(LANE_MANIFEST.read_text(encoding="utf-8"))
    if not isinstance(manifest, dict) or manifest.get("schema_version") != "lane_manifest_v2":
        raise ValueError("unsupported or malformed lane manifest version")
    supported = manifest.get("supported_lane")
    governance = manifest.get("governance_lane")
    extra = manifest.get("semantic_type_scan_extra")
    if not isinstance(supported, list) or not isinstance(governance, list) or not isinstance(extra, list):
        raise ValueError("lane manifest must list supported_lane, governance_lane and semantic_type_scan_extra")
    lanes = supported + governance + extra
    if not all(isinstance(c, str) and c for c in lanes) or len(lanes) != len(set(lanes)):
        raise ValueError("lane manifest must list unique, nonempty crate names")
    return sorted(lanes)


CRATES = _load_crate_list()
PUBLIC_TYPE_RE = re.compile(r"^\s*pub\s+(?:struct|enum|type)\s+([A-Za-z_][A-Za-z0-9_]*)\b")
REQUIRED_DEBT_FIELDS = (
    "id", "name", "owners", "definitions", "failure_mode",
    "why_temporarily_allowed", "removal_condition",
)


def crate_for_path(path: Path) -> str:
    rel = path.relative_to(ROOT)
    parts = rel.parts
    if len(parts) >= 3 and parts[0] == "living-memory" and parts[1] == "living-memory":
        return "living-memory/living-memory"
    return parts[0]


def _nonempty(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def load_allowlist() -> dict[str, tuple[set[str], set[tuple[str, str]]]]:
    raw = json.loads(ALLOWLIST_PATH.read_text(encoding="utf-8"))
    if not isinstance(raw, dict) or set(raw) != {"allowlist"}:
        raise ValueError("public type allowlist must contain only the allowlist field")
    rows = raw["allowlist"]
    if not isinstance(rows, list):
        raise ValueError("public type allowlist must contain an array")
    allow: dict[str, tuple[set[str], set[tuple[str, str]]]] = {}
    debt_ids: set[str] = set()
    for row in rows:
        if not isinstance(row, dict) or set(row) != set(REQUIRED_DEBT_FIELDS):
            raise ValueError("public type allowance has missing or unknown fields")
        if any(not _nonempty(row.get(key)) for key in REQUIRED_DEBT_FIELDS if key not in ("owners", "definitions")):
            raise ValueError("public type allowance missing required debt fields")
        name, debt_id = row["name"], row["id"]
        if name in allow or debt_id in debt_ids:
            raise ValueError(f"duplicate public type allowance name or debt ID: {name}")
        debt_ids.add(debt_id)
        owners = row.get("owners")
        definitions = row.get("definitions")
        if not isinstance(owners, list) or len(owners) < 2 or not all(_nonempty(o) for o in owners) or len(set(owners)) != len(owners):
            raise ValueError(f"invalid owner list for {name}")
        if not isinstance(definitions, list) or len(definitions) < 2:
            raise ValueError(f"missing exact definitions for {name}")
        defined: set[tuple[str, str]] = set()
        defined_owners: set[str] = set()
        for item in definitions:
            if not isinstance(item, dict) or set(item) != {"owner", "path", "domain"}:
                raise ValueError(f"definition has missing or unknown fields for {name}")
            if not all(_nonempty(item.get(k)) for k in ("owner", "path", "domain")):
                raise ValueError(f"incomplete definition for {name}")
            owner, relpath = item["owner"], item["path"]
            path = Path(relpath)
            if path.is_absolute() or ".." in path.parts or not relpath.startswith(owner + "/src/"):
                raise ValueError(f"definition path outside declared owner for {name}: {relpath}")
            if not (ROOT / path).resolve().is_relative_to(ROOT.resolve()):
                raise ValueError(f"definition path escapes root for {name}: {relpath}")
            if (owner, relpath) in defined:
                raise ValueError(f"duplicate definition for {name}: {relpath}")
            defined.add((owner, relpath))
            defined_owners.add(owner)
        if defined_owners != set(owners):
            raise ValueError(f"owner/definition mismatch for {name}")
        allow[name] = (set(owners), defined)
    return allow


def main() -> int:
    try:
        allowlist = load_allowlist()
    except (OSError, ValueError, TypeError, KeyError) as exc:
        print(f"invalid public type debt allowlist: {exc}", file=sys.stderr)
        return 1
    found: dict[str, list[tuple[str, str]]] = defaultdict(list)
    for crate in CRATES:
        src = ROOT / crate / "src"
        if not src.is_dir():
            print(f"missing scanned crate source: {src}", file=sys.stderr)
            return 1
        for path in sorted(src.rglob("*.rs")):
            owner = crate_for_path(path)
            for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
                match = PUBLIC_TYPE_RE.match(line)
                if match:
                    found[match.group(1)].append((owner, str(path.relative_to(ROOT))))
    duplicates = {
        name: entries for name, entries in found.items() if len({owner for owner, _ in entries}) > 1
    }
    disallowed: list[str] = []
    print("public type drift report (names and exact definitions, not semantic equivalence)")
    for name in sorted(duplicates):
        owners = {owner for owner, _ in duplicates[name]}
        definitions = set(duplicates[name])
        print(f"{name}: {', '.join(sorted(owners))}")
        if name not in allowlist or allowlist[name] != (owners, definitions):
            disallowed.append(f"new or changed definition: {name}: {sorted(definitions)}")
    for name in sorted(set(allowlist) - set(duplicates)):
        disallowed.append(f"stale allowance no longer names a cross-crate duplicate: {name}")
    if disallowed:
        print("public type-name drift/debt check failed:", file=sys.stderr)
        for item in disallowed:
            print(f"  {item}", file=sys.stderr)
        return 1
    print(f"public type drift check passed with {len(duplicates)} exact, debt-bound allowance(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
