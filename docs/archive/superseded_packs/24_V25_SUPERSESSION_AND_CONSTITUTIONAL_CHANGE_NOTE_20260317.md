# 24_V25_SUPERSESSION_AND_CONSTITUTIONAL_CHANGE_NOTE_20260317

Supersession note (2026-03-17)

This repo superseded the earlier no-v25 terminal position on 2026-03-17.

What changed:

- `profile-runtime` became the v25 owner crate for effective-constitution composition outputs.
- `stack-ids` became the authority for the shared v25 identity and citation primitives.
- `contract-schema-gen`, `schemas/`, and `examples/` became part of the shipped v25 proof surface.
- repo-local v25 closure work moved from speculative planning into executable checks under `scripts/` and `docs/v25/`.

What this note does not claim:

- It does not claim full v25 production closure by itself.
- It does not widen the supported hardening lane beyond what `SUPPORT_PROFILE.md` names.
- It does not replace the current closure proofs in `STATUS_EVIDENCE_MANIFEST.json` and `release/closeout_receipt_v1.json`.

Current authority remains the live repo truth:

- `scripts/check_v25_repo_truth.sh`
- `scripts/check_v25_production_closure.py`
- `docs/v25/README.md`
