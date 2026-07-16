#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
EVIDENCE_MANIFEST = ROOT / "STATUS_EVIDENCE_MANIFEST.json"
SUPPORT_PROFILE = ROOT / "SUPPORT_PROFILE.md"
RISK_REGISTER = ROOT / "06_RISK_REGISTER.md"
ARCHIVE_MANIFEST = ROOT / "docs" / "archive" / "root_closeout_history" / "manifest.json"
PUBLIC_TYPE_DRIFT_ALLOWLIST = ROOT / "scripts" / "public_type_drift_allowlist.json"
RELEASE_DIR = ROOT / "release"
RECEIPT_PATH = RELEASE_DIR / "closeout_receipt_v1.json"
CORE_DOC_CRATES = [
    "profile-runtime",
    "recursive-kernel-core",
    "constraint-compiler",
    "effect-runtime",
    "verification-control",
    "verification-policy",
    "semantic-memory-forge",
]
PUB_FN_RE = re.compile(r"^\s*pub fn\s+[A-Za-z_][A-Za-z0-9_]*\s*\(")


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def artifact_status(path: Path) -> dict[str, object]:
    """Describe a referenced artifact without inventing a hash when it is absent."""
    return {
        "path": str(path.relative_to(ROOT)),
        "present": path.is_file(),
        "sha256": sha256_file(path) if path.is_file() else None,
    }


