# Master issue matrix

The supplied audit correctly found the top CEA bug, but several “zero tests” claims are now stale or wrong.

This matrix is the canonical finish-line issue list. It distinguishes what is already landed by evidence, what this pack is ready to restore, what remains genuinely open, and what is explicitly deferred.

## BASE-001 — Keep the 2026-03-22 hardening closeout green

- Status: **landed-by-evidence**
- Priority: **P0**
- Phase: **Done**
- Area: `hardening-closeout`
- Owners: root, contract-schema-gen, verification-*, living-memory
- Depends on: —

**Current state**

The active hardening receipt records passing results for repo surface, doc truth, manifest truth, schema registry uniqueness, no production panics, mirror discipline, hotspot budgets, public type drift, root archive manifest, public API docs, schema compatibility, selected cargo tests, and closeout receipt generation.

**Required change**

No architectural change is required. Preserve the active gate set and keep release/closeout_receipt_v1.json reproducible from the same support scope.

**Acceptance**

- STATUS_EVIDENCE_MANIFEST.json remains the release-facing proof ledger for the hardening lane.
- release/closeout_receipt_v1.json regenerates without drift.
- No open allowlist debt is reintroduced for public type drift or production panics.

**Proof**

- `python3 scripts/check_closeout_receipt.py`
- `python3 scripts/check_public_type_drift.py`
- `bash scripts/check_no_prod_panics.sh`

**Touch files**

- none


## BASE-002 — Keep schema publication single-truth across wave/profile manifests

- Status: **landed-by-evidence**
- Priority: **P0**
- Phase: **Done**
- Area: `schema-governance`
- Owners: contract-schema-gen, wave/profile owner crates
- Depends on: BASE-001

**Current state**

The closeout receipt records 17 schema manifests across v16-v25 and P1-P7, each with one owner crate and one canonical publication directory under schemas/.

**Required change**

Preserve the current owner mapping and do not introduce a second publication surface or duplicate owner claims.

**Acceptance**

