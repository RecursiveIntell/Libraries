#!/usr/bin/env python3
"""Run behavioral v11A contract checks instead of marker-string checks."""

import subprocess
import sys


TESTS = [
    ["cargo", "test", "-p", "aidens-contracts", "--lib", "p28_material_done_requires_execution_context_manifests_and_receipts"],
    ["cargo", "test", "-p", "aidens-contracts", "--lib", "p28_material_operator_registry_blocks_undeclared_effects"],
    ["cargo", "test", "-p", "aidens-contracts", "--lib", "p12_operator_invocation_authorization_requires_effects_manifests_receipts_and_finite_taxonomy"],
]


def main() -> int:
    for command in TESTS:
        subprocess.run(command, check=True)
    print("v11A contract behavioral checks passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as exc:
        raise SystemExit(exc.returncode)
