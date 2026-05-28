# P29 Operator Paste-First

Paste this before starting Codex.

---

You are executing **P29 — AiDENs Evidence Repair + v11A Local Release Candidate + v11B Executable Seed** in the AiDENs repository.

## Mission

Repair the failed P28 evidence/package boundary, finish v11A local release-candidate coverage for the supported-local agent path, and enter v11B territory only as an executable seed.

## The P28 failure cannot repeat

P28 advanced code, but the final package was not trustworthy because:

- archive sidecar/run identity drifted;
- P28 docs/handoffs/scripts were archived as stale;
- the active verifier wrapper pointed at a missing verifier;
- evidence manifest referenced files not included in the package;
- package validation passed despite broken extracted-package verification.

P29 must start by repairing these gates.

## Non-negotiable invariants

1. Current run identity must be P29 everywhere.
2. No active P29 docs, handoffs, scripts, or verifier files may be archived as stale.
3. `scripts/verify_current.sh` must delegate to an included, executable `scripts/p29_verify.sh`.
4. The final package must self-replay from an extracted zip.
5. Status/evidence manifest paths must resolve inside the package unless explicitly labeled external/degraded.
6. No material operation path may complete without operator contract, execution context, receipt, and proof/degradation state.
7. AiDENs must not become canonical memory/governance/kernel/provider/schema truth owner.
8. v11B work is seed-level only unless v11A local release gates are green.
9. v11C remains reserved/quarantined only.
10. Every phase must leave a phase report.

## Manual phase injections

Stop for manual operator gates only after:

- Phase 03,
- Phase 07,
- Phase 11,
- Phase 15,
- Phase 19,
- before final package generation.

Do not stop after every phase. Do not skip these gates.

## Final commands

Run and record:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
bash scripts/p29_verify.sh
python3 scripts/assert_p29_package_self_replay.py --package target/p29/package/AiDENs-p29-codex-context.zip
```

If any gate fails, repair before proceeding. Do not call P29 complete.
