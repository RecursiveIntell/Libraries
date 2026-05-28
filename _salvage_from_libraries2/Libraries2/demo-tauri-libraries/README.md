# demo-tauri-libraries

Tauri showcase for the current repository libraries:

- `semantic-memory-forge`
- `forge-memory-bridge`
- `semantic-memory`
- `knowledge-runtime`
- `tauri-queue`

The app demonstrates:

1. Forge export envelope construction.
2. Canonical bridge transform to projection batch.
3. Projection import and projection read APIs in `semantic-memory`.
4. Runtime query/plan/evidence flows in `knowledge-runtime`.
5. Queue job execution and lifecycle events in `tauri-queue`.

## Run

```bash
cd /home/sikmindz/Coding/Libraries/demo-tauri-libraries/src-tauri
cargo tauri dev
```

Prerequisites:

- Rust (stable toolchain)
- `cargo-tauri` CLI (`cargo install tauri-cli`)
- The demo reuses local workspace crates, so run from this repository root checkout.

Open the window and click:

1. `Initialize demo state`
2. Run one or more query/evidence/temporal/projection actions
3. Enqueue jobs and observe queue events in the **Queue events** section

## Notes

- The frontend lives in `ui/` and serves directly through Tauri without a separate frontend framework.
- `run_temporal_query` uses `valid_at` + `recorded_at_or_before` parameters for explicit bitemporal behavior.
- Evidence handles are surfaced only through the explain/audit path (`query_claim_evidence`).
