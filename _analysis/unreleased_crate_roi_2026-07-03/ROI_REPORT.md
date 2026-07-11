# RecursiveIntell Libraries unreleased crate ROI audit

Date: 2026-07-03
Root: /home/sikmindz/Coding/Libraries
Scope: the 39 workspace packages that were not present on crates.io during the live crates.io API check.

## Receipts

Commands run from /home/sikmindz/Coding/Libraries:

- cargo metadata --no-deps --format-version 1
- crates.io API check for every workspace package
- cargo check --workspace --all-targets --quiet
- cargo test --workspace --all-targets --quiet

Observed results:

- 64 top-level workspace packages inventoried by cargo metadata.
- 25 packages already published on crates.io.
- 39 packages appear unpublished.
- cargo check --workspace --all-targets --quiet: passed.
- cargo test --workspace --all-targets --quiet: passed.
- Warnings only: hnsw-bench has unused imports and unused helper functions.
- No unreleased crate README had an obvious visual/diagram signal. That is a public-packaging gap across the whole set.

Generated machine-readable artifacts:

- /home/sikmindz/Coding/Libraries/_analysis/unreleased_crate_roi_2026-07-03/unreleased_inventory.json
- /home/sikmindz/Coding/Libraries/_analysis/unreleased_crate_roi_2026-07-03/unreleased_inventory.csv
- /home/sikmindz/Coding/Libraries/_analysis/unreleased_crate_roi_2026-07-03/INVENTORY.md

## Executive answer

Your first instinct was closer than my first answer. Several of these are not random leftovers. A lot of them are already integrated, tested, and sitting behind weak public presentation.

The highest-ROI move is not “release every crate as a standalone repo.”

The highest-ROI move is:

1. Publish the coherent, already-integrated crates to crates.io after dry-run packaging fixes.
2. Split only a few public-facing families into standalone repos.
3. Build one stack map repo that explains how the crates fit together.
4. Rework READMEs with diagrams, concrete examples, and “what this is not” boundaries.

The unrealized ROI is real. The main bottleneck is positioning and docs, not code existence.

## Product surfaces discovered

### Surface A: Forge / causal edit attribution / patch verification

This is probably the sleeper high-ROI surface.

Crates:
- forge-engine
- forge-pilot
- cea-core
- cea-store
- cea-sqlite
- typed-patch
- check-runner
- check-runner-sys
- sandbox-workspace
- forge-policy
- stabilizer-core
- mindstate-core
- effect-signature

Actual meaning:
A local-first agent patch evaluation system: compile mindstate, apply structured patches, run checks, normalize effects, attribute failures/successes causally, and persist a graph for future risk prediction.

Why this matters:
This is much stronger than “agent framework.” It is closer to “agent change-control and evidence runtime.” That is aligned with your operator-grade/no-shadow-truth doctrine and with current needs in Hermes/AiDENs/agent-memory work.

ROI: 9/10 if positioned as Forge: causal edit attribution and receipt-backed patch verification.

Do not lead with the abstract doctrine names. Lead with:
“Know which agent edit caused which check result.”

### Surface B: Verification / claims / receipt governance

Crates:
- verification-control
- verification-policy
- verification-calibration
- verification-adjudication
- kernel-conformance
- receipt-bench
- contract-schema-gen
- constitutional-memory
- mechanism-runtime
- spec-execution
- discovery-portfolio
- federated-settlement
- remote-oracle-admission

Actual meaning:
Typed artifacts for verification cases, policies, calibration/abstention, adjudication, conformance gates, schemas, proof obligations, receipts, and controlled admission of remote/oracle evidence.

Why this matters:
This is the most “RecursiveIntell” part: bitemporal, claim-ledger, verification, receipts, replayability, and governance boundaries. It is not all equally public-ready, but it is coherent.

ROI: 8.5/10 if packaged as a verification kit. 5/10 if each is released as an isolated crate with abstract READMEs.

### Surface C: Profile / governance / bounded surface crates

Crates:
- profile-runtime
- assurance-runtime
- authority-delegation
- attestation-exchange
- continuity-runtime
- effect-runtime

Actual meaning:
A typed profile composition and governance overlay system: which policies apply, which authority is delegated, what attestations exist, what continuity/incident states exist, and what effects are allowed/observed/compensated.

Why this matters:
This is useful inside your stack and potentially high-ROI later, but public readers will misunderstand it unless the docs aggressively narrow the claim boundaries.

ROI: 7/10 internally now. 6/10 public unless bundled under one repo/story.

### Surface D: Practical app/tool integrations

Crates:
- ai-batch-queue
- tauri-queue
- comfyui-rs
- ollama-vision

Actual meaning:
Usable practical libraries for local apps: batch processing with ETA/resource grouping, Tauri queue bridge, ComfyUI client, Ollama vision parsing/tagging/captioning.

Why this matters:
These may get the easiest external downloads because the names map to existing user searches. They are less doctrine-heavy and more immediately understandable.

ROI: 8/10 for public discoverability if READMEs are upgraded and examples work.

### Surface E: Benchmarks / measurement

