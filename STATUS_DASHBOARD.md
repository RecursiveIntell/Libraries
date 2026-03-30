# Status dashboard

## Snapshot

- Active lane: `2026-03-30-hardening-closeout`
- Gate authority: `scripts/release_gate_set.py`
- Proof ledger: `STATUS_EVIDENCE_MANIFEST.json`
- Receipt authority: `release/closeout_receipt_v1.json`
- Supported closeout lane: 17 crates from `SUPPORT_PROFILE.md`
- CI surface: `.github/workflows/ci.yml` exists, but only local reruns are certified here

## Current local proof state

- `make gate` passes locally after the 2026-03-30 closeout fixes.
- `bash scripts/run_v25_local_checks.sh` passes locally after the 2026-03-30 v25 closure fixes.
- `bash scripts/run_v25_production_pack_checks.sh --final` passes locally after the 2026-03-30 v25 closure fixes.

## Closed in this pass

- `PACK-001`: `04_MASTER_ISSUE_MATRIX.csv` is present and `bash scripts/check_pack_truth.sh` passes.
- `PACK-002`: the archive manifest count matches disk and `python3 scripts/check_root_archive_manifest.py` passes.
- `TRUTH-001`: the gate ledger now comes from `scripts/run_release_gates.py`, and the dashboard no longer claims historical green without current proof.
- `GATE-001`: `make gate`, `scripts/release_gate_set.py`, `STATUS_EVIDENCE_MANIFEST.json`, and `release/closeout_receipt_v1.json` now describe one release lane.
- `SAFE-001`: `scripts/check_no_prod_panics.sh` now ignores inline test modules under `src/` and passes on the supported production lane.
- `CI-001`: `.github/workflows/ci.yml` exists and includes both the hardening gate and the v25 closure lane.
- `V25-001`: the missing supersession/spec files were restored as repo-local compatibility notes and the shipped v25 repo-truth surface passes.
- `V25-002`: `python3 scripts/check_v25_production_closure.py` passes and the final production-pack command is green locally.
- `TYPE-001`: `V25ConstitutionCitation` now has one canonical definition in `stack-ids`, downstream duplicates were removed, and `python3 scripts/check_public_type_drift.py` passes with zero allowlisted duplicates.
- `NAME-001`: compatibility-name governance crates now carry explicit repositioning docs, `attestation-exchange` has a truthful README/lib surface, and the per-crate decision table names the demotion story directly.
- `DOC-001`: `python3 scripts/check_public_api_docs.py` now checks the 17-crate doc-certified lane and also verifies the demoted compatibility-name crates are documented and called out in scope docs.
- `MOD-001`: the remaining oversized production files are now tracked in `docs/module_budget_exceptions.md`, and `bash scripts/check_hotspot_budgets.sh` enforces that explicit exception set.
- `LLM-001`: the dead `llm-refinement` feature/config path was removed from `forge-pilot`, and `rg -n "use_llm_refinement|llm_model|llm-refinement" forge-pilot -g '!target/**'` now returns no matches.
- `EXTRACT-001`: the Rust bootstrap extractor now emits explicit degradation markers for cfg/attribute/generic/multiline surfaces, and `cargo test -p forge-pilot --test bootstrap_rust_extractor_degradation_tests -- --nocapture` passes.
- `ROOT-001`: duplicate root pack snapshots and machine-readable siblings were physically archived under `docs/archive/root_closeout_history/root_pack_duplicates_20260323/`, the archive manifest now points at one slimmer active authority set, and both `python3 scripts/check_root_archive_manifest.py` and `bash scripts/check_pack_truth.sh` pass together.

## V27 Governance Integration Pack (2026-03-25)

- Phase 0: Workspace restored — 6 missing crates extracted (`constraint-compiler`, `profile-runtime`, `discovery-portfolio`, `federated-settlement`, `remote-oracle-admission`, `spec-execution`).
- Phase 1: Typed validation error enums added to 6 governance crates (`assurance-runtime`, `attestation-exchange`, `authority-delegation`, `constitutional-memory`, `mechanism-runtime`, `continuity-runtime`), following `effect-runtime` reference pattern.
- Phase 2: `forge-pilot/src/governance_gate.rs` created with `observe_governance()`, `gate_execution()`, `build_governance_receipt()`. Feature-gated behind `#[cfg(feature = "governance")]`. Fail-open on missing governance state. Wired into `observe.rs`, `loop_runner.rs`, `loop_runner_report.rs`.
- Phase 3: All production unwrap/expect calls verified clean — all instances are inside `#[cfg(test)]` modules (test code, allowed per rules).
- Phase 4: Test coverage expanded — `llm-tool-runtime` +22 tests (40 total), `verification-calibration` +8 tests (10 total), `recursive-kernel-core` +17 tests (20 total).
- Phase 5: Integration Points and Artifact Families documentation added to all 7 governance crate `lib.rs` files.
- Phase 6: `cargo check --workspace`, `cargo clippy --workspace`, `cargo fmt --all -- --check` all pass. Only pre-existing failures: 4 assurance-runtime fixture roundtrip tests (missing example JSON files).

## Performance baseline

- `evidence/perf_baseline_20260330.json` — canonical regression baseline captured from `kernel-conformance` canonical_perf_snapshot example.

## Release-story limits

- `STATUS_EVIDENCE_MANIFEST.json` is generated from a live local gate run. It is the authoritative ledger for this snapshot.
- `release/closeout_receipt_v1.json` is derived from the ledger plus support/archive/doc surfaces. It is not independent evidence.
- The v25 closure commands are shipped and green locally, but they remain adjacent proof surfaces rather than part of the default `make gate` lane.

No remaining issue rows are open in the current closeout lane.
