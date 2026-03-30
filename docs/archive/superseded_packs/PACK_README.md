# Pack README

This is the active hardening pack at the repo root.

The front door is `make gate`, and that command is expected to prove three surfaces together:

1. the numbered hostile-finish pack required by `scripts/check_pack_truth.sh`,
2. the active unnumbered hardening pack documented here, and
3. the release cargo lane claimed in `SUPPORT_PROFILE.md`.

The canonical release gate list lives in `scripts/release_gate_set.py`.
`make gate` runs that exact list, rewrites `STATUS_EVIDENCE_MANIFEST.json`, and then regenerates `release/closeout_receipt_v1.json`.

The numbered `00_` through `17_` files are retained as the hostile remediation sequence. They are not a competing release authority.

## Active authority

- `STATUS_EVIDENCE_MANIFEST.json` is the proof ledger for the current hardening lane.
- `scripts/release_gate_set.py` is the authority for which commands belong to that lane.
- `release/closeout_receipt_v1.json` is generated from the current proof ledger, support profile, archive manifest, and tracked docs.
- `STATUS_DASHBOARD.md` must describe what is reproducible from HEAD, not just what was true in an earlier snapshot.

## Canonical-spec posture

The root `CANONICAL_STACK_SPEC_V6.md` and `CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md` are repo-local compatibility notes that preserve current implementation-truth dependency statements for historical spec labels.
They are not full canonical spec publications and must say so explicitly.

## Non-goals

- Do not reopen v10+ horizon work to close the hardening lane.
- Do not widen the supported release claim beyond the crates named in `SUPPORT_PROFILE.md`.
- Do not patch status surfaces ahead of the underlying repo truth.

## Credibility lane

The doc-certified and build-certified lane is the same 17-crate set in `SUPPORT_PROFILE.md`.
Adjacent governance and artifact-owner crates keep compatibility names for historical continuity, but their honest scope is recorded in `SCOPE_NOTES.md` and `docs/closeout_v21_v24/governance_surface_decision_table.md`.
