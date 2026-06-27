import importlib.util
import sys
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "hermes_replay_eval.py"


def load_module():
    spec = importlib.util.spec_from_file_location("hermes_replay_eval_under_test", SCRIPT)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_parse_target_tokens_list_accepts_comma_list():
    mod = load_module()
    assert mod.parse_int_list("1200, 4000,80000") == [1200, 4000, 80000]


def test_parse_str_list_accepts_comma_list():
    mod = load_module()
    assert mod.parse_str_list("soft_warn, hard_cascade") == ["soft_warn", "hard_cascade"]


def test_write_reports_includes_failed_policy_target_rows(tmp_path):
    mod = load_module()
    candidate = mod.SessionCandidate("sess-1", "title", 42, 1000)
    reports = [
        {
            "ok": False,
            "fixture_id": "sess-1",
            "target_tokens": 1200,
            "budget_mode": "hard_cascade",
            "error": "BudgetExceeded",
        }
    ]

    json_path, md_path = mod.write_reports(tmp_path, reports, [candidate])

    assert json_path.exists()
    md = md_path.read_text()
    assert "Failures: 1" in md
    assert "hard_cascade" in md
    assert "BudgetExceeded" in md
