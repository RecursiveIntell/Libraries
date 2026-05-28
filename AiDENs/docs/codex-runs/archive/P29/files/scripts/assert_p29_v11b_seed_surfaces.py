#!/usr/bin/env python3
"""Run behavioral v11B seed checks and forbid completion overclaims."""

from pathlib import Path
import subprocess
import sys


POLICY_ALLOWLIST = {
    "P29_FORBIDDEN_FINAL_STATE.md",
    "P29_SUPPORT_LABEL_POLICY.md",
    "P29_ACCEPTANCE_GATES.md",
    "P29_MASTER_PACKET.md",
}


def assert_no_v11b_completion_claim() -> None:
    for path in list(Path(".").glob("*.md")) + list(Path("docs").glob("**/*.md")):
        if path.name in POLICY_ALLOWLIST:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore").lower()
        for needle in ("v11b-complete", "v11b complete"):
            idx = text.find(needle)
            if idx < 0:
                continue
            context = text[max(0, idx - 120): idx + 160]
            if any(
                allowed in context
                for allowed in ("forbidden", "must not", "not ", "no v11b-complete", "no v11b complete")
            ):
                continue
            raise RuntimeError(f"forbidden v11B completion claim: {path}")


def main() -> int:
    subprocess.run(
        ["cargo", "test", "-p", "aidens-integration-tests", "--test", "phase_10_minimal_v11b_region"],
        check=True,
    )
    subprocess.run(
        ["cargo", "test", "-p", "aidens-contracts", "--lib", "p10_minimal_v11b_region_seed_covers_failure_repair_support_and_oracle_diff"],
        check=True,
    )
    assert_no_v11b_completion_claim()
    print("v11B seed behavioral checks passed without completion claim")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as exc:
        raise SystemExit(exc.returncode)
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
