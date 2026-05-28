# P31 TASK — v11A Boundary Compiler Microkernel + Receipt Fixtures

You are working in the current repository. Implement the next narrowly-scoped v11A compiler pass.

## Objective

Create or complete one buildable Rust crate/module implementing the v11A structured-boundary compiler microkernel for strict JSON inputs.

This pass must produce executable code, tests, and a short report. It must not drift into the v11B graph/region compiler.

## Source documents to inspect first, if present

Search the repo for these names and read the relevant sections before editing:

- `CANONICAL_STACK_SPEC_V11A_CONSTITUTIONAL_ARTIFACT_RUNTIME_CORE.md`
- `V11_PLUS_ARTIFACT_FAMILY_INDEX.md`
- `V11_PLUS_CONFORMANCE_AND_RELEASE_BAR.md`
- `json_research.md`
- `control.md`
- `Coding-research-next-codex-context-20260511.report.md`

Also run targeted searches:

```bash
rg -n "BoundaryCompilerProfileV1|ParseReceiptV1|RepairReceiptV1|TreatmentIntegrityReceiptV1|BoundaryDecisionV1|BoundaryCompileResultV1" .
rg -n "contract-schema-gen|boundary compiler|structured boundary|duplicate key|canonicalization|repair receipt|treatment integrity" .
rg -n "constraint-compiler|recursive-kernel-core|v11B|CompiledGraphBundleV1|GraphSurfaceDeclarationV1" .
```

Use the existing crate/module if one clearly exists and is buildable. Otherwise create a small standalone crate in the least disruptive location. Acceptable names:

- `boundary-compiler-core`
- `contract-boundary-compiler`
- `v11a-boundary-compiler`

Do not make the root workspace worse. If adding the crate to the root workspace would trigger unrelated missing path dependency failures, keep it standalone and document that decision.

## Required logical artifacts

Implement serializable Rust types for:

- `BoundaryCompilerProfileV1`
- `ParseReceiptV1`
- `RepairReceiptV1`
- `TreatmentIntegrityReceiptV1`
- `BoundaryDecisionV1`
- `BoundaryCompileResultV1`

Use existing stack primitives for IDs, digests, trace/time fields if they are clearly available and buildable. If not, create local newtypes such as `DigestHex`, `SchemaId`, `BoundaryProfileId`, and write TODO notes naming the intended future owner.

## Required function

Implement a strict JSON boundary compiler with this conceptual shape:

```rust
pub fn compile_json_boundary(
    profile: &BoundaryCompilerProfileV1,
    raw: &[u8],
    schema: Option<&serde_json::Value>,
    treatment_critical_paths: &[JsonPointerLikePath],
) -> BoundaryCompileResultV1
```

If the repo already has a better local API style, adapt the signature while preserving the semantics.

## Required behavior

The compiler must:

1. Compute a raw input digest before parsing.
2. Enforce a `max_bytes` resource ceiling before parsing.
3. Parse JSON strictly.
4. Detect duplicate object keys without relying on `serde_json::Value` last-write-wins behavior.
5. Reject or quarantine duplicate keys according to profile policy.
6. Reject malformed JSON.
7. Reject or quarantine unknown fields according to profile policy when schema/profile provides an allowed field set.
8. Reject number/string/null coercion unless the profile explicitly allows it.
9. Enforce resource ceilings after parse:
   - max nesting depth;
   - max total object keys, if feasible.
10. Produce a deterministic canonical representation and canonical digest for accepted input.
11. Emit `ParseReceiptV1` for every parse attempt, including rejects.
12. Emit `RepairReceiptV1` only when actual repair is performed.
13. Default repair policy to `NoRepair` for this pass.
14. Emit `TreatmentIntegrityReceiptV1` whenever treatment-critical paths are missing, ambiguous, changed, repaired, schema-migrated, or otherwise semantically touched.
15. Return one clear decision:
   - `Accept`
   - `Reject`
   - `Quarantine`
   - `RepairedAccept`, only if actual repair exists.

## Duplicate-key rule

