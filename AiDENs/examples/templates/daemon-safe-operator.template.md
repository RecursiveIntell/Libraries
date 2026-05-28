# Daemon-Safe Operator Template

Use this as a safe-mode checklist for AiDENs queue/schedule/wake experiments.

## Supported AiDENs Surfaces

- Profile: `autonomous-daemon` is partial/safe-mode
- Queue: durable queue facade through `aidens-daemon-kit`
- Schedule: one-shot schedule occurrence enqueue
- Wake: explicit wake signal enqueue
- Safety: safe mode blocks risky queue admission
- Evidence: duplicate suppression, leases, cancellations, safe mode, and drain reports

## Workflow

1. Start with `examples/configs/daemon-safe.toml`.
2. Use `aidens queue namespace` to inspect the namespace.
3. Use `aidens queue schedule` or `aidens queue wake` with read-only risk first.
4. Re-submit the same logical occurrence to verify duplicate suppression.
5. Acquire a lease before executing work.
6. Enable safe mode before testing risky wake or write-class work.
7. Drain or cancel queued jobs explicitly.

## Non-Imported Recall Assumptions

- no Recall DB schema;
- no app-specific socket path;
- no desktop UI event bridge;
- no host wake wrapper as runtime truth;
- no Recall memory/session model;
- no full autonomous timer loop claim.
