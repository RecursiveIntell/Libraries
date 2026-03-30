#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def require_pattern(path: str, pattern: str, description: str) -> str | None:
    target = ROOT / path
    if not target.exists():
        print(f"{path}: skipped (path excluded from workspace)")
        return None
    content = target.read_text(encoding="utf-8")
    if re.search(pattern, content, re.MULTILINE | re.DOTALL):
        return None
    return f"{path}: missing {description}"


def forbid_pattern(path: str, pattern: str, description: str) -> str | None:
    target = ROOT / path
    if not target.exists():
        return None  # absent file cannot contain forbidden pattern
    content = target.read_text(encoding="utf-8")
    if re.search(pattern, content, re.MULTILINE | re.DOTALL):
        return f"{path}: found forbidden {description}"
    return None


def main() -> int:
    failures = [
        require_pattern(
            "llm-tool-runtime/src/runtime.rs",
            r"pub\s+async\s+fn\s+execute\s*\(\s*&self,\s*ctx:\s*&crate::ToolCtx,\s*call:\s*&ToolCall,\s*permit:\s*Option<&ToolExecutionPermit>",
            "permit-bearing ToolRuntime::execute signature",
        ),
        require_pattern(
            "llm-tool-runtime/src/runtime.rs",
            r"ctx\.execution_permit\.as_ref\(\)",
            "runtime enforcement against the injected execution permit",
        ),
        require_pattern(
            "llm-tool-runtime/src/starter_tools.rs",
            r"scoped_execution_permit\(",
            "starter-tool direct-invoke permit checks",
        ),
        require_pattern(
            "forge-pilot/src/act.rs",
            r"pub\s+async\s+fn\s+execute_plan\s*\(\s*observation:\s*&Observation,\s*target_key:\s*&str,\s*plan:\s*&PlanKind,\s*permit:\s*&ExecutionPermit,",
            "forge-pilot execute_plan permit gate",
        ),
        require_pattern(
            "LLM-Pipeline/src/tool_loop.rs",
            r"\.execute\(\s*&tool_ctx,\s*&call,\s*request\.execution_permit\.as_ref\(\),\s*ctx\.cancel_flag\(\),\s*\)",
            "llm-pipeline permit forwarding",
        ),
        forbid_pattern(
            "llm-tool-runtime/src/contracts.rs",
            r"approval_token",
            "raw approval token field",
        ),
    ]

    failures = [failure for failure in failures if failure is not None]
    if failures:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1

    print("commit permit path checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
