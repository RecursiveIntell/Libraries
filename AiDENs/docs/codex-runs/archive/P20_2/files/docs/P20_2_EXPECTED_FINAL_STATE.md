# P20.2 Expected Final Repository State

## Required directories

```text
evals/
fixtures/test-agent/
fixtures/runner/
scripts/
crates/aidens-testkit/
crates/aidens-integration-tests/
target/aidens-p20-2-audit/  # generated, not necessarily committed
```

## Required scripts

```text
scripts/p20_2_scan_package_integrity.py
scripts/p20_2_scan_testkit_purity.py
scripts/p20_2_validate_agency_cases.py
scripts/p20_2_verify.sh
scripts/p20_2_verify_release_zip.sh
scripts/p20_2_generate_audit_bundle.sh
```

## Required commands

```bash
P20_2_REQUIRE_CARGO=1 bash scripts/p20_2_verify.sh
cargo test -p aidens-testkit --all-targets
cargo test -p aidens-integration-tests --all-targets
```

## Forbidden leftovers

- production crate dependencies in `aidens-testkit`;
- missing literal include targets;
- manifest entries pointing at absent files;
- fake provider capability claims;
- deleted agency evals;
- docs claiming green release without command output.
