# 04 — Acceptance Gates

## Global gates

```bash
P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh
```

This runs P22 archive contract checks, stale-run hygiene, z.py packaging dry-runs, normal package-clean verification, cargo fmt/check/test/clippy, and the audit-full dry-run.

## Phase 0 gates

- `aidens-contracts` compiles.
- `aidens-boundary-kit` parses strict JSON and emits repair receipts when repair occurs.
- `aidens-config` redacts secrets and validates required fields.
- `aidens-receipts` appends receipts and does not allow mutation through public API.
- `aidens-capability-kit` can represent configured, available, registered, exposed, executable, degraded, disabled, blocked.

## Phase 1 gates

- Provider route truth is exact and test-covered.
- Parser fallback cannot be reported as native.
- Disabled tools are absent from registry/exposure/invocation.
- Write/shell/network risk classes require a permit.
- Runner always emits a run receipt.

## Phase 2 gates

- CLI can scaffold a minimal app.
- `aidens doctor` checks config, provider, tool exposure, receipt sink, and workspace layout.
- Profile expansion prints a visible `AiDENsAppPlanV1`.

## Phase 3+ gates

- Recall extraction does not introduce Tauri/daemon dependencies into core crates.
- Memory adapters preserve no-shadow-database law.
- Queue adapters preserve attempt/lease lineage.
- Schedule adapters never use host wake substrate as canonical schedule truth.
