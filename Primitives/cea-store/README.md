# cea-store

Transactional persistence contract for the CEA **observational association** graph.

## Contract

`cea-store` persists nodes, normalized attribution weights, Beta edge statistics,
negative observations, version-scoped decay, and an idempotent run log. A graph
update and its run-log entry commit or roll back together.

Identified observations deduplicate by `AttributedRunResult::idempotency_key()`;
legacy content-only runs fall back to their content hash. Replaying the same
observation identity with changed content cannot inflate graph evidence.

## Evidence boundary

Only `EvidenceKind::Observational` can enter the edge store. Paired,
ablation, counterfactual, and synthetic-telemetry evidence remain in typed
receipts owned by the execution layer. This prevents a storage round trip from
silently erasing evidence grade.

## Verification

```bash
cargo test -p cea-store --all-targets
```
