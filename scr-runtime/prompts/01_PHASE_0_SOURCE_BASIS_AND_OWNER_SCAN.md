# Phase 0 — Source Basis and Owner Scan

## Objective

Establish the true current repository state and the existing owner-crate boundary map before mutating runtime semantics.

## Required actions

1. Inspect current SCR repo:

```bash
pwd
find . -maxdepth 4 -type f | sort > docs/P31_SOURCE_FILE_LIST.txt
git status --short || true
cargo metadata --format-version 1 --no-deps > docs/P31_SCR_CARGO_METADATA.json
```

2. Inspect the canonical library root:

```bash
find /home/sikmindz/Coding/Libraries -maxdepth 4 -name Cargo.toml -print | sort > docs/P31_LIBRARIES_CARGO_TOMLS.txt
```

3. For likely owner crates, inspect public APIs and docs. At minimum search for:

```bash
rg -n "struct .*Digest|enum .*Digest|ContentDigest|ArtifactId|Receipt|Policy|Permit|Authority|Attestation|EvidenceRef|ExecutionContext|RuntimeQueryProvenance|ControlReceipt|JsonSchema|schema" /home/sikmindz/Coding/Libraries || true
```

4. Create/update `docs/EXTERNAL_CRATE_BOUNDARY_MAP.md` with:

- crate name;
- path;
- concept owned;
- relevant public types/functions found;
- whether SCR currently duplicates the concept;
- planned fix: use dependency, adapter trait, unresolved ambiguity, or explicitly local P0A-only.

5. Update `docs/SOURCE_BASIS.md` to reflect the current truth. It must no longer say no Cargo workspace exists if one exists.

## Acceptance gate

- `docs/EXTERNAL_CRATE_BOUNDARY_MAP.md` exists.
- `docs/SOURCE_BASIS.md` matches current repo state.
- No code changes to runtime semantics yet, except adding docs/scripts needed for source-basis capture.
