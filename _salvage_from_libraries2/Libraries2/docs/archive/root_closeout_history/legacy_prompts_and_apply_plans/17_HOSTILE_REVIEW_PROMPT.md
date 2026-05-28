# Hostile review prompt

Review the implementation as if you are trying to prove it is lying.

## Attack surfaces

1. Did the split passes smuggle semantic edits?
2. Does CI really enforce the written bar now?
3. Did Forge gain additive v13 artifacts, or just prettier comments?
4. Did bridge preserve support/retraction semantics, or invent them?
5. Can semantic-memory answer current-state and as-of queries differently when required?
6. Does a retraction close transaction currentness without erasing history?
7. Can control consume the new artifacts without becoming a secret truth authority?
8. Did package-surface docs become more honest, or just longer?
9. Are compatibility surfaces fenced, or still narrated as normal?
10. Did anyone try to sneak v14/v15 work in before v13 was actually landed?

## Fail the change if

- formatting or lint can drift silently
- schemas drift with no explicit intent
- support/contradiction/retraction remain unqueryable folklore
- control receipts still flatten truth into scalar mush
- a refactor PR changed behavior without owning it
