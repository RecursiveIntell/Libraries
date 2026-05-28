
# 05_MASTER_ISSUE_MATRIX

## Summary

| phase | priority | stream | id | status | gate | title |
|---|---|---|---|---|---|---|
| Phase 0 — release truth and front door | P0 | Packaging truth | PACK-001 | open | Ship blocker | Restore the numbered pack by adding `04_MASTER_ISSUE_MATRIX.csv` |
| Phase 0 — release truth and front door | P0 | Archive truth | PACK-002 | open | Ship blocker | Fix the root archive manifest file count mismatch |
| Phase 0 — release truth and front door | P0 | Status truth | TRUTH-001 | open | Truth blocker | Rewrite status surfaces from current script truth, not historical green claims |
| Phase 0 — release truth and front door | P0 | Gate convergence | GATE-001 | open | Truth blocker | Make `make gate`, the proof ledger, and the receipt describe the same release lane |
| Phase 0 — release truth and front door | P0 | Safety gate hygiene | SAFE-001 | open | Release-truth blocker | Fix the supported-lane panic audit so it measures production code instead of inline test modules |
| Phase 1 — CI and production closure | P1 | CI surface | CI-001 | open | CI blocker | Add a real `.github/workflows/ci.yml` for the claimed release lane |
| Phase 1 — CI and production closure | P1 | Shipped script truth | V25-001 | open | Pack blocker | Restore or retire the broken v25 repo-truth and production-pack surfaces |
| Phase 1 — CI and production closure | P1 | Production closure | V25-002 | open | Conformance blocker | Complete the v25 production-closure marker set across effect/policy/control surfaces |
| Phase 1 — CI and production closure | P1 | Shared primitives | TYPE-001 | open | Drift blocker | Centralize `V25ConstitutionCitation` and widen drift checking to the crates that currently duplicate it |
| Phase 2 — credibility and maintainability | P2 | Naming credibility | NAME-001 | open | Credibility blocker | Stop overcalling thin governance crates “runtime” crates unless they earn it |
| Phase 2 — credibility and maintainability | P2 | Docs scope | DOC-001 | open | Docs blocker | Expand the doc-truth story beyond the curated core-crate list |
| Phase 2 — credibility and maintainability | P2 | Module shape | MOD-001 | open | Maintainability blocker | Break up the oversized modules that still dominate review difficulty |
| Phase 2 — credibility and maintainability | P2 | LLM integration | LLM-001 | open | Clarity blocker | Either implement real `llm-refinement` or remove the feature flag and config path |
| Phase 2 — credibility and maintainability | P2 | Bootstrap extraction | EXTRACT-001 | open | Correctness blocker | Replace or sharply bound the line-based Rust symbol extractor |
| Phase 2 — credibility and maintainability | P2 | Root pack hygiene | ROOT-001 | open | Package polish | Collapse the duplicated root pack surfaces into one active authority lane |

## Detailed rows

### PACK-001 — Restore the numbered pack by adding `04_MASTER_ISSUE_MATRIX.csv`

**Priority:** P0  
**Stream:** Packaging truth  
**Gate:** Ship blocker  
**Depends on:** —

**Why this exists:**  
`bash scripts/check_pack_truth.sh` fails because the required numbered hostile-finish pack is missing exactly one file: `04_MASTER_ISSUE_MATRIX.csv`.

**Acceptance:**  
`bash scripts/check_pack_truth.sh` passes from a clean checkout and the numbered pack contains `.md`, `.json`, and `.csv` forms of the matrix.

**Required proof:**  
`bash scripts/check_pack_truth.sh`

**Primary surface:**  
04_MASTER_ISSUE_MATRIX.csv (+ keep 04_MASTER_ISSUE_MATRIX.md/json in sync)

**Note:**  
This is the first red light a hostile reviewer hits.

### PACK-002 — Fix the root archive manifest file count mismatch

**Priority:** P0  
**Stream:** Archive truth  
**Gate:** Ship blocker  
**Depends on:** PACK-001

**Why this exists:**  
`python3 scripts/check_root_archive_manifest.py` fails because `docs/archive/root_closeout_history/legacy_root_residue` contains 29 files while the manifest still claims 30.

**Acceptance:**  
The manifest count matches the filesystem, or the missing archived file is restored and counted correctly.

