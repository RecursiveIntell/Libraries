#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REQUIRED = [
    ".codex/config.toml",
    ".codex/hooks.json",
    ".codex/prompt_manifest.json",
    ".codex/prompts/MASTER_AUTOMATED_COMPLETION.md",
    ".codex/tools/auto_phase_runner.py",
    ".agents/skills/phase-gate/SKILL.md",
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--quiet", action="store_true")
    args = parser.parse_args()
    missing = [p for p in REQUIRED if not (ROOT / p).exists()]
    if not args.quiet:
        if missing:
            print("missing Codex setup files:")
            for item in missing:
                print(f"- {item}")
        else:
            print("Codex setup present")
    return 1 if missing else 0


if __name__ == "__main__":
    raise SystemExit(main())
