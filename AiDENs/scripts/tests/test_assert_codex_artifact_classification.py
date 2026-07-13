#!/usr/bin/env python3
"""Regression tests for active Codex-artifact classification."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
GATE = REPO_ROOT / "scripts" / "assert_codex_artifact_classification.py"


class CodexArtifactClassificationTests(unittest.TestCase):
    def run_gate(self, root: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", str(GATE), str(root)],
            check=False,
            text=True,
            capture_output=True,
        )

    def setup_root(self, root: Path) -> None:
        (root / "docs" / "codex-runs").mkdir(parents=True)
        (root / "docs" / "codex-runs" / "CURRENT_RUN.json").write_text(
            json.dumps({"active_run": "P32"}), encoding="utf-8"
        )
        (root / "docs" / "codex-runs" / "CODEX_ARTIFACT_CLASSIFICATION.json").write_text(
            json.dumps({"artifacts": []}), encoding="utf-8"
        )

    def test_generated_contract_ownership_evidence_is_not_an_active_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            self.setup_root(root)
            receipt = root / ".codex_evidence" / "contract_ownership" / "final" / "gate.txt"
            receipt.parent.mkdir(parents=True)
            receipt.write_text("receipt", encoding="utf-8")
            result = self.run_gate(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_unclassified_active_codex_artifact_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            self.setup_root(root)
            artifact = root / "P32_ACTIVE_CODEX_NOTES.md"
            artifact.write_text("unclassified", encoding="utf-8")
            result = self.run_gate(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("unclassified run/Codex artifacts", result.stderr)


if __name__ == "__main__":
    unittest.main()