**Required proof:**  
`python3 scripts/check_root_archive_manifest.py`

**Primary surface:**  
docs/archive/root_closeout_history/manifest.json

**Note:**  
Do not patch the dashboard first; patch the archive truth first.

### TRUTH-001 — Rewrite status surfaces from current script truth, not historical green claims

**Priority:** P0  
**Stream:** Status truth  
**Gate:** Truth blocker  
**Depends on:** PACK-002

**Why this exists:**  
`STATUS_DASHBOARD.md`, `STATUS_EVIDENCE_MANIFEST.json`, and `release/closeout_receipt_v1.json` currently describe a greener state than the filesystem/scripts support. Examples: root archive manifest is marked green while its check fails; the dashboard says public type drift allowlist is empty while the receipt records one allowlisted duplicate name.

**Acceptance:**  
Dashboard, evidence manifest, and closeout receipt all describe only what is reproducible from HEAD; stale or historically true claims are marked historical, not current.

**Required proof:**  
Human diff + rerun of the root truth scripts after updates

**Primary surface:**  
STATUS_DASHBOARD.md, STATUS_EVIDENCE_MANIFEST.json, release/closeout_receipt_v1.json

**Note:**  
The current receipt is self-consistent with stale source artifacts; it is not independent evidence.

### GATE-001 — Make `make gate`, the proof ledger, and the receipt describe the same release lane

**Priority:** P0  
**Stream:** Gate convergence  
**Gate:** Truth blocker  
**Depends on:** TRUTH-001

**Why this exists:**  
`Makefile` gate, `STATUS_EVIDENCE_MANIFEST.json`, and `generate_closeout_receipt.py` do not encode one identical gate set. Example: the evidence manifest/receipt record `bash scripts/check_no_prod_panics.sh` as passing, but `make gate` does not run it.

**Acceptance:**  
One authoritative release gate list exists, and it is used consistently by the Makefile, the evidence manifest generation flow, the receipt, and the front-door docs.

**Required proof:**  
One documented gate sequence + a regenerated receipt from that same sequence

**Primary surface:**  
Makefile, STATUS_EVIDENCE_MANIFEST.json, scripts/generate_closeout_receipt.py, PACK_README.md, README.md

**Note:**  
Do not let “receipt truth” diverge from “actual gate truth.”

### SAFE-001 — Fix the supported-lane panic audit so it measures production code instead of inline test modules

**Priority:** P0  
**Stream:** Safety gate hygiene  
**Gate:** Release-truth blocker  
**Depends on:** GATE-001

**Why this exists:**  
`bash scripts/check_no_prod_panics.sh` currently fails on inline test modules stored under `src/` (for example `forge-memory-bridge/src/transform_tests.rs`, `semantic-memory-forge/src/envelope_tests.rs`, `stack-ids/src/ids_tests.rs`, `verification-control/src/lib_tests.rs`, `forge-pilot/src/main_support/tests.rs`). The current receipt still claims this gate is green.

**Acceptance:**  
Either move those inline test modules out of `src/`, or update the audit script to ignore `*_tests.rs`, `tests.rs`, and `lib_tests.rs`; then either run the gate cleanly or stop claiming it as green.

**Required proof:**  
`bash scripts/check_no_prod_panics.sh` or an explicit demotion of the gate from the release story

**Primary surface:**  
scripts/check_no_prod_panics.sh + inline test modules in supported crates

**Note:**  
This is mostly a gate-definition problem, not evidence of rampant production panics in the supported lane.

### CI-001 — Add a real `.github/workflows/ci.yml` for the claimed release lane

**Priority:** P1  
**Stream:** CI surface  
**Gate:** CI blocker  
**Depends on:** SAFE-001

**Why this exists:**  
`python3 scripts/check_v25_production_closure.py` expects `.github/workflows/ci.yml`, but the repo has no workflow file. That means the release process has no canonical hosted execution path.

**Acceptance:**  
A CI workflow exists, installs Rust, runs the root truth scripts, schema checks, and the supported cargo lane, and is referenced by the release docs.

**Required proof:**  
Presence of `.github/workflows/ci.yml` + successful CI on the supported lane

**Primary surface:**  
.github/workflows/ci.yml, README.md, RELEASE_CHECKLIST.md

