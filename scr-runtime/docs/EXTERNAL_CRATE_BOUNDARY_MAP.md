# External Crate Boundary Map

This map records external owner crates that govern canonical concepts and the SCR
runtime adapter strategy used in this repository.

## Canonical owner crates (discovered)

| crate | ownership domain | SCR handling |
|---|---|---|
| attestation-exchange | attestation and provenance envelopes | Adapter-only use; SCR does not own provenance semantics. |
| authority-delegation | authority and delegation artifacts | Adapter-only use; no local canonical reinvention. |
| contract-schema-gen | schema governance and generation process | SCR reuses workspace schema generation patterns and local strict schemas. |
| effect-runtime | effect lifecycle artifacts | Adapter boundary only; SCR only consumes effect/action abstractions. |
| knowledge-runtime | runtime-query provenance and temporal view | Adapter-only; SCR stays deterministic and local. |
| llm-tool-runtime | tool execution receipts and dispatch lineage | Treated as non-authoritative adapter source. |
| semantic-memory-forge | evidence exports and evidence refs | SCR treats references as opaque and local-facing only. |
| stack-ids | identity/digest/tracing primitives | SCR references these canonical types and does not recreate them. |
| verification-control | control-plane receipt and review surfaces | SCR maps local decision outcomes to the canonical boundary contracts. |
| verification-policy | policy and permit artifacts | SCR evaluates policy under policy-domain constraints and compatibility checks. |

## Temporary boundary notes

- Any remaining SCR-local type-like concept not listed here is tracked in `docs/P31_UNRESOLVED_RISKS.md` and must be migrated or justified before release.