Crates:
- hnsw-bench
- receipt-bench

Actual meaning:
Benchmark substrate and concrete HNSW backend comparison with receipts.

Why this matters:
This supports your public “receipts or it did not happen” posture and quant/retrieval work. hnsw-bench needs warning cleanup and probably should not be sold as a generic library; it is a reproducible benchmark artifact.

ROI: 7.5/10 as evidence infrastructure, not as a broad crate.

### Surface F: Linux agent security

Crate:
- agent-guard

Actual meaning:
Linux control plane for AI agent security: BPF LSM, cgroup v2, Landlock, seccomp, eBPF. Current source is only 298 LOC and has no README, but the concept is extremely aligned with AI-agent security.

Why this matters:
This could be insanely high ROI later, but it is underdeveloped relative to the claim surface. Do not publish publicly yet unless you narrow it to “experimental skeleton.”

ROI now: 6/10 internal prototype. Potential ROI: 9/10 after hardening.

## Priority table

| Priority | Crates | Action |
|---|---|---|
| P0 use now | forge-engine, forge-pilot, verification-control, verification-policy, verification-adjudication, receipt-bench, profile-runtime, llm-tool-runtime | integrate into current agent/Hermes/AiDENs workflows; docs second but start now |
| P1 publish soon | typed-patch, check-runner, sandbox-workspace, forge-policy, cea-core, cea-store, cea-sqlite, effect-signature, stabilizer-core, mindstate-core | publish as Forge foundation crates after dry-run fixes |
| P1 public practical | ai-batch-queue, tauri-queue, comfyui-rs, ollama-vision | publish if examples work; strongest easy-download candidates |
| P2 schema/governance bundle | assurance-runtime, authority-delegation, attestation-exchange, continuity-runtime, effect-runtime, constitutional-memory, mechanism-runtime, spec-execution, discovery-portfolio, federated-settlement, remote-oracle-admission, verification-calibration, kernel-conformance, contract-schema-gen | keep in stack repo or one governance-kit repo; do not scatter into 13 disconnected repos |
| P2 benchmark evidence | hnsw-bench | clean warnings, document exact claim boundary, run benchmark receipt before public push |
| Hold | agent-guard, check-runner-sys | agent-guard is high-potential but underdocumented/early; check-runner-sys is unsafe support crate, publish only as dependency if needed |

## Per-crate audit

### agent-guard

Path: /home/sikmindz/Coding/Libraries/agent-guard
LOC/tests: 298 LOC, 4 test attributes
README: none
Reverse deps: none

What it is:
Linux control-plane crate for AI agent security. It declares support direction around BPF LSM, cgroup v2, Landlock, seccomp, and eBPF.

My corrected take:
This is not low-value. It is one of the highest-potential ideas in the set, but it is not currently mature enough for strong public claims. It should be treated as a seed for an “AI agent sandbox/control plane” project.

ROI:
- Current internal ROI: medium.
- Potential public ROI: very high.
- Release now as strong standalone crate: no.

Recommended use:
Use it as the future hard-security boundary for Hermes/AiDENs agent subprocesses. Tie it to check-runner/sandbox-workspace/forge-engine later.

README fix:
Needs a README before any release. Must include:
- Linux-only status.
- Threat model.
- Which mechanisms are implemented vs planned.
- Minimal example.
- Diagram: agent process -> cgroup/seccomp/Landlock/BPF -> receipts.

Decision: HOLD, harden, then make standalone repo later.

### ai-batch-queue

Path: /home/sikmindz/Coding/Libraries/ai-batch-queue
LOC/tests: 2788 LOC, 56 test attributes
README: 27 lines
Reverse deps: none

What it is:
Model-aware batch queue with ETA estimation and resource-aware reordering for Tauri apps.

My corrected take:
This is a practical crate, not a doctrine crate. It is likely higher public ROI than many abstract governance crates because people search for exactly this kind of thing when building local AI apps.

ROI:
- Current project ROI: high for Gloss/Kirsten/Recall-style desktop apps and any local batch inference workflow.
- Public ROI: high if examples are good.

Recommended use:
Use for local desktop AI workloads: captioning batches, embeddings, document ingestion, image jobs, model-swap minimization.

README fix:
Needs concrete examples and visuals:
- Queue lifecycle diagram.
- Resource-key grouping example.
- ETA calculation example.
- Tauri integration note pointing to tauri-queue.

Decision: P1 publish soon; standalone repo is reasonable if polished.

### assurance-runtime

Path: /home/sikmindz/Coding/Libraries/assurance-runtime
LOC/tests: 1184 LOC, 21 test attributes
README: 37 lines
Reverse deps: contract-schema-gen, forge-pilot, profile-runtime

What it is:
Typed deployability and certification surface crate with readiness profiles. It explicitly says it is not an orchestration runtime.

My corrected take:
This is already integrated into profile-runtime/forge-pilot/contract-schema-gen. It is not dead. Its public issue is name/claim ambiguity.

ROI:
- Internal ROI: high for profile-aware release/admission gates.
- Public standalone ROI: medium unless bundled.

