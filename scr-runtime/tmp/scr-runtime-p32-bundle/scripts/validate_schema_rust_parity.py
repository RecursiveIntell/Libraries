#!/usr/bin/env python3
"""
Schema/Rust parity checker for known SCR-P0A required string fields.

This is a static schema check. Keep Rust tests as the source of truth.
"""
from __future__ import annotations

import json
import pathlib
import sys

ROOT = pathlib.Path.cwd()
SCHEMA_DIR = ROOT / "schemas" / "generated"
errors: list[str] = []

required_string_names = {
    "schema_version",
    "input_id",
    "ref_kind",
    "ref_value",
    "owner_hint",
    "valid_time_basis",
    "recorded_time",
    "receipt_id",
    "reason_codes",
    "policy_digest",
    "input_digest",
}

def walk(obj, path="$"):
    if isinstance(obj, dict):
        yield path, obj
        for k, v in obj.items():
            yield from walk(v, f"{path}.{k}")
    elif isinstance(obj, list):
        for i, v in enumerate(obj):
            yield from walk(v, f"{path}[{i}]")

for sp in sorted(SCHEMA_DIR.glob("*.json")):
    schema = json.loads(sp.read_text(encoding="utf-8"))
    for path, node in walk(schema):
        if not isinstance(node, dict):
            continue
        title = str(node.get("title", ""))
        # properties are checked by property name at parent level
        props = node.get("properties")
        if isinstance(props, dict):
            for name, prop in props.items():
                if name in required_string_names and isinstance(prop, dict) and prop.get("type") == "string":
                    if prop.get("minLength") != 1:
                        errors.append(f"{sp}:{path}.properties.{name} missing minLength=1")

if errors:
    print("schema/rust parity errors:")
    for e in errors:
        print(" -", e)
    sys.exit(1)
print("ok schema/rust parity")
