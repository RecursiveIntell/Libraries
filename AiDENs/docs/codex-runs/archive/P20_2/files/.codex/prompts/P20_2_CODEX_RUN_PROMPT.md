# Codex Run Prompt — P20.2 Code Package Closure, Test Agent Proof, and v0.1 Certification

You are working in the AiDENs repository. Your task is to take the project as far as possible in one run while preserving all canonical architecture constraints.

## Mission

Complete P20.2:

1. repair actual source/package integrity;
2. split/purify test topology;
3. make the canonical test agent executable and tested;
4. certify v0.1 release readiness;
5. run guarded stretch work only after all mandatory gates pass.

## Absolute constraints

- Work from code and command output. Do not trust docs as proof.
- AiDENs is an orchestration/wiring/agent-construction layer. Do not reimplement canonical library semantics.
- Do not create compatibility shims to bypass canonical crates.
- Do not delete failing tests unless equivalent coverage remains and the reason is documented.
- Do not advertise provider/tool/native capability without executable tests.
- Do not leave package artifacts missing from the release archive.
- Do not continue after invariant violations without repair or quarantine.

## Required phase order

Execute phases in order:

1. PHASE_00_SOURCE_TRUTH_PREFLIGHT
2. PHASE_01_PACKAGE_INTEGRITY_CLOSURE
3. PHASE_02_TESTKIT_SPLIT_AND_INTEGRATION_CRATE
4. PHASE_03_BUILD_GATE_STABILIZATION
5. PHASE_04_CANONICAL_TEST_AGENT_VERTICAL_SLICE
6. PHASE_05_PROVIDER_TOOL_PERMIT_RECEIPT_HARDENING
7. PHASE_06_AGENCY_EVAL_AND_RECEIPT_HARDENING
8. PHASE_07_V0_1_USABILITY_EXAMPLES
9. PHASE_08_SCANNER_CONFORMANCE_HARDENING
10. PHASE_09_RELEASE_CERTIFICATION_AND_ARCHIVE_REPLAY
11. PHASE_10_GUARDED_STRETCH_LANE

Stop after each phase and produce a phase report. Wait for the operator's injection prompt before continuing when possible. If the environment does not force stops, still run the invariant revalidation internally at the beginning of every phase and record it.

## Required commands by final phase

```bash
python3 scripts/p20_2_scan_package_integrity.py .
python3 scripts/p20_2_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
python3 scripts/p20_2_scan_testkit_purity.py . --require-integration-crate
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
P20_2_REQUIRE_CARGO=1 bash scripts/p20_2_verify.sh
```

## Required implementation outcomes

- `evals/p20_agency_eval_cases.jsonl` exists and validates.
- all literal include targets exist.
- `aidens-testkit` is pure/reference-only.
- `aidens-integration-tests` exists or equivalent integration crate is created.
- test-agent vertical slice passes.
- final audit bundle is generated.
- release archive is unpacked and rechecked.

## Stretch rule

Only if all prior gates pass, improve v0.1 usability: examples, profile smoke tests, operator quickstart, and P21 provider-expansion plan. Do not begin native provider implementation unless explicitly approved.

## Final report

Produce:

- PASS/FAIL per acceptance gate;
- changed files;
- commands run and outputs;
- unresolved risks;
- known limitations;
- final auditor handoff;
- next recommended pass.
