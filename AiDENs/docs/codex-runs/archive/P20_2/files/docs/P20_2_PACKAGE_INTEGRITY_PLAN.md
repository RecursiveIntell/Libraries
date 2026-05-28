# P20.2 Package Integrity Plan

## Required checks

- literal `include_str!` and `include_bytes!` target scan;
- manifest file existence scan;
- script presence scan;
- eval fixture validation;
- release zip replay scan.

## Required restored paths

```text
evals/p20_agency_eval_cases.jsonl
scripts/p20_2_scan_package_integrity.py
scripts/p20_2_scan_testkit_purity.py
scripts/p20_2_validate_agency_cases.py
scripts/p20_2_verify.sh
scripts/p20_2_verify_release_zip.sh
fixtures/test-agent/basic-agent.toml
fixtures/test-agent/coding-agent.toml
fixtures/runner/test_agent_basic.json
fixtures/runner/expected_test_agent_event_log.ndjson
```

## Manifest law

`MANIFEST.txt` and `MANIFEST.json` must be generated from actual files or checked against actual files. They are evidence surfaces, not wish lists.

## Release archive law

A release archive is not valid until it is unpacked into a clean temp directory and checked by package scanners. If sibling crates are required for cargo, the archive report must explicitly state the required layout.
