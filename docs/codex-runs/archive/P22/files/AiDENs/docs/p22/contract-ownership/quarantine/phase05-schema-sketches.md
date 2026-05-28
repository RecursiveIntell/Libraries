# Quarantine: phase05-schema-sketches

STATUS: quarantined historical sketches
DISCOVERED_IN_PHASE: 05
LOCAL FILES:
- `schemas/core_artifact_header_v1.sketch.json`
- `schemas/execution_lineage_graph_v1.sketch.json`
- `schemas/poison_receipt_record_v1.sketch.json`

SUSPECTED CANONICAL OWNER(S): `stack-ids`, `semantic-memory-forge`, `kernel-execution`, `verification-control`, `contract-schema-gen`

SEARCHES PERFORMED:
- `find schemas -name '*.schema.json'`
- `python3 scripts/assert_schema_generation_scope.py`
- `rg -n "sha256:|canonical-digest|CanonicalDigestV1|attestation-envelope|settlement-case|shared-disposition|theory-version|theory-refuter-suite|hypothesis-library" schemas`

WHY AUTOMATIC COLLAPSE IS UNSAFE:
These files are historical `*.sketch.json` design notes, not generated `*.schema.json` artifacts and not part of the schema compatibility gate. They still mention legacy `sha256:` examples and canonical-looking artifact header/lineage terms. Converting them into generated schemas, deleting their historical context, or rewriting their semantics as AiDENs-local schema law would blur the owner boundary.

TEMPORARY ACTION TAKEN:
The Phase 05 schema generator and checked-in generated `*.schema.json` set no longer emit canonical family schemas. `schemas/README.md` now states that sketches are historical design sketches only. The ambiguous sketch files are recorded here for human owner review.

FORBIDDEN ACTIONS:
- Do not promote these sketches into AiDENs-generated schemas.
- Do not treat their `sha256:` examples as canonical digest law.
- Do not use them as schema authority for canonical artifact headers, lineage, poison records, evidence, or digest/content-addressing.

REQUIRED HUMAN DECISION:
Decide whether to archive these sketches outside `schemas/`, rewrite them as non-authoritative docs, or replace them with owner-generated schemas from canonical crates / `contract-schema-gen`.

RECOMMENDED NEXT RUN:
Handle sketch archival in the final docs/auditor phase after wrapper/backpointer decisions are complete.
