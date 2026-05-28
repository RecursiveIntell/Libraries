# 11 — Test and Conformance Plan

## Purpose

AiDENs needs a testkit because it is a wiring/safety stack. Unit tests alone are not enough. The testkit should make the common footguns impossible to miss.

## `aidens-testkit`

Owns:

```text
mock providers
mock tool bundles
mock receipt ledger
config fixtures
schema fixtures
boundary parser corpus
approval harness
queue/schedule harness
capability truth assertions
reference interpreters for small semantic surfaces
```

## Foundation tests

### Contracts

```text
schemas generate
schemas meta-validate
historical schemas still accepted or correctly rejected
breaking changes detected
artifact IDs roundtrip
serde JSON roundtrip
```

### Boundary

```text
strict JSON accepts valid input
strict JSON rejects malformed input
duplicate keys rejected or flagged
oversized payload rejected
repair emits receipt
patch operation order tested
path traversal rejected
symlink escape rejected
```

### Config

```text
missing config creates default if profile allows
invalid config blocks
secrets redacted
apply is validate-then-commit
run pins config generation
hot-swap denied unless enabled
```

### Receipts

```text
every runner path emits run receipt
receipt parent links valid
receipt trace IDs stable
fallback route emits degraded receipt
approval denial emits receipt
timeout emits receipt
```

### Capability truth

```text
provider configured unavailable => blocking
web disabled => no web executable
registered != exposed != executable
parser fallback reported accurately
disabled tools absent
```

## Core agent tests

```text
native OpenAI route fixture
native Ollama route fixture
OpenAI-compatible route fixture
parser fallback route fixture
unknown native provider rejected
read-only tool no approval
write tool requires approval
denied approval blocks execution
approved permit allows exactly one execution if single-use
full registry exposure denied outside debug
```

## Memory tests

```text
memory disabled does not create stores
memory optional degrades visibly
memory required blocks on failure
valid_as_of/recorded_as_of preserved
reranker cannot widen temporal scope silently
dedupe mode recorded
profile memory denied in coding profile unless opt-in
```

## Queue/schedule/daemon tests

```text
queued action gets lease
stale lease job skipped
provider failover child attempt not new job
cancellation propagates
misfire policy deterministic
DST cases deterministic
overlap denied unless policy allows
host wake drift reported
startup recovery does not duplicate jobs
```

## UI/shell tests

```text
Tauri command cannot construct provider directly
approval UI event must go through permit-kit
stream events idempotent by sequence_no
subscription reconnect does not duplicate event source
CLI doctor fails with clear messages
```

## Kernel tests

```text
kernel consumes immutable snapshot
storage graph not accepted as inference graph without compilation
iteration cap enforced
oscillation emits degradation receipt
oracle slice escalation emits receipt
repair proposal does not self-promote
```

## Generated app tests

Every `aidens new` template should include starter tests:

```text
doctor passes
config loads
provider truth available
disabled tools absent
write tool requires approval
parser fallback not hidden
receipt emitted for one sample run
```

## CI gates

1. `cargo check --workspace`
2. `cargo test --workspace`
3. schema generation
4. schema meta-validation
5. dependency law check
6. no forbidden dependencies
7. no unapproved `unwrap()` in non-test public paths for AiDENs crates
8. no shell/network/write tool in safe profiles without explicit approval policy
9. no profile can enable dangerous auto-approval by default

## Dependency law check

Add a script:

```bash
aidens check-deps
```

It should fail if:

```text
contracts depends on runtime/app crates
provider depends on memory/queue/ui
tool-kit depends on app-specific tools
tauri-kit depends on provider construction directly
runner depends on tauri
daemon depends on tauri
queue depends on tauri
```
