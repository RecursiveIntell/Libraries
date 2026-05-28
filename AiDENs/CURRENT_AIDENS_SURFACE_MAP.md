# Current AiDENs Surface Map

This map treats current AiDENs as reference-only. Classification labels:

- `keep-app`: product/UX/app-shell can survive.
- `thin-adapter`: must delegate to canonical stack crates.
- `facade-only`: only re-export/composition; no semantics.
- `compat-only`: forbidden; retained here only as historical audit terminology.
- `rewrite`: dangerous duplicate semantics; rewrite around canonical owners.
- `delete/merge`: remove or merge into adapter/facade.
- `defer`: do not touch until the golden vertical slice and governance gates pass.

| Crate | Classification | Source reference | Current role | Required action |
|---|---|---|---|---|
| `aidens` | `facade-only` | `~/Coding/Libraries/AiDENs/crates/aidens/src/lib.rs` | top-level re-export/prelude | must re-export stack-backed adapters only; no truth |
| `aidens-app-kit` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-app-kit/src/lib.rs` | app plan/status facade | keep UX; remove any semantic authority |
| `aidens-arbiter-kit` | `rewrite` | `~/Coding/Libraries/AiDENs/crates/aidens-arbiter-kit/src/lib.rs` | local arbiter for contradiction decisions | replace with canonical Forge/verification surfaces |
| `aidens-boundary-kit` | `rewrite` | `~/Coding/Libraries/AiDENs/crates/aidens-boundary-kit/src/lib.rs` | boundary parsing/repair/compiler | thin adapter to llm-tool-runtime/contract-schema-gen/forge-memory-bridge; no meaning invention |
| `aidens-budget-kit` | `rewrite` | `~/Coding/Libraries/AiDENs/crates/aidens-budget-kit/src/lib.rs` | budget stop helper | map to verification-control/llm-tool-runtime budget lineage |
| `aidens-capability-kit` | `rewrite` | `~/Coding/Libraries/AiDENs/crates/aidens-capability-kit/src/lib.rs` | local capability truth | move to policy/delegation adapter; no local truth |
| `aidens-cli` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-cli/src/lib.rs` | CLI command inventory | keep command surface; route memory/receipts to adapters |
| `aidens-config` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-config/src/lib.rs` | app config | keep; ensure no canonical semantics encoded |
| `aidens-contracts` | `rewrite` | `~/Coding/Libraries/AiDENs/crates/aidens-contracts/src/lib.rs` | large local artifact taxonomy | collapse to app-only contracts + re-exports only |
| `aidens-daemon-kit` | `defer` | `~/Coding/Libraries/AiDENs/crates/aidens-daemon-kit/src/lib.rs` | daemon facade over local queue | defer until golden slice/governance; no truth |
| `aidens-delegation-kit` | `thin-adapter` | `~/Coding/Libraries/AiDENs/crates/aidens-delegation-kit/src/lib.rs` | delegation/admission | adapter to authority-delegation + verification-policy |
| `aidens-governance-kit` | `thin-adapter` | `~/Coding/Libraries/AiDENs/crates/aidens-governance-kit/src/lib.rs` | local governance/promotion | adapter to verification-control/policy/adjudication/assurance |
| `aidens-kernel-kit` | `thin-adapter` | `~/Coding/Libraries/AiDENs/crates/aidens-kernel-kit/src/lib.rs` | kernel facade | adapter to recursive-kernel-core/constraint-compiler/kernel-execution/kernel-oracles |
| `aidens-memory-kit` | `rewrite` | `~/Coding/Libraries/AiDENs/crates/aidens-memory-kit/src/lib.rs` | local append-only memory plus some canonical imports | replace authoritative behavior with semantic-memory-forge -> bridge -> semantic-memory -> knowledge-runtime adapter |
| `aidens-permit-kit` | `thin-adapter` | `~/Coding/Libraries/AiDENs/crates/aidens-permit-kit/src/lib.rs` | permit policy | adapter to verification-policy/llm-tool-runtime permits |
| `aidens-plan-kit` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-plan-kit/src/lib.rs` | small app plan helper | keep if app-only |
| `aidens-profile-coding` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-profile-coding/src/lib.rs` | profile preset | keep app/profile intent only |
| `aidens-profile-daemon` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-profile-daemon/src/lib.rs` | profile preset | keep app/profile intent only |
| `aidens-profile-desktop` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-profile-desktop/src/lib.rs` | profile preset | keep app/profile intent only |
| `aidens-profile-memory` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-profile-memory/src/lib.rs` | profile preset | keep app/profile intent only |
| `aidens-profile-research` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-profile-research/src/lib.rs` | profile preset | keep app/profile intent only |
| `aidens-provider-kit` | `thin-adapter` | `~/Coding/Libraries/AiDENs/crates/aidens-provider-kit/src/lib.rs` | provider surface | adapter to llm-tool-runtime provider rendering + receipts |
| `aidens-queue-kit` | `defer` | `~/Coding/Libraries/AiDENs/crates/aidens-queue-kit/src/lib.rs` | local durable queue | defer; evaluate libraries2/job-queue later; no truth |
| `aidens-receipts` | `rewrite` | `~/Coding/Libraries/AiDENs/crates/aidens-receipts/src/lib.rs` | receipt ledger with canonical aliases and many local envelope types | make thin adapter/sink around llm-tool-runtime + semantic-memory-forge + verification-control |
| `aidens-repair-kit` | `rewrite` | `~/Coding/Libraries/AiDENs/crates/aidens-repair-kit/src/lib.rs` | local repair helper over local memory | replace with canonical repair/verification/memory adapter |
| `aidens-runner` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-runner/src/lib.rs` | turn/session runner | keep orchestrator; force canonical receipts/tool runtime |
| `aidens-schedule-kit` | `defer` | `~/Coding/Libraries/AiDENs/crates/aidens-schedule-kit/src/lib.rs` | one-shot schedule helpers | defer until phase 7 |
| `aidens-security-kit` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-security-kit/src/lib.rs` | security UX/helpers | keep only app-level checks; no policy authority |
| `aidens-testkit` | `keep-app` | `~/Coding/Libraries/AiDENs/crates/aidens-testkit/src/lib.rs` | fixtures/tests | expand with canonical proof tests |
| `aidens-tool-kit` | `thin-adapter` | `~/Coding/Libraries/AiDENs/crates/aidens-tool-kit/src/lib.rs` | tool dispatch helpers | adapter to llm-tool-runtime; no local receipt truth |
| `aidens-wake-kit` | `defer` | `~/Coding/Libraries/AiDENs/crates/aidens-wake-kit/src/lib.rs` | wake signal construction | defer until phase 7 |
