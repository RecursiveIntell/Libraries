# P25 Phase Plan

## Phase 00 — Preflight, source basis, and scope lock

- Read source basis, prior-run status/evidence, support profile, and Claude audit if present.
- Confirm P25 non-goals and z.py narrow scope.
- Inventory root Markdown files and phase injection files.
- Emit preflight report before code changes.

## Phase 01 — Root Markdown classification

- Classify direct root Markdown files as protected-functional, candidate-archive, ambiguous-stop, or active-current-run.
- Do not move files yet.
- Emit machine-readable classification CSV/JSON.
- Stop at the gate after Phase 01.

## Phase 02 — Surgical z.py root Markdown archiver

- Implement bounded root Markdown archive feature only.
- Add dry-run, verify-only safety, protected allowlist, collision fail-closed behavior, and manifest emission.
- Do not add runtime or semantic behavior to z.py.

## Phase 03 — Phase-gate integrity implementation

- Create P25 phase-injection files with STOP/WAIT language.
- Add verifier script for phase-gate integrity.
- Fail on stale run IDs in active instruction files.
- Stop at the gate after Phase 03.

## Phase 04 — Current-run classification cleanup

- Clean docs/codex-runs current-run files.
- Ensure P25 is current instruction/run truth.
- Archive or mark prior-run evidence appropriately.
- Remove stale prior-run current-instruction classifications.

## Phase 05 — Verifier and evidence manifest hardening

- Create or update scripts/p25_verify.sh and/or scripts/verify_current.sh.
- Run package validation, gate integrity, root-md dry-run, support-claim checks, stale run scans, and cargo gates where available.
- Emit P25_STATUS_EVIDENCE_MANIFEST.json.
- Stop at the gate after Phase 05.

## Phase 06 — Flagship supported-local coding-agent demo

- Build examples/flagship-local-coding-agent.
- Demonstrate fixture repo read/search/status, patch proposal or abstention, permit-gated apply/dry-run, receipts, AiDENsRunBundleV2, deterministic replay.
- No cloud and no fake autonomy.

## Phase 07 — Support profile and docs convergence

- Update README, STATUS, SUPPORT_PROFILE, known limitations, and operator docs.
- Clearly label supported-local, fixture-backed, deferred-cloud, deferred-autonomy.
- Stop at the gate after Phase 07.

## Phase 08 — Large-file containment plan

- Create P25_LARGE_FILE_CONTAINMENT_PLAN.md.
- Identify split candidates and tests for a future pass.
- Do not refactor giant files unless required to satisfy current gates.

## Phase 09 — Final hostile audit and package replay

- Run verifier, package validation, package self-replay, cargo gates where available.
- Emit final audit, known limitations, evidence manifest, and handoff.
- Stop at the gate after Phase 09 before final closure.

## Phase FINAL — Final closure

- After operator injection, emit final summary and no further changes.
- Include exact pass/fail gate results and unresolved risks.
