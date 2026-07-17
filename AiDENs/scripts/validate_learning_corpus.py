"""Validate the immutable Medusa learning-agent corpus v1.

The validator is deliberately independent of the learner: it only reads the
manifest, fixtures, and negative controls and never rewrites them.
"""
from __future__ import annotations

import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


class ValidationError(ValueError):
    pass


REQUIRED_NEGATIVE = {
    "path_escape", "symlink_escape", "descendant_process", "network", "secret_env",
    "malformed_tool_call", "duplicate_key_tool_call", "schema_invalid_tool_call",
    "evaluator_tamper", "stale_dependency_memory", "failing_baseline", "malformed_fixture",
    "rollback_failure", "reward_hacking", "real_to_mock_fallback",
}
SPLITS = ("development", "calibration", "holdout")
TASK_FIELDS = ("id", "family", "split", "fixture", "required_checks", "permitted_effects", "forbidden_effects")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _metadata(root: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    try:
        return (json.loads((root / "manifest.json").read_text(encoding="utf-8")),
                json.loads((root / "oracles.json").read_text(encoding="utf-8")))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValidationError(f"schema-invalid corpus metadata: {exc}") from exc


def split_summary(root: Path) -> dict[str, int]:
    manifest, _ = _metadata(Path(root))
    return dict(Counter(task.get("split") for task in manifest.get("tasks", [])))


def family_aware(summary: dict[str, int]) -> bool:
    """Return whether the corpus has the immutable 60/20/20 denominators."""
    total = sum(summary.get(split, 0) for split in SPLITS)
    return total > 0 and [summary.get(split, 0) for split in SPLITS] == [total * 3 // 5, total // 5, total // 5]


def validate(root: Path) -> str:
    root = Path(root)
    manifest, oracles = _metadata(root)
    if manifest.get("corpus_version") != "v1" or manifest.get("frozen_baseline_policy") != "immutable-after-treatment":
        raise ValidationError("unpinned corpus version or baseline policy")
    if manifest.get("toolchain") != "rust-toolchain.toml@1.86.0" or manifest.get("verifier_manifest") != "verifier-v1@sha256:medusa-local-v1":
        raise ValidationError("unpinned verifier/toolchain")

    tasks = manifest.get("tasks")
    if not isinstance(tasks, list) or len(tasks) < 5:
        raise ValidationError("too few tasks")
    families = {task.get("family") for task in tasks}
    if len(families) < 5 or None in families:
        raise ValidationError("too few families")
    summary = split_summary(root)
    if not family_aware(summary):
        raise ValidationError("split counts must satisfy immutable 60/20/20 ratio with nonempty denominators")

    canonical = manifest.get("canonical_fixture_digests")
    if not isinstance(canonical, dict):
        raise ValidationError("missing canonical fixture digests")
    seen_ids: set[str] = set()
    seen_fixtures: set[str] = set()
    family_splits: dict[str, str] = {}
    actual_fixtures: set[str] = set()
    for item in tasks:
        missing = [key for key in TASK_FIELDS if key not in item]
        if missing:
            raise ValidationError(f"missing task field {missing[0]}")
        if item["id"] in seen_ids:
            raise ValidationError(f"duplicate task id: {item['id']}")
        seen_ids.add(item["id"])
        if item["split"] not in SPLITS:
            raise ValidationError("invalid split")
        family = item["family"]
        previous_split = family_splits.setdefault(family, item["split"])
        if previous_split != item["split"]:
            raise ValidationError(f"family leakage across splits: {family}")
        fixture = item["fixture"]
        if fixture in seen_fixtures:
            raise ValidationError(f"duplicate fixture: {fixture}")
        seen_fixtures.add(fixture)
        if Path(fixture).is_absolute() or ".." in Path(fixture).parts:
            raise ValidationError(f"fixture path escapes corpus: {fixture}")
        path = root / fixture
        if not path.is_file():
            raise ValidationError(f"missing fixture {fixture}")
        actual_fixtures.add(fixture)
        if canonical.get(fixture) != digest(path):
            raise ValidationError(f"fixture digest mismatch: {fixture}")
        if not item["required_checks"] or not item["forbidden_effects"] or not item["permitted_effects"]:
            raise ValidationError("empty required checks/effects denominator")
        if item["split"] == "holdout" and any(key in item for key in ("oracle", "oracle_content", "expected_output")):
            raise ValidationError("holdout oracle leak")
    if set(canonical) != actual_fixtures:
        raise ValidationError("canonical fixture digest inventory mismatch")

    oracle_families = {entry.get("family") for entry in oracles.get("families", [])}
    if oracle_families != families or not all(isinstance(entry.get("oracle"), str) and entry["oracle"] for entry in oracles.get("families", [])):
        raise ValidationError("invalid oracle schema")
    negative = {path.stem for path in (root / "negative").glob("*.json")}
    required = set(manifest.get("required_negative_categories", []))
    if required != REQUIRED_NEGATIVE or not REQUIRED_NEGATIVE <= negative:
        raise ValidationError("negative category inventory drift")
    if manifest.get("holdout_oracles") is not None:
        raise ValidationError("holdout oracle leak")
    return digest(root / "manifest.json")


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    print(validate(args.root))
