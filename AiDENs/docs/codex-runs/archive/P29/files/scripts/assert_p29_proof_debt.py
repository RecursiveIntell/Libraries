#!/usr/bin/env python3
"""Run behavioral proof-debt checks instead of marker-string checks."""

import subprocess


TESTS = [
    ["cargo", "test", "-p", "aidens-contracts", "--lib", "p09_proof_debt_is_queryable_and_expired_waiver_escalates"],
    ["cargo", "test", "-p", "aidens-contracts", "--lib", "p28_proof_waiver_is_not_proof_and_debt_blocks_promotion"],
    ["cargo", "test", "-p", "aidens-contracts", "--lib", "p28_proof_evidence_satisfies_profile_and_allows_promotion"],
]


def main() -> int:
    for command in TESTS:
        subprocess.run(command, check=True)
    print("proof-debt behavioral checks passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as exc:
        raise SystemExit(exc.returncode)
