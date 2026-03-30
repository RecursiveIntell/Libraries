# 04_MASTER_ISSUE_MATRIX

## Summary

| phase | priority | stream | id | status | gate | title |
|---|---|---|---|---|---|---|
| Phase 0 — release truth and front door | P0 | Packaging truth | PACK-001 | open | Ship blocker | Make the front door real: `make gate` must work from a clean checkout |
| Phase 0 — release truth and front door | P0 | Archive truth | PACK-002 | open | Ship blocker | Fix the root archive manifest and the missing archived file count |
| Phase 0 — release truth and front door | P0 | Status truth | PACK-003 | open | Ship blocker | Reconcile STATUS_DASHBOARD, evidence manifest, and closeout claims with current reality |
| Phase 0 — release truth and front door | P0 | Canonical docs | SPEC-001 | open | Ship blocker | Stop using root-level stub specs as if they are canonical law |
| Phase 0 — release truth and front door | P0 | Support lane | GATE-001 | open | Ship blocker | Make the support claim, workspace scope, and gate scope the same story |
| Phase 0 — architecture truth | P0 | Runtime provenance | RUNTIME-001 | open | Correctness blocker | Propagate scheduler degradation reasons into runtime-facing artifacts |
| Phase 1 — artifact convergence | P1 | Execution evidence | EXEC-001 | open | Conformance blocker | Finish execution-evidence convergence across llm-tool-runtime, forge-pilot, and verification crates |
| Phase 1 — artifact convergence | P1 | Shared primitives | TYPE-001 | open | Cleanliness / drift blocker | Centralize duplicated `SurfaceStatus` into one canonical primitive |
| Phase 1 — credibility and naming | P1 | Governance surface | NAME-001 | open | Credibility blocker | Either deepen the thin governance “runtime” crates or rename them honestly |
| Phase 1 — docs and teachability | P1 | API docs | DOC-001 | open | Docs blocker | Document the governance/kernel crates that currently have near-zero API docs |
| Phase 1 — safety and maintainability | P1 | Panic surface | SAFE-001 | open | Reliability blocker | Reduce supported-lane `.unwrap()` hotspots or move them behind typed fallible APIs |
| Phase 1 — safety and maintainability | P1 | Module shape | MOD-001 | open | Maintainability blocker | Split the oversized modules that keep reappearing in audits |
| Phase 2 — package hygiene and polish | P2 | Release hygiene | PACK-004 | open | Package polish | Stop shipping `target-*` trees inside anything called `source-clean` |
| Phase 2 — package hygiene and polish | P2 | Root clutter | DOC-002 | open | Package polish | Reduce the active root/meta clutter once one front-door pack is canonical |
| Phase 2 — package hygiene and polish | P2 | Release bar | REL-001 | open | Package polish | Turn the benchmark/demo story from fixture theater into one honest live proof lane |

## Detailed rows

### PACK-001 — Make the front door real: `make gate` must work from a clean checkout

**Priority:** P0  
**Stream:** Packaging truth  
**Gate:** Ship blocker  
**Depends on:** —

**Why this exists:**
The advertised front-door command fails immediately because `scripts/check_pack_truth.sh` requires a numbered pack that is not present at the repo root. That makes the release surface non-reproducible.

**Acceptance:**
`bash scripts/check_pack_truth.sh` passes from a clean checkout and `make gate` reaches the intended cargo lane instead of dying in pack truth.

**Required proof:**
Passing `bash scripts/check_pack_truth.sh`; updated README / PACK_README / release checklist; no orphan front-door story.

**Primary surface:**
scripts/check_pack_truth.sh + root pack docs

**Note:**
Do not keep two competing root pack conventions.

### PACK-002 — Fix the root archive manifest and the missing archived file count

**Priority:** P0  
**Stream:** Archive truth  
**Gate:** Ship blocker  
**Depends on:** PACK-001

**Why this exists:**
`python3 scripts/check_root_archive_manifest.py` currently fails because `legacy_root_residue` is declared as 30 files while only 29 exist; the archived `04_MASTER_ISSUE_MATRIX.csv` is missing or the manifest is wrong.

**Acceptance:**
`python3 scripts/check_root_archive_manifest.py` passes and the archive directory / manifest / active pack all agree.

