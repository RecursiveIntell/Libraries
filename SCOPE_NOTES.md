# Scope notes — adjacent artifact-owner crates

The hardening receipt (`release/closeout_receipt_v1.json`) certifies exactly the 17 crates listed in `SUPPORT_PROFILE.md`.
The crates below were removed from `SUPPORT_PROFILE.md` because they are adjacent artifact-owner crates for the demo/benchmark substrate, not build-certified members of the hardening receipt.
Their compatibility names are retained for continuity, but the naming-credibility decision table lives in `docs/closeout_v21_v24/governance_surface_decision_table.md`.

## Adjacent crates outside the build-certified lane

- `assurance-runtime`
- `attestation-exchange`
- `authority-delegation`
- `constitutional-memory`
- `continuity-runtime`
- `discovery-portfolio`
- `effect-runtime`
- `federated-settlement`
- `mechanism-runtime`
- `profile-runtime`
- `remote-oracle-admission`
- `spec-execution`

## Scope rule

These crates own real artifact surfaces used by `DEMO-001` and `BENCH-001`.
They are not build-certified by the 2026-03-22 hardening receipt.

Use `SUPPORT_PROFILE.md` for the narrow 17-crate support claim.
Use this file for the adjacent artifact-owner substrate that supports the demo and benchmark packages.
These adjacent crates are also outside the public-doc-certified core tracked by `python3 scripts/check_public_api_docs.py`.

## V27 governance integration status (2026-03-25)

The governance crates listed above now have:
- Typed validation error enums (following `effect-runtime` pattern with `thiserror`)
- Integration Points and Artifact Families documentation in `lib.rs`
- Build-checked status via `cargo check --workspace`
- Feature-gated integration into `forge-pilot` via `governance_gate.rs` (`#[cfg(feature = "governance")]`)

They remain outside the build-certified lane pending full Stack Arena regression verification with governance enabled.

## attestation-exchange integration gap (V29, 2026-03-30)

The `attestation-exchange` crate is wired into the workspace and compiles, but its artifacts are not yet consumed by the governance observation pipeline in `forge-pilot/src/governance_gate.rs`. The `observe_governance()` function reads effect, assurance, authority, continuity, constitutional, and mechanism state — but does not yet read attestation exchange state.

This is a known forward declaration. The attestation surface is planned for V2 of the governance pipeline. The current six-predicate observation scope is sufficient for CLARA V1 submission because attestation exchange state does not gate any execution decision in the current OODA loop.

See also: `forge-pilot/src/governance_gate.rs` module docs, "Not yet observed" section.
