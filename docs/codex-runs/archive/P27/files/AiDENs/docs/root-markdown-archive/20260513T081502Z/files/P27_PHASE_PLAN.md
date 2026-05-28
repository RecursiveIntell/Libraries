# P27 Phase Plan — Super-Pass

## Phase 00 — Intake, no-mutation audit, and scope lock

- Read P27 packet, P26 status/support/source docs, package sidecars, Claude hard audit, GPT hard audit if present.
- Confirm current state of verifier wrappers, CI, source-basis docs, and shipped evidence paths.
- Emit no code changes unless necessary to create `target/p27/audit/` and phase report directories.

Gate: no capability implementation before Phase 01/02 truth repair is complete or quarantined.

## Phase 01 — Verifier and CI hard repair

- Replace missing historical/P26 verifier references with a real `scripts/p27_verify.sh`.
- Make `scripts/verify_current.sh` and `scripts/verify.sh` delegate to P27 current verifier.
- Update CI to call `bash scripts/verify_current.sh` or `scripts/p27_verify.sh`.
- Add script-reference assertion that fails if wrappers point to missing scripts.

Gate: verifier wrapper targets must exist and the script-reference check must pass.

## Phase 02 — Active run truth surface normalization

- Update `STATUS.md`, `SOURCE_BASIS.md`, `SUPPORT_PROFILE.md`, `README.md`, and `AGENTS.md` to agree on P27 current-run truth.
- Archive/downgrade P24/P25/P26 instructions as historical evidence, not active doctrine.
- Add an assertion that active-run docs agree.

Gate: no active doc may claim P22/P23/P24/P25/P26 as current.

## Phase 03 — Package self-replay and sibling-layout classification

- Attempt package self-replay in the extracted package.
- If sibling monorepo layout is required, add a prerequisite checker and honest replay classification.
- Capture stdout/stderr and environment facts.

Gate: replay status must be either green or explicitly classified as environment/sibling-layout blocked, not silently passed.

## Phase 04 — Ownership scanner fail-closed behavior

- Fix `make_type_ownership_inventory.py` and related assertions so canonical-baseline absence cannot produce a clean duplicate-free claim.
- Add fixtures/tests for absent baseline and present baseline.

Gate: absent sibling baseline must emit `canonical_inventory_unavailable=true` or equivalent and fail/return non-clean status.

## Phase 05 — Root Markdown truth and archive hygiene

- Triage ambiguous root Markdown files.
- Archive historical Pxx run docs or label them historical.
- Preserve protected active docs.
- Add/strengthen root Markdown archive policy assertions.

Gate: no stale run doctrine remains unlabeled as active root truth.

## Phase 06 — Scaffold profile crate claim cleanup

- Either remove scaffold-only profile crates from workspace members or fence them with explicit status and assertions.
- Update CLI scaffold disclosure and support profile accordingly.

Gate: scaffold-only crates must not inflate supported product claims.

## Phase 07 — Reproducibility prerequisite and source-basis hardening

- Add `scripts/assert_sibling_workspace_layout.py` or equivalent.
- Update `SOURCE_BASIS.md` with exact sibling crate layout, current run, and replay modes.
- Do not vendor unless time and risk budget allow.

Gate: a fresh reviewer can tell exactly what source tree is required and why.

## Phase 08 — Durable run receipt store v0

- Add a filesystem-backed receipt store for `AiDENsRunBundleV3` and related local evidence.
- Ensure run outputs survive process exit and can be inspected by CLI.
- Keep it AiDENs-local/operator evidence, not canonical memory truth.

Gate: at least one integration test writes a run bundle and later inspects it from disk.

## Phase 09 — Provider path E2E hardening

- Ensure mock-provider Plan→Act→Verify path is executable end-to-end.
- If local Ollama is available, add optional non-required smoke path with clear skip behavior.
- Do not require hosted provider keys for verifier success.

Gate: supported-local loop must be testable without cloud credentials.

## Phase 10 — Patch engine hardening v0

- Harden patch application semantics: dry-run/check, permit requirement, ambiguity refusal, changed-file receipt, failure taxonomy, rollback advice.
- Prefer using `git apply --check` where available or implement narrow strict patch format with explicit limits.

Gate: invalid/ambiguous patches must fail closed with evidence.

## Phase 11 — Coding agent loop uplift

- Connect repo search/read/propose/apply/check receipts into a coherent coding-agent run path.
- Add tests for blocked writes, failed checks, and successful patch+check loop.

Gate: coding-agent path should be safer and more inspectable, not necessarily Claude Code parity.

## Phase 12 — Memory-grounding durable seam

- Strengthen memory-grounded agent evidence with canonical adapter backpointers, query/view disclosure, degradation labels, and no local truth store.
- Add tests/scanners proving no local memory truth was introduced.

Gate: memory evidence exists; memory truth remains sibling-owned.

## Phase 13 — Contract/schema conformance and duplicate-key hardening

- Ensure AgentSpec and RunBundle schemas are generated/validated where tools exist.
- Add strict JSON parse/duplicate-key refusal for evidence-bearing inputs if not already present.

Gate: invalid structured evidence must not be accepted silently.

## Phase 14 — Megafile containment: contracts first

- Split `aidens-contracts/src/lib.rs` into internal domain modules behind a stable re-export facade.
- Avoid crate explosion in this phase unless already trivial.
- Preserve public API compatibility where possible.

Gate: cargo check/test for contracts and affected crates pass or failures are classified.

## Phase 15 — Megafile containment: CLI next

- Split `aidens-cli/src/lib.rs` by command domain behind stable CLI behavior.
- Avoid semantic rewrites.

Gate: CLI help/commands still work; tests pass.

## Phase 16 — Agency/governance eval harness hardening

- Grow eval cases for manipulation/scarcity/urgency/decorative alternatives/relational boundary.
- Keep heuristic-v0.1 label unless formal bar is reached.

Gate: governance blocks remain real and heuristic label remains honest.

## Phase 17 — 11A semantic disclosure layer

- Add exact/approx/degraded/support-tier/proof-check labels to evidence-bearing operator outputs where missing.
- Add reference-semantics TODOs only as fenced, testable obligations.

Gate: no new semantic claim lacks an exactness/support/degradation label.

## Phase 18 — Support profile and operator UX closure

- Update support profile, command docs, examples, known limits, operator quickstart.
- Ensure docs match code and tests.

Gate: support claims are traceable to tests/evidence.

## Phase 19 — Full validation and hostile audit

- Run fmt/check/test/clippy/doc as available.
- Run P27 verifier.
- Run package strict validation.
- Run package self-replay or honest replay classification.
- Draft final audit and final handoff.

Gate: no final success claim if verifier/replay/support truth remains unresolved.

## Phase 20 — Final package and closeout

- Generate final package and sidecars.
- Emit `P27_STATUS_EVIDENCE_MANIFEST.json`.
- Archive stale P27 phase docs as appropriate.
- Produce final auditor handoff.

Gate: final package evidence must be internally self-consistent.

## Stretch-only phase — V10/V11+ geometry notes

Only after all gates are green: record V10/V11 stretch notes. Do not implement regional/hypergraph/federated/mechanism runtime geometry in P27 unless the operator explicitly authorizes a separate pass.
