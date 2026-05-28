#!/usr/bin/env python3
import csv
import argparse
import json
import re
import sys
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--root", default=".")
parser.add_argument(
    "--aidens-overlay-only",
    action="store_true",
    help="Allow an AiDENs-only overlay scan without canonical sibling crates.",
)
args = parser.parse_args()

ROOT = Path(args.root).resolve()
CANONICAL_ROOT = ROOT.parent.resolve()
AIDENS_CONTRACTS_SRC = ROOT / "crates" / "aidens-contracts" / "src"
OUT_DIR = ROOT / "docs" / "contract-ownership"
OUT_DIR.mkdir(parents=True, exist_ok=True)
STATUS_PATH = OUT_DIR / "OWNERSHIP_SCAN_STATUS.json"

TYPE_RE = re.compile(r"^\s*pub\s+(struct|enum|type)\s+([A-Za-z][A-Za-z0-9_]*)\b", re.M)
PUBUSE_RE = re.compile(r"^\s*pub\s+use\s+(.+?);\s*$", re.M)

EXCLUDE_DIR_NAMES = {
    ".git", "target", "AiDENs", "aidens", "Libraries2", "libraries2"
}
CANONICAL_CRATES = {
    "stack-ids",
    "semantic-memory-forge",
    "forge-memory-bridge",
    "semantic-memory",
    "knowledge-runtime",
    "llm-tool-runtime",
    "verification-control",
    "verification-policy",
    "verification-adjudication",
    "verification-calibration",
    "recursive-kernel-core",
    "constraint-compiler",
    "kernel-execution",
    "kernel-oracles",
    "kernel-conformance",
    "attestation-exchange",
    "federated-settlement",
    "mechanism-runtime",
    "remote-oracle-admission",
    "contract-schema-gen",
    "forge-pilot",
}

def rust_files_under(crate_dir: Path):
    if not crate_dir.exists():
        return
    for p in crate_dir.rglob("*.rs"):
        parts = set(p.relative_to(crate_dir).parts)
        if "target" in parts or ".git" in parts:
            continue
        yield p

def line_no(text, idx):
    return text[:idx].count("\n") + 1

def scan_types(path: Path, root: Path, owner: str):
    rows = []
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except Exception:
        return rows
    for m in TYPE_RE.finditer(text):
        rows.append({
            "type_name": m.group(2),
            "kind": m.group(1),
            "owner": owner,
            "file": str(path.relative_to(root)),
            "line": line_no(text, m.start()),
            "definition_kind": "local_def",
        })
    for m in PUBUSE_RE.finditer(text):
        rows.append({
            "type_name": "",
            "kind": "pub_use",
            "owner": owner,
            "file": str(path.relative_to(root)),
            "line": line_no(text, m.start()),
            "definition_kind": "pub_use",
            "pub_use": m.group(1).strip(),
        })
    return rows

canonical_rows = []
for crate in sorted(CANONICAL_CRATES):
    crate_dir = CANONICAL_ROOT / crate
    if not crate_dir.exists():
        continue
    for f in rust_files_under(crate_dir):
        canonical_rows.extend(scan_types(f, CANONICAL_ROOT, crate))

aidens_rows = []
if AIDENS_CONTRACTS_SRC.exists():
    for f in rust_files_under(AIDENS_CONTRACTS_SRC):
        aidens_rows.extend(scan_types(f, ROOT, "aidens-contracts"))

with (OUT_DIR / "CANONICAL_TYPE_INVENTORY.csv").open("w", newline="") as fh:
    w = csv.DictWriter(fh, fieldnames=["type_name","kind","owner","file","line","definition_kind","pub_use"])
    w.writeheader()
    for r in canonical_rows:
        r.setdefault("pub_use", "")
        w.writerow(r)

with (OUT_DIR / "AIDENS_CONTRACTS_TYPE_INVENTORY.csv").open("w", newline="") as fh:
    w = csv.DictWriter(fh, fieldnames=["type_name","kind","owner","file","line","definition_kind","pub_use"])
    w.writeheader()
    for r in aidens_rows:
        r.setdefault("pub_use", "")
        w.writerow(r)

