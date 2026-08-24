# llm-tool-runtime

Provider-agnostic tool contracts, registry, dispatch, and receipt plumbing.

> last-verified: 2026-08-20

## Contract and claim boundary

This crate exposes the typed API and validation/dispatch behavior present in its Rust sources. It does not, by itself, provide deployment, transport, persistence, authorization, truth promotion, or operational guarantees beyond the types and functions documented here. Integrators must supply surrounding services where the source API requires them.

## Install

```toml
[dependencies]
llm-tool-runtime = "0.1.0"
```

Rust 1.75+ and the 2021 edition are declared by the manifest where applicable. Use the workspace manifest when consuming this crate by path.

## Quick start

The checked-in test constructs `ToolDescriptor`, `ToolCall`, `ToolCtx`, and the registry/runtime surfaces, then asserts typed validation and receipt behavior.

```bash
cargo test -p llm-tool-runtime --test core_tests
```

The command is a bounded compile/test entry point for the checked-in source; it does not imply release readiness or external-service availability.

## API overview

| Surface | Role |
|---|---|
| `Tool` | Primary public entry point for this crate's contract. |
| `ToolRegistry` | Typed data or orchestration surface used by callers. |
| `ToolError` | Explicit failure/validation boundary where applicable. |
| `serde`/schema derives | Serialization and schema exchange where declared by source. |

See `src/lib.rs` for the re-export boundary and module list.

## Errors and edge cases

- Treat validation failures as data: do not silently coerce missing, empty, inconsistent, or out-of-scope fields.
- Preserve schema/version identifiers and caller-supplied IDs when serializing artifacts.
- For async/network or queue APIs, handle timeouts, unavailable backends, duplicate/in-flight work, and partial results according to the returned error/status types.
- This README does not claim cryptographic, authorization, scheduler, or release-readiness guarantees unless enforced by the source API.

## Architecture

![llm-tool-runtime architecture](docs/llm-tool-runtime.svg)

Caller-owned input enters the public `Tool` surface and leaves as typed artifacts, status, receipts, or explicit errors. External persistence, transport, policy, and execution remain caller/integration responsibilities unless represented by a public trait.

## Verification

From this crate directory, run the narrow checks that are valid for the workspace:

```bash
cargo test -p llm-tool-runtime
cargo check -p llm-tool-runtime
```

Additional source references: `tests/core_tests.rs; tests/dispatch_tests.rs`. Examples, when present, can be checked with `cargo check -p llm-tool-runtime --examples`.

## Integration path

1. Depend on the published version or workspace path.
2. Construct inputs using the public types and preserve their IDs/schema fields.
3. Call the public API and handle success/error/status explicitly.
4. Connect the result to the owning persistence, policy, transport, or runtime layer in the surrounding workspace.

## Status and roadmap

The manifest version is `0.1.0`. This documentation describes the current source surface, not a promise of release readiness. Future work should follow the crate's existing tests, schema fixtures, and workspace integration requirements; no unsupported roadmap items are asserted here.

## License

`MIT` (from `Cargo.toml`).
