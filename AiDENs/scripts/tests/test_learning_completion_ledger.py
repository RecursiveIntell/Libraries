"""Contract tests for the projection-only learning completion ledger."""
from __future__ import annotations

import hashlib
import json
import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
LEDGER = ROOT / "AiDENs/docs/learning-agent/COMPLETION_LEDGER.md"
BOUNDARY = ROOT / "AiDENs/docs/learning-agent/claim-boundary.json"


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def source_digests(paths: list[str]) -> dict[str, str]:
    return {
        path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
        for path in paths
    }


class LearningCompletionLedgerTests(unittest.TestCase):
    def test_projection_is_current_and_claims_are_bounded(self) -> None:
        self.assertTrue(LEDGER.is_file())
        self.assertTrue(BOUNDARY.is_file())
        boundary = json.loads(BOUNDARY.read_text())
        self.assertEqual(boundary["projection_only"], True)
        self.assertEqual(boundary["truth_authority"], "canonical owner receipts and live execution")
        self.assertTrue(boundary["permitted_claims"])
        self.assertTrue(boundary["forbidden_claims"])

        text = LEDGER.read_text()
        marker = "```json\n"
        start = text.index(marker) + len(marker)
        snapshot = json.loads(text[start:text.index("\n```", start)])
        self.assertEqual(snapshot["branch"], git("branch", "--show-current"))
        self.assertEqual(snapshot["head"], git("rev-parse", "HEAD"))
        self.assertEqual(snapshot["source_digests"], source_digests(snapshot["source_paths"]))
        self.assertIn("historical", snapshot["receipts"])
        self.assertIn("current", snapshot["receipts"])
        self.assertTrue(snapshot["tests"]["historical_passed"])
        self.assertIn("ignored_live_tests", snapshot["tests"])
        self.assertIn("promotion_evidence", snapshot)
        self.assertEqual(snapshot["claim_boundary"], "AiDENs/docs/learning-agent/claim-boundary.json")


if __name__ == "__main__":
    unittest.main()
