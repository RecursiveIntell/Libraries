
# Current code snapshot notes — 2026-03-15

## What matters for this pass

The current workspace already includes the owner crates needed to treat v24 as the terminal general-purpose base horizon, including:
- `effect-runtime`
- `authority-delegation`
- `assurance-runtime`
- `continuity-runtime`
- `attestation-exchange`
- `remote-oracle-admission`
- `federated-settlement`
- `mechanism-runtime`
- `spec-execution`

The repo also already contains:
- `20_TERMINAL_DESIGN_POSITION.md`, which states that growth beyond v24 should default to profile overlays;
- `21_PROFILE_BACKLOG_AFTER_V24.md`, which names the candidate profile lanes explicitly;
- `verification-policy/src/lib.rs`, which already exposes base policy-profile structs for effect, delegation, release, and continuity;
- existing v21–v24 schemas and examples;
- and final-closeout documents that treat the v21–v24 wave as the end of the general-purpose line.

## Design consequence

The next phase should use existing owners and extend the profile layer.
It should not manufacture a new base-spec wave.
