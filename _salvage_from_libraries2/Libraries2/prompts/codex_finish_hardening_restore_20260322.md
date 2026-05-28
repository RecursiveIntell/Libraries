# Codex implementation prompt

You are applying the final finish-line pack for the 2026-03-22 hardening lane.

## Mission

Restore the root control-plane files, keep the hardening receipt green, and then finish only the three remaining substantive items:

1. DEMO-001 — one narrated v21 -> v22 -> v23 demonstrator,
2. BENCH-001 — one benchmark / forge-bench proof package,
3. ARCH-001 — final physical root reduction.

## Constraints

- root is canonical,
- do not edit mirrors first,
- do not invent new owner crates or new schema families,
- do not reopen V10/V14-V20 horizon work,
- do not overstate support beyond the 17-crate lane in SUPPORT_PROFILE.md,
- keep the demo consumer-only with respect to orchestration.

## Required order

1. apply the root docs in this pack,
2. regenerate the receipt,
3. run the static gates,
4. implement DEMO-001,
5. implement BENCH-001,
6. finish ARCH-001,
7. rerun the gates and update STATUS_DASHBOARD.md.
