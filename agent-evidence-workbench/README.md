# Agent Evidence Workbench

AEW stores local legacy reports plus a **provisional V2 deterministic policy evaluator**. V2 uses explicit claim/evidence links and AEW does not issue a terminal release decision; downstream release-decision authority is outside this crate. New V1 `Run` and `Verify` operations rewrite extracted claims to `NotChecked` and `Partial`; V1 regex extraction is not release-grade adjudication. Existing persisted reports are not migrated or automatically reclassified and must not be used as release evidence.

## Opt-in Hermes observer

The observer is deliberately not registered in Hermes configuration. It attempts to append valid JSON input lines to `AEW_EVENTS_PATH`; malformed lines, a missing path, and processing exceptions are ignored. It is a best-effort local observer, not a lossless or non-blocking transport.

```bash
AEW_EVENTS_PATH=.aew/hermes-events.jsonl python3 integrations/hermes/aew-observer.py < hermes-events.jsonl
```

Transcript and Agent Graph imports are explicit `aew import-transcript` and `aew import-graph-result` operations. Semantic-memory promotion is optional and requires `--features semantic-memory`; it uses the configured real embedder and may fail if unavailable.

For the local signing and verification CLI, 32-byte receipt keys are read from `--key-file`; on Unix, group- or world-readable key files are rejected. The CLI does not expose a `--key-hex` option. Receipt serialization does not include key bytes. This is a local storage boundary, not a universal guarantee that secrets supplied to other commands or files cannot be persisted.

V2 evaluates submitted, explicitly linked evidence; it does not independently attest that a command actually executed between source snapshots. The `evaluate-v2` CLI requires the supplied pre/post binding to match its current repository snapshot. The core evaluator validates only consistency of the submitted binding; it does not establish that the snapshots came from Git. The CLI sanitizes its input before evaluation, and the canonical digest covers the complete input passed to the evaluator, observation timestamps, source binding, and policy output; idempotence applies to exact replay. V2 command output and durable event payloads are passed through a bounded local redaction baseline before persistence. Redaction is pattern-based and is not a universal secret-detection or security guarantee. V2 events use a local temporary-file-and-rename write path with idempotent exact replay and conflict rejection. Crash durability and filesystem-specific guarantees are not established here.
