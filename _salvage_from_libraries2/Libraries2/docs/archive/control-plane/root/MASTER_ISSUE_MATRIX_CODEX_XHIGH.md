# Master Issue Matrix — Codex XHigh

This matrix is the **active execution truth** for the current library snapshot.

It is based on:
- full-file static audit of the current snapshot,
- the canonical stack spec,
- prior V7 control-plane work,
- and the current code state rather than stale repo folklore.

## Snapshot verdict

- **Architecture:** strong
- **Canonical read/import lane:** mostly real
- **Forge export richness:** materially improved
- **Repo front door:** now present
- **Packaging / release surface:** still uneven
- **Compatibility debt:** still visible in multiple public crates
- **Main present blocker:** packaging + documentation + compatibility cleanup
- **Main remaining architecture work:** lineage / causal productization / observability cleanup

## Status legend

- **Landed** — should be protected, not reopened
- **Mostly landed** — real enough that the job is hardening and truthful docs
- **Partial** — meaningful substrate exists, but the public contract is not done
- **Open** — missing enough that Codex should plan real work here

## Matrix summary

| issue_id | priority | status | area | title |
| --- | --- | --- | --- | --- |
| GOV-201 | P0 | Mostly landed | Repo governance | Root workspace and front-door controls now exist, but the release surface still needs hardening |
| DOC-201 | P0 | Mostly landed | Documentation surface | The repo now has a real root README/AGENTS surface, but crate-doc coverage is still uneven |
| REL-201 | P0 | Landed | Build / release discipline | Root CI and command entrypoints exist, and the supported Tier 0 gate is now green on the current worktree |
| PKG-201 | P1 | Partial | Packaging metadata | Placeholder and missing-readme contradictions were reduced, but metadata hygiene is still inconsistent |
| DOC-202 | P1 | Partial | Crate docs | Crate README coverage has started, but important crates are still under-documented |
| DOC-203 | P2 | Partial | Rustdoc hygiene | The most obvious context-free public crates were documented, but broader rustdoc coverage is still incomplete |
| PKG-202 | P2 | Open | Naming / topology | Path and naming conventions are inconsistent across the ecosystem |
| ARC-101 | P0 | Landed | Architecture | Core authority split is real and should not be reopened |
| ARC-102 | P0 | Mostly landed | Canonical retrieval path | Imported projection retrieval, temporal filters, and scoped routes are now substantially real |
| LIV-201 | P1 | Mostly landed | Forge export semantics | Forge export is materially richer, but some semantics are still metadata-heavy rather than first-class |
| LIN-201 | P1 | Partial | Lineage / derivation | Claim-version lineage is in good shape, but relation lineage and broader derivation semantics remain partial |
| KR-201 | P1 | Partial | Runtime causal surface | Runtime has strong imported retrieval primitives but still lacks a clearly first-class causal answer surface |
| IDN-201 | P2 | Partial | Identity provenance | Alias handling is useful, but identity merge/split provenance is still not an obvious public contract |
| OBS-201 | P1 | Partial | Trace / retry / replay | Canonical trace primitives exist, but queue/graph/pipeline surfaces still carry migration-era legacy fields |
| CMP-201 | P2 | Partial | Compatibility debt | Bridge compatibility helpers are still prominent enough to confuse new integrations |
| CMP-202 | P2 | Partial | Compatibility debt | semantic-memory still carries a large amount of migration surface and old-story narrative |
| CMP-203 | P2 | Partial | Compatibility debt | Execution crates still expose multiple public eras at once |
| API-201 | P2 | Partial | Public surface design | Some crates are thin wrappers or multi-era re-export surfaces rather than clean product surfaces |
| PRIM-201 | P1 | Open | Primitive crates | The primitive suite is promising but under-governed as a public subsystem |
| PAR-201 | P2 | Partial | Parser subsystem | llm-output-parser now has a real README, but hidden-path packaging still obscures it |
| EXT-201 | P2 | Open | Satellite crates | Client / UI / batch crates are real but under-packaged |
| GOV-202 | P1 | Mostly landed | Release topology | The repo now has an explicit crate-tier map and root CI policy, but tier promotion work remains |
| TST-201 | P1 | Partial | Integration proofs | The repo now has a root proof surface, but cross-workspace smoke coverage is still incomplete |
| DOC-204 | P2 | Open | Narrative drift | Some crate docs still describe yesterday’s story rather than today’s architecture |

