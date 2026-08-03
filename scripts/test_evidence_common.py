#!/usr/bin/env python3
"""Regression tests for release-evidence contract validation."""
from __future__ import annotations

import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from evidence_common import verify_gate_contract


class VerifyGateContractTests(unittest.TestCase):
    def manifest(self) -> dict[str, object]:
        commands = ["first gate", "second gate"]
        receipts = [
            {
                "command": command,
                "argv": ["/bin/sh", "-c", command],
                "cwd": ".",
                "exit_code": 0,
                "result": "pass",
            }
            for command in commands
        ]
        return {
            "gate_definition": {
                "path": "scripts/release_gate_set.py",
                "sha256": "gate-digest",
                "command_count": len(commands),
            },
            "proof_commands": commands,
            "proof_results": [
                {"command": command, "result": "pass"} for command in commands
            ],
            "source_binding": {"command_receipts": receipts},
        }

    def test_accepts_complete_passing_canonical_run(self) -> None:
        self.assertEqual(
            verify_gate_contract(self.manifest(), ["first gate", "second gate"], "gate-digest"),
            [],
        )

    def test_rejects_partial_or_reordered_gate_evidence(self) -> None:
        manifest = self.manifest()
        manifest["proof_commands"] = ["second gate", "first gate"]
        findings = verify_gate_contract(manifest, ["first gate", "second gate"], "gate-digest")
        self.assertIn("proof commands do not match the canonical gate set", findings)

    def test_rejects_failed_receipt_even_when_summary_claims_success(self) -> None:
        manifest = self.manifest()
        receipts = manifest["source_binding"]["command_receipts"]
        receipts[1]["exit_code"] = 1
        findings = verify_gate_contract(manifest, ["first gate", "second gate"], "gate-digest")
        self.assertIn("command receipt is not a passing canonical gate", findings)


if __name__ == "__main__":
    unittest.main()
