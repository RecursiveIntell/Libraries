# P27 Rollback and Quarantine Plan

## Rollback doctrine

Prefer reversible, mechanical changes. Any broad refactor must have a clean fallback.

## Quarantine cases

Quarantine rather than improvise when:

- sibling crate ownership is unclear;
- cargo fails because sibling monorepo is absent;
- package self-replay cannot be attempted;
- patch semantics are ambiguous;
- provider path requires unavailable cloud keys;
- a support claim would require canonical truth not owned by AiDENs;
- a megafile split changes behavior unexpectedly.

## Quarantine record template

```markdown
# Quarantine Record — <ID>

- Date:
- Phase:
- Blocker:
- Files involved:
- Observed command/output:
- Probable owner:
- Support-tier effect:
- Safe next step:
- Operator decision needed:
```
