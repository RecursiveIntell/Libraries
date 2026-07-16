# Validation protocol

## Command receipt

```bash
python3 tools/run_with_receipt.py   --repo /path/to/repo   --output-dir /path/to/run/evidence/commands   --name <stable-name>   --stage <baseline|task|phase|final>   --issue <ID>   -- <argv...>
```

Result vocabulary: `pass`, `fail`, `blocked`, `skipped`. A required blocked/skipped command blocks closure.

## Task

Run issue-specific commands, validate handoff JSON, and ensure receipt commit/tree equals task head.

## Phase

```bash
python3 tools/run_validation_matrix.py   --repo /path/to/repo   --pack-dir /path/to/pack   --matrix /path/to/pack/config/validation_matrix.json   --output-dir /path/to/run/evidence/phase-<n>   --stage phase
```

## Final

Run final matrix, evidence consistency, clean-tree check, and independent hostile audit.
Recording final evidence happens after source commit fixation; verification then reruns read-only.
