# P25 Known Limitations

Record date: `2026-05-04`

## Active limitation statements

- This pass is **not production-cloud-ready**.
- This run includes **no full-autonomy claim**.
- Hosted cloud provider execution is deferred (`deferred-cloud`).
- Native provider streaming and native provider tool loops are deferred (`deferred-cloud`).
- Broad autonomous daemon scheduling is deferred (`deferred-autonomy`).
- Run determinism and replay are scoped to the local coding-agent fixture/replay lane.
- Canonical memory/runtime/verification/governance/repair semantics are owned by sibling crates; AiDENs surfaces operator-grade evidence and routing support only.

## Support wording constraints

- `supported-local` applies only to flows explicitly exercised and evidenced in P25 artifacts.
- `fixture-backed` applies only where fixture evidence is present (`target/p25/flagship-coding-agent/*`, `target/p25/audit/*`).
- No path is considered supported unless evidence is listed in active P25 handoffs/audit evidence.

