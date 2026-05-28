# P20.1 Codex Run Prompt — Code/Package Repair and Final Certification

You are working in the AiDENs repository after P20. Your task is to finish **code/package/test integrity**, not to invent new architecture.

## Mission

Convert the current AiDENs source tree from “architecturally plausible but package/test fragile” into a build-certified, manifest-honest, code-first v0.1 release candidate.

## Hard constraints

- Audit actual code, tests, Cargo manifests, scripts, fixtures, and package manifests first.
- Do not treat docs as proof of implementation.
- Do not add local canonical truth that belongs to sibling libraries.
- Do not claim PASS unless cargo gates run in the real workspace.
- Do not skip manual phase injections.

## Phase sequence

### Phase 00 — code/package audit baseline
Run `scripts/p20_1_hard_code_audit.py`. Confirm include targets, manifest targets, toolchain status, cargo topology, and scanner preconditions.

### Phase 01 — restore missing package files
Restore `evals/p20_agency_eval_cases.jsonl`, `fixtures/runner/expected_event_log.ndjson`, and `supporting/matrices/*.csv` or regenerate `MANIFEST.txt` honestly. Run eval validation.

### Phase 02 — split/repair testkit topology
Make `aidens-testkit` pure reference/fixture code. Move production-integrating tests to an integration test crate or package-local tests.

### Phase 03 — harden ownership scanner
Ensure scanner cannot certify “no duplicate canonical types” if canonical sibling crate inventory is empty/unavailable.

### Phase 04 — cargo gate repair
Run fmt/check/test/clippy in the real workspace. Fix compile/test failures without bypassing canonical crates.

### Phase 05 — revalidate runner/provider/agency surfaces
Prove mock runner vertical slice, provider capability honesty, and agency eval receipts.

### Phase 06 — final audit bundle and archive integrity
Generate final audit output. Confirm archive contains every code/manifest/script-referenced file.

## Required final commands

```bash
python3 scripts/p20_1_hard_code_audit.py --fail-on-blocking
python3 scripts/p20_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/p20_1_generate_audit_bundle.sh
```

## Final output required from Codex

- exact command transcript summary;
- files changed;
- unresolved blockers, if any;
- final PASS/FAIL per acceptance gate;
- final archive integrity status.
