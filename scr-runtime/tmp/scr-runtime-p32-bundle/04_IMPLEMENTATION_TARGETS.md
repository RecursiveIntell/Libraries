# Implementation targets

## A. Replace signal-fixture evaluator with proposed-action evaluator

Current defect: the evaluator mostly derives decisions from explicit `signal` evidence refs. The final evaluator must consider:

- `domain`
- `proposed_action`
- `requested_effect`
- authority check/basis
- evidence check/basis
- owner-boundary check/basis
- rollback/containment basis
- typed control signals
- policy thresholds and hard rules

### Required model additions

Add types in `scr-kernel` or equivalent:

```rust
ControlSignalV1
SignalSourceV1
AuthorityCheckV1
EvidenceCheckV1
OwnerBoundaryCheckV1
RollbackBasisV1
ActionCandidateV1
CandidateSourceV1
CandidateTraceV1
RawInputDigestV1 or equivalent fields
EvaluatorBuildDigestV1 or equivalent fields
```

Do not add bool-only decision fields. Use enums + reason codes + refs.

## B. Keep opaque refs opaque

Forbidden pattern:

```rust
if evidence_ref.ref_kind == "signal" { ... }
if evidence_ref.ref_value.contains("high_hazard") { ... }
```

Allowed pattern:

```rust
for signal in input.control_signals { ... }
```

Legacy fixture adapter may convert fixture cases into typed `ControlSignalV1`, but the evaluator itself must not scan arbitrary refs.

## C. Authority/evidence honesty

If SCR cannot validate external authority/evidence, the receipt must say:

```text
declared_by_adapter
unverified_external_ref
insufficient_authority
unknown_authority
sufficient_authority
```

Do not call recorded refs "verified" unless an adapter check proves that.

## D. Candidate trace

Receipt must include every candidate:

```text
candidate_id
action
source_kind
source_ref
pressure_axis or hard_rule_id
score_bps
threshold_bps
precedence
selected
rejection_reason_codes
```

The selected action must reference `selected_candidate_id`.

## E. Digests

Receipt must include:

- raw input digest when evaluated from JSON/CLI
- typed canonical input digest
- policy digest
- evaluator build/source digest
- schema version
- canonicalization profile id

`evaluator_algorithm_hash` must not pretend to cover more than it covers. Rename or replace.

## F. Schema/Rust parity

Every Rust-required non-empty string must have schema `minLength: 1`.

Recorded time must either:

1. be real RFC3339/date-time and schema says so, or
2. be renamed/documented as opaque basis and does not claim date-time semantics.

## G. CLI split

CLI must separate:

```text
generate-schemas
eval-fixtures
verify-fixtures
explain-receipt
validate-receipt
hash-policy
```

Generation commands may write files. Verification commands must be read-only and fail if drift exists.

## H. Final proof

No final claim without:

```text
cargo fmt/check/test/clippy outputs
schema validator outputs
fixture outputs
static gate outputs
zip/fresh-unzip outputs if packaging occurs
changed-file list
rollback plan
hostile-auditor handoff
```
