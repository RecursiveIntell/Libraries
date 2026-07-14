#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json


RELEASE_GATE_COMMANDS = [
    "python3 scripts/check_action_pinning.py",
    "python3 scripts/check_supply_chain_policy.py",
    "cargo metadata --no-deps --format-version 1 | python3 scripts/check_workspace_hygiene.py",
    "python3 scripts/run_release_gates.py --check-drift",
    "cargo check --workspace",
    "bash scripts/check_pack_truth.sh",
    "bash scripts/check_repo_surface.sh",
    "bash scripts/check_doc_truth.sh",
    "bash scripts/check_manifest_truth.sh",
    "python3 scripts/check_commit_permit_paths.py",
    "python3 scripts/check_constitutional_vocab.py",
    "python3 scripts/check_mandatory_artifact_refs.py",
    "bash scripts/check_schema_registry_uniqueness.sh",
    "bash scripts/check_no_prod_panics.sh",
    "bash scripts/check_mirror_discipline.sh",
    "bash scripts/check_hotspot_budgets.sh",
    "python3 scripts/check_public_type_drift.py",
    "python3 scripts/check_root_archive_manifest.py",
    "python3 scripts/check_public_api_docs.py",
    "bash scripts/check_schema_compat.sh",
    "make release-lane",
]


def gate_sha256() -> str:
    encoded = json.dumps(RELEASE_GATE_COMMANDS, sort_keys=False, separators=(",", ":")).encode(
        "utf-8"
    )
    return hashlib.sha256(encoded).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true", help="print the gate set as JSON")
    args = parser.parse_args()

    if args.json:
        print(
            json.dumps(
                {
                    "commands": RELEASE_GATE_COMMANDS,
                    "count": len(RELEASE_GATE_COMMANDS),
                    "sha256": gate_sha256(),
                },
                indent=2,
            )
        )
        return

    for command in RELEASE_GATE_COMMANDS:
        print(command)


if __name__ == "__main__":
    main()