Recommended use:
Use to make “release readiness” and “regulated/hazard profile” checks explicit in your stack.

README fix:
Needs one worked example: artifact -> assurance profile -> deployability judgment. Add diagram under profile-runtime.

Decision: P2 governance bundle, not standalone first.

### attestation-exchange

Path: /home/sikmindz/Coding/Libraries/attestation-exchange
LOC/tests: 821 LOC, 8 test attributes
README: 38 lines
Reverse deps: contract-schema-gen, forge-pilot, profile-runtime, remote-oracle-admission

What it is:
Typed attestation exchange contracts for envelope, trust-root, and transparency artifacts.

My corrected take:
This is more useful than I initially implied. It is the connective tissue for trust-root and transparency artifacts, and it is already consumed by multiple crates.

ROI:
- Internal ROI: high.
- Public ROI: medium-high if tied to remote-oracle-admission and claim-ledger.

Recommended use:
Use it for signed/replayable evidence boundaries between local and remote runtimes.

README fix:
Needs trust-root diagram and “not a transport runtime” warning.

Decision: P2 governance bundle; could be standalone later.

### authority-delegation

Path: /home/sikmindz/Coding/Libraries/authority-delegation
LOC/tests: 925 LOC, 10 test attributes
README: 37 lines
Reverse deps: contract-schema-gen, forge-pilot, profile-runtime

What it is:
Typed delegated-authority surface: leases, roles, capabilities, emergency delegation, separation of duties.

My corrected take:
This is useful for agent permissioning, especially avoiding silent authority widening. It should not be sold as general auth/RBAC.

ROI:
- Internal ROI: high for agent governance.
- Public ROI: medium until examples exist.

Recommended use:
Use in profile-runtime and Forge decisions where an agent needs bounded delegated permission.

README fix:
Add example: “agent may run tests but cannot publish crate; lease expires; emergency escalation requires receipt.”

Decision: P2 governance bundle.

### cea-core

Path: /home/sikmindz/Coding/Libraries/Primitives/cea-core
LOC/tests: 2444 LOC, 11 test attributes
README: 22 lines
Reverse deps: cea-sqlite, cea-store, forge-engine

What it is:
Causal edit attribution core. Converts structured patches plus check results into weighted cause/effect attribution triples, learns graph weights, predicts future edit risk.

My corrected take:
This is one of the highest-ROI crates in the entire set. I underweighted it earlier because the name is opaque. The idea is strong: agent edits need causal attribution, not just pass/fail logs.

ROI:
- Internal ROI: very high.
- Public ROI: high if renamed/positioned clearly.

Recommended use:
Wire deeply into any agent that changes code: Hermes coding runs, AiDENs, Forge, claim-ledger integration.

README fix:
Needs a diagram: patch -> check result -> located effect -> causal graph -> future risk prediction. Expand acronym immediately.

Decision: P1 publish as part of Forge foundation. Consider standalone repo under “causal-edit-attribution” branding.

### cea-sqlite

Path: /home/sikmindz/Coding/Libraries/Primitives/cea-sqlite
LOC/tests: 1219 LOC, 10 test attributes
README: 24 lines
Reverse deps: forge-engine

What it is:
SQLite persistence adapter for causal edit attribution graphs.

ROI:
- Internal ROI: high because persistence makes CEA useful across runs.
- Public standalone ROI: low-medium by itself.

Recommended use:
Bundle with cea-core/cea-store; do not market independently.

README fix:
Show schema sketch and persistence/replay example.

Decision: P1 publish as support crate only if CEA stack is published.

### cea-store

Path: /home/sikmindz/Coding/Libraries/Primitives/cea-store
LOC/tests: 657 LOC, 5 test attributes
README: 22 lines
Reverse deps: cea-sqlite, forge-engine

What it is:
Storage contract and row types for CEA graphs.

ROI:
- Internal ROI: medium-high.
- Public standalone ROI: low.

Recommended use:
Keep as narrow adapter boundary between CEA semantics and persistence.

README fix:
Add adapter-boundary diagram: cea-core -> cea-store trait -> cea-sqlite.

Decision: P1 support crate.

### check-runner

Path: /home/sikmindz/Coding/Libraries/Primitives/check-runner
LOC/tests: 857 LOC, 12 test attributes
README: 22 lines
Reverse deps: cea-core, cea-sqlite, cea-store, forge-engine

What it is:
Host/container execution backend, environment allowlisting, runtime detection, command output normalization, timing/effect capture.

My corrected take:
High ROI. This is a practical primitive for agent verification, not a boring wrapper.

ROI:
- Internal ROI: very high.
- Public ROI: high if framed as “normalized check execution for agent patch verification.”

Recommended use:
Use in every code-changing agent lane where pass/fail output needs receipts.

README fix:
Needs minimal example and safety model. Mention check-runner-sys relationship.

Decision: P1 publish; likely standalone under Forge family.

### check-runner-sys

Path: /home/sikmindz/Coding/Libraries/Primitives/check-runner-sys
LOC/tests: 42 LOC, 0 test attributes
README: none
Reverse deps: check-runner

What it is:
Unsafe syscall wrappers for check-runner process-group operations.

