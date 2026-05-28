# P30-00 Report

## Scope

Phase slice: build certification, workspace portability, and package integrity.

Matrix inventory from `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`:

- 5 total P30-00 rows.
- Priority split: 4 P0, 1 P1.
- Categories: `BUILD-CERTIFICATION` 1, `WORKSPACE-PORTABILITY` 2, `PACKAGE-INTEGRITY` 2.

Issue IDs addressed by current-run evidence:

- `P30-ABSORB-0001`: the audit environment could not run Cargo, but this current environment can. Multiple cargo commands passed in this session, including `cargo check` and targeted tests.

Issue IDs quarantined as remaining debt:

- `P30-ABSORB-0010`: the nested AiDENs workspace still depends on sibling crates outside the AiDENs directory. This is a real portability constraint, not fixed here.
- `P30-ABSORB-0012`: package certifier evidence still proves package policy, not semantic/build correctness. Current cargo evidence narrows the risk for this run but does not change the package report semantics.
- `P30-ABSORB-0013`: archive hash semantics remain zip-byte hash semantics, not canonical content identity.
- `P30-ABSORB-0148`: nested `.cargo/config.toml` plus sibling path dependencies remains a workspace-assumption risk.

## Changed Files

No P30-00 code changes were made.

## Tests Added Or Updated

No P30-00-specific tests were added.

## Commands Run

Current-run build/cargo evidence from this session includes:

- `cargo check --manifest-path Cargo.toml -p aidens-runner --all-targets --locked`
  - Result: pass.
- `cargo check --manifest-path Cargo.toml -p aidens-cli -p aidens-contracts -p aidens-provider-kit --all-targets --locked`
  - Result: pass.
- `cargo test --manifest-path Cargo.toml -p aidens-runner p30_`
  - Result: pass, 2 tests passed.
- `bash scripts/p30_verify.sh .`
  - Result: pass, `cargo metadata OK`, `static verification completed`, `findings=1841 hard=0`.

## Unresolved Risks And Quarantines

- The current checkout is not a self-contained AiDENs-only workspace. Sibling crate path dependencies are still required.
- Existing package reports should not be read as semantic/build certification.
- Existing archive hashes should not be read as canonical content identity.
- No portability packaging redesign was attempted.

## Invariant Revalidation Checklist

- Cargo is available and usable in the current environment.
- Current evidence is build/check evidence, not release conformance evidence.
- Package-integrity semantics are not overclaimed.
- No v11A/v11B compliance claim is made from this phase.

## Proceed Statement

P30-00 can proceed for `P30-ABSORB-0001` based on current cargo evidence. The workspace portability and package-integrity rows remain explicit debt and must constrain release/package claims.
