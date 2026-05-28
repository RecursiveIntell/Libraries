# Phase 00 — Preflight Source Basis and Contamination Inventory

Do not edit code in this phase except to create the phase report directory if necessary.

## Tasks

1. Read P21 final audit, known limitations, `z.py`, P21 verifiers, root docs, and P22 docs.
2. Inventory all Codex-run artifacts currently in active space:
   - `.codex/**`
   - `.codex_evidence/**`
   - `prompts/P*`, `prompts/p*/**`
   - `docs/p*/**`
   - `handoffs/p*/**`
   - root `CODEX_*`
   - run-specific scripts `scripts/p20*`, `scripts/p21*`, etc.
3. Classify each as:
   - stable current truth doc;
   - current P22 run control file;
   - stale run artifact to archive;
   - executable script to promote to generic current name;
   - ambiguous/unclassified.
4. Inventory package contamination from current `z.py` defaults.
5. Emit `target/p22/audit/phase00_codex_artifact_inventory.json` and `handoffs/p22/PHASE_00_REPORT.md`.

## Acceptance Gate

No source edits except inventory/report files. Phase report must list exact stale active paths and planned archive classes.