## Detailed issue cards

### GOV-201 — Root workspace and front-door controls now exist, but the release surface still needs hardening

- **Priority:** P0
- **Status:** Mostly landed
- **Area:** Repo governance
- **Current state:** The repo now has a root Cargo workspace, root README, root AGENTS, root LICENSE, root command entrypoint, and root GitHub Actions workflow. The root workspace intentionally covers Tier 0 only, with other crates explicitly excluded until their packaging and compatibility surfaces are ready for promotion.
- **Evidence:** `Cargo.toml`; `README.md`; `AGENTS.md`; `LICENSE`; `Makefile`; `.github/workflows/ci.yml`; `CODEX_XHIGH_RELEASE_TIER_MAP.md`.
- **Acceptance:** Add one authoritative root workspace and one active control-plane set: root Cargo.toml (or documented multi-workspace topology), root README.md, root AGENTS.md, license, CI/lint configs, and a single release/test entrypoint.
- **Sequencing:** Do first. Everything else is harder to steer until the repo has a real front door.
### DOC-201 — The repo now has a real root README/AGENTS surface, but crate-doc coverage is still uneven

- **Priority:** P0
- **Status:** Mostly landed
- **Area:** Documentation surface
- **Current state:** The repo now has a live root README and root AGENTS file, plus a documented grouped strategy for tiered workspace coverage. Many crate READMEs now exist, the core authority lane has explicit front doors, and the remaining crate-doc work is mostly in non-core and satellite surfaces under DOC-202.
- **Evidence:** `README.md`; `AGENTS.md`; `CODEX_XHIGH_RELEASE_TIER_MAP.md`; `CODEX_XHIGH_CONFORMANCE_AND_TEST_MATRIX.md`; crate READMEs under `AI-Batch-Queue/`, `Tauri-Queue/`, `ComfyUI-RS/`, `LLM-Pipeline/`, `living-memory/living-memory/`, and `Primitives/`.
- **Acceptance:** Add a root README plus either per-crate READMEs or a clearly documented grouped-doc strategy for primitives/satellites. Add AGENTS.md at the root and reference the active matrix/checklist from it.
- **Sequencing:** Alongside GOV-201.
### REL-201 — Root CI and release gates now exist and are green on the current worktree

- **Priority:** P0
- **Status:** Landed
- **Area:** Build / release discipline
- **Current state:** Root automation now exists through a GitHub Actions workflow, a root `Makefile`, and a manifest-truth gate. The supported Tier 0 commands are documented and executable, and the root `make ci` gate is now green on the current worktree.
- **Evidence:** `.github/workflows/ci.yml`; `Makefile`; `scripts/check_manifest_truth.sh`; `CODEX_XHIGH_CONFORMANCE_AND_TEST_MATRIX.md`.
- **Acceptance:** Add CI for fmt/clippy/test, define supported workspaces/crate subsets, and document the release/test matrix. CI must fail on packaging/doc contradictions and public-surface regressions.
- **Sequencing:** Do with GOV-201 so later work has an executable gate.
### PKG-201 — Manifest metadata hygiene is inconsistent across the 24 audited Cargo manifests

- **Priority:** P1
- **Status:** Partial
- **Area:** Packaging metadata
- **Current state:** Placeholder repository URLs were removed from the satellite queue crates, manifests now point at real sibling READMEs across the core lane and audited package-scoped crates, and Tier 3/internal surfaces are now explicitly fenced with `publish = false`. The remaining gap is broader repository/homepage consistency for crates where the monorepo still lacks one canonical external remote.
- **Evidence:** `stack-ids/Cargo.toml`; `semantic-memory-forge/Cargo.toml`; `forge-memory-bridge/Cargo.toml`; `job-queue/Cargo.toml`; `agent-graph/Cargo.toml`; `LLM-Pipeline/Cargo.toml`; `ComfyUI-RS/Cargo.toml`; `Ollama-Vision-RS/Cargo.toml`; `Primitives/*/Cargo.toml`; `demo-tauri-libraries/src-tauri/Cargo.toml`; `scripts/check_manifest_truth.sh`.
- **Acceptance:** Every public-facing crate has accurate readme/repository/homepage metadata or is explicitly documented as private/internal. No placeholder URLs remain.
- **Sequencing:** Immediately after the root front door exists.
### DOC-202 — Crate README coverage has started, but important crates are still under-documented

