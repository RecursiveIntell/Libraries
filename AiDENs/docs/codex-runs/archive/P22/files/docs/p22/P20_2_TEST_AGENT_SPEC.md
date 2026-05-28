# Canonical Test Agent Spec

## Purpose

The test agent is the first executable proof that AiDENs can assemble a usable agent from canonical wiring without becoming a shadow stack.

## Required files

```text
fixtures/test-agent/basic-agent.toml
fixtures/runner/test_agent_basic.json
fixtures/runner/expected_test_agent_event_log.ndjson
```

## Required test

```bash
cargo test -p aidens-integration-tests test_agent_vertical_slice -- --nocapture
```

## Optional CLI smoke

```bash
cargo run -p aidens-cli -- run-test-agent fixtures/test-agent/basic-agent.toml
```

If `run-test-agent` does not exist yet, Codex should add it only as a thin wrapper over existing runner APIs. Do not invent a separate runner path.

## Assertions

The test must assert:

- provider route recorded;
- tool exposure plan exists;
- permit check occurred;
- tool call executed or was blocked with receipt;
- agency gate evaluated final output;
- receipts/event log are non-empty;
- no unsupported provider/native-tool-loop claim appears.
