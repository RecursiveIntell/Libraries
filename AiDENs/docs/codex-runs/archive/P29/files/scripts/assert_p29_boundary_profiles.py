#!/usr/bin/env python3
"""Run behavioral boundary-profile checks instead of marker-string checks."""

import subprocess


TESTS = [
    ["cargo", "test", "-p", "aidens-boundary-kit"],
    ["cargo", "test", "-p", "aidens-integration-tests", "--test", "phase_09_reference_hostile_tests", "boundary_repair_hard_fails_unverifiable_treatment_change"],
    ["cargo", "test", "-p", "aidens-integration-tests", "--test", "p28_adversarial_conformance", "p28_adversarial_boundary_fixtures_fail_closed"],
]


def main() -> int:
    for command in TESTS:
        subprocess.run(command, check=True)
    print("boundary-profile behavioral checks passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as exc:
        raise SystemExit(exc.returncode)