- **Priority:** P1
- **Status:** Partial
- **Area:** Crate docs
- **Current state:** The repo now has front-door READMEs for the core authority lane, including `stack-ids`, `semantic-memory-forge`, `forge-memory-bridge`, `semantic-memory`, `knowledge-runtime`, and `forge-engine`, plus grouped primitive coverage. The remaining gap is thinner coverage in some non-core and satellite crates rather than the core lane lacking front doors.
- **Evidence:** `stack-ids/README.md`; `semantic-memory-forge/README.md`; `forge-memory-bridge/README.md`; `semantic-memory/README.md`; `knowledge-runtime/README.md`; `living-memory/living-memory/README.md`; `Primitives/README.md`.
- **Acceptance:** Every crate gets either its own README or inclusion in a grouped document with purpose, authority, status, quick start, and compatibility notes.
- **Sequencing:** After PKG-201, because manifests should point to real docs.
### DOC-203 — Several public crates still lack crate-level rustdoc authority and usage framing

- **Priority:** P2
- **Status:** Partial
- **Area:** Rustdoc hygiene
- **Current state:** `forge-engine` and the audited primitive crates now have crate-level rustdoc authority/purpose framing, so the most obvious context-free public roots are no longer blank. The remaining gap is broader rustdoc consistency across the rest of the ecosystem.
- **Evidence:** `living-memory/living-memory/src/lib.rs`; `Primitives/check-runner/src/lib.rs`; `Primitives/cea-store/src/lib.rs`; `Primitives/stabilizer-core/src/lib.rs`; `Primitives/cea-sqlite/src/lib.rs`; `Primitives/effect-signature/src/lib.rs`; `Primitives/forge-policy/src/lib.rs`; `Primitives/mindstate-core/src/lib.rs`; `Primitives/sandbox-workspace/src/lib.rs`; `Primitives/typed-patch/src/lib.rs`.
- **Acceptance:** Every public crate root starts with an authority/purpose/status block. Primitives may be documented individually or via a grouped primitive policy page, but the crates themselves must no longer be context-free.
- **Sequencing:** After DOC-202.
### PKG-202 — Path and naming conventions are inconsistent across the ecosystem

- **Priority:** P2
- **Status:** Partial
- **Area:** Naming / topology
- **Current state:** The snapshot still mixes hidden dirs, PascalCase crate dirs, nested package dirs, and package-name/path mismatches, but the stable exceptions are now documented explicitly instead of being left as unexplained topology drift.
- **Evidence:** `CODEX_XHIGH_RELEASE_TIER_MAP.md`; hidden parser crate at `.parser-lib`; nested forge engine at `living-memory/living-memory`; mixed-case directories such as `LLM-Pipeline`, `AI-Batch-Queue`, `Tauri-Queue`, `ComfyUI-RS`, and `Ollama-Vision-RS`.
- **Acceptance:** Adopt one naming policy. Either normalize directories or document stable exceptions with workspace aliases and publishing guidance.
- **Sequencing:** After root workspace/package strategy is decided.
### ARC-101 — Core authority split is real and should not be reopened

- **Priority:** P0
- **Status:** Landed
- **Area:** Architecture
- **Current state:** The core lane is coherently partitioned: stack-ids for IDs/trace/scope/digest; semantic-memory-forge for export wire truth; forge-memory-bridge for transform only; semantic-memory for durable projections; knowledge-runtime for planning/merge; forge-engine for operational verification.
- **Evidence:** stack-ids/src/lib.rs; semantic-memory-forge/src/lib.rs; forge-memory-bridge/src/lib.rs; semantic-memory/src/lib.rs; knowledge-runtime/src/lib.rs; living-memory/living-memory/src/export.rs.
- **Acceptance:** Keep this closed. No future work may move source truth into runtime or semantics invention into the bridge.
- **Sequencing:** Guardrail, not a todo.
### ARC-102 — Imported projection retrieval, temporal filters, and scoped routes are now substantially real

