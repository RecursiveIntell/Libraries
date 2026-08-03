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


def test_historical_replay_uses_hashed_aggregate_output(tmp_path, monkeypatch):
    mod = load_script("hermes_task_replay_eval.py")
    db = tmp_path / "state.db"
    import sqlite3
    conn = sqlite3.connect(db)
    conn.execute("CREATE TABLE sessions (id TEXT PRIMARY KEY, title TEXT)")
    conn.execute("CREATE TABLE messages (id INTEGER PRIMARY KEY, session_id TEXT, role TEXT, content TEXT, active INTEGER, timestamp REAL, tool_name TEXT)")
    conn.execute("INSERT INTO sessions (id, title) VALUES ('s1', 'secret parser session')")
    contents = [
        ('user', 'Build parser. Acceptance gate: cargo test must pass.'),
        ('assistant', 'Decision: use deterministic JSON parsing.'),
        ('tool', 'error[E0425]: cannot find value parser in /tmp/src/lib.rs'),
        ('assistant', 'Fixed part of it.'),
        ('user', 'Latest task: rerun cargo test and fix remaining parser errors.'),
    ]
    # Add enough messages to pass min_messages and enough bulk to be selected.
    for idx in range(20):
        role, content = contents[idx % len(contents)]
        conn.execute(
            "INSERT INTO messages (session_id, role, content, active, timestamp, tool_name) VALUES (?,?,?,?,?,?)",
            ('s1', role, content + f' filler {idx}', 1, idx, 'terminal' if role == 'tool' else None),
        )
    conn.commit(); conn.close()

    def fake_governor(messages, crate_dir, target_tokens):
        text = mod.render_messages(messages[:1] + messages[-1:])
        exact = mod.render_messages(messages)
        return {"status": "ok", "text": text, "recoverable_text": exact, "tokens": 100, "receipt_id": "ctxr_fake", "warnings": []}

    monkeypatch.setattr(mod, "run_context_governor", fake_governor)
    report = mod.evaluate_db(db, ROOT, limit=10, min_messages=5, target_tokens=500)
    markdown = mod.render_markdown(report)

    assert report["schema"] == "HermesHistoricalAnswerabilityReplayV1"
    assert report["runs"] == 1
    assert report["aggregate"]["context_governor"]["answerability_rate"] >= 0.75
    assert "secret parser session" not in markdown
    assert "cargo test must pass" not in markdown
    assert report["redacted_runs"][0]["question_hashes"]


def test_task_success_synthetic_fixture_has_operational_questions():
    mod = load_script("task_success_eval.py")
    fixture = mod.synthetic_fixture()

    assert fixture["fixture_id"] == "synthetic_task_success"
    assert len(fixture["questions"]) >= 4
    assert fixture["request"]["policy"]["token_counter"] == "provider_chat_approx"


def test_task_success_summary_flags_good_governed_report():
    mod = load_script("task_success_eval.py")
    summary = mod.summarize(
        {
            "fixture_id": "x",
            "receipt_id": "ctxr_test",
            "warnings": [],
            "baselines": [
                {
                    "name": "full",
                    "answerability_rate": 1.0,
                    "incorrect_action_risk": 0,
                    "tokens": 1000,
                    "active_task_visible": True,
                },
                {
                    "name": "head_tail",
                    "answerability_rate": 0.25,
                    "incorrect_action_risk": 0,
                    "tokens": 100,
                    "active_task_visible": True,
                },
                {
                    "name": "context_governor",
                    "answerability_rate": 1.0,
                    "incorrect_action_risk": 0,
                    "tokens": 200,
                    "active_task_visible": True,
                },
            ],
        }
    )

    assert summary["ok"] is True
    assert summary["token_reduction_vs_full"] == 0.8


def test_certify_markdown_reports_gate_status(tmp_path):
    mod = load_script("certify_all.py")
    report = {
        "schema": "ContextGovernorCertificationV1",
        "crate": str(ROOT),
        "quick": True,
        "ok": True,
        "gates": [
            {"name": "cargo-test", "required": True, "ok": True, "returncode": 0},
            {"name": "hermes-plugin-tests", "required": False, "ok": False, "returncode": 1},
        ],
        "task_success_summary": {
            "ok": True,
            "context_governor_answerability": 1.0,
            "head_tail_answerability": 0.25,
            "token_reduction_vs_full": 0.8,
        },
    }
    out = tmp_path / "cert.md"
    mod.write_markdown(report, out)
    text = out.read_text()

    assert "cargo-test" in text
    assert "hermes-plugin-tests" in text
    assert "Task Success" in text