ROI:
- Internal ROI: necessary support.
- Public ROI: low by itself.

Recommended use:
Keep private/support unless crates.io publication is needed as dependency of check-runner.

README fix:
If published, README must be tiny but strict: unsafe Linux syscall wrappers, safety invariants, not user-facing.

Decision: HOLD/support crate only.

### comfyui-rs

Path: /home/sikmindz/Coding/Libraries/comfyui-rs
LOC/tests: 1757 LOC, 23 test attributes
README: 23 lines
Reverse deps: none

What it is:
Async Rust client for ComfyUI with REST, WebSocket progress, fallback polling, model discovery, workflow builder.

My corrected take:
This has obvious public ROI because it maps to an existing ecosystem. It may be one of the easiest crates to get external users.

ROI:
- Current project ROI: high for image workflows and local creative tooling.
- Public ROI: high.

Recommended use:
Use in any local media/image-generation app; can pair with ai-batch-queue.

README fix:
Needs quickstart, workflow JSON example, progress stream example, supported endpoints table.

Decision: P1 public practical crate; standalone repo reasonable.

### constitutional-memory

Path: /home/sikmindz/Coding/Libraries/constitutional-memory
LOC/tests: 662 LOC, 10 test attributes
README: 42 lines
Reverse deps: contract-schema-gen, forge-pilot, kernel-conformance

What it is:
Typed charter/archive surface for amendments, doctrine snapshots, compaction receipts, historical query guarantees, deprecation/retirement bundles.

My corrected take:
This is not a hidden governance runtime; the crate says that explicitly. It is useful as artifact schema for bitemporal/doctrine systems.

ROI:
- Internal ROI: high for your “doctrine as executable invariant” approach.
- Public ROI: medium because name can sound grandiose.

Recommended use:
Use to anchor doctrine/version changes and memory compaction receipts.

README fix:
Needs example of amendment decision and archive compaction receipt.

Decision: P2 schema/governance bundle.

### continuity-runtime

Path: /home/sikmindz/Coding/Libraries/continuity-runtime
LOC/tests: 1244 LOC, 12 test attributes
README: 37 lines
Reverse deps: contract-schema-gen, forge-pilot, profile-runtime

What it is:
Typed continuity and incident surface: recovery profiles, SLOs, incident routing, continuity states.

ROI:
- Internal ROI: medium-high for long-running agent/runtime operations.
- Public ROI: medium.

Recommended use:
Use for local agent session continuity, failed-run recovery, incident routing, postmortem receipts.

README fix:
Needs concrete “incident -> recovery profile -> continuity receipt” flow.

Decision: P2 governance bundle.

### contract-schema-gen

Path: /home/sikmindz/Coding/Libraries/contract-schema-gen
LOC/tests: 1226 LOC, 11 test attributes
README: 44 lines
Reverse deps: none

What it is:
Schema generator/checker for canonical contract artifacts across the governance/verification crates.

My corrected take:
This is strategically important even if not end-user glamorous. It turns the crate family into an auditable contract system instead of hand-wavy types.

ROI:
- Internal ROI: high.
- Public ROI: medium as developer tooling for the stack.

Recommended use:
Use it as the enforcement mechanism before releases: schemas must regenerate and match.

README fix:
Needs command examples and before/after generated schema tree. Add diagram of owner crates -> schema bundle -> conformance.

Decision: P2 tool crate; publish with stack only.

### discovery-portfolio

Path: /home/sikmindz/Coding/Libraries/discovery-portfolio
LOC/tests: 660 LOC, 10 test attributes
README: 40 lines
Reverse deps: contract-schema-gen, kernel-conformance

What it is:
Typed discovery portfolio artifacts: hypothesis sets, information value estimates, campaigns, verification budgets, campaign decisions.

My corrected take:
This is potentially useful for research/experiment management, but easy to overabstract. It should be internal until tied to a concrete workflow.

ROI:
- Internal ROI: medium.
- Public ROI: low-medium today.

Recommended use:
Use for prioritizing experiments in retrieval/quant/memory projects where verification load is scarce.

README fix:
Needs a concrete example with one campaign and budget decision.

Decision: P2 keep in schema bundle; no standalone repo now.

### effect-runtime

Path: /home/sikmindz/Coding/Libraries/effect-runtime
LOC/tests: 2656 LOC, 12 test attributes
README: 39 lines
Reverse deps: contract-schema-gen, forge-pilot

What it is:
Effect intent, preflight, observation, compensation, commit decision surface.

My corrected take:
This is an important bridge between “agent action” and “action receipts.” It is abstract, but conceptually central.

ROI:
- Internal ROI: high for any agent that performs side effects.
- Public ROI: medium unless bundled.

Recommended use:
Use as the canonical vocabulary for effect preflight/commit/compensation in Hermes/Forge.

README fix:
Needs a side-effect lifecycle diagram.

Decision: P2 governance bundle; high internal priority.

### effect-signature

Path: /home/sikmindz/Coding/Libraries/Primitives/effect-signature
LOC/tests: 132 LOC, 5 test attributes
README: 22 lines
Reverse deps: check-runner, forge-engine

