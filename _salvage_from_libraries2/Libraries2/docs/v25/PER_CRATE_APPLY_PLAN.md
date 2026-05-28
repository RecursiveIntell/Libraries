# Per-crate apply plan — v25

## Core owners

| Crate / area | Landed now | Still queued | Acceptance signal |
|---|---|---|---|
| `profile-runtime` | canonical v25 artifact families, reference composition logic, tests, fixtures | broader differential corpus, stricter linting hooks | examples and fixtures parse; reference tests cover block, exception, conflict |
| `stack-ids` | shared ID newtypes for all v25 families | none for v25 vocabulary freeze | repo-truth check finds all required IDs |
| `contract-schema-gen` | v25 family registration and schema publication | regenerate schemas from Rust types once toolchain is present | registry entries exist for all v25 families |
| `knowledge-runtime` | view types and example-backed tests for constitution / obligation / conflict / diff | live query surfaces over stored v25 artifacts | runtime view examples parse and tests exist |

## Existing profile-family owners

| Crate | Current role in v25 | Required next delta |
|---|---|---|
| `verification-policy` | owns P1/P2 plus effect/release/continuity policy profile surfaces | add first-class adapter helpers or direct contribution emitters where family content must flow into composition |
| `authority-delegation` | owns role/delegation/approval/conflict family content | add typed contribution adapters and direct control-case citations |
| `assurance-runtime` | owns deployment / regulated / hazard content | add contribution helpers for release and assurance obligations |
| `attestation-exchange` | owns vendor translation and trust surfaces | add contribution helpers for caveats, trust-root, and downgrade obligations |
| `continuity-runtime` | owns incident routing and escalation content | add contribution helpers for incident-mode and continuity obligations |

## Remaining consumers

| Crate | Why it matters | Concrete next file-touch |
|---|---|---|
| `effect-runtime` | should stop treating raw profiles as ambient permission folklore | add v25-aware preflight/commit citation fields or wrapper surfaces for compiled obligations |
| `verification-control` | review cases should cite the composite constitutional lane | add `effective_constitution_id` / `compiled_obligation_set_id` to effect, release, delegation, and continuity review surfaces |
| `verification-adjudication` | final dispositions should cite the same composite constitutional answer | add composition refs to promotion / refutation / rollback decision surfaces |
| `remote-oracle-admission` | locality, disclosure, and vendor caveats may change remote admissibility | add consumption hooks or documented integration surfaces |
| `federated-settlement` | local and shared settlement should preserve local effective constitution refs | add local constitutional citation path |

## Non-code owners

| Area | Purpose |
|---|---|
| `docs/v25/` | current taught surface, execution notes, and release bar |
| `apply/v25/` | engineer-facing landing sequence and mirror sync |
| `scripts/` | repo-truth and JSON-surface validation without requiring cargo |
| `contracts/fixtures/v25/` | governed fixture corpus and manifest |
| `conformance/v25/` | release-facing conformance notes and corpus expectations |
