# P29 Code Change Targets

## High-priority crates/modules

- `AiDENs/crates/aidens-contracts`
- `AiDENs/crates/aidens-runner`
- `AiDENs/crates/aidens-receipts`
- `AiDENs/crates/aidens-boundary-kit`
- `AiDENs/crates/aidens-tool-kit`
- `semantic-memory`
- `knowledge-runtime`
- `stack-ids`
- `living-memory`
- `z.py`
- `scripts/*`

## Highest-risk implementation surfaces

- HNSW lock ordering and sidecar sync.
- SQLite migration atomicity.
- Search dedup and recency formula.
- ExecutionContextEnvelope and ToolCallReceipt timing/fingerprint.
- Artifact lifecycle transition legality.
- Receipt chain immutability.
- Boundary compiler strictness.
- Proof debt/waiver semantics.
- Package self-replay and manifest path validation.
