# P32 super-pass issue matrix

| ID | Severity | Surface | Defect | Fix direction | Acceptance evidence |
|---|---:|---|---|---|---|
| P32-S0-001 | S0 | Completion proof | Rust build/test/clippy not proven | Run and receipt cargo fmt/check/test/clippy | `docs/P32_COMMAND_RECEIPTS.md` |
| P32-S0-002 | S0 | Final artifacts | P31 final docs missing; P30/P31 drift | Supersede with P32 artifacts | P32 docs exist |
| P32-S0-003 | S0 | Phase gates | Auto gates/hooks weak or inert | Real hooks/scripts/static gates | seeded violation fails |
| P32-S1-001 | S1 | Evaluator | Proposed action/effect weakly used | Action/effect risk model | same signal/different action tests |
| P32-S1-002 | S1 | Signal source | Opaque evidence refs treated as signals | Typed `control_signals`; adapter-only translation | grep + tests |
| P32-S1-003 | S1 | Authority | Authority refs recorded not validated | explicit verified/declared/unknown basis | missing authority tests |
| P32-S1-004 | S1 | Evidence | Evidence refs recorded not validated | explicit evidence result enum + confidence/reasons | insufficient evidence tests |
| P32-S1-005 | S1 | Owner boundary | External owner ambiguity | owner-boundary basis + adapter seams | docs/tests |
| P32-S1-006 | S1 | Rollback | Destructive paths not gated by rollback | rollback basis and hard rule | destructive missing rollback fixture |
| P32-S1-007 | S1 | Schema | Schema weaker than Rust validation | minLength/date-time/strict parity | negative schema tests |
| P32-S1-008 | S1 | Receipts | Losing candidates incomplete | candidate trace model | receipt tests |
| P32-S1-009 | S1 | Digests | Evaluator hash too narrow | honest build/source digest | digest docs/tests |
| P32-S1-010 | S1 | Raw digest | Raw invalid input digest not preserved | hash raw bytes in CLI before parse | unknown field raw digest test |
| P32-S2-001 | S2 | Canonical JSON | Custom canonicalization under-specified | `docs/SCR_CANONICAL_JSON_V1.md` + tests | hash stability tests |
| P32-S2-002 | S2 | CLI | Generation/verification conflated | command split | CLI help + tests |
| P32-S2-003 | S2 | Golden fixtures | Fixture drift rationale weak | policy-change receipt | `docs/P32_POLICY_CHANGE_RECEIPT.md` |
| P32-S2-004 | S2 | Docs | Release/integration overclaim risk | docs state reference-kernel truth | grep audit |
| P32-S2-005 | S2 | Root hygiene | packager/root docs ambiguity | archive/classify root outputs | stale-surface gate |
