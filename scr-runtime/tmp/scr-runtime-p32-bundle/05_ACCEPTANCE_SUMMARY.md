# Acceptance summary

## Release status after this super-pass

Acceptable end states:

### A. Complete SCR-P0A reference runtime

All gates pass. External owner-crate integration remains adapter-seam-only but honestly documented.

### B. Complete except environment blocker

All static gates pass, but Rust toolchain is missing or broken. This is acceptable only if:

- Codex records exact missing toolchain command and environment,
- no completion claim is made,
- remaining work is bounded to rerunning validation after toolchain repair.

### C. Quarantine state

If source ownership cannot be resolved or the implementation requires broad rewrites Codex cannot complete safely, Codex must quarantine the incomplete change and produce a repair packet.

Unacceptable end states:

- "Looks complete" without command receipts.
- New compatibility shims that reinterpret external refs.
- Schema widened to make tests pass.
- Golden fixture updates without policy-change receipt.
- Hooks/phase gates that exist but do not fail seeded violations.
