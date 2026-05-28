# P20 Docs / Code Truth Report

Record time: `2026-04-29T23:46:00Z`

Phase: `02_DOCUMENTATION_TRUTH`

Phase 09 update: `2026-04-30T00:00:00Z`

## Scope

Compared active documentation claims against code paths, tests, and Phase 01 build evidence. This report covers:

- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `MASTER_ISSUE_MATRIX.md`
- `NEXT_CODEX_TASK_MATRIX.md`
- `docs/MASTER_ISSUE_MATRIX.md`
- `CURRENT_AIDENS_HARD_AUDIT_20260426.md`
- `docs/CURRENT_AIDENS_AUDIT.md`
- `docs/p20/CURRENT_AIDENS_AUDIT.md`
- `SUPER_PASS_EXECUTIVE_SUMMARY.md`
- `docs/p20/reports/PHASE_02_REPORT.md`

Historical pass files, handoffs, prompts, and task JSON are treated as evidence packets, not current support claims.

## Evidence Used

| Evidence | Path |
|---|---|
| Phase 01 report | `docs/p20/reports/PHASE_01_REPORT.md` |
| Final fmt log | `target/p20-phase01/logs/06_fmt_check_final.log` |
| Final check log | `target/p20-phase01/logs/07_cargo_check_final.log` |
| Final test log | `target/p20-phase01/logs/08_cargo_test_final.log` |
| Final clippy log | `target/p20-phase01/logs/09_cargo_clippy_final.log` |
| Final repo verify log | `target/p20-phase01/logs/10_repo_verify_final.log` |
| Phase 02 profile list probe | `target/p20-phase02/logs/02_profile_list.log` |
| Phase 02 mock provider probe | `target/p20-phase02/logs/03_provider_check_mock.log` |
| Phase 02 package readiness probe | `target/p20-phase02/logs/04_package_readiness.log` |
| Phase 02 final P20 scan | `target/p20-phase02/scan-post-report/p20_scan.md` |
| Phase 09 hostile reference test log | `target/p20-phase09/logs/02_phase09_reference_hostile_tests.log` |

## Active Claim Corrections

| Claim area | Previous issue | Correct label | Evidence | Action |
|---|---|---|---|---|
| Root README | Read as a P20 Codex handoff packet instead of project truth | `partial`, `adapter/delegated`, `supported` only where tested | Phase 01 logs; CLI probes | Rewritten as project README with support matrix and limitations |
| P00-P19 status ledger | Used broad pass-state wording for many advanced surfaces | `partial`, `adapter/delegated`, `scaffold-only`, `deferred` | Tests and package readiness output | Rewritten as conservative P20 status ledger |
| Source basis | Described archive-generation environment and stale static limitations | `supported` for manifest/build resolution only | Phase 00 metadata/tree; Phase 01 logs | Rewritten to local workspace/source-basis facts |
| Issue matrices | Mixed historical P00-P19 tasks with active next work | `supported` for Phases 00-01; `deferred` for Phases 03-10 | Phase reports and P20 acceptance gates | Rewritten as active P20 matrix |
| Audit docs | Some audit files said they were current while describing old scaffold state | historical evidence, not active truth | Phase 01 logs and current probes | Rewritten as historical or current P20 Phase 02 audit notes |
| Provider capability | Cloud and native tool support could be inferred from older provider language | `fixture-supported-not-cloud` mock, `partial-local-chat` Ollama, `deferred/unavailable` cloud/native loops | provider tests; `provider-check` probe; `docs/p20/PROVIDER_CAPABILITY_MATRIX.md` | README/STATUS now state native loop false unless executable and tested |
| Agency governance | Phase 08 added a boundary policy crate and runner gate | `partial/proved` | `aidens-agency-kit` tests; runner Phase 08 tests; Phase 08 report | Keep scoped to tested final-output/tool-output paths |
| Reference semantics | Phase 09 closed the deferred temporal reference branch and added hostile semantic tests | `partial/proved` | `crates/aidens-testkit/src/lib.rs`; `crates/aidens-integration-tests/tests/phase_09_reference_hostile_tests.rs`; Phase 09 report | Keep scoped to tested supported/delegated surfaces |
| Scaffold crates | Older docs could imply product surface readiness | `scaffold-only` / `deferred` | doctor/scaffold guard script | STATUS lists the scaffold-only profile crates |

## Current Active Support Labels