- **Priority:** P0
- **Status:** Mostly landed
- **Area:** Canonical retrieval path
- **Current state:** semantic-memory exposes projection query APIs and knowledge-runtime consumes them on supported routes, including explicit as_of(valid_t, recorded_t_or_before) helpers and scope-aware projection retrieval.
- **Evidence:** semantic-memory/src/lib.rs query_claim_versions/query_relation_versions/query_episodes/query_entity_aliases/query_evidence_refs; semantic-memory/src/projection_storage.rs; knowledge-runtime/src/adapters/semantic_memory.rs; knowledge-runtime/src/runtime.rs.
- **Acceptance:** Retain proof tests and broaden only where the public contract claims broader coverage than currently implemented.
- **Sequencing:** Do not reopen; only harden and document truthfully.
### LIV-201 — Forge export is materially richer, but some semantics are still metadata-heavy rather than first-class

- **Priority:** P1
- **Status:** Mostly landed
- **Area:** Forge export semantics
- **Current state:** Export now emits claims, relations, aliases, episodes, and evidence refs, and carries trials/refutations/covariates/threats in episode metadata. This is much better than the earlier thin-envelope state, but some causal semantics still live inside JSON blobs.
- **Evidence:** living-memory/living-memory/src/export.rs; living-memory/living-memory/tests/export_tests.rs; semantic-memory-forge/src/envelope.rs; semantic-memory-forge/src/bundle.rs.
- **Acceptance:** Promote the semantically critical causal/verification fields from opaque metadata into explicit first-class export structures where doing so reduces downstream guesswork and schema drift.
- **Sequencing:** After repo/docs surface is fixed, because this is the next real architecture-quality lever.
### LIN-201 — Claim-version lineage is in good shape, but relation lineage and broader derivation semantics remain partial

- **Priority:** P1
- **Status:** Partial
- **Area:** Lineage / derivation
- **Current state:** Claim supersession lineage is preserved when the source knows it. Relation lineage fields exist in the bridge/memory model, but the current Forge exporter still defaults relation supersession to None for emitted relation records. Broader derivation/invalidation semantics are not yet clearly productized.
- **Evidence:** forge-memory-bridge/src/transform.rs; forge-memory-bridge/tests/forge_bridge_memory_proof.rs; living-memory/living-memory/src/export.rs; semantic-memory/src/projection_storage.rs.
- **Acceptance:** Emit real relation supersession when known, define derivation/invalidation expectations explicitly, and leave behind tests proving bounded recomputation semantics across imported claims/relations/episodes/evidence.
- **Sequencing:** Immediately after LIV-201.
### KR-201 — Runtime has strong imported retrieval primitives but still lacks a clearly first-class causal answer surface

- **Priority:** P1
- **Status:** Partial
- **Area:** Runtime causal surface
- **Current state:** knowledge-runtime can query imported claims/relations/episodes and can fetch evidence refs by claim and as-of bounds, but the public narrative still centers on general retrieval rather than an explicit causal query lane backed by imported projections.
- **Evidence:** knowledge-runtime/src/runtime.rs; knowledge-runtime/src/adapters/semantic_memory.rs; knowledge-runtime/src/evidence/support.rs.
- **Acceptance:** Define and test at least one first-class causal query path that answers from imported causal projections with explicit provenance and audit-only evidence dereference.
- **Sequencing:** After LIV-201 and LIN-201.
### IDN-201 — Alias handling is useful, but identity merge/split provenance is still not an obvious public contract

- **Priority:** P2
- **Status:** Partial
- **Area:** Identity provenance
- **Current state:** The data model already contains alias confidence and merge/split provenance fields, and runtime does bounded alias candidate expansion. The missing piece is a clear end-to-end contract and documentation for replayable identity decisions.
- **Evidence:** semantic-memory/src/types.rs; semantic-memory/src/projection_storage.rs; knowledge-runtime/src/adapters/semantic_memory.rs; knowledge-runtime/src/runtime.rs.
- **Acceptance:** Document and test how alias, merge, split, supersession, and review state flow through import, storage, and runtime consumption.
- **Sequencing:** After KR-201 unless identity work is a gating use case sooner.
### OBS-201 — Canonical trace primitives exist, but queue/graph/pipeline surfaces still carry migration-era legacy fields

