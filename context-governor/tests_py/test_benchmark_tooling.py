import csv
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_script(name: str):
    path = ROOT / "scripts" / name
    spec = importlib.util.spec_from_file_location(name.replace(".py", "_under_test"), path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_generate_adversarial_fixtures_writes_expected_probe_metadata(tmp_path):
    mod = load_script("generate_adversarial_fixtures.py")

    fixtures = mod.generate_fixtures()
    assert {fixture["fixture_id"] for fixture in fixtures} >= {
        "latest_user_reversal",
        "critical_error_in_huge_log",
        "prompt_injection_tool_output",
        "personal_social_noise",
    }
    for fixture in fixtures:
        assert fixture["messages"]
        assert fixture["expected_probes"]

    out = tmp_path / "fixtures"
    written = mod.write_fixtures(out, fixtures)
    assert len(written) == len(fixtures)
    sample = json.loads((out / "latest_user_reversal.json").read_text())
    assert sample["expected_invariants"]["latest_user_final"] is True


def test_evaluate_adversarial_fixtures_scores_visible_and_recoverable_without_binary(tmp_path):
    gen = load_script("generate_adversarial_fixtures.py")
    ev = load_script("evaluate_adversarial_fixtures.py")
    fixture_dir = tmp_path / "fixtures"
    gen.write_fixtures(fixture_dir, gen.generate_fixtures())

    report = ev.evaluate_fixture_dir(
        fixture_dir=fixture_dir,
        engine="offline_baseline",
        target_tokens=[8000],
        budget_modes=["soft_warn"],
        crate_dir=ROOT,
        write_responses=None,
    )

    assert report["engine"] == "offline_baseline"
    assert report["runs"] == len(gen.generate_fixtures())
    assert report["failures"] >= 0
    assert all("error" not in run for run in report["results"])
    latest = next(run for run in report["results"] if run["fixture_id"] == "latest_user_reversal")
    assert latest["invariants"]["latest_user_final"]["passed"] is True
    assert latest["probe_scores"]


def test_compare_context_engines_live_records_core_and_unsupported_without_public_raw_markers(tmp_path):
    mod = load_script("compare_context_engines_live.py")

    report = mod.evaluate(target_tokens=320, crate_dir=ROOT, include_external=True)

    assert report["schema"] == "ContextGovernorSameTranscriptComparisonV1"
    assert set(report["aggregate"]) >= {"full", "head_tail", "context_governor", "hermes_builtin_compressor", "squeez", "ogham", "headroom", "llmlingua"}
    assert report["aggregate"]["context_governor"]["runs"] >= 3
    assert report["aggregate"]["context_governor"]["recoverable_anchor_rate"] >= 0.75
    for engine in ["hermes_builtin_compressor", "squeez", "ogham", "headroom", "llmlingua"]:
        assert report["aggregate"][engine]["status"] in {"unsupported", "ok", "partial"}
        if report["aggregate"][engine]["status"] == "unsupported":
            assert report["aggregate"][engine]["reason"]

    markdown = mod.render_markdown(report)
    assert "same-transcript comparison" in markdown
    assert "error[E0425]" not in markdown
    assert "STORE_NEEDLE" not in markdown
    assert "do not say better than Squeez" not in markdown


def test_compare_context_engines_aggregates_reports(tmp_path):
    mod = load_script("compare_context_engines.py")
    report_a = tmp_path / "a.json"
    report_b = tmp_path / "b.json"
    report_a.write_text(json.dumps({
        "engine": "context_governor",
        "mode": "hard_cascade",
        "runs": 2,
        "failures": 0,
        "aggregate": {
            "avg_full_tokens": 1000,
            "avg_compacted_tokens": 100,
            "avg_token_reduction": 0.9,
            "active_task_visible_rate": 1.0,
            "visible_probe_rate": 0.5,
            "recoverable_probe_rate": 1.0,
            "warnings": 2,
        },
    }))
    report_b.write_text(json.dumps({
        "engine": "offline_baseline",
        "mode": "head_tail",
        "runs": 2,
        "failures": 0,
        "aggregate": {
            "avg_full_tokens": 1000,
            "avg_compacted_tokens": 300,
            "avg_token_reduction": 0.7,
            "active_task_visible_rate": 1.0,
            "visible_probe_rate": 0.4,
            "recoverable_probe_rate": 0.4,
            "warnings": 0,
        },
    }))
    out_base = tmp_path / "comparison"

    summary = mod.compare_reports([report_a, report_b], out_base)

    assert summary["best_by_recoverable_rate"]["engine"] == "context_governor"
    assert out_base.with_suffix(".json").exists()
    md = out_base.with_suffix(".md").read_text()
    assert "context_governor" in md
    assert "offline_baseline" in md


def test_semantic_memory_label_template_writes_csv(tmp_path):
    mod = load_script("semantic_memory_label_template.py")
    facts = tmp_path / "facts.json"
    facts.write_text(json.dumps({
        "facts": [
            {
                "fact_id": "f1",
                "namespace": "context_governor_bench",
                "content": "receipt_id: r1\nitem_id: i1\ncontent_blake3: abc\nArchived content:\nDecision: keep it",
                "source": "context-governor receipt r1 item i1",
            }
        ]
    }))
    out = tmp_path / "labels.csv"

    rows = mod.write_label_template(facts, out)

    assert rows == 1
    with out.open() as fh:
        parsed = list(csv.DictReader(fh))
    assert parsed[0]["fact_id"] == "f1"
    assert parsed[0]["source_receipt"] == "r1"
    assert parsed[0]["item_id"] == "i1"
    assert parsed[0]["label"] == ""
