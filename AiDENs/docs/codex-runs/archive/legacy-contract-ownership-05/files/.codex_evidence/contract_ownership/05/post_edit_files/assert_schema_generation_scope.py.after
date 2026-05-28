#!/usr/bin/env python3
import re
import sys
from pathlib import Path

ROOT = Path.cwd()
FILE = ROOT / "crates" / "aidens-contracts" / "src" / "lib.rs"
SCHEMAS_DIR = ROOT / "schemas"

if not FILE.exists():
    print("error: aidens-contracts lib not found", file=sys.stderr)
    sys.exit(2)

text = FILE.read_text(encoding="utf-8", errors="replace")

forbidden_schema_families = {
    "admission-decision",
    "attestation-envelope",
    "canonical-digest",
    "claim-bundle",
    "compiled-region-graph",
    "compaction-report",
    "convergence-report",
    "degradation-event",
    "differential-conformance-finding",
    "episode-bundle",
    "evidence-bundle",
    "execution-context",
    "execution-lineage-graph",
    "execution-receipt",
    "fit-run-report",
    "fit-run-receipt",
    "history-preservation-report",
    "hypothesis-library",
    "invariance-report",
    "invariant-budget",
    "json-repair-receipt",
    "kernel-run-report",
    "kernel-run-receipt",
    "mechanism-bundle",
    "mechanism-report",
    "oracle-slice-request",
    "poison-receipt-record",
    "projection-digest",
    "query-widening-receipt",
    "reference-case",
    "reference-interpreter-report",
    "region-contract",
    "remote-oracle-report",
    "remote-oracle-receipt",
    "removal-frontier",
    "residual",
    "retrieval-policy",
    "runtime-view-request",
    "schema-validation-receipt",
    "settlement-case",
    "shared-disposition",
    "simulator-contract",
    "compaction-receipt",
    "stop-rule-receipt",
    "subtraction-plan",
    "support-core",
    "syndrome",
    "theory-refuter-suite",
    "theory-version",
    "treaty",
    "trust-root",
    "verification-plan",
    "view-disclosure-receipt",
}

registered_families = set(
    re.findall(r'schema_document!\(\s*"([^"]+)"', text, flags=re.M)
)

fail = False
for family in sorted(forbidden_schema_families & registered_families):
    print(f"FAIL: aidens-contracts emits/registers canonical schema family: {family}")
    fail = True

schema_files = []
if SCHEMAS_DIR.exists():
    schema_files = sorted(SCHEMAS_DIR.rglob("*.schema.json"))
    for path in schema_files:
        relative = path.relative_to(SCHEMAS_DIR).as_posix()
        family = path.parent.name
        if family in forbidden_schema_families:
            print(f"FAIL: AiDENs checked-in schema is canonical-owned: {relative}")
            fail = True

# Registry types are allowed only if they are clearly AiDENs-local.
risky_types = [
    "ArtifactFamilyRegistryV1",
    "ArtifactFamilyRegistrationV1",
    "GeneratedSchemaManifestV1",
    "GeneratedSchemaDocumentV1",
]
for ty in risky_types:
    match = re.search(rf"^\s*pub\s+(struct|enum|type)\s+{re.escape(ty)}\b", text, re.M)
    if match:
        window = text[max(0, match.start() - 800): match.end() + 800].lower()
        if "non-authoritative" not in window and "aidens-local" not in window and "display" not in window:
            print(f"FAIL: {ty} exists without clear non-authoritative/AiDENs-local scoping.")
            fail = True

if fail:
    print("Schema generation for canonical artifact families must route through canonical owners / contract-schema-gen.")
    sys.exit(1)

print(
    "PASS: schema generation scope appears AiDENs-local/non-authoritative "
    f"(registered_families={len(registered_families)}, checked_schema_files={len(schema_files)})."
)