What it is:
Stable effect payloads and hashing helpers for normalized check outputs.

ROI:
- Internal ROI: high relative to size.
- Public standalone ROI: low.

Recommended use:
Keep as a tiny foundational primitive for check output comparison and causal attribution.

README fix:
Add one hash example and explain stable comparison across runs.

Decision: P1 support crate.

### federated-settlement

Path: /home/sikmindz/Coding/Libraries/federated-settlement
LOC/tests: 624 LOC, 7 test attributes
README: 40 lines
Reverse deps: contract-schema-gen, kernel-conformance

What it is:
Typed treaty/settlement/shared-view artifact crate. It handles equivalence evidence, dissent, replay requirements, downgrade reasons, settlement receipts.

My corrected take:
This is future-facing. It matters if multiple runtimes/agents need to agree on evidence. It should not be publicly oversold now.

ROI:
- Internal ROI: medium for future multi-runtime work.
- Public ROI: low-medium today.

Recommended use:
Use when local/remote runtime evidence diverges and needs explicit settlement/dissent records.

README fix:
Needs a simple two-runtime divergence example.

Decision: P2 schema bundle, no standalone now.

### forge-engine

Path: /home/sikmindz/Coding/Libraries/living-memory/living-memory
LOC/tests: 16099 LOC, 181 test attributes
README: 39 lines
Reverse deps: forge-pilot, kernel-conformance, llm-pipeline

What it is:
Operational verification/evaluation engine. Owns mindstate compilation, structured patch validation/apply, check execution, scoring, evidence persistence, and CEA updates.

My corrected take:
This is not just “another runtime.” It is one of the most valuable unreleased assets here. Large, tested, already integrated.

ROI:
- Internal ROI: extremely high.
- Public ROI: high if docs explain it without doctrine fog.

Recommended use:
Use as the center of the code-changing agent safety loop. This should feed receipts into claim-ledger/semantic-memory.

README fix:
Needs serious README upgrade:
- architecture diagram
- minimal patch verification example
- receipts emitted
- relation to forge-pilot
- non-goals

Decision: P0 use now. Public repo later as “Forge”.

### forge-pilot

Path: /home/sikmindz/Coding/Libraries/forge-pilot
LOC/tests: 14447 LOC, 78 test attributes
README: 46 lines
Reverse deps: contract-schema-gen, kernel-conformance

What it is:
OODA governance orchestrator over semantic-memory state, verification targets, oracle/paired-patch plans, and canonical Forge evidence export/import.

My corrected take:
This is a major asset. It is the orchestrator that turns the verification crates into an operating loop.

ROI:
- Internal ROI: extremely high.
- Public ROI: high after docs and demo.

Recommended use:
Use as the controller for high-risk codebase modifications where you need observe/orient/decide/act receipts.

README fix:
Needs OODA diagram and a complete toy run.

Decision: P0 use now. Public as part of Forge, not isolated first.

### forge-policy

Path: /home/sikmindz/Coding/Libraries/Primitives/forge-policy
LOC/tests: 445 LOC, 7 test attributes
README: 22 lines
Reverse deps: cea-sqlite, check-runner, forge-engine, sandbox-workspace, typed-patch

What it is:
Filesystem, environment, patch-cap, and SQLite guardrails for Forge.

ROI:
- Internal ROI: high.
- Public standalone ROI: low-medium.

Recommended use:
Centralize safety policy instead of duplicating guardrails across patch/check/storage code.

README fix:
Add policy examples: forbidden paths, allowed env vars, DB identity validation.

Decision: P1 support crate.

### hnsw-bench

Path: /home/sikmindz/Coding/Libraries/hnsw-bench
LOC/tests: 457 LOC, 0 test attributes
README: 31 lines
Reverse deps: none; depends on receipt-bench

What it is:
Benchmark comparing hnsw_rs vs usearch backend behavior at production vector dimensions, emitting receipt-bench receipts.

My corrected take:
This is not a library; it is evidence infrastructure. Good ROI if it produces dated receipts for retrieval decisions.

ROI:
- Internal ROI: high for semantic-memory backend choices.
- Public ROI: medium-high as reproducible benchmark artifact.

Recommended use:
Run it before making claims about HNSW backend choices.

README/code fix:
Clean cargo warnings. Add sample receipt and exact machine/fingerprint boundary.

Decision: P2 evidence artifact; publish only after one clean benchmark receipt.

### kernel-conformance

Path: /home/sikmindz/Coding/Libraries/kernel-conformance
LOC/tests: 3564 LOC, 67 test attributes
README: 38 lines
Reverse deps: none

What it is:
Conformance harness for recursive inference kernel authority, compiler, oracle, and constitutional gates.

My corrected take:
This has high internal value as the gatekeeper for the doctrine-heavy pieces, but public readers may not understand it without the stack map.

ROI:
- Internal ROI: high.
- Public ROI: medium.

Recommended use:
Keep as conformance gate for the verification/schema/governance surface.

README fix:
Needs a matrix mapping gates to crates and why they exist.

