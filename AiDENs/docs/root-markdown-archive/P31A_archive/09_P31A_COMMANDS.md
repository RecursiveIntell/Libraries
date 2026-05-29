# P31A Command Sheet

## Phase gates

```bash
python3 scripts/assert_release_ledger_schema.py
python3 scripts/assert_current_run_truth.py
python3 scripts/assert_release_truth_consistency.py
python3 scripts/assert_support_claims_have_evidence.py
```

```bash
python3 scripts/assert_root_markdown_archive_policy.py
python3 scripts/assert_codex_artifact_classification.py
```

```bash
bash scripts/verify_current.sh
```

## Package commands

```bash
python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P31A
```

Find package:

```bash
find . -maxdepth 4 -name 'AiDENs-*.zip' -printf '%T@ %p\n' | sort -n | tail -1
```

Validate replay:

```bash
python3 scripts/assert_package_validation.py
python3 scripts/assert_package_self_replay.py --package <zip> --require-verifier --receipt-out handoffs/P31A_PACKAGE_REPLAY_RECEIPT.json
```

## Final

```bash
bash scripts/verify_current.sh | tee handoffs/P31A_FINAL_VERIFY.log
```