- contracts/schemas/*/manifest.json keeps one owner crate per schema family.
- schemas/ remains the canonical publication directory.
- bash scripts/check_schema_registry_uniqueness.sh stays green.

**Proof**

- `bash scripts/check_schema_registry_uniqueness.sh`
- `python3 scripts/generate_closeout_receipt.py`

**Touch files**

- none


## BASE-003 — Treat v21-v24 waves and P1-P7 profiles as landed artifact surfaces, not future fiction

- Status: **landed-by-evidence**
- Priority: **P1**
- Phase: **Done**
- Area: `artifact-surfaces`
- Owners: effect-runtime, authority-delegation, assurance-runtime, continuity-runtime, verification-policy, attestation-exchange
- Depends on: BASE-002

**Current state**

The repo already contains v21-v24 and P1-P7 schemas, examples, fixtures, and per-crate fixture conformance tests. The gap is not artifact existence; it is narrated end-to-end demonstration.

**Required change**

Do not reopen owner-crate or schema-family debates. Use the landed fixtures as the substrate for the public demonstrator and benchmark pack.

**Acceptance**

- No new owner crate is invented for v21-v24 or P1-P7.
- Demo and benchmark work consume the existing fixture corpus instead of redefining it.
- Horizon work remains explicitly separate from the finish bar.

**Proof**

- `contracts/fixtures/v21-v24 and p1-p7 remain present`
- `crate fixture_conformance tests remain present`

**Touch files**

- none


## DOC-001 — Restore the canonical root control-plane pack on disk

- Status: **ready-to-apply**
- Priority: **P0**
- Phase: **Phase 0**
- Area: `repo-front-door`
- Owners: root docs
- Depends on: BASE-001

**Current state**

The supplied source tree is missing the root closeout pack even though scripts, archive manifest, and receipt all assume it exists.

**Required change**

Create the root control-plane files, numbered aliases, and prompt files supplied in this pack so the repo root once again matches the hardening receipt and archive manifest.

**Acceptance**

- bash scripts/check_repo_surface.sh passes.
- bash scripts/check_doc_truth.sh passes.
- python3 scripts/check_root_archive_manifest.py passes.
- README.md points contributors to make gate and the active front door.

**Proof**

- `bash scripts/check_repo_surface.sh`
- `bash scripts/check_doc_truth.sh`
- `python3 scripts/check_root_archive_manifest.py`

**Touch files**

- `README.md`
- `00_START_HERE.md`
- `PACK_README.md`
- `MASTER_ISSUE_MATRIX.md`
- `MASTER_ISSUE_MATRIX.json`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`
- `STATUS_DASHBOARD.md`
- `RELEASE_CHECKLIST.md`
- `AGENTS.md`
- `PROMPT.md`


## DOC-002 — Collapse stale scans and hostile audits into one active truth statement

- Status: **ready-to-apply**
- Priority: **P0**
- Phase: **Phase 0**
- Area: `truth-reconciliation`
- Owners: root docs
- Depends on: DOC-001

**Current state**

The stale scan summary still shows pre-closeout failures, while the active 2026-03-22 hardening receipt shows those failures closed. External reviewers can be misled if both are presented without reconciliation.

**Required change**

Make the active docs explicitly mark the stale findings as superseded, preserve the useful critique about external demonstrability, and route readers to the current receipt as authority.

**Acceptance**

- CLAUDE_AUDIT_RECONCILIATION.md explicitly says the stale zero-test claims are wrong now.
- MASTER_ISSUE_MATRIX.md preserves the audit reconciliation note.
- STATUS_DASHBOARD.md names the stale scan as superseded evidence, not current truth.

**Proof**

- `bash scripts/check_doc_truth.sh`

**Touch files**

- `CLAUDE_AUDIT_RECONCILIATION.md`
- `02_HOSTILE_AUDIT_RECONCILED.md`
- `MASTER_ISSUE_MATRIX.md`
- `STATUS_DASHBOARD.md`


## SCOPE-001 — Make the supported closeout lane and adjacent artifact-owner crates explicit

- Status: **ready-to-apply**
- Priority: **P0**
- Phase: **Phase 0**
- Area: `support-scope`
- Owners: root docs
- Depends on: DOC-002

**Current state**

The receipt hardens a 17-crate closeout lane, but the workspace default-members list is broader and includes adjacent owner crates such as effect-runtime, authority-delegation, assurance-runtime, continuity-runtime, profile-runtime, and spec-execution.

**Required change**

Teach the support scope once: the 17-crate hardening lane is the release claim; the adjacent owner crates are landed artifact surfaces and the substrate for the public demonstrator, but they are not the narrow build-certified claim for this receipt.

**Acceptance**

- SUPPORT_PROFILE.md parses into the same 17 crates recorded in the receipt.
- The adjacency list accounts for the remaining default-members not in the support lane.
- No front-door doc claims that the full 29 default-members lane was build-certified by the hardening receipt.

**Proof**

- `python3 scripts/generate_closeout_receipt.py`
- `release/closeout_receipt_v1.json`

**Touch files**

- `SUPPORT_PROFILE.md`
- `09_CRATE_BOUNDARY_MAP.md`
- `PACK_README.md`
- `README.md`


## DEMO-001 — Ship one narrated end-to-end demonstrator across v21 -> v22 -> v23

- Status: **landed-by-evidence**
- Priority: **P0**
- Phase: **Phase 1**
- Area: `external-demonstrability`
- Owners: effect-runtime, authority-delegation, assurance-runtime, verification-control, llm-tool-runtime
- Depends on: SCOPE-001, BASE-003

**Current state**

The narrated demonstration is present and evidence-backed. The bundle, walkthrough, and validating test stitch the effect execution, delegated authority, and release assurance artifacts into one replayable path for review.

**Required change**

No additional implementation is required for the finish bar. Preserve the existing proof surfaces and keep them cited as the release-facing evidence for the stitched `v21 -> v22 -> v23` path.

**Acceptance**

- A reviewer can inspect one bundle and one walkthrough and see preflight -> delegation -> release readiness without hand-assembling wave fixtures.
- The demonstration remains typed-artifact backed, not prose-only.
- The proof set names the participating fixtures and owner crates explicitly.

**Proof**

- `contracts/fixtures/demo/effect_authority_assurance_release.bundle.json`
- `docs/demos/effect_authority_assurance_release.md`
- `verification-control/tests/e2e_effect_authority_assurance_release.rs`

**Touch files**

- `docs/demos/effect_authority_assurance_release.md`
- `contracts/fixtures/demo/effect_authority_assurance_release.bundle.json`
- `verification-control/tests/e2e_effect_authority_assurance_release.rs`
- `README.md`
- `11_BENCHMARK_PLAN.md`


## BENCH-001 — Ship the benchmark / forge-bench proof package

- Status: **partially-landed**
- Priority: **P1**
- Phase: **Phase 2**
- Area: `benchmark-proof`
- Owners: knowledge-runtime, verification-control, forge-pilot
- Depends on: DEMO-001

**Current state**

The forge-bench package is present: casebook, runner, score sheet, and README all exist. The score sheet now includes one execution-verified `temporal_correctness` case alongside the authored fixture-asserted cases.

**Required change**

Preserve the mixed assessment-mode package and keep the remaining fixture-asserted limitation explicit rather than overclaiming full-suite live execution.

**Acceptance**

- Benchmark assets remain stored under `contracts/fixtures/bench/` and `docs/benchmarks/` with a reproducibility note.
- The score sheet clearly distinguishes fixture-asserted and execution-verified cases.
- The benchmark package does not overclaim full-suite live model execution.

**Limitation**

"Most score sheet verdicts are fixture-asserted, not execution-computed. Live execution currently covers one temporal_correctness case."

**Proof**

- `contracts/fixtures/bench/forge_bench_casebook.json`
- `docs/benchmarks/score_sheet.json`
- `docs/benchmarks/README.md`

**Touch files**

- `docs/benchmarks/README.md`
- `contracts/fixtures/bench/**`
- `forge-pilot or knowledge-runtime benchmark harness`


## ARCH-001 — Finish physical root reduction

- Status: **open**
- Priority: **P1**
- Phase: **Phase 3**
- Area: `archive-cleanup`
- Owners: root docs
- Depends on: DOC-001

**Current state**

The archive manifest already describes logical supersession, and the legacy prompt/apply-plan cluster is physically archived, but the receipt still records physical root reduction as open debt.

**Required change**

Move the remaining superseded root residue under docs/archive/root_closeout_history or delete it, then tighten the archive manifest accordingly.

**Acceptance**

- The active root file set is small, stable, and physically matches the archive manifest.
- No stale numbered matrices or prompts remain in the root decision path.
- The receipt's residual debt list no longer includes physical root reduction.

**Proof**

- `python3 scripts/check_root_archive_manifest.py`
- `release/closeout_receipt_v1.json`

**Touch files**

- `docs/archive/root_closeout_history/manifest.json`
- `docs/archive/root_closeout_history/**`
- `STATUS_DASHBOARD.md`
- `release/closeout_receipt_v1.json`


## HORIZON-001 — Keep V10/V14-V20 backlog out of the finish bar

- Status: **deferred**
- Priority: **P2**
- Phase: **Later**
- Area: `horizon-work`
- Owners: kernel, semantic-memory, knowledge-runtime, spec-execution
- Depends on: DEMO-001, BENCH-001, ARCH-001

**Current state**

The repo contains real horizon material for V10 graph/control semantics, V14/V15 causal and remote exchange, and V16-V20 federated/theory/portfolio/constitutional/spec-execution waves. None of that should be reopened while the finish bar still lacks a public demo and benchmark package.

**Required change**

Document the horizon backlog clearly and forbid using it as an excuse to reopen the closeout lane.

**Acceptance**

- 12_V10_HORIZON_BACKLOG.md exists and is explicitly non-blocking.
- AGENTS.md and PROMPT.md instruct implementers to leave horizon work untouched unless the finish bar is already green.
- No release-facing doc treats horizon work as part of the current closeout claim.

**Proof**

- `12_V10_HORIZON_BACKLOG.md`
- `AGENTS.md`
- `PROMPT.md`

**Touch files**

- `12_V10_HORIZON_BACKLOG.md`
- `AGENTS.md`
- `PROMPT.md`
