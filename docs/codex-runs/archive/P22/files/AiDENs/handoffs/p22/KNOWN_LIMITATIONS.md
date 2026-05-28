# P22 Known Limitations

## Current Non-Blocking Risks

- Parent Git repository boundary remains unresolved: `/home/sikmindz/Coding/Libraries` reports `AiDENs/` as untracked.
- Protective filename warnings remain for active Phase 05 secret-redaction prompt/test fixture files:
  - `prompts/phases/PHASE_05_SECRET_REDACTION_AND_API_KEY_WARNING_CLOSURE.md`
  - `scripts/p22_secret_scan_fixture_test.py`
- These filename warnings do not print provider/API-key values and are excluded from normal package content.

## Supported Boundaries

- `z.py` is a stdlib-only source certifier and package builder. It is not a runtime feature.
- Normal packages exclude archived Codex-run history by default.
- `audit-full` mode is the deliberate full-history audit path.
- Operator support-tier JSON is AiDENs reporting metadata only. Canonical sibling crates own semantic truth.

## Partial Or Fixture-Proved Surfaces

- Local mock provider paths are fixture-supported and test-backed.
- Ollama chat routing is partial and depends on a local Ollama service.
- Runner/config/receipt paths are proved for local fixture workflows.
- Memory/runtime/kernel/governance/repair/queue/schedule/wake helpers are adapters or reports over canonical crates, not canonical owners.

## Deferred Or Scaffold Surfaces

- Hosted OpenAI/OpenRouter/Anthropic/cloud provider execution remains deferred.
- Native provider tool loops and streaming remain deferred.
- Full daemon product UX remains deferred.
- Desktop, autonomous memory, and research profile products remain scaffold/deferred.
- Federation, mechanism, recursive-kernel, verification, and memory truth beyond delegated adapter/reporting behavior remains deferred to canonical crates.

## Failed Surfaces

- No active product surface is classified as failed in the final P22 package.
- The secret-scanner fixture intentionally includes a synthetic failing case to prove literal-secret detection remains active.
