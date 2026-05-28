#!/usr/bin/env python3
"""Static guard for P27 strict structured-input boundaries."""

from __future__ import annotations

import sys
from pathlib import Path


REQUIRED_SNIPPETS = [
    "fn permit_policy_from_json",
    "parse_strict_json(input).context(\"failed strict-parse permit JSON\")",
    "fn command_run_receipt_from_arg",
    "failed strict-parse CommandRunReportV1",
    "fn load_agent_spec_file",
    "failed strict-parse AgentSpecV1",
    "pub fn inspect_run_bundle_command",
    "failed strict-parse {}",
    "match parse_strict_json(&actual)",
    "\"schema_validation\": schema_validation",
    "validate_json_schema(&schema, &spec_value)",
    "validate_json_schema(&schema, &bundle)",
]


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    cli_src = root / "crates/aidens-cli/src"
    if not cli_src.exists():
        print(f"FAIL: missing {cli_src}", file=sys.stderr)
        return 2
    cli_files = sorted(cli_src.glob("*.rs"))
    text = "\n".join(path.read_text(encoding="utf-8") for path in cli_files)
    missing = [snippet for snippet in REQUIRED_SNIPPETS if snippet not in text]
    if missing:
        print("FAIL: strict structured-input guard missing snippets:", file=sys.stderr)
        for snippet in missing:
            print(f"- {snippet}", file=sys.stderr)
        return 3
    print("strict structured-input guard OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
