#!/usr/bin/env python3
"""Assert P29 CLI megafile containment remains in place."""

from pathlib import Path
import sys


ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
CLI_SRC = ROOT / "crates" / "aidens-cli" / "src"
LIB = CLI_SRC / "lib.rs"
REQUIRED_MODULES = {
    "agent.rs": ("mod agent;", "pub use agent::*;"),
    "package.rs": ("mod package;", "pub use package::*;"),
    "tests.rs": ("#[cfg(test)]", "mod tests;"),
}
MAX_LIB_LINES = 5200


def line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def main() -> int:
    failures: list[str] = []
    if not LIB.exists():
        failures.append(f"missing CLI facade: {LIB}")
    else:
        lib_text = LIB.read_text(encoding="utf-8")
        lib_lines = line_count(LIB)
        if lib_lines > MAX_LIB_LINES:
            failures.append(f"CLI facade too large: {lib_lines} lines > {MAX_LIB_LINES}")
        for module_name, required_snippets in REQUIRED_MODULES.items():
            module_path = CLI_SRC / module_name
            if not module_path.exists():
                failures.append(f"missing CLI containment module: {module_path}")
                continue
            if line_count(module_path) < 20:
                failures.append(f"CLI containment module is empty: {module_path}")
            for snippet in required_snippets:
                if snippet not in lib_text:
                    failures.append(f"CLI facade missing snippet: {snippet}")

    if failures:
        print("P29 CLI megafile containment FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("P29 CLI megafile containment OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
