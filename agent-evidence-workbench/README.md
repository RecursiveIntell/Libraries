# Agent Evidence Workbench

A local command-line workbench for run reports, evidence manifests, claim statuses, and HMAC-bound receipts.

AEW records commands and repository snapshots, imports transcripts and graph results, and produces inspectable reports. Its claim extraction and verification are bounded heuristics; a reported status is not an independent proof that a change is correct or ready for production. Run and receipt timestamps also mean repeated executions need not produce identical artifacts.

## Build and inspect the CLI

From the Libraries repository root:

```bash
cargo build -p agent-evidence-workbench --bin aew
cargo run -p agent-evidence-workbench --bin aew -- --help
cargo test -p agent-evidence-workbench
```

The binary is `aew`. Run it in the repository whose evidence you want to collect; its `.aew` directory is relative to the current working directory.

Useful commands include `init`, `run`, `verify`, `report`, `claims`, `evidence`, `adjudicate`, `import-transcript`, and `import-graph-result`. Use each command's `--help` for required arguments. `run` executes the supplied command, and imports persist supplied content locally, so inspect inputs and choose the working directory deliberately.

## Opt-in Hermes observer

The observer is not automatically registered in Hermes configuration. It reads JSON lines from stdin and appends events to the configured file, with a receive timestamp:

```bash
# Run from Libraries/agent-evidence-workbench.
mkdir -p .aew
AEW_EVENTS_PATH=.aew/hermes-events.jsonl python3 integrations/hermes/aew-observer.py < hermes-events.jsonl
```

This is a best-effort observer: malformed events and write failures are dropped. Its file I/O is synchronous, so do not treat it as a guaranteed non-blocking or complete audit channel.

## Signing and optional memory promotion

`sign` and `verify-receipt` bind the report digest to a caller-supplied key. They establish integrity relative to that key, not independent trust in the report's claims. The current CLI accepts the key through `--key-hex`; take account of shell-history and process-argument exposure when using that interface. Do not put real keys in example commands or committed files.

`promote` is optional and requires the `semantic-memory` feature. It uses the configured embedder and can fail when the required model service is unavailable. Transcript/graph import and memory promotion are explicit operations; the observer does not silently promote evidence into memory.

See [CLI definitions](src/cli.rs), [claim verification](src/verifier.rs), and [receipt implementation](src/receipt.rs) for the exact current boundaries.
