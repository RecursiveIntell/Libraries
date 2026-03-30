# Risk register and forbidden shortcuts

## Top risks

| Risk | Why it is dangerous | Countermeasure |
|---|---|---|
| Doc-first closure | Makes the repo sound hard-enforced while the code is still advisory. | No row closes without code + proof command. |
| Enum sprawl without ownership | Creates a junk-drawer types crate and moves drift elsewhere. | Own enums locally unless semantics are truly shared. |
| Permit backdoor | One unchecked helper can reintroduce advisory execution. | Sealed permit type + lints + E2E bypass tests. |
| Schema-only closure for future spec families | Creates elegant nouns without executable law. | Every new family ships with builders/validators/tests/refints. |
| Shadow surfaces | Scaffolds and aliases make evidence ambiguous. | Archive scaffolds, rationalize names, keep one authoritative path. |
| Hotspot avoidance theater | Big files remain giant while docs pretend modularity. | Enforce hotspot budgets and module decomposition. |

## Forbidden shortcuts

- Do not close `CCS-001` by adding another boolean.
- Do not close `CCS-002` by renaming `approval_token` to another `String`.
- Do not close `CCS-003` with comments that say “expected values are X/Y/Z”.
- Do not close `CCS-004` with schema generation alone.
- Do not close `CCS-005` with only more fixtures and no reference interpreter.
- Do not close `CCS-006` by marking scripts green without wiring them into CI.
- Do not close `CCS-015` .. `CCS-020` with schema-only stubs.
- Do not claim full constitutional combat strength until the full-spec gap lane is closed.
