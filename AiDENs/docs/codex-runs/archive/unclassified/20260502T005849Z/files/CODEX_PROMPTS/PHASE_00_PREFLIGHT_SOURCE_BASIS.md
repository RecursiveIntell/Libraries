# PHASE 00 — Preflight, Source Basis, Evidence Harness

## Objective

Prepare the repo for safe ownership collapse. Do not edit ownership code yet.

## Required actions

1. Confirm working directory is `~/Coding/Libraries/AiDENs`.
2. Create evidence root:

```bash
mkdir -p .codex_evidence/contract_ownership/00
mkdir -p docs/contract-ownership/quarantine
```

3. Save pre-phase evidence:

```bash
git status --short > .codex_evidence/contract_ownership/00/git_status_before.txt
git diff --binary > .codex_evidence/contract_ownership/00/git_diff_before.patch
```

4. Install/update the bundled scripts and docs if not already present.
5. Create/update:

```text
docs/contract-ownership/COMPATIBILITY_LEDGER.md
docs/contract-ownership/FINAL_QUARANTINE_LEDGER.md
```

The compatibility ledger must contain only a header.

6. Update stale source-basis docs so they identify the 2026-04-28 target snapshot, not older archive names as current.
7. Run:

```bash
bash scripts/assert_no_crate_split.sh
bash scripts/assert_docs_source_basis_current.sh
```

8. Save evidence and write phase report.

## Acceptance

- Evidence directory exists.
- No crate split occurred.
- Source-basis gate passes or exact stale references are listed for Phase 07 repair.
- No ownership code changed yet except installing scripts/docs.

## Stop

Stop after this phase and wait for `GUARDRAIL_00_TO_01`.
