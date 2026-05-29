# P30 Owner / Source-of-Truth Map

| Concern | Primary owner | P30 action |
|---|---|---|
| Material artifact ID policy | `aidens-contracts` for AiDENs-owned contracts; `stack-ids` for shared primitives | replace unsafe generation paths; add display-only unstable ID naming; add guards |
| Tool-call executable boundary | `aidens-runner`, `aidens-provider-kit`, `aidens-boundary-kit` | strict parse path, rejected-call receipts, no permissive executable repair |
| Tool execution and patch application | `aidens-tool-kit` | fail-closed patching, rollback receipts, command sandbox hardening |
| Execution receipts | `aidens-receipts`, `aidens-runner`, `llm-tool-runtime` | durable defaults, failure receipts, attempt lineage |
| Verification semantics | `aidens-contracts`, `verification-*`, `aidens-governance-kit` | advisory != success; proof debt and downgrade law |
| Package/gate truth | `scripts`, `z.py`, `zip.py`, `docs/codex-runs` | gate supersession manifest, source/build/conformance split |
| Root doc authority | `docs/codex-runs`, active doc manifest | classify/archive/supersede root docs |
| v11A contracts | `aidens-contracts` for AiDENs-owned orchestration contracts | seed only with tests and owner notes |
| v11B region/right-graph hooks | `aidens-contracts`, `aidens-repair-kit`, kernel crates | add minimal non-authoritative hooks, not full runtime |

## Ownership ambiguity rule

If Codex cannot identify the owner, it must not implement semantics locally. It must create `handoffs/p30/OWNERSHIP_AMBIGUITY_LEDGER.md` entry with:

- concept;
- candidate owner crates;
- files encountered;
- risk of inventing shadow truth;
- recommended next pass.
