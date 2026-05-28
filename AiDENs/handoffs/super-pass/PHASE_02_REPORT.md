# PHASE 02 REPORT — Security boundary hardening

## Scope

- Backlog rows selected: 70 rows, `AHD-0001` through `AHD-0070`, all with `Suggested_Phase = Phase 02 security boundary hardening`.
- Files/crates touched: `crates/aidens-security-kit/src/lib.rs`, `crates/aidens-tool-kit/src/lib.rs`, `matrices/SUPER_PASS_BACKLOG_1020.csv`, `matrices/SUPER_PASS_BACKLOG_1020.json`.
- Non-goals: this phase does not add cloud/broad-autonomy policy; it hardens supported-local sandbox path enforcement.

## Changes made

| Area | Files | Summary |
|---|---|---|
| Sensitive path policy | `crates/aidens-security-kit/src/lib.rs` | Denies `.git`, `.git-credentials`, `.env`, `.npmrc`, `.aws`, `.ssh`, `.gnupg`, `.cargo`, `.recall`, `.password-store`, and private-key component names. Generic hidden components now fail closed. |
| Path redaction | `crates/aidens-security-kit/src/lib.rs`, `crates/aidens-tool-kit/src/lib.rs` | Sandbox escape errors no longer disclose the host sandbox root; absolute path requests fail without echoing the absolute path; display fallback returns `<sandbox>/name` instead of host paths. |
| Symlink/hardlink hostile cases | `crates/aidens-tool-kit/src/lib.rs` | Existing symlink rejection coverage was extended with symlink escape assertions; Unix hardlinked file targets are rejected for read and write surfaces. |
| Receipt-bearing denials | `crates/aidens-tool-kit/src/lib.rs` | Sandbox denial reason mapping now distinguishes traversal, sensitive prefix, hidden component, hardlink, and escape cases. |

## Tests/commands run

| Command | Result | Evidence/log path |
|---|---|---|
| `cargo test -p aidens-security-kit` | pass | `target/super-pass/audit/phase02-cargo-test-aidens-security-kit.log` |
| `cargo test -p aidens-tool-kit` | pass | `target/super-pass/audit/phase02-cargo-test-aidens-tool-kit.log` |
| `cargo check -p aidens-security-kit -p aidens-tool-kit` | pass | `target/super-pass/audit/phase02-cargo-check-security-tool.log` |

## Issue matrix updates

| Status | Count | IDs |
|---|---:|---|
| fixed | 70 | `AHD-0001` through `AHD-0070` |
| quarantined | 0 |  |
| deferred | 0 |  |
| superseded | 0 |  |
| open-blocking | 0 |  |

## Gate result

- Phase gate: Sandbox/security gate.
- Result: Pass for the scoped local tool sandbox surfaces. Hostile fixtures cover credential paths, symlink escape, hardlink read denial, redacted escape errors, and receipt-bearing denial reason codes.
- Remaining risk: Unicode normalization and cross-platform case-folding remain policy-level coverage on this Linux run; final release still needs the full workspace and package replay gates.

## Notes for next phase

Phase 03 can build on the distinct denial reason codes to prove declared, registered, executable, exposed, and permitted tool sets stay aligned.
