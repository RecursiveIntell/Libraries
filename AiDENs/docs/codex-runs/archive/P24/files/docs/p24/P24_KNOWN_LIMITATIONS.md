# P24 Known Limitations

Record date: `2026-05-03`

## Deferred

- Hosted cloud provider execution is not supported. No cloud provider path was run with credentials and receipts.
- Native provider tool loops and streaming are not supported.
- Autonomous daemon behavior beyond local append-only queue lifecycle is deferred.
- Desktop, autonomous memory, and research profile products remain scaffold/partial.
- V10+ regional decoder, hypergraph, subtraction, federation, and mechanism runtime work was not built in P24.

## Partial

- Memory/runtime seam proof is fixture-backed: `ExportEnvelopeV3 -> forge-memory-bridge -> semantic-memory -> knowledge-runtime` runs locally with mock embeddings.
- Coding-agent support is local fixture support: repo list/read/search/status and patch proposal run; file writes require explicit scoped permit.
- Boundary repair receipts are AiDENs-local display evidence with canonical verification/control backpointers, not canonical repair truth.

## Operational Risks

- The parent worktree was dirty at P24 start and remains outside AiDENs package control.
- Package self-replay is path-parameterized. Operators must set `P24_PACKAGE_SELF_REPLAY` to the package zip when rerunning that verifier check.
