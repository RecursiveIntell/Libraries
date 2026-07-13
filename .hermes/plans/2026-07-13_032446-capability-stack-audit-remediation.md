# Capability Stack Audit Remediation Implementation Plan

> **For Hermes:** Execute this plan immediately using test-first changes, focused verification, then repository-level gates. Preserve unrelated dirty work and commit locally without pushing.

**Goal:** Fix every confirmed semantic-memory/MCP capability-stack defect from the live audit, including real selectable Q8 and Q4 scalar codecs whose applied runtime identity matches governance receipts.

**Architecture:** Keep authority boundaries explicit. `turbo-quant` owns deterministic scalar quantization wire math; `scr-runtime-compression` maps governance profiles to those real codecs; MCP/CLI boundaries reject malformed or oversized input without panic; Hermes and plugin configuration use the canonical `~/.hermes` stores. Runtime stores remain untouched except for read-only verification.

**Tech Stack:** Rust 2021, Cargo, serde/serde_json, Tokio, Python 3.14/pytest, TOML/JSON plugin configuration, Claude/Codex MCP clients.

---

## Current context and safety gates

- Canonical Libraries workspace: `/home/sikmindz/Coding/Libraries`, branch `feat/full-integration`, HEAD `9d73fae`, clean before this pass.
- Active Hermes source: `/home/sikmindz/.hermes/hermes-agent`, branch `feat/moa-reference-assignments`, HEAD `9d7b188f9`, clean before this pass.
- Plugin source: `/home/sikmindz/Coding/agent-memory-kits`, branch `main`, HEAD `ce80747`, already dirty in six Codex-plugin files plus unrelated untracked artifacts. Do not alter, stage, or commit those pre-existing paths.
- Codex config: `/home/sikmindz/.codex/config.toml`, pre-change SHA-256 `13066fcd55805899b5f5860ab1cd272c0db4735d32bb651a7447a52a06926119`.
- Protect `/home/sikmindz/.hermes/semantic-memory.db` and `/home/sikmindz/.hermes/context-governor/` from destructive tests.
- The prior broad Rust chain was killed by the controller. It proves only `semantic-memory --all-features` completed, including doc-tests `3 passed, 0 failed, 1 ignored`; later suites remain unverified.
- HTTP startup authentication is already correctly enforced in `semantic-memory-mcp/src/main.rs`; retain it and verify it. The defect is Codex requesting HTTP without auth from a stdio MCP entry.

## Issue-to-test-to-file matrix

| ID | Finding | RED regression / proof | Production files | GREEN acceptance |
|---|---|---|---|---|
| TQ-1 | TurboQuant parser accepts 44/45-byte valid prefixes then slices through byte 46 | Add valid-prefix boundary tests for lengths 44 and 45; current source panics | `turbo-quant/src/wire.rs` | Both return `MalformedCode`; header constant and all docs/capacities say 46 |
| Q-1 | Governance selects Q8/Q4 but dispatch applies `Uncompressed` | Tests assert selected Q8→`CodecId::Q8`, Q4→`CodecId::Q4`; real round trips and byte reductions | `turbo-quant/src/wire.rs`, `turbo-quant/src/lib.rs`, `scr-runtime-compression/src/lib.rs`, `scr-runtime-compression/src/codec_dispatch.rs` | Q8 and Q4 use distinct wire modes, decode finite vectors, reject corrupt/truncated payloads, Q4 handles odd dimensions, and no selected/applied alias remains |
| AG-1 | Router without routes is accepted and runtime unwraps | Protocol-level `tools/call graph_create` test returns `isError`, plus runtime missing-routes structured-error test | `agent-graph-mcp/src/main.rs` | Empty/missing routes rejected at creation; runtime uses `ok_or_else`, no route unwrap |
| AG-2 | Graphs and full execution states retained forever; no size/step bounds | Tests fill stores and exceed graph/node/edge/input/output/step limits | `agent-graph-mcp/src/main.rs` | Bounded graph count; bounded execution summary deque; full states not retained; explicit constants and structured limit errors |
| PY-1 | Python 3.14 removed `_initializer`/`_initargs` and changed `_worker` signature | Existing `tests/tools/test_daemon_pool.py` fails on 3.14; add explicit worker-context compatibility assertion | `~/.hermes/hermes-agent/tools/daemon_pool.py`, `tests/tools/test_daemon_pool.py` | Daemon/initializer/reuse/fast-exit tests pass on live Python 3.14 without relying on removed fields |
| CG-1 | UTF-8 byte slicing can panic during context truncation | Emoji/CJK test crossing limit | `context-governor/src/llm_summary.rs` | Character-count truncation, no panic, suffix preserved |
| CLI-1 | CEA malformed stdin JSON panics | Unit parser test and subprocess invalid-JSON test | `cea-bridge/src/main.rs` | Exit 1 with normal `error:` text and no `panicked at` |
| CLI-2 | knowledge-router malformed stdin JSON panics | Unit parser test and subprocess invalid-JSON test | `knowledge-router/src/main.rs` | Exit 1 with normal `error:` text and no panic |
| CEA-1 | `file_path` is accepted but ignored in causal signatures | Two otherwise identical records with different paths produce distinct stable signatures; extension is path-derived | `cea-bridge/src/main.rs` | Normalized path contributes to stable context/file identity without persisting raw path in the signature |
| CFG-1 | Codex stdio MCP entry requests unauthenticated HTTP port 1739 | Run exact configured command before/after; parse TOML | `~/.codex/config.toml` | Remove `--http-port 1739`; exact command stays alive over stdio and initializes/lists tools |
| PLUG-1 | Claude plugin defaults can create legacy semantic-memory/context-governor silos | Tests scan active Claude plugin runtime files for legacy durable-store defaults | `agent-memory-kits/claude/plugins/semantic-memory/.mcp.json`, runtime scripts/hooks/docs, plugin/marketplace versions | Defaults are `~/.hermes/semantic-memory.db` and `~/.hermes/context-governor`; plugin validates and local install cache updates to new version |
| HTTP-1 | HTTP must never start without auth | Existing resolver/startup/integration tests plus exact binary probe | `semantic-memory-mcp/src/main.rs` (verify; change only if regression appears) | `--http-port` without token exits 1; token/token-file paths pass; unauthenticated `/health` remains 401 |

