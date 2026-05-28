# P31A Build Scope

## Purpose

This file prevents false build certification by declaring what this pass is responsible for building and why.

## Tier 1 — mandatory AiDENs workspace

Source: `Cargo.toml` workspace members.

- `crates/aidens`
- `crates/aidens-contracts`
- `crates/aidens-boundary-kit`
- `crates/aidens-config`
- `crates/aidens-receipts`
- `crates/aidens-capability-kit`
- `crates/aidens-provider-kit`
- `crates/aidens-tool-kit`
- `crates/aidens-security-kit`
- `crates/aidens-permit-kit`
- `crates/aidens-arbiter-kit`
- `crates/aidens-budget-kit`
- `crates/aidens-agency-kit`
- `crates/aidens-governance-kit`
- `crates/aidens-schedule-kit`
- `crates/aidens-queue-kit`
- `crates/aidens-wake-kit`
- `crates/aidens-daemon-kit`
- `crates/aidens-memory-kit`
- `crates/aidens-kernel-kit`
- `crates/aidens-delegation-kit`
- `crates/aidens-plan-kit`
- `crates/aidens-repair-kit`
- `crates/aidens-runner`
- `crates/aidens-app-kit`
- `crates/aidens-cli`
- `crates/aidens-testkit`
- `crates/aidens-integration-tests`
- `crates/aidens-profile-coding`
- `crates/aidens-profile-research`
- `crates/aidens-profile-desktop`
- `crates/aidens-profile-daemon`
- `crates/aidens-profile-memory`

Required gates: metadata, fmt, check, test, clippy.

## Tier 2 — direct path dependencies

Source: `[workspace.dependencies] path = "../..."`.

Examples: `semantic-memory`, `stack-ids`, `forge-memory-bridge`, `verification-*`, `kernel-*`, `llm-tool-runtime`, `contract-schema-gen`, etc.

Required gate: dependency paths must resolve. If full build includes them through Tier 1, failures are blockers. Do not hide failures by removing dependencies.

## Tier 3 — context-only roots

Source roots included in package for source context/audit but not owned by P31A unless pulled by Cargo.

Required gate: included in package manifest and source basis; not falsely certified as built unless build logs exist.

## Known blockers

| Blocker | Evidence | Effect on certification |
|---|---|---|
