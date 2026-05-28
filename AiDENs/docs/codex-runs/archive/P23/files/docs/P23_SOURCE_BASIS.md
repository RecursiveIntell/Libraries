# P23 Source Basis — AiDENs Product-Capability + Package Replay Closure

## Snapshot basis

- Input package: `AiDENs-aidens-codex-context-20260502.zip`
- Report: `AiDENs-aidens-codex-context-20260502.report.md`
- Manifest: `AiDENs-aidens-codex-context-20260502.manifest.json`
- Findings: `AiDENs-aidens-codex-context-20260502.findings.json`
- Excluded list: `AiDENs-aidens-codex-context-20260502.excluded.json`
- Codex archive report: `AiDENs-aidens-codex-context-20260502.codex-archive.json`

## Observed P22 results that P23 must preserve

- Strict package validation produced `0` errors.
- Normal context package shrank substantially relative to the prior package.
- Codex archival normalization exists and reports `active_stale_after=[]`.
- Normal package mode excludes `docs/codex-runs/archive/` by default.
- Current run is recorded as `P22` in the archive report.
- P22 added release-hygiene and operator-support-tier surfaces.

## Observed P22 defects P23 must close

1. **Package replay is not self-contained.** `scripts/p22_verify.sh` references `scripts/p22_secret_scan_fixture_test.py`, but the package excludes that file because of `secret-like-filename`.
2. **Final audit package identity can diverge from uploaded package identity.** Handoff docs and emitted package reports can name different hashes/manifests unless the package role is explicit.
3. **Stale-run detection is too narrow.** P20/P21/P22-adjacent files remain in active surfaces under `audit/`, `evals/`, `fixtures/`, `templates/`, `repo_overlay/`, `supporting/`, and old `docs/*CODEX*` paths.
4. **`z.py` contains P22-specific logic.** It should be generic across future `--codex-current-run P23/P24/...` without code edits.
5. **Script reference checking is optional and insufficiently authoritative.** Strict packaging should catch included scripts that reference missing or excluded script dependencies.
6. **Legacy `zip.py` remains a footgun.** It must be removed, archived, or converted into a hard-failing wrapper to `z.py`.
7. **Package modes are conflated.** `codex-context` currently mixes next-Codex context, release context, and current-run evidence.

## Capability gap P23 must start closing

P23 is not a secretary pass. The packaging fixes are a support lane. The main capability lane must move AiDENs toward the vision:

- concrete receipt-bearing agent run bundles,
- product-facing test-agent / coding-agent vertical slice,
- command and fixture paths that prove AiDENs can assemble, inspect, and run a local agent lane,
- explicit execution context / support-tier / package provenance in outputs,
- no new shadow truth plane.

## Research law basis to preserve

- AiDENs is a directing/wiring/orchestration layer. Sibling crates own canonical memory, evidence, IDs, contracts, verification, kernel, and tool receipts.
- Execution is evidence: retries, queue hops, deadlines, provider routes, dispatch outcomes, degradation, and replay lineage must be modeled as receipts.
- Episode identity, execution context, and multi-view runtime provenance are inseparable.
- Approximate or convenience outputs must not outrank witnesses, certificates, residuals, syndromes, repair records, or explicit proof obligations.
- Lawful subtraction applies to repo cleanup and archive compaction: never destroy evidence silently; demote old run material from active instruction space into archived evidence.
