#!/usr/bin/env python3
"""Run behavioral receipt-chain checks instead of marker-string checks."""

import subprocess


TESTS = [
    ["cargo", "test", "-p", "aidens-receipts", "phase01_concurrent_canonical_appends_keep_single_digest_chain"],
    ["cargo", "test", "-p", "aidens-receipts", "phase01_duplicate_receipt_ids_are_rejected"],
    ["cargo", "test", "-p", "aidens-receipts", "phase01_corrupt_trailing_record_is_quarantined_not_history_poisoning"],
    ["cargo", "test", "-p", "aidens-receipts", "p28_canonical_log_digest_chain_detects_tampering"],
]


def main() -> int:
    for command in TESTS:
        subprocess.run(command, check=True)
    print("receipt-chain behavioral checks passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as exc:
        raise SystemExit(exc.returncode)
