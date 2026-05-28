# P25 z.py Root Markdown Archiver Spec

## Scope

This is the only allowed z.py feature in P25.

`z.py` may gain root workspace Markdown archive hygiene. It must not become a runtime, agent capability, compatibility shim, schema adapter, or semantic owner.

## Feature name

Recommended flags:

```text
--archive-root-markdown-noise
--root-markdown-archive-root <path>
--root-markdown-archive-dry-run
```

This can be integrated into existing strict package modes if that matches the current z.py style.

## Target set

Only Markdown files directly in the archive root/workspace root.

Do not recursively scan nested directories.

## Protected root Markdown names

Always preserve:

```text
AGENTS.md
CLAUDE.md
README.md
CONTRIBUTING.md
LICENSE.md
CHANGELOG.md
SECURITY.md
CODE_OF_CONDUCT.md
SUPPORT.md
SUPPORT_PROFILE.md
SOURCE_BASIS.md
STATUS.md
ARCHITECTURE.md
DESIGN.md
ROADMAP.md
```

## Candidate patterns

Candidates include direct root Markdown whose names indicate run/audit/spec/prompt/matrix/planning residue:

```text
*AUDIT*.md
*HARD_AUDIT*.md
*ISSUE_MATRIX*.md
*RISK_REGISTER*.md
*PROMPT*.md
*MASTER*.md
*SNAPSHOT*.md
*STATUS_DASHBOARD*.md
*IMPLEMENTATION_PLAYBOOK*.md
*CONFORMANCE*.md
*HARDENING*.md
*PLAN*.md
*TENSOR*.md
*MATRIX*.md
```

## Archive destination

```text
docs/root-markdown-archive/<YYYYMMDDTHHMMSSZ>/files/<original_name>.md
```

## Manifest

Emit:

```text
docs/root-markdown-archive/<timestamp>/ROOT_MARKDOWN_ARCHIVE_MANIFEST.json
```

Each entry:

```json
{
  "original_path": "MASTER_ISSUE_MATRIX.md",
  "archived_path": "docs/root-markdown-archive/20260503T000000Z/files/MASTER_ISSUE_MATRIX.md",
  "sha256": "...",
  "bytes": 12345,
  "mtime_utc": "...",
  "reason": "root-markdown-noise",
  "classification": "candidate-archive"
}
```

## Safety behavior

Must fail closed if:
- destination collision exists with different hash;
- candidate is referenced by protected docs or active current-run docs;
- file is ambiguous;
- file is protected;
- verify-only or dry-run is active.

## Required reporting

The package report must include:
- root Markdown inspected count,
- protected count,
- candidate count,
- ambiguous count,
- moved count,
- archive manifest path,
- collisions/errors.

## Tests

Required tests:
1. dry-run does not move files;
2. protected docs remain;
3. candidate files are moved under strict mode;
4. collisions fail closed;
5. ambiguous files remain and are reported;
6. generated archive does not become active context unless explicitly requested.
