#!/usr/bin/env python3
"""Assert P29 module budgets and ownership-oriented splits are present."""

from pathlib import Path
import sys


ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")

MODULE_BUDGETS = {
    "crates/aidens-cli/src/lib.rs": 5000,
    "crates/aidens-tool-kit/src/lib.rs": 3300,
    "crates/aidens-contracts/src/tests.rs": 3400,
    "crates/aidens-runner/src/lib.rs": 1900,
    "crates/aidens-provider-kit/src/lib.rs": 1600,
    "crates/aidens-boundary-kit/src/lib.rs": 1800,
}

REQUIRED_SPLITS = {
    "crates/aidens-cli/src/lib.rs": {
        "modules": ["agent.rs", "package.rs", "tests.rs"],
        "snippets": ["mod agent;", "mod package;", "mod tests;"],
    },
    "crates/aidens-tool-kit/src/lib.rs": {
        "modules": ["canonical_stack.rs", "exposure.rs"],
        "snippets": ["pub mod canonical_stack;", "mod exposure;", "pub use exposure::"],
        "forbidden_snippets": ["pub mod canonical_stack {"],
    },
    "crates/aidens-runner/src/lib.rs": {
        "modules": ["execution.rs", "finalization.rs", "provider_tool.rs", "receipts.rs", "replay.rs", "tests.rs"],
        "snippets": ["mod execution;", "mod finalization;", "mod provider_tool;", "mod receipts;", "mod replay;", "mod tests;"],
    },
    "crates/aidens-contracts/src/lib.rs": {
        "modules": ["artifact.rs", "boundary.rs", "execution.rs", "operator.rs", "proof.rs", "semantic.rs", "reserved_v11.rs", "schema_catalog.rs", "tests.rs"],
        "snippets": ["mod artifact;", "mod boundary;", "mod execution;", "mod operator;", "mod proof;", "mod semantic;", "mod reserved_v11;", "mod schema_catalog;", "mod tests;"],
    },
}


def line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def main() -> int:
    failures: list[str] = []
    for rel, max_lines in MODULE_BUDGETS.items():
        path = ROOT / rel
        if not path.exists():
            failures.append(f"missing budgeted file: {rel}")
            continue
        actual = line_count(path)
        if actual > max_lines:
            failures.append(f"module budget exceeded: {rel} has {actual} lines > {max_lines}")

    for rel, spec in REQUIRED_SPLITS.items():
        facade = ROOT / rel
        if not facade.exists():
            failures.append(f"missing facade file: {rel}")
            continue
        text = facade.read_text(encoding="utf-8")
        base = facade.parent
        for module_name in spec.get("modules", []):
            module_path = base / module_name
            if not module_path.exists():
                failures.append(f"missing ownership split module: {module_path.relative_to(ROOT)}")
            elif line_count(module_path) < 10:
                failures.append(f"ownership split module is too small to be meaningful: {module_path.relative_to(ROOT)}")
        for snippet in spec.get("snippets", []):
            if snippet not in text:
                failures.append(f"facade missing ownership split snippet in {rel}: {snippet}")
        for snippet in spec.get("forbidden_snippets", []):
            if snippet in text:
                failures.append(f"facade still owns inline implementation in {rel}: {snippet}")

    if failures:
        print("P29 module ownership boundaries FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("P29 module ownership boundaries OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
