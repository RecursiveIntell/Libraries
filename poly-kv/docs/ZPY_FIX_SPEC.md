# z.py fix specification

## Required behavior

`z.py` must produce a pass-off package that is self-contained for the current repo's validators and clean enough to hand to a hostile auditor.

## Required allowlist changes

Add `.pyi` to `ALLOWED_TEXT_EXTENSIONS`.

Add `py.typed` to `ALLOWED_BASENAMES`.

Do not solve this by disabling unsupported-file checks globally.

## Required command-log handling

`commands_run.log` is material evidence for a Codex run. It should not be silently excluded in `next-codex-context`, `codex-run-full`, or `audit-full` packages.

Implementation options, in preferred order:

1. Include `.codex-runs/<run-id>/commands_run.log` when mode is `next-codex-context`, `codex-run-full`, or `audit-full`, subject to normal size and secret scanning.
2. Or generate/include `.codex-runs/<run-id>/commands_run.receipts.jsonl` and keep raw logs excluded.
3. Do not keep the current behavior where both raw command log and receipt equivalent are absent.

Recommended narrow implementation:

```python
CONTEXT_LOG_BASENAMES = {
    "commands_run.log",
    "validation.log",
    "stdout.log",
    "stderr.log",
}

CONTEXT_LOG_ROLES = {
    "next-codex-context",
    "codex-run-full",
    "audit-full",
}


def is_context_receipt_log(rel: str, package_role: str) -> bool:
    p = Path(rel)
    return (
        package_role in CONTEXT_LOG_ROLES
        and p.name in CONTEXT_LOG_BASENAMES
        and len(p.parts) >= 2
        and p.parts[0] == ".codex-runs"
    )
```

Then in `include_decision`, before the general `.log` exclusion:

```python
if suffix in LOG_EXTENSIONS:
    if is_context_receipt_log(rel, policy.package_role):
        text_reason = text_file_policy_reason(path, limit_bytes=1024 * 1024)
        if text_reason:
            return False, text_reason
        return True, "included-context-receipt-log"
    if not policy.include_logs:
        return False, "log-disabled"
```

## Required root package hygiene archival

Add an archival pass that runs before file collection, after stale Codex-run normalization and root Markdown cleanup.

Name: `archive_root_package_artifacts`.

Default enabled for:

- `next-codex-context`
- `codex-run-full`
- `audit-full`

CLI flags:

```text
--archive-root-package-artifacts / --no-archive-root-package-artifacts
--verify-root-package-hygiene
--root-package-archive-root docs/source-packages/archive
--root-package-archive-dry-run
--include-root-package-archive
```

Default archive root:

```text
docs/source-packages/archive/<UTC_STAMP>/files/
```

Manifest path:

```text
docs/source-packages/archive/<UTC_STAMP>/PACKAGE_ARTIFACT_ARCHIVE_MANIFEST.json
```

## Root package artifact classification

Move these root-level files before packaging when they are not the current reserved output path:

| Pattern | Reason |
|---|---|
| `*-next-codex-context-*.zip` | prior-context-archive |
| `*.manifest.json` if generated sidecar | prior-generated-sidecar |
| `*.report.md` if generated sidecar | prior-generated-sidecar |
| `*.excluded.json` if generated sidecar | prior-generated-sidecar |
| `*.findings.json` if generated sidecar | prior-generated-sidecar |
| `*.codex-archive.json` | prior-codex-archive-report |
| `README_BUNDLE.md` | prior-bundle-readme |
| `BUNDLE_MANIFEST.json` | prior-bundle-manifest |
| `*_BUNDLE.md` | prior-bundle-doc |
| `*_PROMPT.md` | root-prompt-residue |
| `*_AUDIT.md` | root-audit-residue |
| `*_ISSUE_MATRIX.md` | root-issue-matrix-residue |
| `*_RISK_REGISTER.md` | root-risk-register-residue |

Never move protected source files:

```text
README.md
AGENTS.md
Cargo.toml
Cargo.lock
pyproject.toml
z.py
LICENSE*
CHANGELOG*
CONTRIBUTING*
SECURITY*
```

## Destination collision policy

Use the existing `unique_archive_destination` behavior: if destination exists with same hash, remove active duplicate; if different hash, append `.dup-<shortsha>` before the suffix.

## Report integration

Add `root_package_archive` to:

- markdown report summary
- manifest JSON
- console output

Fields:

```json
{
  "enabled": true,
  "dry_run": false,
  "verify_only": false,
  "archive_root": "...",
  "archive_dir": "...",
  "manifest_path": "...",
  "inspected_count": 0,
  "protected_count": 0,
  "candidate_count": 0,
  "planned_count": 0,
  "moved_count": 0,
  "skipped_existing_count": 0,
  "collision_count": 0,
  "candidate_paths": [],
  "protected_paths": [],
  "moved": [],
  "errors": []
}
```

## Strict failure behavior

Under `--strict`, packaging must fail if:

- root package artifacts remain after archival normalization;
- package sidecar allowlist excludes required Python sidecar files;
- current run command evidence is absent;
- generated sidecars from previous runs remain at root;
- root Markdown classifier reports ambiguous files.