Decision: P2 publish with stack; not standalone first.

### llm-tool-runtime

Path: /home/sikmindz/Coding/Libraries/llm-tool-runtime
LOC/tests: 4552 LOC, 66 test attributes
README: 37 lines
Reverse deps: forge-engine, forge-pilot, llm-pipeline, verification-control, verification-policy

What it is:
Provider-agnostic tool contracts, registry, dispatch, semantic-memory starter tools, and receipt plumbing.

My corrected take:
Very high ROI. This is directly relevant to Hermes/agent-memory and current LLM tool calling work. It should probably be used more immediately.

ROI:
- Internal ROI: extremely high.
- Public ROI: high if docs are concrete.

Recommended use:
Use as the canonical tool-contract/receipt layer for agent runtimes instead of scattering ad hoc tool definitions.

README fix:
Needs provider/tool dispatch diagram and receipt example.

Decision: P0 use now; likely standalone-worthy after packaging.

### mechanism-runtime

Path: /home/sikmindz/Coding/Libraries/mechanism-runtime
LOC/tests: 623 LOC, 10 test attributes
README: 41 lines
Reverse deps: contract-schema-gen, forge-pilot, kernel-conformance

What it is:
Typed mechanism/theory surface with fit/refuter evaluators. It is not a standalone runtime.

ROI:
- Internal ROI: medium-high for research claims and theory versioning.
- Public ROI: medium-low today.

Recommended use:
Use when promoting/refuting mechanism hypotheses in experiments.

README fix:
Needs one theory bundle + fit run + refuter example.

Decision: P2 schema bundle.

### mindstate-core

Path: /home/sikmindz/Coding/Libraries/Primitives/mindstate-core
LOC/tests: 285 LOC, 7 test attributes
README: 22 lines
Reverse deps: forge-engine

What it is:
Serializable mindstate payload types for forge-engine: evidence items, trace refs, hashes, question signatures, budget evidence.

ROI:
- Internal ROI: high as a small stable primitive.
- Public standalone ROI: low.

Recommended use:
Use to standardize what the agent thought it was doing before patching.

README fix:
Show rendered mindstate and hash/signature example.

Decision: P1 support crate.

### ollama-vision

Path: /home/sikmindz/Coding/Libraries/ollama-vision
LOC/tests: 769 LOC, 6 test attributes
README: 22 lines
Reverse deps: none

What it is:
Ollama vision tagging/captioning toolkit with robust parsing strategies for LLM output formats.

My corrected take:
This is practical and likely useful immediately, especially for local-first AI apps. It should be public-facing if examples are clean.

ROI:
- Internal ROI: high for media/image local tools.
- Public ROI: high because Ollama has a large user base.

Recommended use:
Use with ai-batch-queue for image-library tagging/captioning.

README fix:
Needs input/output examples, parser failure modes, model recommendations, and batch integration example.

Decision: P1 public practical crate.

### profile-runtime

Path: /home/sikmindz/Coding/Libraries/profile-runtime
LOC/tests: 4255 LOC, 17 test attributes
README: 40 lines
Reverse deps: contract-schema-gen

What it is:
Canonical effective constitution/profile composition runtime. It composes family-specific profile overlays into replayable answers with conflicts and exceptions.

My corrected take:
High ROI internally. This is the thing that prevents policy/profile drift across all the bounded surface crates.

ROI:
- Internal ROI: very high.
- Public ROI: medium-high if shown with examples.

Recommended use:
Make this the central composition layer for authority, assurance, continuity, attestation, and effect policies.

README fix:
Needs profile overlay diagram and conflict example.

Decision: P0/P2 hybrid: use now internally; publish with governance kit.

### receipt-bench

Path: /home/sikmindz/Coding/Libraries/receipt-bench
LOC/tests: 2075 LOC, 27 test attributes
README: 27 lines
Reverse deps: hnsw-bench

What it is:
Replayable benchmark substrate: structured receipts timestamped and keyed to commit hash and machine fingerprint.

My corrected take:
High ROI because it supports your credibility. It is the right answer to benchmark skepticism.

ROI:
- Internal ROI: high.
- Public ROI: high if paired with example receipts.

Recommended use:
Use for semantic-memory, turbo-quant, HNSW, compression, and retrieval benchmarks.

README fix:
Needs sample JSON receipt and replay/diff instructions.

Decision: P0 use now; P1 publish.

### remote-oracle-admission

Path: /home/sikmindz/Coding/Libraries/remote-oracle-admission
LOC/tests: 713 LOC, 6 test attributes
README: 28 lines
Reverse deps: contract-schema-gen

What it is:
Typed remote oracle admission contracts: lease, slice request/result, cross-runtime replay ticket, exactness/disclosure classes, revocation/supersession.

ROI:
- Internal ROI: medium-high for remote proof/oracle workflows.
- Public ROI: medium.

Recommended use:
Use only when a local runtime admits remote evidence and must preserve exactness/disclosure boundaries.

README fix:
Needs local-vs-remote admission diagram.

Decision: P2 schema bundle.

### sandbox-workspace

