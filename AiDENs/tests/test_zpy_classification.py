import z


def test_stale_codex_reason_uses_classification_for_anchored_variants():
    classification = {"scripts/p30_guard.py": "durable-script"}

    reason = z.stale_codex_reason_for_rel(
        "AiDENs/scripts/p30_guard.py",
        "P32",
        classification,
    )

    assert reason is None


def test_rewrite_workspace_manifest_members_keeps_workspace_dependencies():
    original = """[workspace]
resolver = "2"
members = [
    "agent-graph",
    "AiDENs",
]
default-members = [
    "agent-graph",
    "AiDENs",
]

[workspace.dependencies]
schemars = "0.8"
"""

    rewritten = z.rewrite_workspace_manifest_members(original, ["AiDENs", "attestation-exchange"])

    assert '"agent-graph"' not in rewritten
    assert '"AiDENs"' in rewritten
    assert '"attestation-exchange"' in rewritten
    assert 'default-members = [' in rewritten
    assert '[workspace.dependencies]\nschemars = "0.8"' in rewritten
