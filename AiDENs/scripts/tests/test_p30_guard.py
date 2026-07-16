import importlib.util
import json
import sys
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "p30_guard.py"
spec = importlib.util.spec_from_file_location("p30_guard_under_test", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = module
spec.loader.exec_module(module)


def test_hard_fixture_under_aidens_workspace_is_detected(tmp_path):
    runner = tmp_path / "AiDENs" / "crates" / "aidens-runner" / "src"
    runner.mkdir(parents=True)
    (runner / "provider_tool.rs").write_text("let x = calls.iter().filter_map(|call| Some(call));\n")

    receipt = module.build_receipt(tmp_path)

    findings = [f for f in receipt["findings"] if f["level"] == "hard"]
    assert any(f["name"] == "PARSER_DROP_FILTER_MAP" for f in findings)
    assert "AiDENs/crates/aidens-runner/src/provider_tool.rs" in {
        f["path"] for f in findings
    }


def test_missing_configured_target_is_reported(tmp_path):
    receipt = module.build_receipt(tmp_path)

    assert receipt["missing_targets"]
    assert any(t.endswith("crates/aidens-runner/src/provider_tool.rs") for t in receipt["missing_targets"])


def test_receipt_reports_root_and_rule_coverage(tmp_path):
    receipt = module.build_receipt(tmp_path)

    assert receipt["discovered_roots"] == ["."]
    assert receipt["target_count"] == len(module.HARD_PATTERNS)
    assert set(receipt["rule_coverage"]) == {name for name, _, _ in module.HARD_PATTERNS}


def test_every_hard_rule_has_a_known_bad_self_test():
    coverage = module.self_test_hard_rules()

    assert all(coverage.values())


def test_release_gate_distinguishes_advisory_inventory_from_hard_findings(tmp_path):
    source = tmp_path / "sample.rs"
    source.write_text("fn advisory_only() { let _ = serde_json::json!({}); }\n")

    receipt = module.build_receipt(tmp_path)

    assert receipt["summary"]["warning_findings"] > 0
    assert receipt["summary"]["hard_findings"] == 0
    assert receipt["release_gate"]["status"] == "blocked"
    assert "missing-configured-targets" in receipt["release_gate"]["blockers"]
    assert receipt["release_gate"]["warning_policy"] == "advisory-inventory-not-release-blocking"