**Required proof:**
Passing root archive manifest check; manifest diff; either restored file or corrected count with truthful rationale.

**Primary surface:**
docs/archive/root_closeout_history/manifest.json + archived root residue

**Note:**
This is a direct filesystem-vs-manifest contradiction, not an interpretation dispute.

### PACK-003 — Reconcile STATUS_DASHBOARD, evidence manifest, and closeout claims with current reality

**Priority:** P0  
**Stream:** Status truth  
**Gate:** Ship blocker  
**Depends on:** PACK-002

**Why this exists:**
The dashboard says the hardening gates are green, including root archive manifest, but the current repo state disproves that. Evidence files and front-door summaries have drifted apart.

**Acceptance:**
STATUS_DASHBOARD, STATUS_EVIDENCE_MANIFEST, and release receipt either all match the current state or explicitly declare a historical snapshot that is no longer reproducible from HEAD.

**Required proof:**
Human-readable diff plus passing spot-check commands mentioned in the dashboard.

**Primary surface:**
STATUS_DASHBOARD.md, STATUS_EVIDENCE_MANIFEST.json, release/closeout_receipt_v1.json

**Note:**
Stop claiming current green if the claim is only historically true.

### SPEC-001 — Stop using root-level stub specs as if they are canonical law

**Priority:** P0  
**Stream:** Canonical docs  
**Gate:** Ship blocker  
**Depends on:** PACK-001

**Why this exists:**
The root `CANONICAL_STACK_SPEC_V6.md` and `CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md` are 470- and 512-byte excerpts that only satisfy string-matching doc checks. That is governance theater, not spec publication.

**Acceptance:**
Either restore the real canonical documents at the root, or rename the stubs as excerpts / compatibility notes and update doc-truth to stop pretending they are canonical specs.

**Required proof:**
Updated files plus a strengthened `scripts/check_doc_truth.sh` that checks identity and purpose, not just string presence.

**Primary surface:**
CANONICAL_STACK_SPEC_V6.md, CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md, scripts/check_doc_truth.sh

**Note:**
The current state incentivizes tiny compliant placeholders.

### GATE-001 — Make the support claim, workspace scope, and gate scope the same story

**Priority:** P0  
**Stream:** Support lane  
**Gate:** Ship blocker  
**Depends on:** PACK-001

**Why this exists:**
`SUPPORT_PROFILE.md` claims a 17-crate closeout lane, the workspace has 30 members / 29 default members, and `Makefile` runs `cargo test --workspace --exclude forge-engine`. Those are three different surfaces.

**Acceptance:**
One explicit release lane is named in `SUPPORT_PROFILE.md`, `Makefile`, the receipt, and the dashboard; cargo commands target that same lane; adjacent crates are clearly demoted to non-certified status.

**Required proof:**
One passing release command sequence plus receipt regeneration / support-profile hash update.

**Primary surface:**
SUPPORT_PROFILE.md, Cargo.toml, Makefile, release/closeout_receipt_v1.json, STATUS_DASHBOARD.md

**Note:**
A support claim that is narrower than the gate is how release truth rots.

### RUNTIME-001 — Propagate scheduler degradation reasons into runtime-facing artifacts

**Priority:** P0  
**Stream:** Runtime provenance  
**Gate:** Correctness blocker  
**Depends on:** GATE-001

**Why this exists:**
`kernel_execution::ScheduledExecution` sets `degraded_reason` for real cases like `budget_exhausted` and `explicit_changed_nodes_required_for_delta`, but `knowledge-runtime` drops that reason in advisory/explanation/risk-gate outputs.

**Acceptance:**
InferenceAdvisory, InferenceExplanation, risk-gate outputs, and schema generation all surface the precise degradation reason and preserve it through query provenance.

**Required proof:**
New schema fields, runtime tests, and one cross-crate fixture proving the reason survives end-to-end.

**Primary surface:**
kernel-execution/src/lib.rs, knowledge-runtime/src/inference.rs, knowledge-runtime/src/obs/trace.rs, contract-schema-gen/src/lib.rs

**Note:**
Right now the scheduler knows why it degraded and the user-facing runtime does not.

### EXEC-001 — Finish execution-evidence convergence across llm-tool-runtime, forge-pilot, and verification crates

