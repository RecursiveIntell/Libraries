# Capability Stack Audit Remediation — Verification Receipt

Date: 2026-07-13
Scope: semantic-memory, MCP/runtime boundaries, real Q8/Q4, Hermes Python 3.14 compatibility, Codex/Claude integration.

## Implemented

- TurboQuant parser minimum corrected to the actual 46-byte header; malformed 44/45-byte valid prefixes return `MalformedCode` without panic.
- Real self-describing per-vector affine Q8 and packed Q4 codecs implemented in `turbo-quant`; Q4 supports odd dimensions.
- `scr-runtime-compression` now applies Q8/Q4 instead of mapping both to uncompressed; selected and applied codec identities match.
- `agent-graph-mcp` validates routers, edges, route targets, graph/input/output limits, and retains bounded execution summaries rather than full states.
- Context-governor truncation is Unicode-scalar safe.
- CEA and knowledge-router malformed JSON boundaries return exit 1 without Rust panic.
- CEA file paths now affect privacy-preserving stable file identity and path-derived extension; explicit `file_path` wins over tool args.
- Hermes daemon pool supports CPython 3.14 WorkerContext while retaining the CPython 3.8–3.13 compatibility branch.
- Codex semantic-memory MCP is stdio-only; stale unauthenticated `--http-port 1739` was removed.
- Claude plugin durable-store defaults are canonical: `$HOME/.hermes/semantic-memory.db` and `$HOME/.hermes/context-governor`; plugin updated locally from 0.6.2 to 0.6.3.

## Verification

| Command / probe | Result |
|---|---|
| `cargo test -p turbo-quant` | PASS; 58 unit tests plus integration/doc suites, exit 0 |
| `cargo test -p scr-runtime-compression` | PASS; 26 unit tests, doc-tests 1 passed/1 ignored, exit 0 |
| `cargo test -p scr-runtime-compression --no-default-features --lib` | PASS; 14 tests, exit 0 |
| `cargo test --manifest-path agent-graph-mcp/Cargo.toml` | PASS; 24 tests, exit 0 |
| `cargo test --manifest-path context-governor/Cargo.toml --all-targets` | PASS, exit 0 |
| `cargo test --manifest-path cea-bridge/Cargo.toml` | PASS; 7 tests, exit 0 |
| `cargo test --manifest-path knowledge-router/Cargo.toml` | PASS; 11 tests, exit 0 |
| `cargo test --manifest-path semantic-memory-mcp/Cargo.toml --all-features` | PASS; integration 25 passed, overall exit 0 |
| `cargo test -p semantic-memory --all-features` | PASS; overall exit 0; doc-tests 3 passed/0 failed/1 ignored |
| Hermes focused daemon/memory provider suites | PASS; 115 tests, exit 0 |
| Claude canonical-path regression | PASS; 1 test, exit 0 |
| `claude plugin validate` for plugin and marketplace | PASS |
| Claude installed plugin readback | PASS; version 0.6.3 with canonical MCP environment paths |
| Codex TOML parse + exact MCP initialize/tools-list | PASS; exit 0, semantic-memory-mcp 0.5.4, 11 tools |
| HTTP startup without token | Expected refusal; exit 1 with `--http-port requires --http-auth-token or --http-auth-token-file` |
| Malformed JSON subprocess probes | Expected refusal; CEA and knowledge-router exit 1, zero `panicked at` occurrences |
| Task-owned `git diff --check` | PASS in all repositories |

## Claim boundaries

- Q8/Q4 claims are limited to real encoding/decoding, deterministic wire identity, bounded fixture error, malformed-input rejection, and byte reduction relative to f32 fixtures. No retrieval-quality or external superiority claim was tested.
- Existing live semantic-memory database was not destructively modified.
- No push was performed.
- Installed source changes require applicable host/process restart to be loaded by already-running clients; Claude explicitly reported restart required after plugin update.
