# P25 Root Markdown Archive Test Plan

## Test fixtures

Create a temporary workspace with:

Protected:
- AGENTS.md
- CLAUDE.md
- README.md
- SOURCE_BASIS.md
- SUPPORT_PROFILE.md

Candidates:
- MASTER_ISSUE_MATRIX.md
- PROMPT.md
- AUDIT_2026-04-01.md
- HOSTILE_AUDIT_SYNTHESIS_V5.md
- V9_IMPLEMENTATION_PLAYBOOK.md

Ambiguous:
- NOTES.md
- IDEAS.md

## Tests

### Dry run

Command:
```bash
python z.py --archive-root-markdown-noise --root-markdown-archive-dry-run
```

Expected:
- no files moved;
- candidates listed;
- protected files listed as protected;
- ambiguous files listed as ambiguous.

### Strict run

Expected:
- candidates moved to deterministic archive path;
- protected remain;
- ambiguous remain;
- manifest emitted.

### Collision

Pre-create an archive destination with a different hash.

Expected:
- fail closed;
- no partial moves.

### Verify-only

Expected:
- no moves;
- findings report policy state.

### Active reference check

If a candidate root Markdown is referenced by active current-run docs, it must not be moved without explicit operator approval.
