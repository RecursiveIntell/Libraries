# P31 Boundary Compiler Microkernel Report

**Run:** P31 - v11A Boundary Compiler Microkernel + Receipt Fixtures  
**Date:** 2026-05-12  
**Repository root:** `/home/sikmindz/Coding/Libraries/AiDENs`

## 1. Chosen crate/module

- Chosen location: `scaffold/boundary-compiler-core`
- Existing or new: completed existing P31 scaffold crate
- Reason for selection: it already matched the requested crate name, module shape, public API, and fixture set while remaining narrow to v11A strict JSON boundary compilation.
- Workspace integration decision: kept standalone. The root workspace depends on many sibling path crates, and adding this microkernel to the root workspace would increase the blast radius for an intentionally narrow P31 pass.

## 2. Files changed

```text
scaffold/boundary-compiler-core/Cargo.toml
scaffold/boundary-compiler-core/Cargo.lock
scaffold/boundary-compiler-core/src/lib.rs
scaffold/boundary-compiler-core/src/types.rs
scaffold/boundary-compiler-core/src/digest.rs
scaffold/boundary-compiler-core/src/strict_json.rs
scaffold/boundary-compiler-core/src/canonical.rs
scaffold/boundary-compiler-core/src/treatment.rs
scaffold/boundary-compiler-core/src/json_boundary.rs
scaffold/boundary-compiler-core/tests/json_boundary_fixtures.rs
docs/codex-runs/P31_BOUNDARY_COMPILER_MICROKERNEL_REPORT.md
```

## 3. Artifact/contract types implemented

- [x] `BoundaryCompilerProfileV1`
- [x] `ParseReceiptV1`
- [x] `RepairReceiptV1`
- [x] `TreatmentIntegrityReceiptV1`
- [x] `BoundaryDecisionV1`
- [x] `BoundaryCompileResultV1`

Notes:

```text
The crate uses local P31 newtypes such as DigestHex and JsonPointerLikePath. Code TODOs identify stack-ids / boundary-artifact owner crates as the future owners once this standalone compiler is wired into the larger workspace.
```

## 4. Strict JSON behavior

Duplicate-key detection method:

```text
StrictJsonValue implements a custom Serde Visitor and MapAccess loop. Object keys are checked before insertion into a BTreeMap, so {"a":1,"a":2} errors before ordinary serde_json::Value last-write-wins conversion can happen.
```

Malformed input behavior:

```text
Malformed JSON returns BoundaryDecisionV1::Reject and always includes ParseReceiptV1 with raw input digest and the parse error.
```

Unknown-field behavior:

```text
P31 implements a minimal top-level allowlist from either BoundaryCompilerProfileV1.allowed_top_level_fields or a schema object's properties map. UnknownFieldPolicy::Reject rejects; UnknownFieldPolicy::Quarantine quarantines when unknown fields are the only post-parse errors; UnknownFieldPolicy::Allow skips the allowlist check.
```

Coercion behavior:

```text
CoercionPolicy::RejectByDefault compares actual top-level JSON value kinds against declared expected field types and rejects mismatches. No number/string/null coercion is performed.
```

Resource ceiling behavior:

```text
max_bytes is enforced before parse. max_nesting_depth and max_object_keys are enforced after strict parse. Ceiling failures emit ParseReceiptV1 and BoundaryErrorKind::ResourceCeiling.
```

## 5. Canonicalization

Canonicalization profile:

```text
StableSortedJsonV1: object keys sorted lexicographically by BTreeMap order, no insignificant whitespace, arrays preserve order, strings are JSON-serializer encoded, booleans/null emitted normally. This is not claimed as RFC 8785/JCS.
```

Digest algorithm:

```text
SHA-256 over raw input bytes for raw_digest. SHA-256 over StableSortedJsonV1 bytes for canonical_digest.
```

Limitations:

```text
Full JSON Schema validation, canonical stack digest types, generated schemas, and cross-crate owner wiring remain P32+ work.
```

## 6. Receipts

Parse receipts emitted for:

- [x] accepted inputs
- [x] malformed inputs
- [x] duplicate-key inputs
- [x] unknown-field inputs
- [x] resource-ceiling failures

Repair receipt behavior:

```text
RepairPolicy defaults to NoRepair. No repair operator is implemented in P31, no RepairReceiptV1 is emitted, and BoundaryDecisionV1::RepairedAccept is never returned without actual repair.
```

Treatment integrity receipt behavior:

