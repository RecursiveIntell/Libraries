# P20 Acceptance Gates

Each phase has a hard pass/fail gate. A phase may not advance until its gate passes or the operator explicitly accepts a documented quarantine/failure.

## Global gates

- No shadow truth.
- No silent semantic widening.
- No compatibility-layer invention.
- No fake provider capability.
- No scaffold promotion.
- No untested completion claims.
- No phase transition without invariant revalidation.
- No missing final audit bundle.

## Phase gates

| Phase | Gate |
|---|---|
| 00 | Arbitration artifact and baseline plan created; repo/source basis confirmed |
| 01 | `cargo fmt/check/test/clippy` and repo verify either pass or failures are fixed/quarantined with docs corrected |
| 02 | README/STATUS/source docs match actual code and tests |
| 03 | Contract ownership inventory exists; no unaddressed duplicate/ambiguous canonical concepts |
| 04 | Static scanner integrated into `scripts/p20_verify.sh`; machine-readable reports emitted |
| 05 | Provider matrix matches executable support; unsupported providers cannot imply native tool support |
| 06 | End-to-end runner fixture proves config → provider → tool → result → receipts |
| 07 | Canonical adapter tests prove delegation for memory/kernel/verification/repair surfaces claimed |
| 08 | Agency/influence gate exists in runner path; evals pass; receipts emitted |
| 09 | Reference interpreters implemented or feature claims demoted; hostile tests pass |
| 10 | Final audit bundle generated; known limitations and unresolved risks explicit |

## Required final commands

```bash
bash scripts/p20_verify.sh
bash scripts/p20_generate_audit_bundle.sh
```

If either fails, P20 is not complete.
