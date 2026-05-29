# Crate and Surface Target Map

This is a routing map, not a substitute for code inspection.

| Surface | Primary risks | Required hardening |
|---|---|---|
| `crates/aidens-runner` | provider honesty, done-state receipts, replay, budget, mixed ownership/concurrency | typed provider mocks, local/mock separation, no done without receipts, execution-context enforcement |
| `crates/aidens-tool-kit` | patch_apply, command execution, file reads, sandbox TOCTOU | transactional patch, structured command execution, path hostile tests |
| `crates/aidens-security-kit` | sandbox policy, secret paths, symlinks/hardlinks | deny/quarantine fixtures and receipts |
| `crates/aidens-receipts` | durability, hash chains, corruption, concurrency | file locks/single-writer, verification command, quarantine |
| `crates/aidens-queue-kit` / daemon/schedule kits | leases, idempotency, races, safe mode | concurrent tests, lease enforcement, queue-hop receipts |
| `crates/aidens-boundary-kit` | strict JSON/schema/repair/treatment integrity | full validator or unsupported rejection, durable repair receipts |
| `crates/aidens-contracts` | artifact/effect lifecycle, proof debt, budget consumed | operator effect enforcement, terminal state constructors, modular split if needed |
| `semantic-memory` | HNSW TOCTOU, pool timeouts, search threshold/date parsing | Claude F-006..F-012 fixes/tests |
| `forge-pilot`, `effect-runtime`, `verification-*`, `federation`, `attestation`, `authority-delegation`, `recursive-kernel-core` | unaudited high-risk layers | audit/quarantine before broad claims |
| docs/evidence scripts | marker tests, stale labels, open bug list ambiguity | semantic fixtures, status classification, final handoff |
