# Preflight and baseline

## Preconditions

Complete checkout; documented native dependencies; no unrelated dirty work. Dirty state stops
semantic editing and is recorded.

## Verify/install/bootstrap

```bash
python3 tools/verify_pack.py --pack /path/to/pack
bash /path/to/pack/scripts/bootstrap_run.sh   --repo ~/Coding/Libraries   --pack-dir /path/to/pack
```

Bootstrap records branch/commit/tree/status, lock/manifests, toolchain/platform, workspace
inventory, existing evidence inconsistencies, ID-boundary findings, and placeholder-codec findings.

## Baseline validation

Failures are valid baseline evidence and do not become passes.

```bash
python3 tools/run_validation_matrix.py   --repo ~/Coding/Libraries   --pack-dir /path/to/pack   --matrix /path/to/pack/config/validation_matrix.json   --output-dir <run>/evidence/baseline   --stage baseline   --continue-on-failure
```

## Reconciliation

Create `run/baseline_reconciliation.md`:

| Issue | Locator status | Current files | Existing tests | Decision |
|---|---|---|---|---|

Statuses: confirmed, moved, partially fixed, closed with evidence, not found, superseded.
Absence of a text pattern is not closure.
