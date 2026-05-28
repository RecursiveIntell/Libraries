# P22 Deletion, Archival, and Quarantine Rules

## Rule 1 — Archive historical run artifacts

Old Codex artifacts are not trash. They are execution evidence. Move them to archive with receipt metadata.

## Rule 2 — Quarantine semantic ambiguity

If a file cannot be confidently assigned to a run, move it to `docs/codex-runs/archive/unclassified/<stamp>/` with reason `unclassified-codex-artifact`.

## Rule 3 — Do not move canonical project truth

Never archive source crates, root Cargo files, active README/STATUS/SOURCE_BASIS/AGENTS, examples, fixtures, or tests because of loose regexes.

## Rule 4 — Promote reusable scripts deliberately

If a run-specific script is still useful, rename or copy it to a generic current script and update docs/tests. Archive the old run-specific script.

## Rule 5 — Existing archives are immutable

Do not rewrite existing archive files or manifests. New archival events produce new manifests or appended index entries.
