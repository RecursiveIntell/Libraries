# AGENTS.md Appendix — P31A Release Truth Law

Add this section to `AGENTS.md` or merge it into the existing release/governance section.

## P31A release-truth law

`docs/codex-runs/CURRENT_RUN.json` is the canonical release-truth ledger. Protected docs and scripts may mirror it but must not become independent sources of truth.

Protected mirror files:

- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`
- `docs/codex-runs/CURRENT_RUN.md`

Required distinction:

- `last_certified_run` is the most recent run with evidence-backed certification.
- `active_run` is the run currently being executed.
- `target_run` is the run this pass is trying to complete.
- `certification_status` may not become `certified` unless evidence-backed final gates pass.

Forbidden behavior:

- editing docs to manufacture certification;
- treating package validation as build certification;
- treating missing cargo/toolchain as success;
- leaving root Markdown ambiguity active;
- using old P24–P30 docs/scripts as current instructions;
- allowing P31 boundary compiler work before P31A release truth passes;
- adding runtime receipt/ID/schema families inside P31A.

Any positive certification claim must cite evidence recorded in `CURRENT_RUN.json`.
