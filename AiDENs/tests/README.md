# Workspace Test Notes

Rust workspace-level tests should live inside the crate they test. This directory documents cross-crate conformance scenarios to implement in `aidens-testkit` and crate-specific `tests/` folders.

Priority tests:

- disabled means absent,
- parser fallback has receipt,
- provider route truth exact,
- config redaction,
- receipt append-only behavior,
- dangerous tools require permits,
- runner does not import shell/UI/daemon crates.
