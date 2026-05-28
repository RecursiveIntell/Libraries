# Crate Ownership and Boundary Map

## Boundary doctrine

- `aidens-contracts` owns only shared primitive contracts and versioned envelope shapes. It must not absorb domain behavior.
- `aidens-provider-kit` owns provider dialects and provider route truth. It does not own tools.
- `aidens-tool-kit` owns descriptors, exposure plans, dispatch, and tool receipts. It does not own provider execution.
- `aidens-receipts` owns durable execution evidence and receipt stores. It does not own domain truth.
- `aidens-memory-kit` owns episode-first bitemporal memory. It does not own runtime planning or provider calls.
- `aidens-runner` coordinates one run/turn. It must not become a durable truth store.
- `aidens-governance-kit` owns promotion/disposition decisions. It cannot self-promote advisory output.
- `aidens-repair-kit` owns repair records and contradiction transitions. It cannot delete truth silently.
- `aidens-kernel-kit` owns region/inference/subtraction algorithms. It is advisory unless governance promotes outputs.
- `aidens-daemon-kit`, `aidens-queue-kit`, `aidens-schedule-kit`, and `aidens-wake-kit` own execution scheduling and leases. They own execution history only.
- Profile crates own default configuration bundles, never hidden authority.

## Allowed dependency direction

```text
contracts <- all crates
security <- tool/app/runtime consumers
permit <- tool/governance/runtime consumers
tool <- runner/app/cli
provider <- runner/app/cli
receipts <- runner/tool/provider/memory/daemon/governance consumers
memory <- runner/governance/kernel/query consumers
kernel <- governance/runtime consumers, not contracts
```

## Forbidden dependency patterns

- Provider crate depends on tool crate to decide exposure.
- Tool crate depends on provider crate to claim native capability.
- Runner persists truth directly instead of going through memory/receipts.
- Memory invokes provider/tool execution.
- Governance mutates memory without receipt and explicit disposition.
