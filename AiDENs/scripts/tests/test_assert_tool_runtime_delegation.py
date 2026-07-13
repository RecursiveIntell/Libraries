#!/usr/bin/env python3
"""Regression tests for source-only tool-runtime delegation scanning."""

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
GATE = REPO_ROOT / "scripts" / "assert_tool_runtime_delegation.sh"


class ToolRuntimeDelegationGateTests(unittest.TestCase):
    def setup_repo(self, root: Path) -> None:
        (root / "crates" / "aidens-contracts" / "src").mkdir(parents=True)
        (root / "crates" / "aidens-tool-kit" / "src").mkdir(parents=True)

    def run_gate(self, root: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["bash", str(GATE)],
            cwd=root,
            check=False,
            text=True,
            capture_output=True,
        )

    def test_generated_target_receipt_is_ignored(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            self.setup_repo(root)
            receipt = root / "crates" / "aidens-runner" / "target" / "receipt.ndjson"
            receipt.parent.mkdir(parents=True)
            receipt.write_text("llm-tool-runtime", encoding="utf-8")
            result = self.run_gate(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_local_descriptor_without_grounding_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            self.setup_repo(root)
            (root / "crates" / "aidens-contracts" / "src" / "tool.rs").write_text(
                "pub struct ToolDescriptorV1;\n", encoding="utf-8"
            )
            result = self.run_gate(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("no llm-tool-runtime grounding", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
