"""Validate the immutable Medusa learning-agent corpus v1."""
from __future__ import annotations
import hashlib, json, re
from pathlib import Path

class ValidationError(ValueError): pass
REQUIRED_NEGATIVE = {"path_escape","symlink_escape","descendant_process","network","secret_env","malformed_tool_call","duplicate_key_tool_call","schema_invalid_tool_call","evaluator_tamper","stale_dependency_memory","failing_baseline","malformed_fixture","rollback_failure","reward_hacking","real_to_mock_fallback"}

def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def validate(root: Path) -> str:
    root = Path(root)
    try:
        manifest = json.loads((root/"manifest.json").read_text())
        oracles = json.loads((root/"oracles.json").read_text())
    except Exception as exc: raise ValidationError(f"schema-invalid corpus metadata: {exc}") from exc
    if manifest.get("corpus_version") != "v1" or manifest.get("frozen_baseline_policy") != "immutable-after-treatment":
        raise ValidationError("unpinned corpus version or baseline policy")
    if manifest.get("toolchain") != "rust-toolchain.toml@1.86.0" or manifest.get("verifier_manifest") != "verifier-v1@sha256:medusa-local-v1":
        raise ValidationError("unpinned verifier/toolchain")
    tasks = manifest.get("tasks", [])
    if len(tasks) < 5: raise ValidationError("too few tasks")
    families = {x.get("family") for x in tasks}
    if len(families) < 5: raise ValidationError("too few families")
    seen = set(); file_digests = manifest.get("canonical_fixture_digests", {})
    for item in tasks:
        for key in ("id","family","split","fixture","required_checks","permitted_effects","forbidden_effects"): 
            if key not in item: raise ValidationError(f"missing task field {key}")
        if item["split"] not in {"development","calibration","holdout"}: raise ValidationError("invalid split")
        key = (item["family"], item["split"])
        if key in seen: raise ValidationError(f"duplicate family split: {key}")
        seen.add(key)
        if item["split"] == "holdout" and any(k in item for k in ("oracle","oracle_content","expected_output")): raise ValidationError("holdout oracle leak")
        p = root / item["fixture"]
        if not p.is_file(): raise ValidationError(f"missing fixture {item['fixture']}")
        actual = digest(p)
        if file_digests.get(item["fixture"]) != actual: raise ValidationError(f"fixture digest mismatch: {item['fixture']}")
        if not item["required_checks"] or not item["forbidden_effects"]: raise ValidationError("empty required checks/effects denominator")
    if len(seen) != len(tasks): raise ValidationError("duplicate family/split assignment")
    if not isinstance(oracles.get("families"), list) or not all("family" in x and "oracle" in x for x in oracles["families"]): raise ValidationError("invalid oracle schema")
    negative = {p.stem for p in (root/"negative").glob("*.json")}
    missing = REQUIRED_NEGATIVE - negative
    if missing: raise ValidationError("missing required negative category: " + ",".join(sorted(missing)))
    if manifest.get("holdout_oracles") is not None: raise ValidationError("holdout oracle leak")
    return hashlib.sha256(b"".join((root/"manifest.json").read_bytes() for _ in [0])).hexdigest()

if __name__ == "__main__":
    import argparse
    p=argparse.ArgumentParser(); p.add_argument("root", type=Path); a=p.parse_args(); print(validate(a.root))
