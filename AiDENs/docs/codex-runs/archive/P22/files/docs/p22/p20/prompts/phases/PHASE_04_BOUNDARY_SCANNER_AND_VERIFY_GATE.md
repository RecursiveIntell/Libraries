# Phase 04 — Boundary Scanner and Verify Gate Integration

## Objective

Turn doctrine into executable static checks.

## Required actions

- Improve `scripts/p20_scan_aidens.py` if needed.
- Ensure `scripts/p20_verify.sh` runs scanner and validation.
- Scanner must emit JSON + markdown.
- Checks must cover docs truth, provider claims, shadow types, deferred reference semantics, scaffold promotion, forbidden compatibility language.

## Acceptance gate

`bash scripts/p20_verify.sh` runs P20 scanner and fails on architectural violations.
