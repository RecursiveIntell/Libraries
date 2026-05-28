# Expected Output (Observed Evidence Template)

## Commands

```bash
cargo run -p aidens-cli -- run-coding-agent examples/flagship-local-coding-agent/aidens.toml --out target/p25/flagship-coding-agent/no-permit
cargo run -p aidens-cli -- permit request --tool-id aidens:patch-apply:1 --risk file-write --sandbox-root fixtures/p25/coding-agent-repo
cargo run -p aidens-cli -- permit approve --request-id ... --tool-id aidens:patch-apply:1 --risk file-write --sandbox-root fixtures/p25/coding-agent-repo --decided-by operator > /tmp/permit.json
cargo run -p aidens-cli -- run-coding-agent examples/flagship-local-coding-agent/aidens.toml --out target/p25/flagship-coding-agent/with-permit --permit-json "$(cat /tmp/permit.json)"
cargo run -p aidens-cli -- inspect-run target/p25/flagship-coding-agent/no-permit/run-bundle.json
cargo run -p aidens-cli -- inspect-run target/p25/flagship-coding-agent/with-permit/run-bundle.json
```

## Expected evidence artifacts

- `target/p25/flagship-coding-agent/no-permit/coding-agent-report.json`
- `target/p25/flagship-coding-agent/no-permit/run-bundle.json`
- `target/p25/flagship-coding-agent/no-permit/coding-agent-summary.md`
- `target/p25/flagship-coding-agent/no-permit/event-log.ndjson`
- `target/p25/flagship-coding-agent/with-permit/coding-agent-report.json`
- `target/p25/flagship-coding-agent/with-permit/run-bundle.json`
- `target/p25/flagship-coding-agent/with-permit/coding-agent-summary.md`
- `target/p25/flagship-coding-agent/with-permit/event-log.ndjson`

## Expected assertions

- Without permit:
  - `coding-agent-report.json#steps[]` contains `patch_apply_permit_gate` blocked status and an `approval_request`.
  - run bundle `support.support_tier = supported-local`.
  - `coding-agent` status remains unchanged on disk.
- With permit:
  - permit receipt appears in the report and in `run-bundle.json`.
  - run bundle `support.support_tier = supported-local`.
  - `inspect-run` returns `event_log_digest_verified = true`.
- Replay:
  - deterministic `inspect-run` for each run with stable normalized replay comparison data.
