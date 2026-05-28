# P22 `z.py` Codex-Run Archival Specification

## Intent

`z.py` must become a state-normalizing source certifier. It must archive stale Codex-run materials before normal packaging so future context bundles are not polluted by old prompts, phase docs, handoffs, or evidence directories.

## Required CLI

```text
--archive-codex-runs / --no-archive-codex-runs
--archive-only
--verify-codex-archive-hygiene
--include-codex-archive
--codex-current-run P22
--codex-archive-root docs/codex-runs/archive
--codex-archive-report-out <path>
--codex-archive-dry-run-report <path>  # optional if same as report-out under --dry-run
```

`--no-archive-codex-runs` is diagnostic only. It must not become the default.

## Required mode

Add:

```text
audit-full
```

Policy:

| Mode | Archive before zipping | Include archived history by default | Use case |
|---|---:|---:|---|
| `source-clean` | yes | no | clean source handoff |
| `codex-context` | yes | no | future Codex context |
| `full-context` | yes | no unless explicit | broad source context |
| `research-context` | optional | no unless explicit | research docs |
| `audit-full` | verify/archive | yes when `--include-codex-archive` | hostile audit / full run replay |

## Archive layout

```text
docs/codex-runs/
  ARCHIVAL_POLICY.md
  CODEX_RUN_INDEX.md
  CURRENT_RUN.md
  archive/
    P20/
      ARCHIVE_MANIFEST.json
      SUPERSESSION.md
      RUN_SUMMARY.md
      files/...
    P21/
      ARCHIVE_MANIFEST.json
      SUPERSESSION.md
      RUN_SUMMARY.md
      files/...
    unclassified/
      20260501T235959Z/
        ARCHIVE_MANIFEST.json
        files/...
```

## Detection rules

Archive stale run artifacts from active space:

- `.codex/**`, except stable non-run config only if explicitly retained;
- `.codex_evidence/**`;
- `prompts/P[0-9]*`, `prompts/p[0-9]*/**`;
- `docs/p[0-9]*/**`, `docs/P[0-9]*`;
- `handoffs/p[0-9]*/**`, `handoffs/P[0-9]*`;
- root `CODEX_*`, `*_CODEX_RUN_PROMPT.md`, `NEXT_CODEX_*` run control docs;
- run-specific scripts such as `scripts/p20_*`, `scripts/p21_*` unless promoted to a generic current verifier name;
- generated target audit logs only when explicit archive mode is invoked or when they are copied into a release/audit archive with receipts.

Do **not** archive:

- `README.md`, `STATUS.md`, `SOURCE_BASIS.md`, `AGENTS.md`, `Cargo.toml`, `Cargo.lock`, source crates, examples, fixtures, tests, current stable docs;
- existing files under `docs/codex-runs/archive/**`;
- files under `.git`, `target`, build outputs, package zips.

## Run ID assignment

Use first strong match:

1. path segment matching `pNN`, `P_NN`, `P20_2`, etc.;
2. filename prefix matching `P[0-9]+` or `p[0-9]+`;
3. parent directory like `contract_ownership/NN` maps to `legacy-contract-ownership-NN` unless an owning run is known;
4. otherwise `unclassified/<UTC-stamp>`.

Normalize IDs as uppercase where practical: `P20`, `P20_2`, `P21`, `P22`.

## Manifest schema

Each archive manifest must include:

```json
{
  "archive_manifest_version": "CodexRunArchiveManifestV1",
  "created_utc": "...",
  "tool": "z.py",
  "tool_version": "...",
  "repo_root": "...",
  "run_id": "P21",
  "superseded_by": "P22 or active-source-cleanup",
  "files": [
    {
      "original_path": "prompts/p21/P21_CODEX_RUN_PROMPT.md",
      "archived_path": "docs/codex-runs/archive/P21/files/prompts/p21/P21_CODEX_RUN_PROMPT.md",
      "sha256": "...",
      "bytes": 1234,
      "mtime_utc": "...",
      "reason": "stale-run-prompt"
    }
  ],
  "collisions": [],
  "skipped_existing": [],
  "unclassified": []
}
```

## Idempotence

Running the archival phase twice must not move already archived files or rewrite manifests in place. If the exact same file content is already archived, record `skipped_existing`. If a different file would collide with an existing archive path, use a deterministic suffix and record the collision.

## Normal package exclusion

Normal `codex-context` packages must exclude:

- `docs/codex-runs/archive/**`;
- `.codex_evidence/**`;
- stale `docs/pNN/**`, `prompts/pNN/**`, `handoffs/pNN/**` if any remain;
- generated target audit logs unless explicitly selected.

`CODEX_RUN_INDEX.md`, `CURRENT_RUN.md`, and `ARCHIVAL_POLICY.md` may be included because they describe state, not stale instructions.

## Strict failure conditions

Under `--strict`, fail if:

- a stale active Codex artifact remains after normalization;
- a candidate archive operation would overwrite an existing file;
- more than one run claims to be current;
- archive manifest cannot be written;
- `--include-codex-archive` is absent but archived run history would be included;
- zipping proceeds after archival errors.
