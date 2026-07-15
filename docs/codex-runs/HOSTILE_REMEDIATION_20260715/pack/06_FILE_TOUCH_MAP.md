# File touch map

Locators, not blanket write permission. Hermes assigns exact scopes and locks shared files.

## Phase 0

| Issue | Primary code | Regression tests |
|---|---|---|
| AG-001 | `agent-graph/src/{engine,interrupt,error}.rs` and event/checkpoint status code | `agent-graph/tests/interrupt_failure_contract.rs` |
| GOV-001 | `forge-pilot/src/governance_gate.rs` and observation/loop call sites | `forge-pilot/tests/governance_fail_closed.rs` |
| CMP-001 | `scr-runtime-compression/src/{codec_dispatch,exact_fallback_adapter,error}.rs` | `scr-runtime-compression/tests/decode_contract.rs` plus fixtures |

## Phase 1

| Workstream | Paths |
|---|---|
| Canonical API | `stack-ids/**` |
| Graph/queues | `agent-graph/**`, `ai-batch-queue/**`, `job-queue/**`, `tauri-queue/**` |
| Ledger | `claim-ledger/**` |
| Memory/AiDENs | `semantic-memory/**`, `AiDENs/crates/aidens-contracts/**`, consumers |
| Codec IDs | `poly-kv/crates/quant-codec-core/**`, codec backends |
| Enforcement | repository scripts/config/manifests |

Root `Cargo.toml` and `Cargo.lock` remain integration-owned.

## Phase 2

- DIG-001: `stack-ids/src/digest.rs`, versioned compatibility, golden fixtures.
- SCP-001: `stack-ids/src/scope.rs`, bridge/storage callers.
- LED-001: `claim-ledger/src/{ledger,ids,types}.rs`, receipts/tests.

## Phase 3

- Contract: `poly-kv/crates/quant-codec-core/src/{codec,digest,ids,profile,wire,error}.rs`.
- Turbo: `turbo-quant/**`.
- Fib: `fib-quant/**`.
- Consumer: `semantic-memory/src/vector_codec.rs`, search/index/config/receipts.
- Adapter: `scr-runtime-compression/**`.
- Pool: assigned `poly-kv` exact-authority/receipt paths.

## Phase 4

- QUE-001: `ai-batch-queue/src/{queue,executor,types}.rs`.
- QUE-002: `job-queue/src/{lib,executor,db,types,events}.rs`.
- SEM-001: `semantic-memory/src/{search,db,types}.rs`.

## Phase 5

`.github/workflows/**`, workspace manifests, lint/evidence/release scripts, status/support docs,
codec/KV READMEs, claims manifest.

## Broad-edit prohibitions

No repository-wide formatting/replacement from narrow branches; no generated evidence in
implementation branches; no uncoordinated schema/reader edits; no deletion of old wire/storage
fields before compatibility and rollback gates.
