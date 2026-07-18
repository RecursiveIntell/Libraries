#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import shutil
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "validate_learning_corpus_v2.py"
SPEC = importlib.util.spec_from_file_location("validate_learning_corpus_v2", SCRIPT)
assert SPEC and SPEC.loader
validator = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validator)
CORPUS = Path(__file__).resolve().parents[2] / "fixtures" / "learning-coding-agent" / "v2"


def write_manifest(root: Path, manifest: dict) -> None:
    manifest.pop("corpus_digest", None)
    manifest["corpus_digest"] = validator.sha256_bytes(validator.canonical_bytes(manifest))
    (root / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")


class ExecutableLearningCorpusV2Tests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory, Path]:
        temp = tempfile.TemporaryDirectory(prefix="aidens-corpus-v2-test-")
        root = Path(temp.name) / "v2"
        shutil.copytree(CORPUS, root)
        return temp, root

    def test_valid_static_corpus_is_non_promoting(self) -> None:
        receipt = validator.validate(CORPUS, False, None)
        self.assertEqual(receipt["task_count"], 15)
        self.assertEqual(receipt["family_count"], 5)
        self.assertEqual(receipt["pair_count"], 0)
        self.assertEqual(receipt["promotion_evidence_status"], "unavailable")

    def test_fixture_digest_drift_is_rejected(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        source = next(root.glob("tasks/*/*/fixture/src/lib.rs"))
        source.write_text(source.read_text() + "// drift\n")
        with self.assertRaisesRegex(ValueError, "fixture tree digest mismatch"):
            validator.validate(root, False, None)

    def test_metadata_only_fixture_is_rejected(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        next(root.glob("tasks/*/*/fixture/Cargo.toml")).unlink()
        with self.assertRaisesRegex(ValueError, "metadata-only or incomplete fixture"):
            validator.validate(root, False, None)

    def test_fixture_symlink_is_rejected_without_following_it(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        fixture = next(root.glob("tasks/*/*/fixture"))
        (fixture / "linked.rs").symlink_to(fixture / "src" / "lib.rs")
        with self.assertRaisesRegex(ValueError, "fixture symlink is forbidden"):
            validator.validate(root, False, None)

    def test_public_oracle_exposure_is_rejected_even_with_rehashed_manifest(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        manifest = json.loads((root / "manifest.json").read_text())
        entry = manifest["tasks"][0]
        task_path = root / entry["task_path"]
        task = json.loads(task_path.read_text())
        task["patch"] = {"forbidden": True}
        task_path.write_text(json.dumps(task, indent=2, sort_keys=True) + "\n")
        entry["task_digest"] = validator.sha256_bytes(validator.canonical_bytes(task))
        write_manifest(root, manifest)
        with self.assertRaisesRegex(ValueError, "public task exposes evaluator data"):
            validator.validate(root, False, None)

    def test_duplicate_task_identity_is_rejected(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        manifest = json.loads((root / "manifest.json").read_text())
        manifest["tasks"][1]["task_id"] = manifest["tasks"][0]["task_id"]
        write_manifest(root, manifest)
        with self.assertRaisesRegex(ValueError, "duplicate task ID"):
            validator.validate(root, False, None)

    def test_duplicate_material_under_distinct_identity_is_rejected(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        manifest = json.loads((root / "manifest.json").read_text())
        for field in ("task_digest", "fixture_tree_digest", "oracle_digest"):
            manifest["tasks"][1][field] = manifest["tasks"][0][field]
        write_manifest(root, manifest)
        with self.assertRaisesRegex(ValueError, "duplicate task, fixture, or oracle material"):
            validator.validate(root, False, None)

    def test_duplicate_oracle_patch_id_with_distinct_bytes_is_rejected(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        manifest = json.loads((root / "manifest.json").read_text())
        first, second = manifest["tasks"][0], manifest["tasks"][1]
        first_oracle = json.loads(
            (root / f"oracles/{first['task_id']}.patch.json").read_text()
        )
        second_oracle_path = root / f"oracles/{second['task_id']}.patch.json"
        second_oracle = json.loads(second_oracle_path.read_text())
        second_oracle["patch_id"] = first_oracle["patch_id"]
        second_oracle["summary"] += " distinct bytes"
        second_oracle_path.write_text(
            json.dumps(second_oracle, indent=2, sort_keys=True) + "\n"
        )
        second["oracle_digest"] = validator.sha256_bytes(
            validator.canonical_bytes(second_oracle)
        )
        second_task_path = root / second["task_path"]
        second_task = json.loads(second_task_path.read_text())
        second_task["oracle_digest"] = second["oracle_digest"]
        second_task_path.write_text(
            json.dumps(second_task, indent=2, sort_keys=True) + "\n"
        )
        second["task_digest"] = validator.sha256_bytes(
            validator.canonical_bytes(second_task)
        )
        write_manifest(root, manifest)
        with self.assertRaisesRegex(ValueError, "invalid or duplicate oracle patch ID"):
            validator.validate(root, False, None)

    def test_in_root_oracle_role_confusion_is_rejected_before_read(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        manifest = json.loads((root / "manifest.json").read_text())
        manifest["tasks"][0]["task_path"] = (
            f"oracles/{manifest['tasks'][0]['task_id']}.patch.json"
        )
        write_manifest(root, manifest)
        with self.assertRaisesRegex(ValueError, "task topology mismatch"):
            validator.validate(root, False, None)

    def test_family_split_leakage_is_rejected_with_counts_preserved(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        manifest = json.loads((root / "manifest.json").read_text())
        development = next(task for task in manifest["tasks"] if task["split"] == "development")
        calibration = next(task for task in manifest["tasks"] if task["split"] == "calibration")
        development["split"], calibration["split"] = calibration["split"], development["split"]
        write_manifest(root, manifest)
        with self.assertRaisesRegex(ValueError, "canonical family-to-split mapping mismatch"):
            validator.validate(root, False, None)

    def test_unknown_family_is_rejected_even_with_refreshed_manifest_digest(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        manifest = json.loads((root / "manifest.json").read_text())
        manifest["tasks"][0]["family"] = "renamed-family"
        write_manifest(root, manifest)
        with self.assertRaisesRegex(ValueError, "canonical family-to-split mapping mismatch"):
            validator.validate(root, False, None)

    def test_manifest_path_escape_is_rejected_even_when_target_exists(self) -> None:
        temp, root = self.copied()
        self.addCleanup(temp.cleanup)
        outside = root.parent / "outside.json"
        outside.write_text("{}\n")
        manifest = json.loads((root / "manifest.json").read_text())
        manifest["tasks"][0]["task_path"] = "../outside.json"
        write_manifest(root, manifest)
        with self.assertRaisesRegex(ValueError, "task topology mismatch"):
            validator.validate(root, False, None)

    def test_execute_rejects_unbound_runtime_before_invocation(self) -> None:
        temp = tempfile.TemporaryDirectory(prefix="aidens-fake-runtime-")
        self.addCleanup(temp.cleanup)
        runtime = Path(temp.name) / "podman"
        runtime.write_text("#!/bin/sh\nexit 0\n")
        runtime.chmod(0o755)
        with self.assertRaisesRegex(ValueError, "runtime digest mismatch"):
            validator.validate(CORPUS, True, "localhost/example@sha256:" + "0" * 64, runtime, "0" * 64)

    def test_sealed_argv_contains_every_declared_isolation_mechanism(self) -> None:
        command = validator.sealed_command(
            "/usr/bin/podman",
            "localhost/example@sha256:" + "0" * 64,
            Path("/HOST_WORKSPACE"),
            Path("/HOST_CIDFILE"),
        )
        for expected in (
            "--pull=never",
            "--network=none",
            "--read-only",
            "--cap-drop=all",
            "--userns=keep-id",
            "--security-opt=no-new-privileges",
            "--pids-limit=256",
            "--tmpfs=/tmp:rw,noexec,nosuid,nodev,size=256m",
        ):
            self.assertIn(expected, command)
        self.assertIn("--cidfile=/HOST_CIDFILE", command)


if __name__ == "__main__":
    unittest.main()
