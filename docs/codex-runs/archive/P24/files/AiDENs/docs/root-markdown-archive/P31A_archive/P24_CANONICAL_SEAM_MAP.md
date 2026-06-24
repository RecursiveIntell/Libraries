# P24 Canonical Seam Map

AiDENs is the product, compiler, orchestration, operator, fixture, and display/report layer. It consumes canonical stack seams; it does not replace them.

| Seam | Canonical owner | AiDENs P24 behavior | Evidence |
|---|---|---|---|
| IDs, digests, trace, attempt, trial | `stack-ids` | `AiDENsRunBundleV2` embeds `TraceCtx`, `AttemptId`, `TrialId`, and `ContentDigest` values from `stack-ids` types. | `crates/aidens-contracts/src/lib.rs`; `tests/fixtures/p24/aidens_run_bundle_v2.json` |
| Execution context | `semantic-memory-forge` | Run bundles carry `ExecutionContextV1` and backpoint to Forge ownership. | `target/p24/test-agent/run-bundle.json`; `target/p24/coding-agent/run-bundle.json` |
| Forge export | `semantic-memory-forge` | Memory fixture creates `ExportEnvelopeV3`; AiDENs does not define export truth locally. | `target/p24/memory-seam/export-envelope-v3.json` |
| Export-to-import bridge | `forge-memory-bridge` | AiDENs calls `transform_forge_export` through `aidens-memory-kit`. | `target/p24/memory-seam/projection-import-batch-v3.json` |
| Memory storage/import | `semantic-memory` | AiDENs opens canonical `MemoryStore` with mock embedder for fixture import. | `target/p24/memory-seam/memory-runtime-seam-report.json` |
| Runtime query/provenance | `knowledge-runtime` | AiDENs queries through `KnowledgeRuntime` and discloses view/widening/degradation. | `target/p24/memory-seam/memory-runtime-seam-report.json` |
| Tool runtime semantics | `llm-tool-runtime` | AiDENs local tool receipts are display/operator wrappers with backpointers to runtime tool receipt semantics. | `target/p24/coding-agent/coding-agent-report.json` |
| Verification/control/repair | `verification-*` | Boundary repair records and verification/control pointers remain canonical-owner backpointers; local repair is display evidence only. | `crates/aidens-boundary-kit/src/lib.rs`; boundary tests in `target/p24/audit/` |
| Daemon queue receipts | AiDENs local operator lane over stack IDs | Append-only local queue evidence is supported-local only; no autonomous external side effects are claimed. | `target/p24/daemon-safe/queue.ndjson` |

## Enforcement

- `scripts/p24_verify.sh` runs no-shadow-truth, no local canonical type duplicate, no local digest law, schema-scope, wrapper-backpointer, run-bundle, coding-agent, memory seam, and package dry-run checks.
- `scripts/verify.sh` now delegates to `scripts/p24_verify.sh`.