Path: /home/sikmindz/Coding/Libraries/Primitives/sandbox-workspace
LOC/tests: 386 LOC, 11 test attributes
README: 22 lines
Reverse deps: check-runner, forge-engine, typed-patch

What it is:
Safe workspace staging and patch filesystem helpers for controlled file access and patch application.

ROI:
- Internal ROI: very high.
- Public standalone ROI: medium.

Recommended use:
Use everywhere a patch is applied before checks run.

README fix:
Needs concrete temp workspace/staged patch example and file-access boundary diagram.

Decision: P1 support crate.

### spec-execution

Path: /home/sikmindz/Coding/Libraries/spec-execution
LOC/tests: 729 LOC, 10 test attributes
README: 42 lines
Reverse deps: contract-schema-gen, kernel-conformance

What it is:
Typed spec/proof surface with generated schema/interpreter/conformance/migration/proof artifacts and veto/challenge baselines.

ROI:
- Internal ROI: high for doctrine/spec-driven work.
- Public ROI: medium if examples are grounded.

Recommended use:
Use when turning specs into generated artifacts plus proof obligations.

README fix:
Needs simple spec -> generated schema bundle -> conformance corpus example.

Decision: P2 schema bundle.

### stabilizer-core

Path: /home/sikmindz/Coding/Libraries/Primitives/stabilizer-core
LOC/tests: 487 LOC, 6 test attributes
README: 22 lines
Reverse deps: forge-engine

What it is:
Attempt-phase and delta-policy primitives for forge-engine: innovate/stabilize/clamp phases, strategy tags, novelty, approach family.

ROI:
- Internal ROI: high because it directly addresses repeated failed agent fix loops.
- Public ROI: medium.

Recommended use:
Use to enforce bounded fix attempts and prevent infinite thrashing.

README fix:
Needs example showing attempt 1 innovate -> attempt 2 stabilize -> attempt 3 clamp/stop.

Decision: P1 support crate.

### tauri-queue

Path: /home/sikmindz/Coding/Libraries/tauri-queue
LOC/tests: 1503 LOC, 33 test attributes
README: 22 lines
Reverse deps: none

What it is:
Tauri integration for job-queue background processing; event emitter bridge, coalescing/drop policy, frontend events.

ROI:
- Internal ROI: high for desktop apps.
- Public ROI: high if it cleanly integrates with job-queue.

Recommended use:
Use in Gloss/Kirsten/Recall-style apps where background work should report progress without UI spam.

README fix:
Needs Tauri command + frontend listener example, event flow diagram, coalescing/drop policy explanation.

Decision: P1 public practical crate.

### typed-patch

Path: /home/sikmindz/Coding/Libraries/Primitives/typed-patch
LOC/tests: 987 LOC, 6 test attributes
README: 22 lines
Reverse deps: cea-core, forge-engine, stabilizer-core

What it is:
Structured patch schema plus validation/apply helpers: file edits, anchors, line ranges, line attribution maps, validation results.

My corrected take:
This is a central primitive and high ROI. Agent code editing should use typed patches, not raw string diffs, when you need receipts.

ROI:
- Internal ROI: very high.
- Public ROI: high as part of Forge.

Recommended use:
Use as the canonical edit object for patch application and CEA.

README fix:
Needs example patch object and rendered diff.

Decision: P1 publish; likely standalone-worthy within Forge family.

### verification-adjudication

Path: /home/sikmindz/Coding/Libraries/verification-adjudication
LOC/tests: 1381 LOC, 10 test attributes
README: 35 lines
Reverse deps: contract-schema-gen, forge-pilot, kernel-conformance

What it is:
Canonical promotion/refutation/rollback decision artifacts and adjudication logic.

ROI:
- Internal ROI: very high.
- Public ROI: high in verification kit.

Recommended use:
Use after verification-control produces cases/receipts to decide promotion/refutation/rollback.

README fix:
Needs decision flow diagram: case -> evidence -> promotion/refutation/rollback.

Decision: P0/P1 verification kit.

### verification-calibration

Path: /home/sikmindz/Coding/Libraries/verification-calibration
LOC/tests: 354 LOC, 10 test attributes
README: 32 lines
Reverse deps: contract-schema-gen, forge-pilot, kernel-conformance, verification-adjudication

What it is:
Calibration and abstention artifacts: calibration snapshots, nuisance state, evaluator.

ROI:
- Internal ROI: high if used to prevent overconfident promotion.
- Public ROI: medium.

Recommended use:
Use to separate “verified”, “abstain”, and “nuisance/noise” decisions.

README fix:
Needs one calibration snapshot example.

Decision: P2 support in verification kit.

### verification-control

Path: /home/sikmindz/Coding/Libraries/verification-control
LOC/tests: 3493 LOC, 19 test attributes
README: 38 lines
Reverse deps: contract-schema-gen, forge-pilot, kernel-conformance, verification-adjudication, verification-calibration, verification-policy

What it is:
Verification cases, check plans, attempts, control receipts, ledger entries, governance review artifacts, scheduling, promotion eligibility, ledger replay.

My corrected take:
This is one of the highest-value unreleased crates. It is already depended on by six crates and defines the operational control plane for verification.

