# Agent Evidence Workbench

AEW stores deterministic local run reports, evidence manifests, claim statuses, and HMAC receipts.

## Opt-in Hermes observer

The observer is deliberately not registered in Hermes configuration. It only appends received JSON events and never blocks the producer:

```bash
AEW_EVENTS_PATH=.aew/hermes-events.jsonl python3 integrations/hermes/aew-observer.py < hermes-events.jsonl
```

Transcript and Agent Graph imports are explicit `aew import-transcript` and `aew import-graph-result` operations. Semantic-memory promotion is optional and requires `--features semantic-memory`; it uses the configured real embedder and may fail if unavailable. Secrets are supplied as `--key-hex` and are never persisted.
