# P20 Deletion, Removal, and Quarantine Rules

## Delete or redirect immediately

- Local canonical truth duplicates.
- Compatibility layers that silently reinterpret canonical crate semantics.
- Fake provider capability declarations.
- Docs that claim unsupported features as complete.
- Stale Codex-run documents in root that confuse current project state.

## Quarantine rather than delete

Quarantine when a surface may contain useful work but violates ownership clarity.

Quarantine location:

```text
docs/p20/quarantine/
```

Quarantine record must include:

- object/file/type name;
- reason;
- canonical owner if known;
- risk if retained;
- repair option;
- delete/redirect/defer recommendation.

## Supersede rather than rewrite silently

For docs and reports, preserve historical context under:

```text
docs/archive/codex-runs/
```

Do not let stale docs remain in root or active docs without `historical` labeling.
