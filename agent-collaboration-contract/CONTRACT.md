# Agent Collaboration Contract V1

This crate defines transport-neutral Rust data contracts for tasks, leases, artifacts, events, and receipts. `contract-schema-gen` produces the checked-in JSON Schema projections.

- Versioned top-level records require `schema_version`; their Rust `validate()` methods require it to equal `agent_collaboration_contract_v1`. Nested reference types without that field inherit their enclosing record’s version context. Generated schema projections require the field where present but do not encode the exact-value check.
- Timestamps are RFC3339 strings; `occurred_at` describes source time and `recorded_at` describes authority receipt time.
- Within these V1 contracts, IDs are represented as opaque `stack-ids` values; URLs, filenames, prompts, and model text are not treated as identity.
- Task envelopes and receipts carry artifact references rather than inline prompt history; consumers remain responsible for enforcing declared resource limits.
- Optional fields do not imply success. Protocols that need unavailable or redacted states must represent them explicitly rather than relying on omission or null.
- Task envelopes carry an idempotency key and request digest for later stateful idempotency enforcement; this crate does not perform deduplication.
- The transition helper gives statuses classified as terminal no outgoing transitions. Reconciliation transitions are available only from `completion_unknown`; this is an in-memory transition table, not durable lifecycle enforcement.
- Authority and execution-permit references are optional data fields in V1. They do not grant authority or authorize effects; authority and effect admission are outside this phase.
- Generated JSON Schemas are projections of Rust types and must not be hand-edited.
