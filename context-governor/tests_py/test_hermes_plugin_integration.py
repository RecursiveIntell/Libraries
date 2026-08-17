"""Hermes adapter boundary tests for ri-context-governor.

The Rust core has richer Message fields than Hermes sends to providers.  These
checks keep the adapter's final emitted role/content projection, receipt hashes,
and durable exact-fallback state bound together.
"""

from __future__ import annotations

import json
import os
from pathlib import Path


def _default_hermes_root() -> Path:
    for candidate in (
        Path.home() / "Coding" / "Ares",
        Path.home() / "Coding" / "hermes-agent",
        Path.home() / ".hermes" / "hermes-agent",
    ):
        if (candidate / "plugins" / "context_engine").is_dir():
            return candidate
    return Path.home() / "Coding" / "hermes-agent"


HERMES_ROOT = Path(os.environ.get("HERMES_SOURCE_ROOT") or _default_hermes_root())


def _configure_certified_store(engine, tmp_path, binary: str):
    """Initialize Ares-owned temporary canonical key state for this fixture."""
    from plugins.context_engine._context_governor.key_state import ContextGovernorKeyState

    home = tmp_path / "hermes-home"
    home.mkdir(mode=0o700, exist_ok=True)
    state = ContextGovernorKeyState(home, binary)
    binding = state.initialize_first_install()
    engine._key_state = state
    engine._key_binding = binding
    return binding


def _load_engine():
    from plugins.context_engine import load_context_engine

    engine = load_context_engine("ri-context-governor")
    assert engine is not None
    return engine


def test_plugin_finalizes_emitted_projection_and_persists_exact_fallback(
    tmp_path, monkeypatch
):
    """Plugin receipts must describe the exact transcript Hermes will emit."""
    monkeypatch.syspath_prepend(str(HERMES_ROOT))
    engine = _load_engine()
    binary = str(
        Path(__file__).resolve().parents[1] / "target" / "debug" / "context-governor"
    )
    assert Path(binary).is_file(), (
        "build the debug context-governor binary before this test"
    )
    engine.binary = binary

    engine.store_dir = tmp_path / "receipts"
    engine._policy["token_budget"] = 180
    engine.protect_first_n = 0
    engine.protect_last_n = 1
    engine._policy["summary_max_chars"] = 400
    _configure_certified_store(engine, tmp_path, binary)

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

    emitted = engine.compress(messages, current_tokens=10_000)

    assert emitted != messages
    assert emitted[-1]["role"] == "user"
    assert emitted[-1]["content"] == "LATEST_USER_MUST_SURVIVE"
    # Hermes SessionDB has no durable generic provider-message-id column.
    # The adapter deliberately retains only tool-call identity so a stored
    # receipt and a resumed transcript normalize to the same exact prefix.
    assert "id" not in emitted[-1]
    assert emitted[-1]["metadata"] == {"source": "host"}
    assert all({"role", "content"}.issubset(message) for message in emitted)
    assert not list(engine.store_dir.glob("ctxr_*.json"))
    assert list((engine.store_dir / ".pending").glob("ctxr_*.json"))
    assert engine.validate_pending_compression(emitted) is True
    assert engine.commit_pending_compression(emitted) is True
    assert engine.compression_count == 1

    files = list(engine.store_dir.glob("ctxr_*.json"))
    assert len(files) == 1
    persisted = json.loads(files[0].read_text())
    assert [
        engine._message_from_governor(message)
        for message in persisted["compacted_messages"]
    ] == emitted
    assert persisted["receipt"]["compacted_message_count"] == len(emitted)
    assert (
        persisted["receipt"]["summary_loss_report"]["exact_recovery_state"]
        == "persisted"
    )
    assert persisted["receipt"]["recovery_durability"] == "persisted"
    # Persisting via the descriptor-authorized adapter verifies the response
    # before it can become a parent or exact-fallback authority.
    assert persisted["hmac"].startswith(persisted["receipt"]["signing_key_id"] + ":")