## Implementation phases

### Phase 0 — Preserve evidence and establish RED

1. Save this plan and baseline repository status/checksums.
2. Run each focused pre-fix test or hostile probe separately; capture exit status. Never use a single `A && B && ...` chain as the only receipt.
3. Add regression tests before production changes. Tests become locked except for genuine harness defects.

### Phase 1 — TurboQuant header and real Q8/Q4

1. In `turbo-quant/src/wire.rs`, add `TURBO_CODE_WIRE_HEADER_LEN = 46`; use it in encode capacity, documentation, and `parse_header` bounds.
2. Add 44/45-byte valid-prefix tests and confirm RED by panic/failure before the parser fix.
3. Add a public deterministic scalar wire API in `turbo-quant`:
   - `ScalarQuantMode::{Q8,Q4}`;
   - self-describing V1 header with magic, version, bit width, dimension, finite positive scale, zero point, and payload length;
   - per-vector affine Q8 payload (`i8 × dims`);
   - per-vector affine Q4 payload (two 4-bit values per byte, odd final dimension supported);
   - checked integer conversions, non-finite-input rejection, exact-length validation, reserved-field validation, and no production unwraps.
4. Re-export the API from `turbo-quant/src/lib.rs`.
5. Extend `scr-runtime-compression::CodecId` with Q8 and Q4 and make display/serde identities explicit.
6. Map governed Q8/Q4 to those exact codec IDs; wire adapter, encode, and decode paths to `ScalarQuantWireV1`.
7. Add tests for deterministic encoding, finite reconstruction, bounded Q8/Q4 error on deterministic fixtures, actual compressed size at embedding dimensions, odd Q4 dimensions, corruption rejection, and selected/applied identity.
8. Run `cargo test -p turbo-quant wire` and `cargo test -p scr-runtime-compression codec_dispatch` before full crate suites.

### Phase 2 — Agent graph protocol hardening

1. Add explicit limits: graph count, serialized graph bytes, nodes, edges, recursion/steps, input bytes, output bytes, and execution summary count.
2. Validate router routes are present and non-empty at `tool_graph_create`; validate route targets exist or equal `END`.
3. Replace `routes.as_ref().unwrap()` with a structured error even though creation validates it (defense in depth).
4. Store bounded metadata summaries in a `VecDeque`, not full execution state/output.
5. Reject oversized input/output and graph specs before retention; reject a new graph when at capacity while allowing replacement of an existing ID.
6. Add direct and MCP protocol tests, then run the crate suite.

### Phase 3 — Boundary panic fixes

1. Context governor: add emoji/CJK truncation tests; implement `chars().take(max_chars)` semantics so `max_chars` means characters.
2. CEA bridge: make stdin parsing return `Result`; propagate with `?`; derive stable path identity and actual extension in `synthesize_signature`; test malformed input and distinct paths.
3. Knowledge router: make stdin parsing return `Result`; propagate with `?`; test malformed input and subprocess behavior.
4. Run each focused crate suite independently.

### Phase 4 — Hermes Python 3.14 compatibility