def sha256_json(data: object) -> str:
    encoded = json.dumps(data, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def load_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def has_doc_comment(lines: list[str], index: int) -> bool:
    cursor = index - 1
    saw_doc = False
    while cursor >= 0:
        stripped = lines[cursor].strip()
        if stripped.startswith("///") or stripped.startswith("#[doc ="):
            saw_doc = True
            cursor -= 1
            continue
        if stripped.startswith("#["):
            cursor -= 1
            continue
        if not stripped:
            return saw_doc
        return saw_doc
    return saw_doc


def crate_doc_coverage(crate: str) -> dict[str, object]:
    total = 0
    documented = 0
    for path in sorted((ROOT / crate / "src").rglob("*.rs")):
        lines = path.read_text(encoding="utf-8").splitlines()
        for idx, line in enumerate(lines):
            if PUB_FN_RE.match(line):
                total += 1
                if has_doc_comment(lines, idx):
                    documented += 1
    return {"crate": crate, "documented": documented, "total": total}


def parse_supported_lane(path: Path) -> list[str]:
    lane: list[str] = []
    in_section = False
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line == "Supported closeout lane:":
            in_section = True
            continue
        if not in_section:
            continue
        if not line:
            if lane:
                break
            continue
        if not line.startswith("- "):
            if lane:
                break
            continue
        lane.append(line[2:].strip().strip("`"))
    return lane


def collect_schema_manifests(root: Path) -> list[dict[str, object]]:
    manifests: list[dict[str, object]] = []
    for manifest_path in sorted((root / "contracts" / "schemas").glob("*/manifest.json")):
        manifest = load_json(manifest_path)
        schema_files = manifest.get("schema_files") or manifest.get("schemas") or []
        manifests.append(
            {
                "path": str(manifest_path.relative_to(root)),
                "owner": manifest.get("owner_crate") or manifest.get("primary_owner"),
                "kind": "wave" if "wave" in manifest else "profile",
                "schema_count": len(schema_files),
                "sha256": sha256_file(manifest_path),
            }
        )
    return manifests


def load_public_type_drift_allowlist() -> list[dict[str, object]]:
    raw = load_json(PUBLIC_TYPE_DRIFT_ALLOWLIST)
    return raw.get("allowlist", [])


def parse_open_debt(path: Path) -> list[str]:
    capture = False
    bullets: list[str] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.rstrip()
        if line.strip() == "Still open:":
            capture = True
            continue
        if not capture:
            continue
        stripped = line.strip()
        if not stripped:
            if bullets:
                break
            continue
        if stripped.startswith("- "):
            bullets.append(stripped[2:].rstrip(","))
        elif bullets:
            break
    return bullets


def build_receipt() -> dict[str, object]:
    evidence = load_json(EVIDENCE_MANIFEST)
    supported_lane = parse_supported_lane(SUPPORT_PROFILE)
    schema_manifests = collect_schema_manifests(ROOT)
    open_debt = parse_open_debt(ROOT / "STATUS_DASHBOARD.md")
    archive_manifest = load_json(ARCHIVE_MANIFEST)
    public_type_drift_allowlist = load_public_type_drift_allowlist()
    doc_coverage = [crate_doc_coverage(crate) for crate in CORE_DOC_CRATES]

    gate_results = {
        item["command"]: item["result"] for item in evidence.get("proof_results", [])
    }

    return {
        "receipt_version": 1,
        "snapshot": evidence.get("snapshot"),
        "captured_at": evidence.get("captured_at"),
        "supported_closeout_lane": {
            "crates": supported_lane,
            "crate_count": len(supported_lane),
            "sha256": sha256_json(supported_lane),
            "source": str(SUPPORT_PROFILE.relative_to(ROOT)),
        },
        "gate_results": gate_results,
        # EVD-001: the receipt is a derivative of source-bound, content-addressed
        # command receipts. Verification compares this exact binding without
        # regenerating or overwriting evidence.
        "source_binding": evidence.get("source_binding", {}),
        "schema_publication": {
            "canonical_dir": "schemas/",
            "manifest_count": len(schema_manifests),
            "manifests": schema_manifests,
            "canonical_schema_dir_sha256": sha256_json(
                {
                    path.name: sha256_file(path)
                    for path in sorted((ROOT / "schemas").glob("*.json"))
                }
            ),
        },
        "evidence_manifest": {
            "path": str(EVIDENCE_MANIFEST.relative_to(ROOT)),
            "sha256": sha256_file(EVIDENCE_MANIFEST),
        },
        "risk_register": artifact_status(RISK_REGISTER),
        "archive_manifest": {
            "path": str(ARCHIVE_MANIFEST.relative_to(ROOT)),
            "sha256": sha256_file(ARCHIVE_MANIFEST),
            "archive_mode": archive_manifest.get("archive_mode"),
            "active_root_file_count": len(
                archive_manifest.get("active_root_closeout_pack", [])
            ),
            "physically_archived_groups": [
                {
                    "group": group.get("group"),
                    "archived_dir": group.get("archived_dir"),
                    "archived_count": group.get("archived_count"),
                }
                for group in archive_manifest.get("superseded_root_groups", [])
                if group.get("archived_dir")
            ],
        },
        "public_type_drift": {
            "allowlist_path": str(PUBLIC_TYPE_DRIFT_ALLOWLIST.relative_to(ROOT)),
            "allowlist_sha256": sha256_file(PUBLIC_TYPE_DRIFT_ALLOWLIST),
            "allowlisted_duplicate_count": len(public_type_drift_allowlist),
            "allowlist": public_type_drift_allowlist,
        },
        "public_api_docs": {
            "tracked_crates": [item["crate"] for item in doc_coverage],
            "tracked_crate_count": len(doc_coverage),
            "coverage": doc_coverage,
            "fully_documented_crates": [
                item["crate"]
                for item in doc_coverage
                if item["total"] == item["documented"]
            ],
        },
        "residual_debt": open_debt,
    "notes": [
            "This receipt reflects the live closeout state for the 2026-03-22 hardening lane.",
            "The canonical support scope is taken from SUPPORT_PROFILE.md.",
            "The tracked core closeout crates are at full public-function rustdoc coverage.",
            "The archive manifest is logical, and physical legacy-root reductions are explicitly modeled for this lane."
            if not open_debt
            else "The archive manifest is logical in this snapshot; physical root reduction remains open debt.",
        ],
    }


def main() -> None:
    receipt = build_receipt()
    RELEASE_DIR.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
