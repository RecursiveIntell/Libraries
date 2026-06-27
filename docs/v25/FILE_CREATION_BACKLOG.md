# File creation backlog — v25

This pass intentionally created the files required to make the repo self-consistent and actionable for v25.
The following remain reasonable next-file candidates after cargo-backed validation is available:

## Consumer integration files

- `effect-runtime/src/v25.rs` or equivalent helper surface for compiled obligations
- `verification-control/src/v25.rs` or direct field additions for composite constitutional refs
- `verification-adjudication/src/v25.rs` or direct field additions for composite constitutional refs
- consumer-facing example JSONs once those schema changes are admitted

## Conformance and CI files

- CI workflow step that runs `scripts/check_v25_repo_truth.sh`
- CI workflow step that runs `python3 scripts/check_v25_json_surface.py`
- no-local-recomposition grep or lint rules once consumer integrations are real

## Generated or regenerated files

- regenerated v25 schemas from `contract-schema-gen`
- compatibility reports for any field additions in consumer crates
- updated release notes after consumer adoption lands
