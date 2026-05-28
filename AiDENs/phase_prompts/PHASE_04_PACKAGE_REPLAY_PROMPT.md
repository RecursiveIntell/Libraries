# Phase 4 Prompt — Package Validation and Extracted Self-Replay

Use the actual repo `z.py` interface. Package with current run P31A:

```bash
python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P31A
```

Then validate sidecars and extracted package replay:

```bash
python3 scripts/assert_package_validation.py
python3 scripts/assert_package_self_replay.py --package <zip> --require-verifier --receipt-out handoffs/P31A_PACKAGE_REPLAY_RECEIPT.json
```

If replay is blocked by external path deps or missing cargo, record blocker status and keep certification false.
