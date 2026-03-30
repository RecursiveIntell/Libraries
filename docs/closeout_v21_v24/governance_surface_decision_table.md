# Governance Surface Decision Table

| Crate | Role | Build Status | Doc Coverage | Feature Gate |
|-------|------|-------------|-------------|--------------|
| `effect-runtime` | Effect lifecycle artifacts, validators, compensation | Build-checked | Doc-certified | `governance` |
| `assurance-runtime` | Assurance case, certification, deployment readiness | Build-checked | - | `governance` |
| `continuity-runtime` | Incident management, recovery, SLO, error budget | Build-checked | - | `governance` |
| `mechanism-runtime` | Mechanism bundles, theory, fit runs | Build-checked | - | `governance` |
| `authority-delegation` | Delegation chains, approval, capability leases | Build-checked | - | `governance` |
| `attestation-exchange` | Vendor trust, certification adapters | Build-checked | - | `governance` |
| `constitutional-memory` | Amendments, effective constitutions | Build-checked | - | `governance` |
| `profile-runtime` | Constitutional composition engine (adapters.rs) | Build-checked | Doc-certified | Always |
| `forge-pilot` | OODA loop, governance gate, observation, receipts | Build-certified | Doc-certified | Always |

## Integration Surface

- `forge-pilot/src/governance_gate.rs` — Observes governance state from semantic-memory projections, gates execution
- `forge-pilot/src/loop_runner.rs` — Populates governance receipt in loop iteration reports, honors gate results
- `forge-pilot/src/observe.rs` — Calls `observe_governance()` during observation phase
- `profile-runtime/src/adapters.rs` — Projects governance profiles into `ObligationContributionV1` streams

## Decision: Default-Enabled (V28)

As of V28, the `governance` feature is enabled by default in `forge-pilot/Cargo.toml`. This ensures:
1. All governance crates compile in default builds
2. `cargo test` exercises governance observation, gating, and receipt generation
3. DARPA CLARA evaluators see governance traces in loop output
