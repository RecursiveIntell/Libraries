import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).parents[2]
SCRIPT = ROOT / "scripts" / "validate_learning_corpus.py"
spec = importlib.util.spec_from_file_location("learning_validator", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = module
spec.loader.exec_module(module)


def test_v1_corpus_validates_and_is_deterministic():
    first = module.validate(ROOT / "fixtures" / "learning-coding-agent" / "v1")
    second = module.validate(ROOT / "fixtures" / "learning-coding-agent" / "v1")
    assert first == second
    assert len(first) == 64


def test_validator_rejects_changed_fixture_digest(tmp_path):
    source = ROOT / "fixtures" / "learning-coding-agent" / "v1"
    import shutil
    dest = tmp_path / "v1"
    shutil.copytree(source, dest)
    task = next((dest / "tasks").rglob("*.json"))
    task.write_text(task.read_text() + "\n", encoding="utf-8")
    try:
        module.validate(dest)
    except module.ValidationError as exc:
        assert "digest" in str(exc)
    else:
        raise AssertionError("changed fixture must be rejected")


def test_validator_rejects_duplicate_family_split(tmp_path):
    source = ROOT / "fixtures" / "learning-coding-agent" / "v1"
    import shutil
    dest = tmp_path / "v1"
    shutil.copytree(source, dest)
    manifest = json.loads((dest / "manifest.json").read_text())
    manifest["tasks"][1]["family"] = manifest["tasks"][0]["family"]
    manifest["tasks"][1]["split"] = manifest["tasks"][0]["split"]
    (dest / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    try:
        module.validate(dest)
    except module.ValidationError as exc:
        assert "family" in str(exc)
    else:
        raise AssertionError("duplicate family split must be rejected")


def test_validator_rejects_holdout_oracle_leak(tmp_path):
    source = ROOT / "fixtures" / "learning-coding-agent" / "v1"
    import shutil
    dest = tmp_path / "v1"
    shutil.copytree(source, dest)
    oracle = json.loads((dest / "oracles.json").read_text())
    manifest = json.loads((dest / "manifest.json").read_text())
    manifest["holdout_oracles"] = oracle["families"][0]
    (dest / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    try:
        module.validate(dest)
    except module.ValidationError as exc:
        assert "oracle" in str(exc)
    else:
        raise AssertionError("holdout oracle leak must be rejected")
