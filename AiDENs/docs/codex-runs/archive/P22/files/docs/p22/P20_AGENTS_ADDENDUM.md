# P20 AGENTS Addendum

These instructions are binding for the P20 run.

## Operating doctrine

AiDENs is an orchestration and product surface over canonical libraries. It must not become the source of truth for evidence, memory, kernel inference, verification, repair, or control.

## Required behavior

- Make minimal, targeted changes.
- Prefer thin adapters over local copies of canonical law.
- Preserve existing P00–P19 artifacts unless they are false; then correct them or move them to historical/archive docs.
- Do not hide build/test/clippy failures.
- Do not mark a feature done without an executable test or explicit proof artifact.
- Do not add fake provider support.
- Do not add an agency policy type without routing at least one real runner path through it.

## Canonical owner map

| Surface | Canonical owner |
|---|---|
| shared IDs, digests, trace primitives | `stack-ids` |
| raw evidence/export truth | `semantic-memory-forge` |
| export/import transform and digest/backpointer preservation | `forge-memory-bridge` |
| queryable projected memory truth | `semantic-memory` |
| runtime view use, widening, result provenance | `knowledge-runtime` |
| recursive kernel witnesses/syndromes/residuals/oracles | kernel crates |
| verification/control/adjudication/calibration | `verification-*` crates |
| provider/tool receipts | `llm-tool-runtime` and AiDENs adapter receipt layer |
| closed-loop orchestration | `forge-pilot` / AiDENs runner as consumer-only |
| agency/influence policy for AiDENs surface | `aidens-agency-kit` unless promoted to a canonical sibling crate later |

## Naming rule

Do not name an AiDENs-local type as if it owns canonical law unless it is explicitly a re-export or adapter.

Bad:

```rust
pub struct CanonicalEvidenceBundleV1 { ... } // local duplicate
```

Allowed:

```rust
pub struct AidensEvidenceDisplayReportV1 { ... } // non-authoritative view
```

## Final proof rule

The run is not complete until:

```bash
bash scripts/p20_verify.sh
bash scripts/p20_generate_audit_bundle.sh
```

both complete, or the final report explicitly says `P20 FAILED`.
