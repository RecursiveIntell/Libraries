# P25 Acceptance Gates

## Global pass gate

P25 passes only if all P0 and P1 gates pass or are explicitly operator-approved as deferred with quarantine records.

## P0 gates

### G0 — Package health

- strict package validation still yields 0 errors and 0 warnings;
- generated outputs are excluded or classified correctly;
- archive sidecar shows no active stale artifacts.

### G1 — z.py scope

- only root Markdown archive functionality added;
- no runtime/agent/semantic behavior added;
- dry-run and verify-only do not move files;
- collisions fail closed;
- ambiguous files are not moved.

### G2 — Root Markdown hygiene

- protected docs preserved;
- candidates archived with manifest;
- direct root Markdown inventory emitted;
- archive manifest includes original path, archived path, hash, bytes, mtime, and reason.

### G3 — Phase gate enforcement

- active phase-injection docs use P25 naming;
- active injection docs contain STOP/WAIT language;
- verifier fails stale run IDs or stale `target/p##` / `handoffs/p##` references;
- every-other-phase gate sequence is documented and checked.

### G4 — Current-run truth

- `CURRENT_RUN.md` says P25;
- classification map does not classify stale prior-run instructions as current;
- prior-run files are evidence/archive, not current instructions.

### G5 — Verifier

- `scripts/p25_verify.sh` or `scripts/verify_current.sh` runs all relevant checks;
- failure output is deterministic and actionable;
- P25 evidence manifest emitted.

## P1 gates

### G6 — Flagship demo

- local fixture demo exists;
- patch proposal or abstention is supported;
- write/apply is permit-gated;
- receipts and AiDENsRunBundleV2 emitted;
- replay works.

### G7 — Support truth

- README, STATUS, SUPPORT_PROFILE, known limitations updated;
- supported-local / fixture-backed / deferred-cloud / deferred-autonomy distinction is explicit.

### G8 — Large-file containment

- large file plan emitted;
- no risky refactor mixed into P25 without need.

## Final gate

Final handoff must include:
- command list,
- changed-file summary,
- validation outputs,
- unresolved risks,
- support claims,
- package/hash references,
- operator gate compliance record.
