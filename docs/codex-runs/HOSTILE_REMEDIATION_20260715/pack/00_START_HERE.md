# Start here

## Mission

Repair the audited correctness, authority, interchangeability, release-proof, and efficiency
defects without new shadow truth, fake compatibility, false completion, or destructive migrations.

## Authored baseline

- Repository: `RecursiveIntell/Libraries`
- Audited branch: `p32-schema-compat`
- Audited commit: `c65972dbdf0ee5a7b472019b12c905a9de77c5c9`
- Pack date: `2026-07-15`
- Audit mode: GitHub connector-backed static inspection
- Independent local Cargo execution by auditor: **not performed**

Hermes must treat these values as locators, not current truth. It must capture the actual checkout
commit/tree/toolchain and reconcile each locator before editing.

## First commands

```bash
python3 tools/verify_pack.py --pack .
bash scripts/bootstrap_run.sh --repo ~/Coding/Libraries --pack-dir "$PWD"
```

## Release posture

Release remains blocked until every P0 and P1 issue is independently reviewed, merged,
post-merge validated, and included in a source-bound final receipt.
