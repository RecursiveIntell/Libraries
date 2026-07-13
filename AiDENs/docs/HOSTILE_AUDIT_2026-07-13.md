# AiDENs Hostile Audit — 2026-07-13

**Target:** `/home/sikmindz/Coding/Libraries/AiDENs`
**Baseline:** `feat/full-integration` at `9d73fae1204062e24beb284c9972779ad69cded6`; the completion work is intentionally uncommitted while final review runs.
**Authority:** current source and fresh controller-owned command receipts outrank historical P31B/P32 statements and agent summaries.

## Current verdict

**Local supported lane: build-verified, not release-certified.** The original P0/P1 local defects were repaired and focused gates passed. A fresh full workspace build/lint/test pass completed before final release-ledger synchronization. The release ledger remains **blocked** because package/extracted-replay certification has not been rerun, autonomous-cycle receipts are explicitly non-durable process history, and no authenticated OpenAI-compatible HTTP endpoint was configured for a live receipt.

## Original audit findings and disposition

| ID | Severity | Disposition | Current evidence |
|---|---:|---|---|
| HA-001 | P0 | Fixed | Root mirrors and `CURRENT_RUN.json` now agree on P32/P30; `assert_current_run_truth.py` passed before final ledger synchronization. |
| HA-002 | P0 | Fixed | Phase 08 fixture now has canonical claim IDs; full workspace test passed on the remediation tree. |
| HA-003 | P1 | Fixed | Empty compatibility ledger gate passed; data-row rejection remains covered. |
| HA-004 | P1 | Fixed | README, STATUS, SUPPORT_PROFILE, and the historical P20 audit are explicitly separated from current-run truth. |
| HA-005 | P2 | Fixed | Delegation remains documented as disabled/quarantined, not operational. |

## Follow-up hardening disposition

| Area | Disposition | Evidence |
|---|---|---|
| Approval binding | Fixed | Approval request IDs are derived from the exact tool/risk/sandbox/run/attempt context; context substitution is rejected. |
| Permit exposure | Fixed | Pre-run exposure does not authorize side effects without run/attempt IDs; dispatch remains the exact-context authorization boundary. |
| Receipt append integrity | Fixed | Corrupt, syntactically valid, and malformed histories are validated under append lock and block new records. `aidens-receipts`: 8 tests passed. |
| Runner governance truth | Fixed | Tool execution success produces non-promotable advisory governance evidence rather than a claimed causal verification. |
| Patch rollback truth | Fixed | Existing versus absent targets are retained; failed new-file writes are removed and receipts distinguish restoration/removal/residual state. |
| Autonomous durable completion | Fixed | Queue completion occurs before in-memory task completion. |
| Autonomous receipt material | Fixed, non-durable | Hash material is framed and includes viscosity; cycle mode/errors and per-cycle gaps are retained in an inspectable process-owned ledger. It is not a durable audit log. |
| Provider parsing and truth | Fixed | Mixed malformed native tool arrays fail explicitly; unavailable OpenAI-compatible routes are not advertised executable. |
| Receipt-store doctor | Fixed | A configured corrupt canonical log is degraded/unavailable rather than healthy. |
| UI safety | Fixed | TUI truncation is Unicode-safe. |
| Test isolation | Fixed | Environment-mutating P30 tests use an async lock; focused test and Clippy passed. |

## Fresh live provider evidence

- **Ollama:** loopback `127.0.0.1:11434`, model `llama3.2:3b`; discovery returned 35 models, a chat completion returned `AIDENS_LIVE_OLLAMA_RUN_OK`, and a native function-call probe returned one `aidens_probe` call. Sanitized receipt: `.codex_evidence/live-provider/ollama-live-2026-07-13.json`.
- **OpenAI-compatible HTTP:** not certified. The controller environment had no `OPENAI_API_KEY`, base URL, or model configuration. Codex OAuth is available for coding-agent operation, but it is not an OpenAI-compatible API endpoint credential and is not evidence of the runtime boundary.

## Remaining external limits

The following are not certified by this pass: package/extracted-replay lifecycle, durable autonomous cycle-receipt persistence/restart recovery, authenticated OpenAI-compatible HTTP behavior, federation, attestation, settlement, remote-oracle behavior, and production network failure/cancellation/rate-limit behavior.

## Final local command bar

The settled-tree command bar is authoritative only after its final rerun immediately before commit:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --locked --all-targets -- -D warnings
cargo test --workspace --locked --all-targets --no-fail-fast
bash scripts/phase_verify_contract_ownership.sh final
bash scripts/verify_current.sh
python3 scripts/p30_guard.py
git diff --check
```

No claim of full product or production certification is made by this audit.
