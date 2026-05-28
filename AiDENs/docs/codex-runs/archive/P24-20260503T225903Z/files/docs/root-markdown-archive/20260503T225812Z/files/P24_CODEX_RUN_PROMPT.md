# P24 Codex run prompt — AiDENs V9 seam lock and completion super-pass

You are operating on the latest AiDENs source package. Your job is to push AiDENs as close to complete as possible in one super-pass without violating canonical stack ownership.

## Non-negotiable doctrine

AiDENs is a product/compiler/orchestration/operator layer. It must not become a shadow truth engine.

Canonical ownership remains with the library crates:

- `stack-ids` owns IDs, digests, scopes, trace primitives, attempt/trial/episode identities.
- `semantic-memory-forge` owns Forge export, evidence bundles, `EpisodeBundleV1`, `ExecutionContextV1`, and Forge tool receipts.
- `forge-memory-bridge` owns export-to-import transformation and digest/backpointer preservation.
- `semantic-memory` owns canonical memory storage/import truth.
- `knowledge-runtime` owns runtime query/view/widening provenance.
- `llm-tool-runtime` owns provider/tool runtime receipt semantics where applicable.
- `verification-*` crates own verification policy/control/adjudication/calibration semantics.
- AiDENs owns display/report/operator artifacts, app profiles, local tool routing UX, local fixture execution, and support-tier evidence.

## Primary goal

Finish P24 as a V9 seam-consuming AiDENs product layer:

1. harden verifier/package self-replay;
2. lock canonical seam ownership;
3. emit typed `AiDENsRunBundleV2` with canonical `ExecutionContextV1` and replay-normalized evidence;
4. promote the local coding-agent lane to supported-local;
5. prove canonical memory/runtime import/query via a fixture;
6. optionally promote daemon-safe queue if fully tested;
7. harden parser/repair/verification failure honesty;
8. update docs/support profile/current run state;
9. produce a final package and handoff with command evidence.

## Do not do

- Do not build V10+ regional decoder/hypergraph/subtraction/federation/mechanism runtime as this pass's main target.
- Do not add local canonical substitutes.
- Do not promote scaffold profiles with prose.
- Do not delete historical docs without archive/quarantine evidence.
- Do not claim cloud/native provider support unless a runnable provider path and receipts exist.
- Do not mark unsupported paths as complete.

## Required phase sequence

Run the phases from `P24_PHASE_PLAN.md` in order. For each phase, create `AiDENs/handoffs/p24/PHASE_XX_REPORT.md`.

### Phase 00 — preflight hostile audit

Record package sidecars, source metrics, active docs, support claims, and rust/tooling availability. Create `P24_SOURCE_BASIS.md`.

### Phase 01 — verifier hardening

Create `scripts/p24_verify.sh`. Add timeouts/pruning/receipts. Fix any script that scans archive/generated/target paths or can hang.

### Phase 02 — canonical seam lock

Create `P24_CANONICAL_SEAM_MAP.md`. Add tests/scripts proving AiDENs does not define canonical owner types locally unless aliasing/re-exporting canonical owners.

### Phase 03 — AiDENsRunBundleV2

Define typed V2 bundle. It must include or backpoint to `ExecutionContextV1`, `TraceCtx`, `AttemptId`, `TrialId`, event-log digest, provider/tool receipts, budget/deadline/degradation, support tier, replay normalization, and failure taxonomy. Upgrade `run-test-agent` and `inspect-run`.

### Phase 04 — coding-agent supported-local lane

Implement/run a local coding-agent fixture. It must read/list/search/status a local repo, propose patch or abstain, require permit for writes, and emit receipts.

### Phase 05 — memory/runtime seam

Add fixture proving ExportEnvelopeV3 -> forge-memory-bridge -> semantic-memory -> knowledge-runtime query. Preserve digest/backpointers and disclose view/widening/degradation.

### Phase 06 — daemon-safe lane

Only promote if fully runnable. Append-only local queue lifecycle with receipts, idempotency, duplicate suppression, and no external side effects.

### Phase 07 — boundary/repair/verification hardening

Strict JSON/patch parsing. Duplicate keys fail. Treatment-critical ambiguity fails. Repairs carry before/after digests and canonical verification/control backpointers or degraded/no-plan reasons.

### Phase 08 — docs/support profile/UX

Update active docs to P24. Keep support claims exact. Add operator commands/docs. Mark scaffold/partial/deferred honestly.

### Phase 09 — final audit and package

Run cargo fmt/check/test/clippy, p24 verifier, package strict modes, package self-replay, final audit, and handoff. If something fails, produce exact blocked handoff.

## Required final artifacts

- `AiDENs/docs/p24/P24_FINAL_AUDIT_REPORT.md`
- `AiDENs/docs/p24/P24_KNOWN_LIMITATIONS.md`
- `AiDENs/P24_STATUS_EVIDENCE_MANIFEST.json`
- `AiDENs/handoffs/p24/*`
- package sidecars
- run-bundle V2 example + replay receipt
- coding-agent evidence if supported
- memory/runtime seam evidence
- daemon-safe evidence if supported

## Final answer format inside Codex

Return:

1. completed phases;
2. changed files;
3. commands run and pass/fail;
4. generated artifacts and hashes;
5. support profile delta;
6. unresolved risks;
7. exact next pass if any.