**Note:**  
Without CI, the repo still relies on oral tradition for its release bar.

### V25-001 — Restore or retire the broken v25 repo-truth and production-pack surfaces

**Priority:** P1  
**Stream:** Shipped script truth  
**Gate:** Pack blocker  
**Depends on:** CI-001

**Why this exists:**  
`bash scripts/check_v25_repo_truth.sh` and `bash scripts/run_v25_local_checks.sh` fail because `24_V25_SUPERSESSION_AND_CONSTITUTIONAL_CHANGE_NOTE_20260317.md` is missing. `bash scripts/run_v25_production_pack_checks.sh` fails because `docs/v25/PRODUCTION_MASTER_ISSUE_MATRIX_20260318.csv` is missing.

**Acceptance:**  
Either the missing v25 files are restored and the scripts pass, or the scripts and docs are explicitly retired so shipped commands no longer reference dead surfaces.

**Required proof:**  
`bash scripts/check_v25_repo_truth.sh` and `bash scripts/run_v25_production_pack_checks.sh`

**Primary surface:**  
scripts/check_v25_repo_truth.sh, scripts/run_v25_local_checks.sh, scripts/run_v25_production_pack_checks.sh, docs/v25/*

**Note:**  
Broken shipped scripts are credibility debt even when they are not on the default gate.

### V25-002 — Complete the v25 production-closure marker set across effect/policy/control surfaces

**Priority:** P1  
**Stream:** Production closure  
**Gate:** Conformance blocker  
**Depends on:** V25-001

**Why this exists:**  
`python3 scripts/check_v25_production_closure.py` currently fails because key v25 constitution/profile markers are still absent from `effect-runtime/src/effect.rs`, `effect-runtime/src/observation.rs`, `effect-runtime/src/compensation.rs`, and `verification-policy/src/lib.rs`.

**Acceptance:**  
The production-closure script passes with Cargo available and all required markers/schemas/examples exist.

**Required proof:**  
`python3 scripts/check_v25_production_closure.py`

**Primary surface:**  
effect-runtime/src/*.rs, verification-policy/src/lib.rs, contract-schema-gen/src/lib.rs, schemas/, examples/

**Note:**  
This is a real code/doc convergence gap, not just a packaging problem.

### TYPE-001 — Centralize `V25ConstitutionCitation` and widen drift checking to the crates that currently duplicate it

**Priority:** P1  
**Stream:** Shared primitives  
**Gate:** Drift blocker  
**Depends on:** V25-002

**Why this exists:**  
`V25ConstitutionCitation` is defined separately in `effect-runtime`, `remote-oracle-admission`, `federated-settlement`, and `verification-control`. The current public-type-drift script under-scopes this because it does not scan all of those crates.

**Acceptance:**  
One canonical definition exists in a primitive crate, all runtime crates reuse it, and the drift check covers the actual duplication surface.

**Required proof:**  
`rg -n "pub struct V25ConstitutionCitation"` returns one canonical definition

**Primary surface:**  
stack-ids or equivalent primitive crate + affected runtime crates + scripts/check_public_type_drift.py

**Note:**  
The current allowlist hides only part of the actual duplication.

### NAME-001 — Stop overcalling thin governance crates “runtime” crates unless they earn it

**Priority:** P2  
**Stream:** Naming credibility  
**Gate:** Credibility blocker  
**Depends on:** TYPE-001

**Why this exists:**  
Four workspace crates are still pure type shells in production code (`assurance-runtime`, `attestation-exchange`, `authority-delegation`, `continuity-runtime`), and several others remain near-empty runtime/governance crates with 1–3 public functions and zero docs.

**Acceptance:**  
Either rename/reposition these crates honestly (`*-types`, `*-schema`, `*-profile`) or deepen them with actual runtime logic and documentation.

**Required proof:**  
Per-crate decision table + updated Cargo metadata/README surfaces

**Primary surface:**  
Affected crate `Cargo.toml`, `README.md`, and `src/lib.rs` surfaces

**Note:**  
This is an external-credibility problem, not an internal build-breaker.

### DOC-001 — Expand the doc-truth story beyond the curated core-crate list

**Priority:** P2  
**Stream:** Docs scope  
**Gate:** Docs blocker  
**Depends on:** NAME-001

**Why this exists:**  
`python3 scripts/check_public_api_docs.py` currently tracks 13 curated crates and passes, but most governance shells still have zero public doc comments. The gate is narrower than the repo narrative.

**Acceptance:**  
Either widen the docs gate to the credibility-critical shell crates or explicitly demote those crates from the public-facing surface and say so in the support docs.

**Required proof:**  
Updated docs gate + no surprise zero-doc shells in the claimed release story

**Primary surface:**  
scripts/check_public_api_docs.py, SUPPORT_PROFILE.md, README.md, thin governance crates

**Note:**  
Green coverage on a curated subset is fine only if the subset is named honestly.

### MOD-001 — Break up the oversized modules that still dominate review difficulty

**Priority:** P2  
**Stream:** Module shape  
**Gate:** Maintainability blocker  
**Depends on:** DOC-001

**Why this exists:**  
Large production files remain concentrated in critical crates: `profile-runtime/src/adapters.rs` (1776 LOC), `semantic-memory/src/db.rs` (1609), `semantic-memory/src/lib.rs` (1600), `forge-pilot/src/main_support/mod.rs` (1592), `forge-pilot/src/loop_runner.rs` (1034), and `knowledge-runtime/src/runtime/core.rs` (1199).

**Acceptance:**  
Critical files are split into reviewed submodules with focused tests, or explicit budget exceptions are documented and justified.

**Required proof:**  
File-size diff + preserved test coverage

**Primary surface:**  
profile-runtime, semantic-memory, forge-pilot, knowledge-runtime

**Note:**  
The architecture is real; the review surface is still harder than it needs to be.

### LLM-001 — Either implement real `llm-refinement` or remove the feature flag and config path

**Priority:** P2  
**Stream:** LLM integration  
**Gate:** Clarity blocker  
**Depends on:** MOD-001

**Why this exists:**  
`forge-pilot` still exposes an `llm-refinement` feature/config path, but the current decision path only appends a check hint when enabled; it does not perform real model-guided decision refinement.

**Acceptance:**  
A real refinement step exists with tests and clear failure semantics, or the feature flag and config field are removed.

**Required proof:**  
No dead feature path remains in `forge-pilot/src/decide.rs` and `Cargo.toml`

**Primary surface:**  
forge-pilot/Cargo.toml, forge-pilot/src/config.rs, forge-pilot/src/decide.rs

**Note:**  
This is the cleanest example of a capability being narrated more strongly than implemented.

### EXTRACT-001 — Replace or sharply bound the line-based Rust symbol extractor

**Priority:** P2  
**Stream:** Bootstrap extraction  
**Gate:** Correctness blocker  
**Depends on:** LLM-001

**Why this exists:**  
`forge-pilot/src/bootstrap/extract/rust.rs` still uses a line/prefix parser. It will miss multiline signatures, attributes, cfg-gated items, and richer Rust forms.

**Acceptance:**  
Use a Rust parser (`syn`/tree-sitter) for symbol extraction, or add explicit degradation markers and fixtures proving the exact unsupported surface.

**Required proof:**  
Extractor fixtures covering multiline/attribute/generic/cfg cases

**Primary surface:**  
forge-pilot/src/bootstrap/extract/rust.rs + bootstrap tests/docs

**Note:**  
This is acceptable for a cheap bootstrap only if the limitation is enforced and surfaced.

### ROOT-001 — Collapse the duplicated root pack surfaces into one active authority lane

**Priority:** P2  
**Stream:** Root pack hygiene  
**Gate:** Package polish  
**Depends on:** EXTRACT-001

**Why this exists:**  
The repo root currently has 54 top-level files, including duplicated `SOURCE_BASIS`, three `MASTER_ISSUE_MATRIX` markdown files, two matrix JSONs, two exact-file-touch maps, two dashboards, and two evidence manifests. The duplication is explicit in filenames, not inferred.

**Acceptance:**  
One active pack remains authoritative at the root; numbered or superseded materials are archived or clearly demoted.

**Required proof:**  
Root file count and active-pack manifest shrink without losing history

**Primary surface:**  
Root pack docs + docs/archive/root_closeout_history/manifest.json

**Note:**  
This is not the first blocker, but it is the easiest way for a strong repo to still look messy.
