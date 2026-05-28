# Flagship Local Coding-Agent Demo (P25)

This example demonstrates a supported-local coding lane using the existing
`aidens-cli run-coding-agent` flow with fixture-backed, deterministic artifacts.

## Purpose

- Load a local fixture repository.
- Read/list/search/status using supported local tools.
- Propose a patch and demonstrate explicit permit-gated apply behavior.
- Emit:
  - tool/receipt evidence,
  - provider route evidence,
  - support-tier disclosure,
  - `AiDENsRunBundleV2`,
  - deterministic replay report.

## Files

- `aidens.toml` — local coding-agent config for this demo.
- `task.md` — deterministic task target.
- `expected_output.md` — expected evidence artifacts and fields.
- `fixtures` are rooted at `../../fixtures/p25/coding-agent-repo`.

## Run sequence (supported-local + fixture-backed)

```bash
cargo run -p aidens-cli -- run-coding-agent examples/flagship-local-coding-agent/aidens.toml --out target/p25/flagship-coding-agent/no-permit
```

Request a patch-apply permit:

```bash
REQ=$(cargo run -p aidens-cli -- permit request --tool-id aidens:patch-apply:1 --risk file-write --sandbox-root fixtures/p25/coding-agent-repo)
APP=$(cargo run -p aidens-cli -- permit approve --request-id "$(
    jq -r '.request_id' <<<"$REQ"
)" --tool-id aidens:patch-apply:1 --risk file-write --sandbox-root fixtures/p25/coding-agent-repo --decided-by operator)
cargo run -p aidens-cli -- run-coding-agent examples/flagship-local-coding-agent/aidens.toml --out target/p25/flagship-coding-agent/with-permit --permit-json "$APP"
```

Inspect replay evidence:

```bash
cargo run -p aidens-cli -- inspect-run target/p25/flagship-coding-agent/no-permit/run-bundle.json
cargo run -p aidens-cli -- inspect-run target/p25/flagship-coding-agent/with-permit/run-bundle.json
```

## Notes

- No cloud provider is used in this demo.
- No autonomous loops are started.
- Receipt evidence is local and deterministic for a fixed fixture checkout and fixed output directory.