```text
TreatmentIntegrityReceiptV1 is emitted when treatment-critical paths are supplied by the profile or function argument. Missing paths create TreatmentIntegrityDecision::MissingCriticalPath and a rejecting result. Duplicate-key, malformed parse, and pre-parse max_bytes failures also emit a treatment receipt when treatment-critical paths were declared, recording that the path could not be proven through the boundary. Type mismatch or unknown-field errors overlapping a treatment-critical path mark the receipt ChangedWithoutWaiver.
```

## 7. Commands run

```bash
cargo test --manifest-path scaffold/boundary-compiler-core/Cargo.toml
# first run failed because Cargo saw the scaffold as inside the root workspace but not listed as a member/exclude

cargo fmt --manifest-path scaffold/boundary-compiler-core/Cargo.toml
cargo test --manifest-path scaffold/boundary-compiler-core/Cargo.toml
cargo clippy --manifest-path scaffold/boundary-compiler-core/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path scaffold/boundary-compiler-core/Cargo.toml --check
# intermediate clippy runs failed on helper argument count; helpers were refactored into TerminalMetadata and the final clippy run passed
```

## 8. Test results

```text
cargo fmt: pass
cargo fmt --check: pass
cargo test: pass, 28 integration tests passed
cargo clippy --all-targets -- -D warnings: pass
```

Required fixture coverage:

- [x] valid minimal accepted + canonical digest
- [x] malformed rejected + receipt
- [x] duplicate key rejected/quarantined
- [x] duplicate key not last-write-wins
- [x] unknown field rejected/quarantined
- [x] string/number coercion rejected by default
- [x] max bytes ceiling
- [x] max nesting depth ceiling
- [x] treatment-critical missing path receipt
- [x] NoRepair no fake RepairedAccept
- [x] canonical digest stable across object ordering
- [x] accepted and rejected results both have receipts
- [x] duplicate-key quarantine policy
- [x] max object-key ceiling
- [x] schema-derived valid acceptance
- [x] parse/resource failures with treatment-critical paths emit treatment receipts
- [x] canonical bytes match StableSortedJsonV1
- [x] treatment-critical type/unknown-field policy errors mark ChangedWithoutWaiver
- [x] function argument treatment paths override profile paths
- [x] JSON pointer escaping works for treatment-critical paths
- [x] compile results serialize/deserialize as receipt artifacts
- [x] raw digest is SHA-256 over original bytes on pre-parse reject
- [x] post-parse rejects keep canonical digest in ParseReceiptV1 without returning accepted value
- [x] NoRepair emits no repair receipt across accept/reject/quarantine terminal decisions

## 9. P30 issue traceability

```text
This pass implements a new P31 microkernel covering the boundary classes represented by P30-ABSORB-0002, P30-ABSORB-0005, P30-ABSORB-0029, P30-ABSORB-0136, and the dynamic-JSON boundary findings P30-ABSORB-0365 through P30-ABSORB-0387.

It does not claim those existing AiDENs production paths are fully remediated; wiring this compiler into concrete import/export/tool-output paths remains P32 integration work.
```

## 10. Blockers / dependency issues

```text
The first targeted cargo test failed until the crate was made explicitly standalone with an empty [workspace] table. No root workspace build was attempted because it would pull in unrelated sibling path dependencies and broad workspace state.
```

## 11. Invariant revalidation checklist

- [x] Raw digest computed before parse.
- [x] max_bytes enforced before parse.
- [x] Duplicate keys rejected/quarantined before serde_json::Value collapse.
- [x] Malformed JSON rejected with a parse receipt.
- [x] No fake repair receipt or RepairedAccept under NoRepair.
- [x] Treatment-critical missing path emits treatment integrity receipt.
- [x] Accepted input has deterministic canonical digest.
- [x] Phase can proceed to P32 after this narrow P31 pass.

## 12. Confirmation of scope boundary

- [x] No v11B graph compiler work added.
- [x] No region runtime work added.
- [x] No recursive kernel work added.
- [x] No lawful subtraction work added.
- [x] No broad repo cleanup attempted beyond selected crate/module.

## 13. P32 follow-up

Recommended next pass:

```text
P32 - Schema Compatibility + Reference Boundary Fixtures
```

Remaining work:

```text
Replace the P31 minimal schema subset with full generated/meta-validated schema compatibility, publish reference fixtures, and wire one real structured import/export/tool-output path through this compiler with durable receipt propagation.
```
