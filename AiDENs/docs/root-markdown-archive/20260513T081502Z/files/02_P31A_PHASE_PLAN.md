# P31A Phase Plan

## Phase 0 — Inventory and stop-rule check

No code changes except reports.

Produce:

```text
handoffs/P31A_PHASE_00_INVENTORY.md
```

Inventory:

- every protected doc and its run/status claims,
- existing release/package scripts and stale defaults,
- current CI workflow,
- root Markdown ambiguity count,
- Codex artifact classification gaps,
- package certifier command and current defaults,
- existing package self-replay behavior,
- whether cargo/rust toolchain is available locally.

Gate:

- no unresolved ambiguity about canonical owner for current-run truth;
- no feature work started.

## Phase 1 — Release ledger and protected docs

Create/update:

```text
docs/codex-runs/CURRENT_RUN.json
docs/codex-runs/CURRENT_RUN.md
docs/codex-runs/RUN_LEDGER.jsonl
README.md
STATUS.md
SOURCE_BASIS.md
SUPPORT_PROFILE.md
AGENTS.md
scripts/assert_release_ledger_schema.py
scripts/assert_current_run_truth.py
scripts/assert_release_truth_consistency.py
scripts/assert_support_claims_have_evidence.py
```

Gate:

```bash
python3 scripts/assert_release_ledger_schema.py
python3 scripts/assert_current_run_truth.py
python3 scripts/assert_release_truth_consistency.py
python3 scripts/assert_support_claims_have_evidence.py
```

## Phase 2 — Build scope and final command bar

Create/update:

```text
docs/codex-runs/BUILD_SCOPE.md
scripts/verify_current.sh
.github/workflows/ci.yml
```

Gate:

```bash
bash scripts/verify_current.sh
```

If cargo is missing or build fails, the pass may continue only to record blocker evidence. Certification remains false.

## Phase 3 — Root Markdown and Codex artifact classification

Create/update:

```text
docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json
docs/codex-runs/archive/**
docs/root-markdown-archive/**
scripts/assert_root_markdown_archive_policy.py
scripts/assert_codex_artifact_classification.py
```

Use archive/quarantine, not deletion.

Gate:

```bash
python3 scripts/assert_root_markdown_archive_policy.py
python3 scripts/assert_codex_artifact_classification.py
```

## Phase 4 — Package validation and self-replay

Create/update:

```text
scripts/assert_package_validation.py
scripts/assert_package_self_replay.py
```

Run:

```bash
python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P31A
python3 scripts/assert_package_validation.py
python3 scripts/assert_package_self_replay.py --package <zip> --require-verifier --receipt-out handoffs/P31A_PACKAGE_REPLAY_RECEIPT.json
```

Gate:

- package findings are zero,
- manifest current_run equals P31A,
- extracted self-replay runs `verify_current.sh`,
- blocked replay states are explicit and reflected in the ledger.

## Phase 5 — Broad warning triage, not suppression

Run:

```bash
python3 scripts/p30_guard.py --repo . --json > handoffs/P31A_P30_GUARD_WARNINGS.json
python3 scripts/p30_guard.py --repo . --fail-broad
```

If broad warnings remain, do not silence them. Create:

```text
docs/codex-runs/P31A_BROAD_WARNING_TRIAGE.json
```

Each waiver requires:

```json
{
  "rule_id": "...",
  "path_glob": "...",
  "symbol": "...",
  "reason": "...",
  "owner": "...",
  "expires_after": "P31B",
  "evidence": "..."
}
```

Gate:

- hard violations zero;
- broad warnings either fixed or expiring waiver recorded;
- no line-number-only allowlist.

## Phase 6 — Final verifier, ledger update, and report

Run final command bar:

```bash
bash scripts/verify_current.sh | tee handoffs/P31A_FINAL_VERIFY.log
```

If all build/package/replay gates pass, update the ledger booleans and evidence refs. If any required gate fails or is blocked, keep certification false/blocked and cite evidence.

Produce:

```text
handoffs/P31A_FINAL_REPORT.md
handoffs/P31A_DEFERRED_RUNTIME_EVIDENCE_ISSUES.md
```

Gate:

- no positive certification without evidence;
- no stale docs disagree with ledger;
- no final status ambiguity.