ROI:
- Internal ROI: extremely high.
- Public ROI: high if packaged as verification-control/agent-verification-kit.

Recommended use:
Use as the central object model for code/claim verification in current projects.

README fix:
Needs full lifecycle diagram and a minimal check-plan example.

Decision: P0 use now; P1 publish.

### verification-policy

Path: /home/sikmindz/Coding/Libraries/verification-policy
LOC/tests: 2657 LOC, 22 test attributes
README: 36 lines
Reverse deps: contract-schema-gen, forge-pilot, kernel-conformance, profile-runtime, verification-adjudication

What it is:
Canonical verification policy and approval artifacts: effect policy, delegation policy, release policy, approval matching, policy_as_of, evaluate_policy.

My corrected take:
High-value. This is how you avoid silent widening and shadow truth in release/effect/delegation decisions.

ROI:
- Internal ROI: extremely high.
- Public ROI: high in verification kit.

Recommended use:
Use in every release/publish/side-effect gate.

README fix:
Needs policy examples and “policy as of time T” bitemporal example.

Decision: P0 use now; P1 publish.

## Repo/publishing recommendation

Do not make 39 standalone repos.

Make these public-facing repos/families first:

1. recursiveintell/forge
   - forge-engine
   - forge-pilot
   - typed-patch
   - check-runner
   - sandbox-workspace
   - forge-policy
   - cea-core
   - cea-store
   - cea-sqlite
   - effect-signature
   - stabilizer-core
   - mindstate-core

2. recursiveintell/agent-verification-kit
   - verification-control
   - verification-policy
   - verification-adjudication
   - verification-calibration
   - receipt-bench
   - contract-schema-gen
   - kernel-conformance

3. recursiveintell/agent-governance-profiles
   - profile-runtime
   - assurance-runtime
   - authority-delegation
   - attestation-exchange
   - continuity-runtime
   - effect-runtime
   - constitutional-memory
   - mechanism-runtime
   - spec-execution
   - discovery-portfolio
   - federated-settlement
   - remote-oracle-admission

4. Standalone practical repos:
   - ai-batch-queue
   - tauri-queue
   - comfyui-rs
   - ollama-vision

5. Hold/harden:
   - agent-guard
   - check-runner-sys
   - hnsw-bench until warnings and benchmark receipt are cleaned

## README policy for this set

Every public README should have:

1. One-sentence use case.
2. “What this crate owns.”
3. “What this crate explicitly does not own.”
4. Minimal runnable example.
5. Architecture or flow diagram.
6. Integration map to adjacent RecursiveIntell crates.
7. Claim boundary: what is tested, what is experimental, what is not claimed.

Current gap:
All 39 unreleased crates lacked obvious README visual/diagram signals in the inventory scan. That is the fastest trust improvement.

## Highest ROI current-project integrations

1. Hermes/AiDENs coding-agent control:
   - llm-tool-runtime
   - typed-patch
   - check-runner
   - sandbox-workspace
   - forge-engine
   - verification-control
   - verification-policy
   - verification-adjudication
   - receipt-bench

2. Semantic-memory/claim-ledger trust loop:
   - verification-control
   - verification-policy
   - verification-calibration
   - verification-adjudication
   - receipt-bench
   - constitutional-memory
   - profile-runtime
   - attestation-exchange

3. Local desktop AI apps:
   - ai-batch-queue
   - tauri-queue
   - ollama-vision
   - comfyui-rs

4. Agent side-effect safety:
   - effect-runtime
   - authority-delegation
   - profile-runtime
   - forge-policy
   - agent-guard later

## Blunt keep/kill

Keep and invest:
- forge-engine, forge-pilot, llm-tool-runtime, verification-control, verification-policy, verification-adjudication, profile-runtime, receipt-bench, cea-core, typed-patch, check-runner, sandbox-workspace, ai-batch-queue, tauri-queue, comfyui-rs, ollama-vision.

Keep as support crates:
- cea-store, cea-sqlite, forge-policy, effect-signature, stabilizer-core, mindstate-core, check-runner-sys.

Keep bundled, do not over-market standalone yet:
- assurance-runtime, authority-delegation, attestation-exchange, continuity-runtime, effect-runtime, constitutional-memory, mechanism-runtime, spec-execution, discovery-portfolio, federated-settlement, remote-oracle-admission, verification-calibration, kernel-conformance, contract-schema-gen.

Hold/harden before public claims:
- agent-guard.

Clean before evidence publication:
- hnsw-bench warnings and sample receipt.

## Bottom line

The high-ROI assets are real. The strongest hidden project is Forge: causal edit attribution + typed patch verification + check normalization + receipts. That should become a first-class part of your current agent stack.

The second strongest is the verification/profile/governance kit. It should not be scattered as 20 abstract crates; it needs a stack map and diagrams.

The easiest public download wins are the practical integration crates: ai-batch-queue, tauri-queue, comfyui-rs, ollama-vision.

The biggest immediate flaw is README quality, not code health. The workspace builds and tests passed. The public surface does not yet explain why these crates matter.
