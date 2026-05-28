# P20 Source Basis

## Static basis used to create this packet

- User-supplied source archive: `libraries-source-clean-20260429.zip`.
- User-supplied research archive: `Full Provenance+ Research 4/26/26.zip`.
- New agency/manipulation research file: `manipulation.md`.
- Extracted source root observed in this environment: `/mnt/data/aidens_latest`.

## Observed workspace shape

The snapshot contains a Rust workspace with these AiDENs crates:

```text
crates/aidens
crates/aidens-contracts
crates/aidens-boundary-kit
crates/aidens-config
crates/aidens-receipts
crates/aidens-capability-kit
crates/aidens-provider-kit
crates/aidens-tool-kit
crates/aidens-security-kit
crates/aidens-permit-kit
crates/aidens-arbiter-kit
crates/aidens-budget-kit
crates/aidens-governance-kit
crates/aidens-schedule-kit
crates/aidens-queue-kit
crates/aidens-wake-kit
crates/aidens-daemon-kit
crates/aidens-memory-kit
crates/aidens-kernel-kit
crates/aidens-delegation-kit
crates/aidens-plan-kit
crates/aidens-repair-kit
crates/aidens-runner
crates/aidens-app-kit
crates/aidens-cli
crates/aidens-testkit
crates/aidens-profile-coding
crates/aidens-profile-research
crates/aidens-profile-desktop
crates/aidens-profile-daemon
crates/aidens-profile-memory
```

## Critical source-basis caveat

The archive is not self-contained if sibling path dependencies are absent. The real build expects canonical crates next to AiDENs under the broader Libraries root, including `stack-ids`, `semantic-memory`, `semantic-memory-forge`, `forge-memory-bridge`, `knowledge-runtime`, kernel crates, and `verification-*` crates.

P20 must report whether the workspace was built in the real sibling-crate layout or from an incomplete archive.

## P20 source-basis proof requirement

Generate:

```bash
cargo metadata --format-version=1 > target/aidens-final-audit/cargo-metadata.json
cargo tree --workspace > target/aidens-final-audit/cargo-tree.txt
```

If either command fails due to missing sibling crates, P20 cannot claim build certification.
