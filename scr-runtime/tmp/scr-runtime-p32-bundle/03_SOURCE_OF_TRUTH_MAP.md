# scr-runtime source-of-truth map

## Local crate ownership

| Concept | Owner in this repo | Rule |
|---|---|---|
| SCR-P0A input envelope | `crates/scr-kernel` | Rust type is canonical; schema generated/checked from it. |
| SCR-P0A receipt envelope | `crates/scr-kernel` | Receipt must be complete enough for replay/audit. |
| Score/weight basis points | `crates/scr-kernel` | Durable scores are integer bps only; no float score fields. |
| Proposed action/effect semantics | `crates/scr-reference` + docs | Must materially affect evaluation. |
| Policy model and hard-rule registry | `crates/scr-reference/src/policy.rs` | Unknown hard rules must reject. |
| Reference evaluator | `crates/scr-reference` | Deterministic, no network/LLM/time dependency. |
| CLI schema/fixture/explain flows | `crates/scr-cli` | Generation, verification, and explanation must be separate. |
| Audit fixture adapter | `crates/scr-audit-adapter` | May translate legacy fixture signals into typed control signals. |

## External ownership boundaries

| Concept | Likely external owner | SCR rule |
|---|---|---|
| Cross-crate stable IDs/digests | `stack-ids` or equivalent | Do not claim integration unless compiled/tested. Use opaque refs otherwise. |
| Authority/delegation truth | `authority-delegation` or equivalent | SCR may record declared/adapter-supplied authority basis; it must not invent authority. |
| Effect lifecycle / execution receipts | `effect-runtime` or equivalent | SCR may decide whether to permit effect, not execute effects. |
| Verification/control receipts | `verification-control` or equivalent | SCR receipt adapter seam may map into this later. |
| Evidence exports | `semantic-memory-forge` / ClaimLedger-like evidence | SCR may reference evidence, not fetch/mutate raw evidence. |
| Schema-generation convention | `contract-schema-gen` or repo-local generator | Rust types remain canonical; generated schemas must be checked. |

## Boundary law

If an external owner crate is not available in the current source tree, Codex must not fake integration. It must:

1. keep an opaque adapter boundary,
2. document the seam,
3. add tests proving SCR does not reinterpret refs,
4. record ambiguity in `docs/SourceTruthAmbiguityRecord.md`.
