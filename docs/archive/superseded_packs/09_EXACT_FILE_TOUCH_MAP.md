
# 09_EXACT_FILE_TOUCH_MAP

This is the shortest plausible file-first map for the current finish pass.

## PACK-001 — Restore the numbered pack by adding `04_MASTER_ISSUE_MATRIX.csv`

Primary surfaces:
- 04_MASTER_ISSUE_MATRIX.csv (+ keep 04_MASTER_ISSUE_MATRIX.md/json in sync)

Suggested first edit:
- 04_MASTER_ISSUE_MATRIX.csv (+ keep 04_MASTER_ISSUE_MATRIX.md/json in sync)

## PACK-002 — Fix the root archive manifest file count mismatch

Primary surfaces:
- docs/archive/root_closeout_history/manifest.json

Suggested first edit:
- docs/archive/root_closeout_history/manifest.json

## TRUTH-001 — Rewrite status surfaces from current script truth, not historical green claims

Primary surfaces:
- STATUS_DASHBOARD.md
- STATUS_EVIDENCE_MANIFEST.json
- release/closeout_receipt_v1.json

Suggested first edit:
- STATUS_DASHBOARD.md

## GATE-001 — Make `make gate`, the proof ledger, and the receipt describe the same release lane

Primary surfaces:
- Makefile
- STATUS_EVIDENCE_MANIFEST.json
- scripts/generate_closeout_receipt.py
- PACK_README.md
- README.md

Suggested first edit:
- Makefile

## SAFE-001 — Fix the supported-lane panic audit so it measures production code instead of inline test modules

Primary surfaces:
- scripts/check_no_prod_panics.sh
- inline test modules in supported crates

Suggested first edit:
- scripts/check_no_prod_panics.sh + inline test modules in supported crates

## CI-001 — Add a real `.github/workflows/ci.yml` for the claimed release lane

Primary surfaces:
- .github/workflows/ci.yml
- README.md
- RELEASE_CHECKLIST.md

Suggested first edit:
- .github/workflows/ci.yml

## V25-001 — Restore or retire the broken v25 repo-truth and production-pack surfaces

Primary surfaces:
- scripts/check_v25_repo_truth.sh
- scripts/run_v25_local_checks.sh
- scripts/run_v25_production_pack_checks.sh
- docs/v25/*

Suggested first edit:
- scripts/check_v25_repo_truth.sh

## V25-002 — Complete the v25 production-closure marker set across effect/policy/control surfaces

Primary surfaces:
- effect-runtime/src/*.rs
- verification-policy/src/lib.rs
- contract-schema-gen/src/lib.rs
- schemas/
- examples/

Suggested first edit:
- effect-runtime/src/*.rs

## TYPE-001 — Centralize `V25ConstitutionCitation` and widen drift checking to the crates that currently duplicate it

Primary surfaces:
- stack-ids or equivalent primitive crate
- affected runtime crates
- scripts/check_public_type_drift.py

Suggested first edit:
- stack-ids or equivalent primitive crate + affected runtime crates + scripts/check_public_type_drift.py

## NAME-001 — Stop overcalling thin governance crates “runtime” crates unless they earn it

Primary surfaces:
- Affected crate `Cargo.toml`
- `README.md`
- and `src/lib.rs` surfaces

Suggested first edit:
- Affected crate `Cargo.toml`

## DOC-001 — Expand the doc-truth story beyond the curated core-crate list

Primary surfaces:
- scripts/check_public_api_docs.py
- SUPPORT_PROFILE.md
- README.md
- thin governance crates

Suggested first edit:
- scripts/check_public_api_docs.py

## MOD-001 — Break up the oversized modules that still dominate review difficulty

Primary surfaces:
- profile-runtime
- semantic-memory
- forge-pilot
- knowledge-runtime

Suggested first edit:
- profile-runtime

## LLM-001 — Either implement real `llm-refinement` or remove the feature flag and config path

Primary surfaces:
- forge-pilot/Cargo.toml
- forge-pilot/src/config.rs
- forge-pilot/src/decide.rs

Suggested first edit:
- forge-pilot/Cargo.toml

## EXTRACT-001 — Replace or sharply bound the line-based Rust symbol extractor

Primary surfaces:
- forge-pilot/src/bootstrap/extract/rust.rs
- bootstrap tests/docs

Suggested first edit:
- forge-pilot/src/bootstrap/extract/rust.rs + bootstrap tests/docs

## ROOT-001 — Collapse the duplicated root pack surfaces into one active authority lane

Primary surfaces:
- Root pack docs
- docs/archive/root_closeout_history/manifest.json

Suggested first edit:
- Root pack docs + docs/archive/root_closeout_history/manifest.json
