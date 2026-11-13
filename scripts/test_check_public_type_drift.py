"""Negative fixtures for the existing public-type drift gate.

This checks only duplicate public Rust names and declared definition paths. It
is not semantic-equivalence, wire-compatibility, or runtime-admission proof.
"""
from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("check_public_type_drift.py")
spec = importlib.util.spec_from_file_location("public_type_drift_under_test", SCRIPT)
assert spec is not None and spec.loader is not None
checker = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = checker
spec.loader.exec_module(checker)


class PublicTypeDriftGateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="public-type-drift-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "scripts").mkdir()
        self.allowlist_path = self.root / "scripts/public_type_drift_allowlist.json"
        self.addCleanup(setattr, checker, "LANE_MANIFEST", getattr(checker, "LANE_MANIFEST"))
        setattr(checker, "ROOT", self.root)
        setattr(checker, "ALLOWLIST_PATH", self.allowlist_path)
        setattr(checker, "CRATES", ["claim-ledger", "semantic-memory-forge"])
        self.definition("claim-ledger/src/types.rs")
        self.definition("semantic-memory-forge/src/bundle.rs")
        self.allowance()

    def definition(self, path: str) -> None:
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("pub struct EvidenceBundle { }\n", encoding="utf-8")

    def allowance(self, *, extra: list[dict] | None = None, **override: object) -> None:
        row = {
            "id": "TYPE-CLAIM-FORGE-001",
            "name": "EvidenceBundle",
            "owners": ["claim-ledger", "semantic-memory-forge"],
            "definitions": [
                {"owner": "claim-ledger", "path": "claim-ledger/src/types.rs", "domain": "claim_support"},
                {"owner": "semantic-memory-forge", "path": "semantic-memory-forge/src/bundle.rs", "domain": "causal_verification"},
            ],
            "failure_mode": "same name could be reinterpreted across domains",
            "why_temporarily_allowed": "these V1 bundles have distinct source contracts",
            "removal_condition": "replace with versioned, explicitly mapped families",
        }
        row.update(override)
        self.allowlist_path.write_text(json.dumps({"allowlist": [row, *(extra or [])]}), encoding="utf-8")

    def run_gate(self) -> tuple[int, str]:
        text = io.StringIO()
        with contextlib.redirect_stdout(text), contextlib.redirect_stderr(text):
            status = checker.main()
        return status, text.getvalue()

    def test_exact_domain_qualified_allowance_passes(self) -> None:
        status, output = self.run_gate()
        self.assertEqual(status, 0, output)

    def test_new_definition_path_in_existing_owner_fails(self) -> None:
        self.definition("claim-ledger/src/shadow.rs")
        status, output = self.run_gate()
        self.assertEqual(status, 1, output)

    def test_new_owner_for_same_name_fails(self) -> None:
        getattr(checker, "CRATES").append("new-adapter")
        self.definition("new-adapter/src/lib.rs")
        status, output = self.run_gate()
        self.assertEqual(status, 1, output)

    def test_stale_allowance_fails(self) -> None:
        (self.root / "semantic-memory-forge/src/bundle.rs").write_text("pub struct Other { }\n", encoding="utf-8")
        status, output = self.run_gate()
        self.assertEqual(status, 1, output)

    def test_duplicate_debt_id_or_name_fails(self) -> None:
        row = json.loads(self.allowlist_path.read_text())["allowlist"][0]
        self.allowance(extra=[row])
        status, output = self.run_gate()
        self.assertEqual(status, 1, output)

    def test_missing_debt_fields_fail(self) -> None:
        self.allowance(removal_condition="")
        status, output = self.run_gate()
        self.assertEqual(status, 1, output)

    def test_unknown_allowance_field_fails(self) -> None:
        self.allowance(unreviewed_override=True)
        status, output = self.run_gate()
        self.assertEqual(status, 1, output)

    def test_missing_scanned_crate_source_fails(self) -> None:
        getattr(checker, "CRATES").append("missing-owner")
        status, output = self.run_gate()
        self.assertEqual(status, 1, output)

    def test_missing_lane_manifest_is_not_a_fallback_scan(self) -> None:
        setattr(checker, "LANE_MANIFEST", self.root / "scripts/missing-lanes.json")
        with self.assertRaises(ValueError):
            checker._load_crate_list()

    def test_extra_type_scan_owner_comes_from_versioned_manifest(self) -> None:
        manifest = self.root / "scripts/lane_manifest.json"
        manifest.write_text(json.dumps({
            "schema_version": "lane_manifest_v2",
            "supported_lane": ["semantic-memory-forge"],
            "governance_lane": [],
            "semantic_type_scan_extra": ["new-adapter"],
        }), encoding="utf-8")
        setattr(checker, "LANE_MANIFEST", manifest)
        crates = checker._load_crate_list()
        self.assertIn("new-adapter", crates)
        self.assertNotIn("claim-ledger", crates)

    def test_missing_or_unknown_type_scan_manifest_contract_fails(self) -> None:
        manifest = self.root / "scripts/lane_manifest.json"
        setattr(checker, "LANE_MANIFEST", manifest)
        for payload in (
            {"schema_version": "lane_manifest_v2", "supported_lane": [], "governance_lane": []},
            {"schema_version": "lane_manifest_v999", "supported_lane": [], "governance_lane": [], "semantic_type_scan_extra": []},
        ):
            manifest.write_text(json.dumps(payload), encoding="utf-8")
            with self.assertRaises(ValueError):
                checker._load_crate_list()


if __name__ == "__main__":
    unittest.main()
