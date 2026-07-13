#!/usr/bin/env python3
"""Regression tests for the ownership compatibility-ledger gate."""

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
GATE = REPO_ROOT / "scripts" / "assert_no_compatibility_ledgers.sh"
HEADER = "| Shim name | File path | Reason |\n|---|---|---|\n"


class CompatibilityLedgerGateTests(unittest.TestCase):
    def run_gate(self, ledger: str) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "docs" / "contract-ownership").mkdir(parents=True)
            (root / "docs" / "contract-ownership" / "COMPATIBILITY_LEDGER.md").write_text(
                ledger, encoding="utf-8"
            )
            (root / "crates" / "aidens-contracts").mkdir(parents=True)
            return subprocess.run(
                ["bash", str(GATE)],
                cwd=root,
                check=False,
                text=True,
                capture_output=True,
            )

    def test_empty_markdown_ledger_passes(self) -> None:
        result = self.run_gate(HEADER)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_data_row_fails(self) -> None:
        result = self.run_gate(
            HEADER + "| legacy-shim | src/shim.rs | temporary |\n"
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("has entries", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