Do not parse directly into `serde_json::Value` and then check for duplicates. That has already lost the evidence.

Implement one of:

- a custom Serde `Visitor` / `MapAccess` parser that tracks keys while parsing; or
- a strict tokenizer/parser; or
- a crate-local parser already capable of duplicate-key detection.

A test must prove that `{"a":1,"a":2}` is not silently accepted as `{"a":2}`.

## Canonicalization rule

Use a stable canonicalization profile. For this pass, it is acceptable to implement `StableSortedJsonV1`:

- object keys sorted lexicographically;
- no insignificant whitespace;
- arrays preserve order;
- strings encoded by JSON serializer;
- booleans/null emitted normally.

Do not call this RFC 8785/JCS unless you fully implement that standard. Name the profile honestly.

## Schema behavior for this pass

If the repository already has JSON Schema validation, use it.

If not, implement a minimal profile/schema subset sufficient for tests:

- optional allowed top-level fields;
- optional per-field expected JSON type;
- reject/quarantine unknown fields according to policy;
- reject string/number/null coercion by default.

Document that full JSON Schema validation is P32 unless you implement it completely here.

## Required tests

Create targeted tests. Names may vary, but cover these cases exactly:

1. `valid_minimal_json_is_accepted_and_gets_canonical_digest`
2. `malformed_json_is_rejected_with_parse_receipt`
3. `duplicate_key_is_rejected_or_quarantined`
4. `duplicate_key_is_not_silently_last_write_wins`
5. `unknown_field_policy_rejects_surprise_structure`
6. `string_number_coercion_is_rejected_by_default`
7. `resource_ceiling_rejects_large_input`
8. `resource_ceiling_rejects_deep_input`
9. `treatment_critical_missing_path_requires_integrity_receipt`
10. `no_repair_policy_never_emits_fake_repair_accept`
11. `canonical_digest_is_stable_for_equivalent_object_ordering`
12. `accepted_and_rejected_results_both_have_receipts`

## Required commands

Run the narrowest useful command set. Do not run a whole-repo build if unrelated path dependencies are broken.

Preferred:

```bash
cargo fmt --manifest-path <chosen-crate>/Cargo.toml
cargo test --manifest-path <chosen-crate>/Cargo.toml
```

If the crate is a clean workspace member:

```bash
cargo fmt -p <crate-name>
cargo test -p <crate-name>
```

Optional if local dependency state allows:

```bash
cargo clippy --manifest-path <chosen-crate>/Cargo.toml --all-targets -- -D warnings
```

If a command fails due to unrelated workspace/dependency breakage, do not hide it. Switch to a targeted standalone manifest command and document the blocker.

## Required report

Write:

```text
docs/codex-runs/P31_BOUNDARY_COMPILER_MICROKERNEL_REPORT.md
```

The report must include:

- chosen crate/module;
- why that location was chosen;
- files changed;
- commands run;
- tests passing/failing;
- receipt types implemented;
- duplicate-key detection method;
- canonicalization profile used;
- schema validation level implemented;
- what remains for P32;
- dependency/workspace blockers;
- confirmation that no v11B graph/region compiler work was added.

Use `05_P31_REPORT_TEMPLATE.md` from this pack if available.

## Explicitly out of scope

Do not implement:

- `GraphSurfaceDeclarationV1`;
- `CompiledGraphBundleV1`;
- region contracts;
- recursive inference;
- causal attribution bundles;
- lawful subtraction;
- external/federated admission;
- generated self-hosting spec compiler;
- broad workspace cleanup;
- a new global architecture document.

## Acceptance criteria

The pass is complete only when:

- the selected crate/module builds under a targeted `cargo test` command;
- all required tests pass or explicitly documented failures are limited to external workspace blockers;
- duplicate keys are never silently accepted;
- every result has a parse receipt;
- repair receipts are not faked when repair is disabled;
- treatment-critical missing/touched paths produce treatment integrity receipts;
- accepted input has a deterministic canonical digest;
- a P31 report exists;
- no v11B graph/region/subtraction scope has been added.

Stop when the above is satisfied. Do not continue into P32 in this pass.