def test_plugin_aborts_lossy_compaction_when_durable_store_fails(tmp_path, monkeypatch):
    """A dead exact-fallback pointer must never be returned as a compaction."""
    monkeypatch.syspath_prepend(str(HERMES_ROOT))
    engine = _load_engine()
    binary = str(
        Path(__file__).resolve().parents[1] / "target" / "debug" / "context-governor"
    )
    engine.binary = binary
    engine.store_dir = tmp_path / "receipts"
    engine._policy["token_budget"] = 180
    engine.protect_first_n = 0
    engine.protect_last_n = 1
    engine._policy["summary_max_chars"] = 400
    _configure_certified_store(engine, tmp_path, binary)

    def fail_store(_response):
        raise RuntimeError("synthetic durable store failure")

    monkeypatch.setattr(engine, "_prepare_response", fail_store)

    messages = [
        {"role": "tool", "content": "bulk " * 1_000},
        {"role": "user", "content": "LATEST_USER_MUST_SURVIVE"},
    ]
    assert engine.compress(messages, current_tokens=10_000) == messages
    assert engine.compression_count == 0
    assert "synthetic durable store failure" in (engine.last_error or "")
    assert (
        engine.get_status()["last_compaction_metrics"]["integrity_result"]
        == "failed_closed_to_authoritative_source"
    )


def test_plugin_two_generation_restart_expands_exact_original_marker(
    tmp_path, monkeypatch
):
    """The host adapter must resume Rust-owned lineage without adapter state."""
    monkeypatch.syspath_prepend(str(HERMES_ROOT))
    binary = str(
        Path(__file__).resolve().parents[1] / "target" / "debug" / "context-governor"
    )
    marker = "PYTHON_ADAPTER_OMITTED_MARKER_d6e2a917"

    def new_engine():
        engine = _load_engine()
        engine.binary = binary
        engine.store_dir = tmp_path / "receipts"
        engine._policy["token_budget"] = 180
        engine.protect_first_n = 0
        engine.protect_last_n = 1
        engine._policy["summary_max_chars"] = 320
        engine._summary_mode = "extractive"
        if not (tmp_path / "hermes-home/context-governor/keys/current.json").exists():
            _configure_certified_store(engine, tmp_path, binary)
        else:
            engine._key_state = type(engine._key_state)(tmp_path / "hermes-home", binary)
        engine.on_session_start("adapter-lineage-session")
        return engine

    first_engine = new_engine()
    first_messages = [
        {"role": "system", "content": "Preserve exact evidence."},
        {
            "role": "tool",
            "content": ("old output " * 1_500) + marker + (" more output" * 1_500),
        },
        {"role": "assistant", "content": "I inspected the output."},
        {"role": "user", "content": "Continue the verification."},
    ]
    first_emitted = first_engine.compress(first_messages, current_tokens=10_000)
    assert marker not in json.dumps(first_emitted)
    assert first_engine.commit_pending_compression(first_emitted) is True
    first_files = sorted(first_engine.store_dir.glob("ctxr_*.json"))
    assert len(first_files) == 1
    first = json.loads(first_files[0].read_text())
    assert first["receipt"]["generation"] == 1
    source = next(
        item
        for item in first["source_evidence"]
        if marker in item["message"]["content"]
    )

    # New engine instance simulates a backend restart. It supplies no parent
    # locator; compact-v2 must recover the unique canonical tip from the store.
    restarted_engine = new_engine()
    second_input = first_emitted + [
        {"role": "tool", "content": "new restart output " * 1_500},
        {"role": "assistant", "content": "restart checkpoint"},
        {"role": "user", "content": "Compact once more."},
    ]
    second_emitted = restarted_engine.compress(second_input, current_tokens=10_000)
    assert marker not in json.dumps(second_emitted)
    assert restarted_engine.commit_pending_compression(second_emitted) is True
    receipts = [
        json.loads(path.read_text())
        for path in restarted_engine.store_dir.glob("ctxr_*.json")
    ]
    second = next(
        receipt for receipt in receipts if receipt["receipt"]["generation"] == 2
    )
    assert (
        second["receipt"]["parent_receipt"]["receipt_id"]
        == first["receipt"]["receipt_id"]
    )

    recovered = restarted_engine._run_json(
        [
            "expand", "--dir", str(restarted_engine.store_dir),
            "--receipt", second["receipt"]["receipt_id"],
            "--item", source["source_id"],
            *restarted_engine._certified_store_args(),
        ],
        {},
    )
    assert marker in recovered["content"]