| Surface | Label | Evidence |
|---|---|---|
| Build gate | `supported` | Phase 01 fmt/check/test/clippy/verify logs |
| Mock provider local run | `fixture-supported-not-cloud` | `aidens-runner`, `aidens-provider-kit`, and CLI tests; Phase 02 mock probe |
| Provider truth reporting | `supported` | provider matrix/readiness tests; `provider-check` output |
| CLI fixture workflows | `supported` for tested local fixtures | `aidens-cli` tests and `scripts/verify.sh` |
| Safe read-only tools | `supported` | tool-kit tests and runner tool-loop tests |
| Permit-gated side-effect tools | `partial` | permit/tool tests; operator permit still required |
| Receipts | `partial` | durable log tests; canonical crates own payload semantics |
| Memory/runtime | `adapter/delegated` and `partial` | memory/runtime tests; canonical memory crates own truth |
| Kernel/governance/repair/federation/mechanism helpers | `adapter/delegated` and `partial` | adapter/helper tests; canonical crates own semantics |
| Cloud providers | `deferred/unavailable` | provider readiness tests |
| Native provider tool loops | `deferred` | provider tests assert false |
| Agency/influence governance | `partial/proved` | Phase 08 agency kit and runner gate tests |
| Reference interpreter and hostile semantic surfaces | `partial/proved` | Phase 09 temporal reference interpreter and hostile semantic tests |
| `aidens-plan-kit` | `partial/execution-plan-assembly-only` | plan-kit tests; Phase 04 plan compile/validate proof |
| `aidens-profile-daemon`, `aidens-profile-desktop`, `aidens-profile-memory`, `aidens-profile-research` | `scaffold-only` | STATUS crate table; scaffold guard |

## Verification Performed In Phase 02

| Command | Result | Log |
|---|---|---|
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase02/scan-initial` | ran; high findings remained | `target/p20-phase02/logs/01_p20_scan_initial.log` |
| `cargo run -q -p aidens-cli -- profile list` | pass | `target/p20-phase02/logs/02_profile_list.log` |
| `cargo run -q -p aidens-cli -- provider-check --config examples/aidens.mock.toml` | pass | `target/p20-phase02/logs/03_provider_check_mock.log` |
| `cargo run -q -p aidens-cli -- package readiness --root . --config examples/aidens.mock.toml` | pass | `target/p20-phase02/logs/04_package_readiness.log` |
| `bash scripts/assert_no_scaffold_promoted.sh .` | pass after report creation | `target/p20-phase02/logs/12_assert_no_scaffold_promoted_post_report.log` |
| `bash scripts/assert_docs_source_basis_current.sh` | pass after report creation | `target/p20-phase02/logs/13_assert_docs_source_basis_post_report.log` |
| `bash scripts/assert_docs_match_cargo.sh .` | pass after report creation | `target/p20-phase02/logs/14_assert_docs_match_cargo_post_report.log` |
| `bash scripts/assert_no_fake_completion.sh .` | pass after report creation | `target/p20-phase02/logs/15_assert_no_fake_completion_post_report.log` |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase02/scan-post-report` | ran; high findings remain for later phases | `target/p20-phase02/logs/16_p20_scan_post_report.log` |
| Active-doc forbidden phrase scan | 0 matches | `target/p20-phase02/logs/17_active_docs_forbidden_phrase_post_report.log` |

Final scan summary:

```text
public_types: 194
medium public type hints: 6
pattern findings: 931
high pattern findings: 47
active docs overclaim candidates in Phase 02 patched files: 0
```

Remaining high scan findings are code/test marker hits for negative implementation markers inside `crates/`; they are not fixed in Phase 02 because the user asked for documentation honesty only. Phase 04 owns scanner integration and hard-finding policy.

## Remaining Risks

- `aidens-contracts` still has six medium scanner public-type hints requiring Phase 03 ownership inventory:
  - `AttestationVerificationStatusV1`
  - `StopRuleEvidenceV1`
  - `ResidualV1`
  - `SyndromeKindV1`
  - `SyndromeV1`
  - `JsonRepairReportV2`
- P20 scanner still needs Phase 04 hardening so historical docs and test marker fixtures can be classified without noisy false positives.
- P20 agency/influence governance is wired for tested runner final-output/tool-output paths as of Phase 08; broader product coverage remains scoped.
- Phase 09 reference/hostile coverage is scoped to temporal/as-of behavior, bridge digest/backpointer atomicity, provider truth, agency decisions, boundary repair integrity, runtime widening disclosure, and repair-record invariants.
- Final P20 audit bundle has not been generated.

## Phase 02 Gate Result

Phase 02 documentation truth gate: `PASS`.

The active docs listed in this report now use evidence-backed labels and disclose limitations. This does not advance P20 beyond Phase 02.
