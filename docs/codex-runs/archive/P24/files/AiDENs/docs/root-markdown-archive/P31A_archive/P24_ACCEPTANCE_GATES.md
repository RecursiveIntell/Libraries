# P24 acceptance gates

## Gate 0 — Source and package truth

- `AiDENs/docs/p24/P24_SOURCE_BASIS.md` exists and records package name, archive SHA, report path, manifest path, findings path, excluded path, and codex-archive path.
- The package sidecars show zero validation findings or the run records exact failures and blocks promotion.
- `AiDENs/docs/codex-runs/CURRENT_RUN.md` identifies P24 as current.

## Gate 1 — Verifier does not hang

- `AiDENs/scripts/p24_verify.sh` exists.
- Every assertion command is timeout-wrapped.
- The verifier prunes `.git`, `target`, `target-*`, archive, generated package output, and generated sidecar paths.
- The verifier emits a JSON receipt with command, exit code, duration, stdout/stderr path, and pass/fail.

## Gate 2 — Canonical seam ownership

- `P24_CANONICAL_SEAM_MAP.md` names every canonical seam consumed by AiDENs and its owning library crate.
- No AiDENs crate defines canonical-looking replacements for `EpisodeBundleV*`, `ExecutionContextV*`, `EvidenceBundle`, `ExportEnvelopeV*`, `ProjectionImportBatchV*`, `RepairRecord`, or `VerificationPlan` unless the definition is an alias/re-export of the owning crate.
- Display/report DTOs include support tier and canonical backpointer or explicit degraded/no-canonical-ref reason.

## Gate 3 — AiDENsRunBundleV2

- `run-test-agent` emits `AiDENsRunBundleV2`.
- The bundle contains or references canonical `ExecutionContextV1`, `TraceCtx`, `AttemptId`, `TrialId`, provider route, tool route, budget/deadline, degradation markers, replay link, event-log digest, and receipt digests.
- `inspect-run` validates bundle schema and digest linkage.
- Replay comparison distinguishes byte equality from normalized semantic equality.

## Gate 4 — Supported-local coding-agent lane

- `aidens run-coding-agent` or equivalent command exists.
- It can operate on a local fixture repo without network access.
- It emits read/list/search/status receipts, permit decisions, patch proposal or abstention, and final support-tier report.
- It refuses writes without explicit permit/approval and records the denial receipt.

## Gate 5 — Canonical memory/runtime seam

- A test imports an `ExportEnvelopeV3` fixture through `forge-memory-bridge` and `aidens-memory-kit`.
- Query output discloses view model, temporal coordinates, widening/degradation, and source backpointers.
- No AiDENs-local memory truth store is used as authority.

## Gate 6 — Daemon-safe local queue lane

This gate is required only if the pass promotes daemon-safe from scaffold/partial.

- Queue lifecycle is append-only and replayable.
- Lease/heartbeat/finish/fail/cancel/drain states emit receipts.
- Duplicate suppression is tested.
- No external side effects occur.

## Gate 7 — Boundary/repair/failure honesty

- Strict parser fixtures reject duplicate keys and treatment-critical ambiguity.
- Repair records include before/after digests, reason, actor, confidence/status, and canonical backpointer where applicable.
- Risk-bearing outputs require verification plan reference or explicit abstention/degradation.

## Gate 8 — Support profile honesty

- README, STATUS, SUPPORT_PROFILE, RUN_ORDER, AGENTS, and Known Limitations all agree.
- Supported / partial / scaffold / deferred are explicitly separated.
- Coding-agent lane is supported-local only if Gate 4 passes.
- daemon-safe is supported-local or partial only if Gate 6 passes.
- desktop/memory/research profiles remain scaffold/partial unless tested.

## Gate 9 — Final package and handoff

- `cargo fmt --all --check`, `cargo check --workspace --all-targets`, `cargo test --workspace --all-targets`, and `cargo clippy --workspace --all-targets -- -D warnings` pass in the real repo environment.
- Package self-replay passes for release and next-codex-context profiles.
- Final handoff contains command transcript, artifact hashes, changed files, known limitations, support claims, and unresolved risk register.