- **Priority:** P1
- **Status:** Partial
- **Area:** Trace / retry / replay
- **Current state:** stack-ids provides TraceCtx and legacy conversion. job-queue, agent-graph, tauri-queue, and llm-pipeline still expose compatibility-era trace_id and retry fields in public surfaces.
- **Evidence:** stack-ids/src/trace.rs; job-queue/src/events.rs; agent-graph/src/event_sink.rs; Tauri-Queue/src/lib.rs; LLM-Pipeline/src/lib.rs; LLM-Pipeline/src/exec_ctx.rs; LLM-Pipeline/src/trace.rs.
- **Acceptance:** Define one canonical public observability surface built on TraceCtx/AttemptId/TrialId, keep legacy fields fenced, and leave behind an explicit removal/migration plan.
- **Sequencing:** After root repo/docs work, because this is the biggest execution-surface cleanup theme.
### CMP-201 — Bridge compatibility helpers are still prominent enough to confuse new integrations

- **Priority:** P2
- **Status:** Partial
- **Area:** Compatibility debt
- **Current state:** forge-memory-bridge keeps a legacy module plus deprecated hidden re-exports for the old envelope path.
- **Evidence:** forge-memory-bridge/src/lib.rs; forge-memory-bridge/src/legacy.rs.
- **Acceptance:** Fence bridge compatibility paths more aggressively: feature gate them, isolate them behind clearer names, and remove them from the center of active docs/examples.
- **Sequencing:** After the current docs/install pass so the canonical path is taught first.
### CMP-202 — semantic-memory still carries a large amount of migration surface and old-story narrative

- **Priority:** P2
- **Status:** Partial
- **Area:** Compatibility debt
- **Current state:** semantic-memory keeps deprecated projection_import, compat modules, JSON compat import paths, and still opens with an add_fact()-centric quick start that under-describes the canonical import lane.
- **Evidence:** semantic-memory/src/lib.rs; semantic-memory/src/projection_storage.rs.
- **Acceptance:** Make the canonical batch import and projection-query story the default narrative. Fence legacy JSON/V10 import surfaces as compatibility-only and stop teaching them in front-door docs.
- **Sequencing:** With DOC-204 and after root docs exist.
### CMP-203 — Execution crates still expose multiple public eras at once

- **Priority:** P2
- **Status:** Partial
- **Area:** Compatibility debt
- **Current state:** agent-graph, job-queue, tauri-queue, and llm-pipeline all preserve migration-era public fields or entire legacy APIs. This is reasonable during migration but creates a blurry public contract.
- **Evidence:** agent-graph/src/event_sink.rs; job-queue/src/events.rs; Tauri-Queue/src/lib.rs; LLM-Pipeline/src/lib.rs.
- **Acceptance:** Choose and document the primary public API era for each crate. Legacy surfaces remain only if clearly fenced and backed by a removal condition.
- **Sequencing:** After OBS-201.
### API-201 — Some crates are thin wrappers or multi-era re-export surfaces rather than clean product surfaces

- **Priority:** P2
- **Status:** Partial
- **Area:** Public surface design
- **Current state:** tauri-queue largely re-exports job-queue, and llm-pipeline intentionally exposes both the new payload API and the original pipeline API. This works, but it increases ambiguity about what is actually primary.
- **Evidence:** Tauri-Queue/src/lib.rs; LLM-Pipeline/src/lib.rs.
- **Acceptance:** Mark the primary surface explicitly in docs and examples, and reduce duplicate entrypoints where they no longer earn their keep.
- **Sequencing:** After CMP-203.
### PRIM-201 — The primitive suite is promising but under-governed as a public subsystem

- **Priority:** P1
- **Status:** Open
- **Area:** Primitive crates
- **Current state:** Several primitive crates have no crate-level rustdoc, no README metadata, no repository metadata, and zero test annotations in the audited snapshot.
- **Evidence:** Primitives/check-runner, effect-signature, mindstate-core, sandbox-workspace, stabilizer-core, cea-store, cea-sqlite all lack readme/repository metadata; several also show zero test annotations.
- **Acceptance:** Decide which primitives are public, which are internal-only, and document/test them accordingly. Public primitives need docs, smoke tests, and a release story. Internal ones should be grouped and marked internal.
- **Sequencing:** After GOV-201/PKG-201, before any public release claims.
### PAR-201 — llm-output-parser now has a real front door, but hidden-path packaging still obscures it

