# Module budget exceptions

These exceptions keep the canonical owner modules intact. They do not waive
formatting, clippy, tests, or API review.

## `agent-graph/src/error.rs`

`AgentGraphError` is the canonical typed error owner for the agent-graph
runtime. Its variants remain together so checkpoint, execution, serialization,
and lifecycle failures retain stable discriminants and display contracts.

## `profile-runtime/src/adapters.rs`

The adapter module contains the complete cross-provider admission and receipt
translation boundary. Splitting it would separate coupled ownership checks and
create a second adapter contract.

## `semantic-memory/src/db.rs`

This is the canonical SQLite schema, migration, pragma, embedding codec, and
integrity boundary. Its size reflects one durable storage owner; extraction
would risk duplicating migration/version semantics.

## `semantic-memory/src/lib.rs`

This is the public MemoryStore facade and canonical async boundary. It keeps the
public API, connection lifecycle, embedding orchestration, and integrity hooks
under one owner rather than introducing a parallel facade.

## `forge-pilot/src/main_support/mod.rs`

The module owns the CLI support wiring and typed orchestration boundary for
forge-pilot. Its coupled command/config/receipt support remains one owner.

## `forge-pilot/src/loop_runner.rs`

The loop runner owns budget, halt, cooldown, evidence, and recovery transitions.
Keeping those transitions together preserves one lifecycle state machine.

## `knowledge-runtime/src/runtime/core.rs`

The runtime core owns projection admission and lifecycle coordination. The
exception prevents splitting state-transition semantics across shadow helpers.
