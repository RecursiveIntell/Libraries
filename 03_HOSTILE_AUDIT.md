
# 03_HOSTILE_AUDIT

## Executive posture

This repo now looks like **a real artifact stack wrapped in an unreliable closure story**.

The center of gravity is real:
- `semantic-memory` is substantial and deeply tested.
- `living-memory/living-memory` is substantial and deeply tested.
- `knowledge-runtime`, `semantic-memory-forge`, `forge-memory-bridge`, and `forge-pilot` are not shells.
- the kernel lane is real enough to matter.

The weak flank is **not** “there is no system here.”
The weak flank is:

1. release-truth drift,
2. pack/gate/receipt inconsistency,
3. unfinished production-closure surfaces,
4. thin governance/runtime shell credibility,
5. oversized review surfaces.

## Fresh findings

### 1. The front door still fails on file truth
`check_pack_truth.sh` fails because the numbered hostile pack is missing `04_MASTER_ISSUE_MATRIX.csv`.

That is not an architecture problem.
That is a packaging-truth failure.

### 2. The archive manifest is currently false
`check_root_archive_manifest.py` fails because the manifest says `legacy_root_residue` has 30 files while the directory currently has 29.

That means the archive surface is not self-consistent.

### 3. The dashboard/receipt/evidence surfaces are stale
The current dashboard says:
- root archive manifest is green,
- production panic guard is green,
- public type drift allowlist is empty.

The current repo contradicts all three:
- root archive manifest check fails,
- panic guard script fails,
- the receipt records one allowlisted duplicate semantic type.

This is the single biggest “hostile reviewer” problem in the snapshot.

### 4. The closeout receipt is not independent evidence
`check_closeout_receipt.py` only checks whether the stored receipt matches the output of `generate_closeout_receipt.py`.

`generate_closeout_receipt.py` in turn derives from:
- `STATUS_EVIDENCE_MANIFEST.json`,
- `SUPPORT_PROFILE.md`,
- `STATUS_DASHBOARD.md`,
- archive manifest,
- selected doc-coverage checks.

So the receipt can be perfectly “fresh” while still describing stale truth if its inputs are stale.
That makes the receipt **self-consistent**, not necessarily **world-consistent**.

### 5. `make gate` and the recorded gate set are not the same story
The Makefile gate does **not** run `check_no_prod_panics.sh`, but the evidence manifest and receipt record it as passing.

That is a release-governance bug, not just a docs bug.

### 6. The current panic audit is mostly catching inline test modules
The failing locations are concentrated in `*_tests.rs`, `tests.rs`, and `lib_tests.rs` files stored under `src/`.

That means the panic audit is currently measuring a mixture of:
- production code,
- inline test harnesses,
- src-side roundtrip fixtures.

This must be cleaned up before the gate can honestly sit in the release story.

### 7. CI is still folklore
There is no `.github/workflows/ci.yml`.

For a repo that claims a front-door `make gate`, that is a gap.

### 8. The v25 lane is unfinished in two different ways
The shipped v25 scripts reference missing files.
Separately, the v25 production-closure script still finds real closure gaps in effect/policy/control surfaces.

So this is not one bug; it is both:
- missing package surfaces,
- incomplete code/schema convergence.

### 9. Thin governance/runtime shells still create external drag
The older hostile audit overstated the problem, but it did not invent it.

Current workspace members still include four pure type shells in production code:
- `assurance-runtime`
- `attestation-exchange`
- `authority-delegation`
- `continuity-runtime`

Several more remain near-empty “runtime” crates with 1–3 public functions and zero public doc comments.

### 10. The core review hot spots remain hot
The largest files are still concentrated in the most important crates:
- `profile-runtime/src/adapters.rs`
- `semantic-memory/src/db.rs`
- `semantic-memory/src/lib.rs`
- `forge-pilot/src/main_support/mod.rs`
- `forge-pilot/src/loop_runner.rs`
- `knowledge-runtime/src/runtime/core.rs`

These are not disqualifying, but they are still the sharpest maintenance seam.

### 11. `llm-refinement` is still mostly a story
The feature exists in `forge-pilot/Cargo.toml`.
The config path exists.
The current decision path only appends a hint string when enabled.

That is not real refinement.

### 12. The line-based Rust symbol extractor is still brittle
`forge-pilot/src/bootstrap/extract/rust.rs` still does prefix-based line parsing.
That is fast, but fragile, and the limitation should be enforced or replaced.

## The strongest positive surprise

The older “zero tests / zero docs” narrative is now materially stale for the core lane.

Core snapshot highlights:

| crate | prod_loc | tests | pub_fn | doc_comments |
|---|---|---|---|---|
| semantic-memory | 17505 | 303 | 208 | 847 |
| forge-pilot | 9772 | 54 | 92 | 59 |
| living-memory/living-memory | 9126 | 181 | 138 | 645 |
| knowledge-runtime | 5665 | 145 | 75 | 692 |
| semantic-memory-forge | 3930 | 46 | 34 | 302 |
| forge-memory-bridge | 1723 | 44 | 10 | 271 |
| kernel-conformance | 1376 | 47 | 20 | 20 |
| kernel-execution | 1217 | 10 | 7 | 7 |
| kernel-oracles | 1007 | 12 | 8 | 8 |
| llm-tool-runtime | 2454 | 11 | 32 | 31 |
| stack-ids | 2309 | 56 | 37 | 416 |

That matters because it changes the finish posture. The repo no longer needs a defense that “the architecture is real.”
It needs a truthful closure pass.