- **Priority:** P2
- **Status:** Partial
- **Area:** Parser subsystem
- **Current state:** The parser crate now has a real README that explains its role in `llm-pipeline` and `ollama-vision`, but it still lives in the hidden `.parser-lib` path and still lacks repository metadata.
- **Evidence:** `.parser-lib/Cargo.toml`; `.parser-lib/README.md`; `LLM-Pipeline/Cargo.toml`; `Ollama-Vision-RS/Cargo.toml`; `.parser-lib/src/lib.rs`.
- **Acceptance:** Give the parser a canonical path/name, real README, and an explicit relationship to llm-pipeline and ollama-vision.
- **Sequencing:** After PKG-202.
### EXT-201 — Client / UI / batch crates are real but under-packaged

- **Priority:** P2
- **Status:** Open
- **Area:** Satellite crates
- **Current state:** ComfyUI-RS, Ollama-Vision-RS, AI-Batch-Queue, and Tauri-Queue still need clearer release stance and more consistent metadata, but the placeholder repository URLs in the two queue crates are gone.
- **Evidence:** `ComfyUI-RS/Cargo.toml`; `Ollama-Vision-RS/Cargo.toml`; `AI-Batch-Queue/Cargo.toml`; `Tauri-Queue/Cargo.toml`; `AI-Batch-Queue/README.md`; `Tauri-Queue/README.md`.
- **Acceptance:** Decide whether each satellite is public, experimental, or internal. Then make the manifests/docs/tests match that decision.
- **Sequencing:** After core repo/package governance is fixed.
### GOV-202 — The repo now has an explicit crate-tier map and root CI policy, but tier promotion work remains

- **Priority:** P1
- **Status:** Mostly landed
- **Area:** Release topology
- **Current state:** The repo now has an explicit tier map, a documented root workspace scope, and a root CI policy that ties Tier 0 support to the root gate. What remains is promoting or fencing the other tiers crate-by-crate until the release story matches the map.
- **Evidence:** `CODEX_XHIGH_RELEASE_TIER_MAP.md`; `README.md`; `Cargo.toml`; `.github/workflows/ci.yml`; `Makefile`.
- **Acceptance:** Publish a crate map with tiers (core / execution / primitives / satellites / internal) and tie CI, docs, and publishing rules to those tiers.
- **Sequencing:** Alongside GOV-201 and REL-201.
### TST-201 — The repo now has a root proof surface, but cross-workspace smoke coverage is still incomplete

- **Priority:** P1
- **Status:** Partial
- **Area:** Integration proofs
- **Current state:** The repo now has a root workspace, a root conformance/test matrix, and a root `test-core` entrypoint. What remains is a broader, explicit smoke path across parser, pipeline, queue/graph, forge, memory, and runtime for the crates that are still package-scoped.
- **Evidence:** `Cargo.toml`; `Makefile`; `CODEX_XHIGH_CONFORMANCE_AND_TEST_MATRIX.md`; `knowledge-runtime/tests/cross_crate_proof.rs`; `forge-memory-bridge/tests/forge_bridge_memory_proof.rs`.
- **Acceptance:** Add a root-level conformance/test matrix and at least a minimal set of cross-crate smoke proofs spanning parser -> pipeline -> queue/graph -> forge -> memory -> runtime where applicable.
- **Sequencing:** After REL-201.
### DOC-204 — Some crate docs still describe yesterday’s story rather than today’s architecture

- **Priority:** P2
- **Status:** Open
- **Area:** Narrative drift
- **Current state:** semantic-memory still opens with a store-a-fact quick start even though the broader stack now has a mature canonical import/query lane; forge-engine exports a large public surface with no crate-level authority statement.
- **Evidence:** semantic-memory/src/lib.rs; living-memory/living-memory/src/lib.rs.
- **Acceptance:** Update crate docs so the public narrative matches the actual architecture and compatibility boundaries in the current code.
- **Sequencing:** After GOV-201/DOC-201 so the front door can point to the right stories.

## Blunt summary

Do **not** let Codex waste time re-inventing the authority split.  
That part is already the best part of the repo.

The current highest-leverage work order is:

1. root workspace / root docs / release gates,
2. manifest and crate-doc cleanup,
3. relation lineage + causal runtime surface,
4. trace/retry/API canonicalization,
5. compatibility burn-down,
6. primitives + satellites brought under one explicit release policy.
