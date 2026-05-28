#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from pathlib import Path

ALLOWED_TESTKIT_DEPS = {
    "aidens-contracts",
    "anyhow",
    "chrono",
    "serde",
    "serde_json",
    "thiserror",
    "toml",
    "uuid",
}
FORBIDDEN_AIDENS_DEPS = {
    "aidens-agency-kit",
    "aidens-app-kit",
    "aidens-arbiter-kit",
    "aidens-boundary-kit",
    "aidens-budget-kit",
    "aidens-capability-kit",
    "aidens-cli",
    "aidens-config",
    "aidens-daemon-kit",
    "aidens-delegation-kit",
    "aidens-governance-kit",
    "aidens-kernel-kit",
    "aidens-memory-kit",
    "aidens-permit-kit",
    "aidens-plan-kit",
    "aidens-provider-kit",
    "aidens-queue-kit",
    "aidens-receipts",
    "aidens-repair-kit",
    "aidens-runner",
    "aidens-schedule-kit",
    "aidens-security-kit",
    "aidens-tool-kit",
    "aidens-wake-kit",
}
FORBIDDEN_SOURCE_IMPORT_RE = re.compile(r"\b(?:use|extern\s+crate)\s+(aidens_[A-Za-z0-9_]+)")


def dependency_names(cargo_toml: Path) -> dict[str, list[str]]:
    data = tomllib.load(open(cargo_toml, "rb"))
    return {
        "dependencies": sorted(data.get("dependencies", {})),
        "dev-dependencies": sorted(data.get("dev-dependencies", {})),
        "build-dependencies": sorted(data.get("build-dependencies", {})),
    }


def scan_source_imports(testkit_root: Path) -> list[dict[str, str]]:
    findings = []
    for source in testkit_root.rglob("*.rs"):
        rel = source.relative_to(testkit_root.parent.parent).as_posix()
        text = source.read_text(encoding="utf-8", errors="ignore")
        for match in FORBIDDEN_SOURCE_IMPORT_RE.finditer(text):
            crate_name = match.group(1).replace("_", "-")
            if crate_name in FORBIDDEN_AIDENS_DEPS:
                findings.append({"file": rel, "crate": crate_name})
    return findings


def main() -> int:
    ap = argparse.ArgumentParser(description="Verify aidens-testkit remains pure/reference-only.")
    ap.add_argument("root", nargs="?", default=".")
    ap.add_argument("--require-integration-crate", action="store_true")
    ap.add_argument("--json-out", default="target/aidens-p20-2-audit/testkit-purity.json")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    testkit = root / "crates" / "aidens-testkit"
    cargo = testkit / "Cargo.toml"
    integration_cargo = root / "crates" / "aidens-integration-tests" / "Cargo.toml"

    report = {
        "ok": True,
        "missing_testkit": False,
        "missing_integration_crate": False,
        "dependencies": {},
        "forbidden_dependencies": [],
        "forbidden_source_imports": [],
    }

    if not cargo.exists():
        report["ok"] = False
        report["missing_testkit"] = True
    else:
        deps_by_section = dependency_names(cargo)
        report["dependencies"] = deps_by_section
        forbidden = []
        for section, deps in deps_by_section.items():
            for dep in deps:
                if dep in FORBIDDEN_AIDENS_DEPS or (
                    dep.startswith("aidens-") and dep not in ALLOWED_TESTKIT_DEPS
                ):
                    forbidden.append({"section": section, "dependency": dep})
        report["forbidden_dependencies"] = forbidden
        source_imports = scan_source_imports(testkit)
        report["forbidden_source_imports"] = source_imports
        if forbidden or source_imports:
            report["ok"] = False

    if args.require_integration_crate and not integration_cargo.exists():
        report["ok"] = False
        report["missing_integration_crate"] = True

    out = root / args.json_out
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if report["ok"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
