# 03 — Implementation Phases

## Phase 0 — Workspace and contracts

Create/finish:

- `Cargo.toml`
- `aidens`
- `aidens-contracts`
- `aidens-boundary-kit`
- `aidens-config`
- `aidens-receipts`
- `aidens-capability-kit`
- `aidens-testkit`

Required output:

- schema-derivable `AiDENsAppPlanV1`, `AidensRunContextV1`, `RunReportV1`, `RuntimeCapabilityTruthV1`, `ToolExposureSetV1`.
- config redaction and validation.
- canonical receipt sink/log with append-only semantics.
- boundary repair receipt model.
- test fixtures for footguns.

## Phase 1 — Core agent runtime

Create/finish:

- `aidens-provider-kit`
- `aidens-tool-kit`
- `aidens-security-kit`
- `aidens-permit-kit`
- `aidens-arbiter-kit`
- `aidens-budget-kit`
- `aidens-runner`

Required output:

- provider route truth enum,
- native/parser-disabled mode model,
- tool descriptor/registry/exposure model,
- disabled means absent,
- basic permit model,
- route decision model,
- budget guard model,
- runner emits receipts even in mocked mode.

## Phase 2 — App creation speed layer

Create/finish:

- `aidens-app-kit`
- `aidens-cli`
- profile crates or built-in profiles.

Required output:

```bash
aidens new coding-agent my-agent
aidens doctor
aidens check-config
aidens list-tools
aidens provider-check
```

## Phase 3 — Recall extraction adapters

Integrate generalized code from `~/Coding/Recall`:

- provider bridge to `llm-pipeline`,
- `llm-tool-runtime` registry/dispatch adapters,
- config and path safety helpers,
- initial Recall tool bundles only as optional profile/application-specific bundles.

## Phase 4 — Memory / queue / daemon / UI

Create/finish:

- `aidens-memory-kit`
- `aidens-queue-kit`
- `aidens-schedule-kit`
- `aidens-wake-kit`
- `aidens-daemon-kit`
- `aidens-tauri-kit`

Do not start these before Phase 0/1 footgun tests pass.

## Phase 5 — Advanced runtime geometry

Create/finish:

- `aidens-kernel-kit`
- `aidens-delegation-kit`
- `aidens-plan-kit`
- `aidens-repair-kit`
- future `aidens-federation-kit`
- future `aidens-mechanism-kit`

These should follow the right-graph and small-region laws.
