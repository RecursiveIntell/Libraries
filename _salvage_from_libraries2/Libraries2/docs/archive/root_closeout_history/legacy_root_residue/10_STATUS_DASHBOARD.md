# Status dashboard

## Snapshot

- Active lane: `2026-03-22-hardening-closeout`
- Receipt authority: `release/closeout_receipt_v1.json`
- Support scope: 17-crate hardening lane
- Workspace default-members: 29 crates
- Adjacent artifact-owner crates outside support lane: 12

## What is green now

- repo surface law
- doc truth
- manifest truth
- schema registry uniqueness
- production panic guard
- mirror discipline
- hotspot budgets
- public type drift
- root archive manifest
- tracked public API rustdoc coverage
- schema compatibility (when cargo is present)
- selected cargo tests
- closeout receipt generation

## What is superseded

- the stale scan summary is not the current release truth,
- older matrix variants are backlog only,
- horizon research is not part of the finish bar.

Still open:
- physical root reduction is still incomplete; logical supersession is in place but the full root residue is not yet physically archived
- no single operator-facing end-to-end demo yet stitches `effect-runtime`, `authority-delegation`, and `assurance-runtime` into one narrated runnable path
- no benchmark / forge-bench package yet proves superiority on replayable evidence-bound reasoning against simpler baselines

## Finish condition

The repo is *done done* when:

1. this root pack is present and scripts agree,
2. DEMO-001 ships one runnable story across v21 -> v22 -> v23,
3. BENCH-001 ships one reproducible benchmark score sheet,
4. ARCH-001 removes the remaining physical root residue.
