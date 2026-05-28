# P29 Bundle Install Plan

Copy these files into AiDENs root before running Codex:

- all top-level `P29_*` docs;
- `prompts/phases/*`;
- `matrices/*`;
- `scripts/*`;
- `fixtures/p29/*`;
- `docs/templates/*`.

Set executable bit on shell scripts:

```bash
chmod +x scripts/p29_verify.sh
```

Create directories if absent:

```bash
mkdir -p handoffs/p29 docs/p29 docs/p29/quarantine target/p29/audit target/p29/package
```
