# AiDENs — P32+ Completion Status

**Status**: P32+ with P11 regional decoder and semantic-memory wiring complete

**Run**: P32 | **Role**: schema-compatibility | **Status**: candidate | **Label**: p32-schema-compat-candidate

All 18 verification gates pass. 470 tests, 0 failures.

**Last certified run**: P30 — see `docs/codex-runs/CURRENT_RUN.json`.

## Quick Links

- Current run state: `docs/codex-runs/CURRENT_RUN.json`
- Phase completion: `COMPLETION_BLUEPRINT_P32.md`

## Gate Results

| Gate | Result |
|---|---|
| release_ledger_schema | PASS |
| current_run_truth | PASS |
| release_truth_consistency | PASS |
| root_markdown_archive_policy | PASS |
| codex_artifact_classification | PASS |
| support_claims_have_evidence | PASS |
| no_fake_completion | PASS |
| no_shadow_truth | PASS |
| adapter_delegation | PASS |
| tool_runtime_delegation | PASS |
| no_canonical_type_duplicates | PASS |
| no_local_substitute_dependencies | PASS |
| p30_guard (0 hard) | PASS |
| cargo_metadata | PASS |
| cargo_fmt | PASS |
| cargo_check | PASS |
| cargo_test | PASS |
| cargo_clippy | PASS |

## Crate Inventory

| Crate | Status |
|---|---|
| `aidens` | implemented |
| `aidens-agency-kit` | implemented |
| `aidens-app-kit` | implemented |
| `aidens-arbiter-kit` | implemented |
| `aidens-boundary-kit` | implemented |
| `aidens-budget-kit` | implemented |
| `aidens-capability-kit` | implemented |
| `aidens-cli` | implemented |
| `aidens-config` | implemented |
| `aidens-contracts` | implemented |
| `aidens-daemon-kit` | implemented |
| `aidens-delegation-kit` | implemented (quarantined) |
| `aidens-governance-kit` | implemented |
| `aidens-integration-tests` | implemented |
| `aidens-kernel-kit` | implemented |
| `aidens-memory-kit` | implemented |
| `aidens-permit-kit` | implemented |
| `aidens-plan-kit` | implemented |
| `aidens-profile-coding` | implemented |
| `aidens-profile-daemon` | scaffold-only (honest) |
| `aidens-profile-desktop` | scaffold-only (honest) |
| `aidens-profile-memory` | scaffold-only (honest) |
| `aidens-profile-research` | scaffold-only (honest) |
| `aidens-provider-kit` | implemented |
| `aidens-queue-kit` | implemented |
| `aidens-receipts` | implemented |
| `aidens-repair-kit` | implemented |
| `aidens-runner` | implemented |
| `aidens-schedule-kit` | implemented |
| `aidens-security-kit` | implemented |
| `aidens-testkit` | implemented |
| `aidens-tool-kit` | implemented |
| `aidens-wake-kit` | implemented |
| `boundary-compiler-core` | implemented |

## Key Repairs

1. Decertified prior false claims; set P31B candidate
2. Fixed z.py `normalize_codex_run_id` regex for letter-suffix run IDs
3. Added manifest `run` field; updated validation script for normalized comparison
4. Fixed `GENERATED_PACKAGE_RE` timestamp regex and `FINISH_PACK_RE` exclusion
5. Classifying 659 prior artifacts as run-evidence
6. Restored `SHADOW_SEMANTICS_AUDIT.md` to root; added sibling deps to `SOURCE_BASIS.md`
7. Self-replay script: added `PermissionError` environmental blocker classification

## Source Basis Declarations

This clean source bundle is accepted as source basis. STATUS.md is not a product-conformance or release-package claim. All zip-byte hashes are verified via the receipt chain.