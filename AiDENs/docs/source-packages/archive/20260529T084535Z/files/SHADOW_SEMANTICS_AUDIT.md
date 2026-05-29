# Shadow Semantics Audit — P31A

Record date: `2026-05-29`
Ledger: `docs/codex-runs/CURRENT_RUN.json`

This audit records shadow semantics findings from the P30 hostile audit and their P31A resolution status.

## Method

Every finding tagged `SHADOW-*` in the P30 hostile audit was classified by severity (P0–P3) and triaged against the canonical owner map. A shadow semantic is any local definition in AiDENs that duplicates, reinterprets, or silently widens semantics owned by a sibling crate without explicit delegation or receipt.

## Findings

| Finding | Status | Severity | Resolution |
|---|---|---|---|
| `SHADOW-P0-001` local ExecutionContextV1 | `resolved` | P0 | Removed; aidens-contracts re-exports semantic-memory-forge::ExecutionContextV1 |
| `SHADOW-P0-002` local MemoryStore | `resolved` | P0 | Removed; aidens-memory-adapter opens canonical semantic-memory::MemoryStore |
| `SHADOW-P0-003` local ArtifactId/ClaimId/EvidenceId | `resolved` | P0 | Removed; all stack IDs consumed from stack-ids crate |
| `SHADOW-P0-004` local EpisodeBundleV1 | `resolved` | P0 | Removed; re-exported from semantic-memory-forge |
| `SHADOW-P0-005` local tool invocation DTOs | `resolved` | P0 | Removed; aidens-receipts is a ToolReceiptSink over llm-tool-runtime receipts |
| `SHADOW-P1-001` local ReceiptEnvelopeV1 | `resolved` | P1 | Removed; canonical receipt types from llm-tool-runtime + verification-control |
| `SHADOW-P1-002` local RunReceipt/PromotionReceipt | `resolved` | P1 | Removed; canonical verification-control receipts used |
| `SHADOW-P1-003` local ClaimRecordV1/EvidenceRecordV1 | `resolved` | P1 | Removed; canonical types from semantic-memory-forge |
| `SHADOW-P1-004` local ProjectionRecordV1 | `resolved` | P1 | Removed; canonical projection from semantic-memory |
| `SHADOW-P1-005` local SchemaCatalogEntryV1 owner_crate | `resolved` | P1 | owner_crate set to aidens-orchestration (display-only metadata) |
| `SHADOW-P2-001` provider status overclaim | `resolved` | P2 | Provider matrix reclassified cloud providers as BoundaryUnavailable |
| `SHADOW-P2-002` budget stop without receipt | `resolved` | P2 | Budget exhaustion now emits canonical ControlReceipt |
| `SHADOW-P3-001` broad allow/lint suppression | `resolved` | P3 | Workspace lints deny unsafe_code, todo, dbg_macro |

## Zero-tolerance rule

No P0 or P1 shadow semantic may remain unresolved for a release audit to pass. All P0/P1 findings above are marked `resolved`. P2/P3 findings are tracked but do not block release audit.

## Evidence

- `CANONICAL_OWNER_MAP.md` — canonical ownership declarations and forbidden AiDENs behavior
- `crates/aidens-contracts/src/lib.rs` — no forbidden local truth type declarations
- `crates/aidens-provider-kit/src/lib.rs` — provider matrix with BoundaryUnavailable for cloud providers
- `tests/fixtures/p07/artifact_family_registry_v1.json` — no aidens-contracts owner_crate entries