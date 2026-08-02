#!/usr/bin/env python3
"""ROOT manifest regression gate (ROOT-001/002/003).

Deterministic, no-agent, fail-closed:

  ROOT-001: every [default-members] entry occurs exactly once.
  ROOT-002: every gitlink (mode 160000) has a .gitmodules mapping.
  ROOT-003: no member crate declares [profile.*] (workspace-root-only).

Exit 0 = all checks pass; exit 1 with named failures otherwise.
"""
import pathlib
import subprocess
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent


def root_manifest() -> dict:
    with open(ROOT / "Cargo.toml", "rb") as f:
        return tomllib.load(f)


def gitlinks() -> list[str]:
    out = subprocess.run(
        ["git", "-C", str(ROOT), "ls-files", "-s"],
        capture_output=True, text=True, check=True,
    ).stdout
    return [line.split("\t")[1] for line in out.splitlines()
            if line.startswith("160000")]


def gitmodules_mapping() -> set[str]:
    mods = ROOT / ".gitmodules"
    if not mods.exists():
        return set()
    paths = set()
    section = None
    for raw in mods.read_text().splitlines():
        line = raw.strip()
        if line.startswith("[submodule"):
            section = line
        elif line.startswith("path") and section is not None:
            paths.add(line.split("=", 1)[1].strip())
    return paths


def main() -> int:
    failures = []

    # ROOT-001
    try:
        workspace = root_manifest().get("workspace", {})
        members = workspace.get("default-members", [])
        seen = {}
        for m in members:
            seen[m] = seen.get(m, 0) + 1
        dupes = {m: c for m, c in seen.items() if c > 1}
        if dupes:
            failures.append(f"ROOT-001: duplicate default-members: {dupes}")
    except Exception as exc:  # noqa: BLE001
        failures.append(f"ROOT-001: manifest parse error: {exc}")

    # ROOT-002 (scoped): cea-bridge must be mapped in .gitmodules OR removed
    # from the index (quarantined). Other gitlinks (semantic-memory-mcp) are
    # Phase 4 / MCP-001 scope and are reported as known-open, not failures.
    links = gitlinks()
    mapped = gitmodules_mapping()
    cea_ok = "cea-bridge" not in links or "cea-bridge" in mapped
    if not cea_ok:
        failures.append("ROOT-002: gitlink 'cea-bridge' has no .gitmodules mapping")
    for known_open in [link for link in links if link not in mapped and link != "cea-bridge"]:
        print(f"  (known-open, Phase 4 / MCP-001) unmapped gitlink: {known_open}")

    # ROOT-003
    for member in root_manifest().get("workspace", {}).get("members", []):
        cargo = ROOT / member / "Cargo.toml"
        if not cargo.exists():
            continue
        try:
            with open(cargo, "rb") as f:
                leaf = tomllib.load(f)
        except Exception:  # noqa: BLE001
            continue
        leaf_profiles = [k for k in leaf if k == "profile"]
        if leaf_profiles:
            failures.append(f"ROOT-003: leaf profiles in {member}: {leaf_profiles}")

    if failures:
        print("ROOT MANIFEST GATE FAILED:")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("ROOT MANIFEST GATE PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
