# P32 Follow-up Pass — Schema Compatibility + Reference Boundary Fixtures

Run P32 only after P31 targeted tests pass.

## Objective

Wrap the P31 boundary compiler behavior in reference/conformance artifacts and connect it to one real import/export/tool-output path.

## P32 target artifacts

- `ReferenceInterpreterBundleV1`
- `ConformanceRunReceiptV1`
- schema-generation artifacts for P31 receipt/profile types
- schema meta-validation result
- compatibility check baseline

## P32 expected work

1. Generate JSON Schema from the P31 Rust types if possible.
2. Meta-validate generated schemas.
3. Add a reference fixture corpus for boundary compiler behavior.
4. Emit a `ReferenceInterpreterBundleV1` describing supported surface/dialect/profile.
5. Emit or simulate `ConformanceRunReceiptV1` for implementation vs reference fixtures.
6. Connect one real structured import/export/tool-output path to the P31 compiler.
7. Add tests proving that boundary compiler records survive into the artifact/receipt path.

## P32 non-goals

- Full bitemporal query reference interpreter.
- Full v11B graph compiler.
- Region runtime.
- Lawful subtraction.
- External federation.

## P32 starting prompt

```text
Implement P32 — Schema Compatibility + Reference Boundary Fixtures. Build on the P31 boundary compiler microkernel. Generate and meta-validate schemas for the P31 artifact types, create a reference fixture bundle for strict JSON boundary behavior, emit or model ConformanceRunReceiptV1 results, and wire one real structured import/export/tool-output path through the boundary compiler. Do not implement v11B graph/region/subtraction surfaces.
```
