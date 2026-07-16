# Phase 4 — Queues and search integrity

Issues: `QUE-001`, `QUE-002`, `SEM-001`.

- ai-batch: atomic claim, strict transitions, terminal completion, light handles, event wakeup.
- job-queue: explicit cancellation/heartbeat/lease failure, ownership enforcement, measured connection strategy.
- semantic-memory: explicit strict/degraded corruption policy in results/receipts.

Exit: race tests pass; infrastructure error cannot become benign; search degradation is caller-accepted.
