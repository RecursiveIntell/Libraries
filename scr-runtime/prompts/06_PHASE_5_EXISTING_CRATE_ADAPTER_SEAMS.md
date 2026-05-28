# Phase 5 — Existing-Crate Adapter Seams

## Objective

Make SCR explicitly aware of and bounded by existing crates in `~/Coding/Libraries` without inventing replacement semantics.

## Required actions

1. Use `docs/EXTERNAL_CRATE_BOUNDARY_MAP.md` from Phase 0.
2. For each owner concept, choose one:
   - integrate existing crate type directly;
   - add optional feature-gated adapter conversion;
   - add trait seam with explicit canonical owner in docs;
   - create `SourceTruthAmbiguityRecord` if ownership/API is unclear.
3. Add or update `scripts/assert_existing_crate_boundaries.py`.
4. Add `docs/SCR_ADAPTER_SEAMS.md` documenting:
   - SCR-local types;
   - canonical owner crate if any;
   - adapter path;
   - migration/deferred integration issue;
   - why local type is allowed, if allowed.
5. Add compile feature plan:

```toml
[features]
standalone-reference = []
stack-integration = []
default = ["standalone-reference"]
```

Only add actual path dependencies after inspecting owner APIs. Do not guess.
6. Ensure local types are named honestly:
   - opaque refs are `OpaqueExternalRef` or equivalent;
   - basis records say `recorded_unverified` unless actually verified;
   - no local type claims to be canonical owner if it is not.

## Acceptance gate

```bash
python3 scripts/assert_existing_crate_boundaries.py
cargo test --workspace --all-targets
```
