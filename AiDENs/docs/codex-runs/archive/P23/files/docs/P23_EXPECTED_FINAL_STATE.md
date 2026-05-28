# P23 Expected Final State

## Required files/directories

- `docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json`
- `docs/codex-runs/CODEX_RUN_INDEX.md`
- `docs/codex-runs/CURRENT_RUN.md`
- `handoffs/p23/FINAL_AUDIT_REPORT.md`
- `handoffs/p23/KNOWN_LIMITATIONS.md`
- `scripts/p23_verify.sh`
- `scripts/assert_package_self_replay.py`
- `scripts/assert_codex_artifact_classification.py`
- `target/p23/audit/*` evidence outputs
- `target/p23/runs/*` capability run receipts

## Forbidden leftovers

- runnable legacy `zip.py` with independent packager behavior,
- hard-coded P22-only z.py current-run logic,
- included verifier scripts with excluded dependencies,
- unclassified P20/P21/P22 active artifacts,
- final audit docs naming stale package hashes as current truth,
- cloud/native/autonomous claims without executable proof.

## Expected product posture

AiDENs after P23 should be describable as:

> a local provenance-safe agent builder/runner/doctor that can assemble and run a tested fixture-backed agent lane, emit receipts, inspect the run, and package itself without stale Codex contamination.