1. Run the existing daemon-pool tests on Python 3.14 and preserve the expected failure.
2. In `_adjust_thread_count`, use `_create_worker_context()` with Python 3.14’s `_worker(executor_reference, ctx, work_queue)` contract when available; retain the older initializer/initargs worker contract as the compatibility branch.
3. Keep daemon threads unregistered from `_threads_queues` on both paths.
4. Run `tests/tools/test_daemon_pool.py`, then the previously focused memory-provider test group. Record exact counts.

### Phase 5 — Client and plugin canonical-path repair

1. Remove only `--http-port`, `1739` from the Codex semantic-memory stdio MCP args. Do not modify unrelated Codex settings.
2. Parse the TOML and execute the exact resulting server command using an MCP initialize/tools-list probe.
3. Update Claude plugin durable-store defaults in `.mcp.json`, `run-server.sh`, hook resolver, context-governor scripts, ingest script, and setup docs. Keep benchmark receipt output directories separate unless they are authoritative store defaults.
4. Fix `run-server.sh` directory creation for a database-file path by creating its parent directory rather than a directory named `semantic-memory.db`.
5. Add a canonical-path regression test covering active runtime files.
6. Bump plugin and marketplace versions together, validate with `claude plugin validate`, update the local directory-marketplace install, and read back the installed cache.
7. Preserve all pre-existing dirty agent-memory-kits paths and stage only task-owned files.

### Phase 6 — Focused-to-broad verification

Run separately and classify each as passed, failed, timed out, killed, or untested:

```bash
cargo test -p turbo-quant
cargo test -p scr-runtime-compression
cargo test --manifest-path agent-graph-mcp/Cargo.toml
cargo test --manifest-path context-governor/Cargo.toml --all-targets
cargo test --manifest-path cea-bridge/Cargo.toml
cargo test --manifest-path knowledge-router/Cargo.toml
cargo test --manifest-path semantic-memory-mcp/Cargo.toml --all-features
cargo test -p semantic-memory --all-features
python -m pytest tests/tools/test_daemon_pool.py -o 'addopts=' -q
python -m pytest <focused memory-provider tests> -o 'addopts=' -q
python -m pytest tests/<new plugin canonical-path test>.py -q
```

Then:

- `cargo fmt --all -- --check` for the Libraries workspace.
- Targeted `cargo clippy` for changed crates if time permits; do not call it passed if not run.
- Exact invalid JSON subprocess probes for CEA and knowledge-router.
- Exact Codex MCP stdio initialize/tools-list probe.
- Semantic-memory HTTP no-token startup refusal and authenticated/unauthenticated health behavior without mutating the live DB.
- Diff inspection and forbidden-pattern searches: no route unwrap, no Q8/Q4→Uncompressed alias, no 44-byte header claim, no legacy durable-store defaults in active Claude plugin runtime files.

### Phase 7 — Local commits, no push

1. Re-read status/diffs after all long tests.
2. Commit Libraries task-owned changes together with the plan and verification receipt.
3. Commit Hermes source changes in its repository.
4. Commit agent-memory-kits task-owned changes only; explicitly exclude the six pre-existing modified Codex-plugin files and unrelated untracked files.
5. Treat `/home/sikmindz/.codex/config.toml` and installed plugin cache as live configuration receipts, not repository commits.
6. Do not push.

## Risks and rollback

- **Wire compatibility:** TurboCode header correction changes no valid bytes; it only makes the documented minimum match the existing 46-byte layout. Scalar Q8/Q4 get a new magic/version and therefore cannot be confused with existing TurboCode artifacts.
- **Lossy quality:** Tests prove fixture-level reconstruction/error and byte size only. Do not claim retrieval-quality improvement without a real corpus evaluation.
- **Graph bounds:** Existing clients exceeding new limits receive structured errors rather than process exhaustion. Defaults should exceed normal workflows while preventing unbounded growth.
- **Plugin dirty tree:** Use path-scoped staging. Never run broad checkout/reset/clean in agent-memory-kits.
- **Live services:** Source changes require rebuilding/restarting binaries to affect running processes. Do not restart Hermes mid-session. Report any restart requirement.
- **Rollback:** Revert only task commits per repository; restore Codex config from its pre-change checksum-backed diff; Claude plugin can be reinstalled at the prior cached version. Never roll back the live semantic-memory database.

## Completion definition

- [x] Every matrix row has a locked regression and passing focused test/probe.
- [x] Q8 and Q4 are distinct real codecs with self-describing wire metadata and round-trip reconstruction.
- [x] Selected and applied codec identities match.
- [x] No confirmed panic or unbounded-retention finding remains.
- [x] Canonical store paths are installed, not only edited in source.
- [x] HTTP auth enforcement remains green.
- [x] Broad suites are reported individually and honestly.
- [x] Task-owned changes are committed locally in each repository; nothing is pushed.
