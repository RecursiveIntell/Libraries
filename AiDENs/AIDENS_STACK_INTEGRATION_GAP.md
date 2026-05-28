# AiDENs Stack Integration Gap

## Manifest gap

Current AiDENs root workspace dependencies include actual stack package names:
**15** canonical sibling crates at `Cargo.toml:L58-L72`.

The remaining gap is semantic, not manifest-level: any AiDENs surface must stay
an orchestration/config/reporting adapter over canonical crates. Major local
truth DTOs for IDs, memory/evidence, receipt envelopes, verification plans,
repair records, governance decisions, and kernel receipts have been collapsed or
removed.

## Correct first dependency

For any AiDENs crate needing canonical identity, use the workspace dependency:

```toml
stack-ids.workspace = true
```

The workspace root must continue to resolve that dependency to the canonical
`../stack-ids` sibling under `~/Coding/Libraries`. Do not point this dependency
to `libraries2`.

## Current doctrine

Symbol overlap is acceptable only for orchestration, CLI/UI/config surfaces,
adapter calls, and display reports. If a symbol implies memory, evidence,
receipt, verification, repair, kernel, or governance truth, it must be imported
from `~/Coding/Libraries`.

## Recommended order

1. `aidens-contracts` → canonical IDs.
2. `aidens-receipts` → canonical execution receipts/evidence.
3. `aidens-memory-kit` → forge/bridge/memory/runtime crates.
4. `aidens-kernel-kit` → recursive kernel crates.
5. Governance/permit/arbiter kits → verification and authority crates.
6. Runner/CLI/app kits → consumers, not truth owners.
