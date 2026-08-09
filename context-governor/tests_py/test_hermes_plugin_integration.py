"""Hermes adapter boundary tests for ri-context-governor.

The Rust core has richer Message fields than Hermes sends to providers.  These
checks keep the adapter's final emitted role/content projection, receipt hashes,
and durable exact-fallback state bound together.
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


HERMES_ROOT = Path.home() / ".hermes" / "hermes-agent"


def _load_engine():
    from plugins.context_engine import load_context_engine

    engine = load_context_engine("ri-context-governor")
    assert engine is not None
    return engine


def _finalize(binary: str, response: dict) -> dict:
    result = subprocess.run(
        [binary, "finalize"],
        input=json.dumps(response),
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(result.stdout)


def test_plugin_finalizes_emitted_projection_and_persists_exact_fallback(
    tmp_path, monkeypatch
):
    """Plugin receipts must describe the exact transcript Hermes will emit."""
    monkeypatch.syspath_prepend(str(HERMES_ROOT))
    engine = _load_engine()
    module = __import__(engine.__class__.__module__, fromlist=["BINARY"])
    binary = str(Path(__file__).resolve().parents[1] / "target" / "debug" / "context-governor")
    assert Path(binary).is_file(), "build the debug context-governor binary before this test"
    monkeypatch.setattr(module, "BINARY", binary)

    engine.store_dir = tmp_path / "receipts"
    engine._target_tokens = 180
    engine._protect_first_n = 0
    engine._protect_last_n = 1
    engine._summary_max_chars = 400
    engine._hmac_key_path = ""

    messages = [
        {
            "id": "tool-1",
            "role": "tool",
            "name": "terminal",
            "metadata": {"preserve": "only in core input"},
            "content": "build output " * 1_000,
        },
        {
            "id": "active-user",
            "role": "user",
            "metadata": {"source": "host"},
            "content": "LATEST_USER_MUST_SURVIVE",
        },
    ]

    emitted = engine.compress(messages, current_tokens=10_000, force=True)

    assert emitted != messages
    assert emitted[-1] == {"role": "user", "content": "LATEST_USER_MUST_SURVIVE"}
    assert all(set(message) == {"role", "content"} for message in emitted)
    assert engine.compression_count == 1

    files = list(engine.store_dir.glob("ctxr_*.json"))
    assert len(files) == 1
    persisted = json.loads(files[0].read_text())
    assert persisted["compacted_messages"] == emitted
    assert persisted["receipt"]["compacted_message_count"] == len(emitted)
    assert persisted["receipt"]["summary_loss_report"]["exact_recovery_state"] == "persisted"
    assert persisted["receipt"]["recovery_durability"] == "persisted"

    # Re-finalizing the durable response must be idempotent: receipt hashes and
    # token counts already bind the role/content transcript returned above.
    rebound = _finalize(binary, persisted)
    assert rebound["receipt"]["compacted_transcript_blake3"] == persisted["receipt"][
        "compacted_transcript_blake3"
    ]
    assert rebound["receipt"]["compacted_transcript_sha256"] == persisted["receipt"][
        "compacted_transcript_sha256"
    ]
    assert rebound["receipt"]["compacted_approx_tokens"] == persisted["receipt"][
        "compacted_approx_tokens"
    ]


def test_plugin_aborts_lossy_compaction_when_durable_store_fails(tmp_path, monkeypatch):
    """A dead exact-fallback pointer must never be returned as a compaction."""
    monkeypatch.syspath_prepend(str(HERMES_ROOT))
    engine = _load_engine()
    module = __import__(engine.__class__.__module__, fromlist=["BINARY"])
    binary = str(Path(__file__).resolve().parents[1] / "target" / "debug" / "context-governor")
    monkeypatch.setattr(module, "BINARY", binary)
    engine.store_dir = tmp_path / "receipts"
    engine._target_tokens = 180
    engine._protect_first_n = 0
    engine._protect_last_n = 1
    engine._summary_max_chars = 400
    monkeypatch.setattr(engine, "_persist_receipt", lambda response: False)

    messages = [
        {"role": "tool", "content": "bulk " * 1_000},
        {"role": "user", "content": "LATEST_USER_MUST_SURVIVE"},
    ]
    assert engine.compress(messages, current_tokens=10_000, force=True) == messages
    assert engine.compression_count == 0
    assert engine.get_status()["last_safety_scan"] == "persistence-failed"