with (OUT_DIR / "TYPE_OWNERSHIP_INVENTORY.csv").open("w", newline="") as fh:
    w = csv.DictWriter(fh, fieldnames=["type_name","kind","owner","file","line","definition_kind","pub_use"])
    w.writeheader()
    for r in canonical_rows + aidens_rows:
        r.setdefault("pub_use", "")
        w.writerow(r)

canon_by_name = {}
for r in canonical_rows:
    if r["definition_kind"] == "local_def":
        canon_by_name.setdefault(r["type_name"], []).append(r)

findings = []
for r in aidens_rows:
    if r["definition_kind"] != "local_def":
        continue
    for c in canon_by_name.get(r["type_name"], []):
        findings.append({
            "type_name": r["type_name"],
            "aidens_file": r["file"],
            "aidens_line": r["line"],
            "canonical_owner": c["owner"],
            "canonical_file": c["file"],
            "canonical_line": c["line"],
            "severity": "P0" if r["type_name"] in {
                "AttestationEnvelopeV1",
                "SharedDispositionV1",
                "SettlementCaseV1",
                "TheoryRefuterSuiteV1",
                "TheoryVersionV1",
                "HypothesisLibraryV1",
            } else "P1_REVIEW",
        })

with (OUT_DIR / "CANONICAL_DUPLICATE_FINDINGS.csv").open("w", newline="") as fh:
    w = csv.DictWriter(fh, fieldnames=["type_name","aidens_file","aidens_line","canonical_owner","canonical_file","canonical_line","severity"])
    w.writeheader()
    for r in findings:
        w.writerow(r)

# Also write final aliases for convenience.
for src, dst in [
    ("TYPE_OWNERSHIP_INVENTORY.csv", "FINAL_TYPE_OWNERSHIP_INVENTORY.csv"),
]:
    srcp = OUT_DIR / src
    if srcp.exists():
        (OUT_DIR / dst).write_text(srcp.read_text())

canonical_local_def_count = len([r for r in canonical_rows if r["definition_kind"] == "local_def"])
aidens_local_def_count = len([r for r in aidens_rows if r["definition_kind"] == "local_def"])
canonical_inventory_unavailable = canonical_local_def_count == 0
status = {
    "artifact_kind": "local_operator_ownership_scan_status",
    "support_tier": "verification",
    "semantic_status": "exact_check",
    "root": str(ROOT),
    "canonical_root": str(CANONICAL_ROOT),
    "aidens_overlay_only": bool(args.aidens_overlay_only),
    "canonical_inventory_unavailable": canonical_inventory_unavailable,
    "canonical_local_def_count": canonical_local_def_count,
    "aidens_contracts_local_def_count": aidens_local_def_count,
    "duplicate_findings": len(findings),
    "known_limits": [],
}
if canonical_inventory_unavailable:
    status["known_limits"].append(
        "Canonical sibling crates were unavailable or yielded no local type definitions; duplicate-free claims are not authoritative."
    )
STATUS_PATH.write_text(json.dumps(status, indent=2, sort_keys=True) + "\n", encoding="utf-8")

print(f"canonical_types={canonical_local_def_count}")
print(f"aidens_contracts_types={aidens_local_def_count}")
print(f"duplicate_findings={len(findings)}")
print(f"canonical_inventory_unavailable={str(canonical_inventory_unavailable).lower()}")
if not args.aidens_overlay_only and canonical_inventory_unavailable:
    print(
        "FAIL: canonical_inventory_unavailable=true; canonical type inventory is empty. "
        "Rerun with canonical sibling crates available or pass --aidens-overlay-only for an overlay-only package scan.",
        file=sys.stderr,
    )
    sys.exit(2)
if findings:
    for f in findings:
        print(f"{f['severity']}: {f['type_name']} local {f['aidens_file']}:{f['aidens_line']} canonical {f['canonical_owner']} {f['canonical_file']}:{f['canonical_line']}")
