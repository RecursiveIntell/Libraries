# P20 Expected Final Repository State

## Required active files/directories

```text
AGENTS.md
docs/p20/
docs/p20/reports/
docs/p20/CONTRACT_OWNERSHIP_INVENTORY.md
docs/p20/PROVIDER_CAPABILITY_MATRIX.md
docs/p20/DOCS_CODE_TRUTH_REPORT.md
docs/p20/AGENCY_EVAL_REPORT.md
docs/p20/KNOWN_LIMITATIONS.md
scripts/p20_scan_aidens.py
scripts/p20_verify.sh
scripts/p20_generate_audit_bundle.sh
crates/aidens-agency-kit/       # if agency is implemented as separate crate
```

## Required integrations

- Root or workspace verify path calls P20 scanner.
- Runner vertical slice test exists and passes.
- Provider matrix is executable-truth aligned.
- Contract ownership inventory is current.
- Agency gate is on at least one real runner/generation path.
- Final audit bundle is generated under `target/aidens-final-audit/`.

## Forbidden leftovers

- Root README that reads like a Codex packet instead of project README.
- `complete/implemented` claims without tests.
- Scaffold crates listed as production-ready.
- Local canonical truth clones.
- Provider native tool support claims without tests.
- Deferred reference semantics for supported features.
- Hidden compatibility shims.
