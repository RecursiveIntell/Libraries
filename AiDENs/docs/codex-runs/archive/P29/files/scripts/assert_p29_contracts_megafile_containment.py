#!/usr/bin/env python3
"""Assert P29 contracts megafile containment remains in place."""

from pathlib import Path
import sys


ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
CONTRACTS_SRC = ROOT / "crates" / "aidens-contracts" / "src"
LIB = CONTRACTS_SRC / "lib.rs"
REQUIRED_MODULES = {
    "artifact.rs": ("mod artifact;", "pub use artifact::*;"),
    "boundary.rs": ("mod boundary;", "pub use boundary::*;"),
    "execution.rs": ("mod execution;", "pub use execution::*;"),
    "operator.rs": ("mod operator;", "pub use operator::*;"),
    "proof.rs": ("mod proof;", "pub use proof::*;"),
    "semantic.rs": ("mod semantic;", "pub use semantic::*;"),
    "tests.rs": ("#[cfg(test)]", "mod tests;"),
}
MAX_LIB_LINES = 900


def line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def main() -> int:
    failures: list[str] = []
    if not LIB.exists():
        failures.append(f"missing contracts facade: {LIB}")
    else:
        lib_text = LIB.read_text(encoding="utf-8")
        lib_lines = line_count(LIB)
        if lib_lines > MAX_LIB_LINES:
            failures.append(
                f"contracts facade too large: {lib_lines} lines > {MAX_LIB_LINES}"
            )
        for module_name, required_snippets in REQUIRED_MODULES.items():
            module_path = CONTRACTS_SRC / module_name
            if not module_path.exists():
                failures.append(f"missing contracts containment module: {module_path}")
                continue
            if line_count(module_path) < 20:
                failures.append(f"contracts containment module is empty: {module_path}")
            for snippet in required_snippets:
                if snippet not in lib_text:
                    failures.append(f"contracts facade missing snippet: {snippet}")

    if failures:
        print("P29 contracts megafile containment FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("P29 contracts megafile containment OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
