# Crate Hardening Matrix

<!-- LIB-002: Explicit per-crate hardening status to prevent over-reading the closeout receipt. -->

Generated: 2026-03-30

## Legend

| Column | Meaning |
|---|---|
| **Build** | `cargo check --workspace` passes |
| **Release** | In the 17-crate supported closeout lane (release/closeout_receipt_v1.json) |
| **Panic** | Zero production `unwrap`/`expect`/`panic`/`todo`/`unsafe` in src (scripts/check_no_prod_panics.sh) |
| **Doc** | Public API rustdoc coverage tracked and passing (scripts/check_public_api_docs.py) |
| **Bench** | Covered by evidence/perf_baseline_20260330.json |
| **Lints** | Workspace-level deny lints enforced (LIB-005) |

## Matrix

| Crate | Build | Release | Panic | Doc | Bench | Lints | Lane |
|---|---|---|---|---|---|---|---|
| assurance-runtime | Y | — | Y | — | — | Y | governance |
| attestation-exchange | Y | — | Y | — | — | Y | governance |
| authority-delegation | Y | — | Y | — | — | Y | governance |
| constitutional-memory | Y | — | Y | — | — | Y | governance |
| constraint-compiler | Y | — | Y | Y | — | Y | supported (via forge-pilot) |
| continuity-runtime | Y | — | Y | — | — | Y | governance |
| contract-schema-gen | Y | Y | Y | — | — | Y | supported |
| discovery-portfolio | Y | — | Y | — | — | Y | extension (v18) |
| effect-runtime | Y | — | Y | Y | — | Y | governance |
| federated-settlement | Y | — | Y | — | — | Y | extension (v16) |
| forge-memory-bridge | Y | Y | Y | — | Y | Y | supported |
| forge-pilot | Y | Y | Y | Y | Y | Y | supported |
| kernel-conformance | Y | Y | Y | — | Y | Y | supported |
| kernel-execution | Y | Y | Y | — | Y | Y | supported |
| kernel-oracles | Y | Y | Y | — | — | Y | supported |
| knowledge-runtime | Y | Y | Y | — | — | Y | supported |
| living-memory/living-memory | Y | Y | Y | — | — | Y | supported |
| llm-tool-runtime | Y | Y | Y | — | — | Y | supported |
| mechanism-runtime | Y | — | Y | — | — | Y | governance |
| profile-runtime | Y | — | Y | Y | — | Y | extension (v25) |
| recursive-kernel-core | Y | Y | Y | Y | — | Y | supported |
| remote-oracle-admission | Y | — | Y | — | — | Y | extension |
| semantic-memory | Y | Y | Y | — | — | Y | supported |
| semantic-memory-forge | Y | Y | Y | Y | — | Y | supported |
| spec-execution | Y | — | Y | — | — | Y | extension (v20) |
| stack-ids | Y | Y | Y | — | — | Y | supported |
| verification-adjudication | Y | Y | Y | — | — | Y | supported |
| verification-calibration | Y | Y | Y | — | — | Y | supported |
| verification-control | Y | Y | Y | Y | — | Y | supported |
| verification-policy | Y | Y | Y | Y | — | Y | supported |

## Lane definitions

- **supported**: In the 17-crate closeout lane. Full release-gate coverage.
- **governance**: Build-checked and functionally tested. Not in the narrow closeout scope.
  Feature-gated behind `forge-pilot`'s `governance` feature flag.
- **extension**: Schema-wave crates that extend the contract surface. Build-checked, not
  release-gated. These are safe to use but are not covered by the closeout receipt.

## What this matrix does NOT cover

- Integration test coverage per crate (see individual crate test suites)
- Cross-crate regression coverage (see kernel-conformance property tests)
- Performance under release profile (LIB-006 — perf baseline is dev-profile only)