**Priority:** P1  
**Stream:** Execution evidence  
**Gate:** Conformance blocker  
**Depends on:** RUNTIME-001

**Why this exists:**
`ExecutionContextV1` and `EpisodeBundleV1` exist, but the control/orchestration/tool seams still look partly parallel rather than fully canonical. The stack is close; the danger is staying close forever.

**Acceptance:**
Tool receipts, pilot loop artifacts, and verification/control artifacts all carry or strongly reference the same execution-evidence family with one schema owner and one taught mental model.

**Required proof:**
Schema registry entries, round-trip serde fixtures, and one cross-crate lineage fixture spanning tool runtime -> pilot -> verification/control.

**Primary surface:**
semantic-memory-forge/src/v9.rs, llm-tool-runtime/src/contracts.rs, llm-tool-runtime/src/runtime.rs, forge-pilot/src/*, verification-*/src/*, contract-schema-gen/src/lib.rs

**Note:**
This is the difference between telemetry and admissible execution evidence.

### TYPE-001 — Centralize duplicated `SurfaceStatus` into one canonical primitive

**Priority:** P1  
**Stream:** Shared primitives  
**Gate:** Cleanliness / drift blocker  
**Depends on:** SPEC-001

**Why this exists:**
`SurfaceStatus` is redefined in five crates (`spec-execution`, `mechanism-runtime`, `federated-settlement`, `discovery-portfolio`, `constitutional-memory`). That is textbook drift bait.

**Acceptance:**
One canonical shared definition exists in a primitive crate and all leaf crates reuse it.

**Required proof:**
Greppable removal of duplicate enum definitions; schema regen and compatibility proof.

**Primary surface:**
stack-ids/src/* or another primitive crate; affected leaf crates

**Note:**
This is small, obvious, and should already be done.

### NAME-001 — Either deepen the thin governance “runtime” crates or rename them honestly

**Priority:** P1  
**Stream:** Governance surface  
**Gate:** Credibility blocker  
**Depends on:** TYPE-001

**Why this exists:**
Several governance leaf crates are still tiny schema carriers with runtime names (`mechanism-runtime`, `discovery-portfolio`, `constitutional-memory`, `federated-settlement`, `spec-execution`, and the small post-v20 surfaces). The names overpromise the behavior.

**Acceptance:**
For each thin crate, either (a) add runtime logic that justifies the name, or (b) rename / reposition it as schema/types/profile surface in both Cargo metadata and docs.

**Required proof:**
Crate-by-crate decision table, updated README/Cargo descriptions, and no remaining misleading top-level crate names.

**Primary surface:**
Affected crate Cargo.toml + README + src/lib.rs surfaces

**Note:**
DARPA-grade reviewers will open these crates. Pretending they will not is fantasy.

### DOC-001 — Document the governance/kernel crates that currently have near-zero API docs

**Priority:** P1  
**Stream:** API docs  
**Gate:** Docs blocker  
**Depends on:** NAME-001

**Why this exists:**
`forge-pilot`, `kernel-conformance`, `llm-tool-runtime`, `kernel-execution`, `contract-schema-gen`, and `kernel-oracles` have essentially no public API documentation despite carrying differentiating logic or interfaces.

**Acceptance:**
Public-item rustdoc coverage becomes intentional for the thin-doc crates, not just for the already-well-documented ones.

**Required proof:**
Expanded rustdoc counts plus passing public API docs check against the intended crate set.

**Primary surface:**
forge-pilot/src/**, kernel-conformance/src/**, llm-tool-runtime/src/**, kernel-execution/src/lib.rs, contract-schema-gen/src/**, kernel-oracles/src/lib.rs

**Note:**
Right now the repo documents the storage core far better than the governance edge it keeps selling.

### SAFE-001 — Reduce supported-lane `.unwrap()` hotspots or move them behind typed fallible APIs

**Priority:** P1  
**Stream:** Panic surface  
**Gate:** Reliability blocker  
**Depends on:** DOC-001

**Why this exists:**
Supported-lane production code still carries notable `.unwrap()` density in `semantic-memory-forge`, `stack-ids`, `forge-memory-bridge`, `knowledge-runtime`, `kernel-conformance`, and `forge-pilot`. Meanwhile the repo’s “production panic guard” only checks two `contract-schema-gen` files.

**Acceptance:**
Either remove / gate the hotspot unwraps on the closeout lane or explicitly scope the panic guard and stop implying repo-wide coverage.

**Required proof:**
Targeted grep / audit report for supported-lane unwraps, widened panic guard or renamed check, and regression tests for replaced fallible paths.

**Primary surface:**
semantic-memory-forge/src/envelope.rs, semantic-memory-forge/src/v11.rs, stack-ids/src/ids.rs, forge-memory-bridge/src/transform.rs, knowledge-runtime/src/**, kernel-conformance/src/**, forge-pilot/src/main_support/mod.rs, scripts/check_no_prod_panics.sh

**Note:**
A narrow guard with a broad name is worse than no guard.

### MOD-001 — Split the oversized modules that keep reappearing in audits

**Priority:** P1  
**Stream:** Module shape  
**Gate:** Maintainability blocker  
**Depends on:** DOC-001

**Why this exists:**
Large files remain concentrated in the exact places where the repo is semantically important: `semantic-memory-forge/src/envelope.rs`, `living-memory/.../evidence.rs`, `semantic-memory/src/projection_storage.rs`, `stack-ids/src/ids.rs`, `verification-control/src/lib.rs`, `forge-pilot/src/main_support/mod.rs`, `forge-pilot/src/loop_runner.rs`, `LLM-Pipeline/src/llm_call.rs`, `forge-memory-bridge/src/transform.rs`, and `constraint-compiler/src/lib.rs`.

**Acceptance:**
Each hotspot is split by responsibility with tests preserved and public surface unchanged or improved.

**Required proof:**
Before/after file-size report plus unchanged conformance/tests.

**Primary surface:**
Hotspot files above

**Note:**
These files are where architecture turns into hand-maintained sediment.

### PACK-004 — Stop shipping `target-*` trees inside anything called `source-clean`

**Priority:** P2  
**Stream:** Release hygiene  
**Gate:** Package polish  
**Depends on:** PACK-003

**Why this exists:**
The archive contains multiple build artifact directories (`target-closeout-check`, `target-closeout-verify`, `target-full`, `target-serial`, `target-serial2`, `target-vc`). The label “source-clean” is therefore false on its face.

**Acceptance:**
Release packaging excludes build outputs or the package is renamed to reflect what it actually is.

**Required proof:**
Clean archive listing with no `target-*` roots.

**Primary surface:**
Packaging pipeline / archive manifest / release docs

**Note:**
Small issue, loud signal.

### DOC-002 — Reduce the active root/meta clutter once one front-door pack is canonical

**Priority:** P2  
**Stream:** Root clutter  
**Gate:** Package polish  
**Depends on:** PACK-001

**Why this exists:**
The root currently carries 32 top-level doc/json/txt files and ~465 non-archive meta files across docs/plans/prompts/scaffolds/reference/snippets/schemas/scripts. The repo is in danger of teaching planning instead of product.

**Acceptance:**
One front-door pack is canonical, duplicate or superseded meta files are archived or demoted, and examples teach the canonical lane first.

**Required proof:**
Updated archive manifest, root file inventory, and grep proof that docs/examples teach the canonical story.

**Primary surface:**
Root docs + archive manifest + examples / walkthroughs

**Note:**
A compatibility lane can exist without owning the reader’s brain.

### REL-001 — Turn the benchmark/demo story from fixture theater into one honest live proof lane

**Priority:** P2  
**Stream:** Release bar  
**Gate:** Package polish  
**Depends on:** EXEC-001

**Why this exists:**
The repo already has benchmark/demo infrastructure, but the strongest remaining credibility move is one replayable live proof path that demonstrates the canonical artifact story without relying on authored verdicts.

**Acceptance:**
At least one live benchmark/demo case is execution-verified end-to-end and clearly separated from fixture-asserted cases.

**Required proof:**
A reproducible command, generated receipt(s), and a dashboard that distinguishes fixture-asserted from execution-verified lanes.

**Primary surface:**
docs/benchmarks/*, demo fixtures, verification-control/tests, release docs

**Note:**
One real proof lane beats five narrated ones.
