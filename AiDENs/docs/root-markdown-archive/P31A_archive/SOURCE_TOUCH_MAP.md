# Source Touch Map

This is a practical map for Codex. It names the current files that are most likely to change by pass.

## P00-P01: honesty and no-op/API parity

- `README.md`
- `AGENTS.md`
- `docs/*`
- `.github/workflows/ci.yml`
- `scripts/*`
- `crates/aidens-app-kit/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`

Known constraints:

- P00 owns the root README/status source-basis lock for the 2026-04-26 snapshot;
- `AiDENsApp::from_plan(...).build()` must not ignore provider-required semantics;
- `MemoryMode::Required` must fail doctor/run unless `[memory].store_root` is configured.

## P02-P03: provider/tool runtime

- `crates/aidens-provider-kit/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-budget-kit/src/lib.rs`
- `crates/aidens-contracts/src/lib.rs`

Known constraints:

- `ProviderCapabilitiesV1::advertised_by_kind()` needs separation from executable capabilities;
- provider backends beyond Ollama/mock are represented as unavailable;
- runner currently performs one completion call rather than a full tool loop.

## P04-P06: safety/evidence/boundaries

- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-permit-kit/src/lib.rs`
- `crates/aidens-security-kit/src/lib.rs`
- `crates/aidens-receipts/src/lib.rs`
- `crates/aidens-boundary-kit/src/lib.rs`
- `crates/aidens-contracts/src/lib.rs`

Known constraints:

- direct low-level runner construction without a configured durable store remains explicit `ReceiptLevelV1::Minimal`;
- `canonical_json_digest()` now uses SHA-256 over deterministic canonical JSON; keep new evidence digests cryptographic;
- boundary handling now rejects duplicate keys, emits schema-validation receipts, and records degraded repair provenance.

## P07-P09: schemas/reference/memory

- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-testkit/src/lib.rs`
- `crates/aidens-memory-kit/src/lib.rs`
- `schemas/*`
- `tests/fixtures/*`

Known constraints:

- `aidens schemas generate` and `aidens schemas check` own schema files under `schemas/<family>/vN.schema.json`;
- P08 reference interpreters live in `aidens-testkit` and production crates should compare against them from tests instead of copying production behavior;
- memory crate owns append-only episode storage and bitemporal as-of query semantics after P09.

## P10-P14: product-grade runtime and daemon/control

- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-security-kit/src/lib.rs`
- `crates/aidens-queue-kit/src/lib.rs`
- `crates/aidens-schedule-kit/src/lib.rs`
- `crates/aidens-wake-kit/src/lib.rs`
- `crates/aidens-daemon-kit/src/lib.rs`
- `crates/aidens-governance-kit/src/lib.rs`
- `crates/aidens-repair-kit/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`

Known constraints:

- queue/schedule/wake/daemon/governance/repair are scaffold-only;
- side-effect tool suite must be permit-gated and receipt-bearing;
- Codex packet generator does not yet exist as a canonical artifact.

## P15-P19: advanced completion horizon

- `crates/aidens-kernel-kit/src/lib.rs`
- `crates/aidens-repair-kit/src/lib.rs`
- `crates/aidens-delegation-kit/src/lib.rs`
- `crates/aidens-governance-kit/src/lib.rs`
- `crates/aidens-memory-kit/src/lib.rs`

Known constraints:

- kernel/delegation remain scaffold-only;
- advanced work must not begin until earlier passes make evidence, contracts, memory, and governance real.
