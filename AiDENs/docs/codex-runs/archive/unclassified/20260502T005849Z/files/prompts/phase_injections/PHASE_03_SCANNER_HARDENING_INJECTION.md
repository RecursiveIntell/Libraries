# Phase 03 injection — scanner hardening

Fix the false-confidence hole.

Required:

- ownership scanner must detect whether canonical sibling crates were scanned;
- if canonical type count is zero, fail unless explicit `--aidens-overlay-only` mode is used;
- final report must state whether duplicate scan is authoritative.

Forbidden:

- treating `canonical_types=0` as a clean result.
