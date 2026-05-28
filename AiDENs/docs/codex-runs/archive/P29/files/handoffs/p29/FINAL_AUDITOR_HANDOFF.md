# P29 Final Auditor Handoff

Status: Phase 21 pre-package draft. Final auditor handoff remains incomplete until strict package sidecars, extracted package self-replay, known limitations, and unresolved risks are recorded after Injection 6.

## Audit Posture

P29 has passed manual gates after Phases 03, 07, 11, 15, and 19. The current state is a pre-final release candidate posture:

- P28 evidence/package failures have explicit postmortem and verifier coverage.
- Runtime and contract audit items are fixed or quarantined with phase evidence.
- v11A evidence is present only for the declared supported-local `run-coding-agent` path.
- v11B graph/region/subtraction work is executable seed only.
- v11C remains reserved-only.

## Required Final Evidence

Pre-package command evidence has passed:

- `cargo fmt --all -- --check`: `target/p29/audit/phase21_cargo_fmt_check.log`
- `cargo check --workspace --all-targets`: `target/p29/audit/phase21_cargo_check.log`
- `cargo test --workspace --all-targets`: `target/p29/audit/phase21_cargo_test.log`
- `cargo clippy --workspace --all-targets -- -D warnings`: `target/p29/audit/phase21_cargo_clippy.log`
- `cargo doc --workspace --no-deps`: `target/p29/audit/phase21_cargo_doc.log`
- `bash scripts/p29_verify.sh`: `target/p29/audit/phase21_p29_verify.log`
- `bash scripts/verify_current.sh`: `target/p29/audit/phase21_verify_current.log`

Still required before any release claim:

- `python3 scripts/assert_p29_package_self_replay.py --package target/p29/package/AiDENs-p29-codex-context.zip`

## Package Targets

The final package targets remain pending:

- `target/p29/package/AiDENs-p29-codex-context.zip`
- `target/p29/package/AiDENs-p29-codex-context.report.md`
- `target/p29/package/AiDENs-p29-codex-context.manifest.json`
- `target/p29/package/AiDENs-p29-codex-context.findings.json`
- `target/p29/package/AiDENs-p29-codex-context.excluded.json`

## Auditor Warning

Do not accept a final claim if any package sidecar is missing, if manifest paths do not resolve or classify as explicitly external/degraded, if `scripts/verify_current.sh` stops delegating to `scripts/p29_verify.sh`, or if any doc/script/handoff classifies active P29 material as stale.
