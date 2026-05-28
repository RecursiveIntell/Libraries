# Re-run and Verification Instructions

## Current P22 Re-run

Run the full local P22 gate:

```bash
P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh
```

Run the non-cargo P22 packaging/assertion gate:

```bash
bash scripts/p22_verify.sh
```

Run the release package replay verifier:

```bash
bash scripts/p22_verify_release_archive.sh target/p22/aidens-p22-release-context.zip
```

## Focused Assertions

```bash
python3 scripts/assert_p22_zpy_archive_contract.py z.py
python3 scripts/assert_p22_codex_archival_hygiene.py .
python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/p22_verify_codex_context.manifest.json
python3 scripts/p22_secret_scan_fixture_test.py
```

## Direct z.py Proofs

Normal package dry run:

```bash
python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run
```

Audit/full-history dry run:

```bash
python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run
```

## Regression Triggers

Re-run the full P22 gate after any change to:

- `z.py`;
- `scripts/p22_*`;
- `scripts/assert_p22_*`;
- root packaging docs;
- `Cargo.toml`;
- provider/config/redaction code;
- archive policy or current-run docs.

Older contract-ownership, P20, and P21 command packets are historical evidence only. They are not the P22 final release gate.
