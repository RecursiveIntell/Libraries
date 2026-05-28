# Phase 00 injection — code/package audit baseline

Run the hard code audit before touching architecture.

Required:

```bash
python3 scripts/p20_1_hard_code_audit.py --out target/p20-1/phase00-audit.json --markdown target/p20-1/phase00-audit.md
```

Report:

- missing include targets;
- missing manifest entries;
- missing eval/support files;
- cargo/rustc availability;
- `aidens-testkit` topology;
- ownership scanner preconditions.

Do not proceed until every P0 finding has a concrete repair action.
