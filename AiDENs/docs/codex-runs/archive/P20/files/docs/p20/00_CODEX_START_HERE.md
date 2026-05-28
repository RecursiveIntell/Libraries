# 00 — Codex Start Here

AiDENs should be built from the current Recall codebase at `~/Coding/Recall`, not Recall-Coding.

The current Recall code should be treated as an unfinished application that contains reusable primitives and known failure modes. The Codex run should create the new AiDENs workspace in `~/Coding/Libraries/AiDENs`.

## Target outcome

Create the main crate family for building applications around the RecursiveIntell Rust libraries.

AiDENs must be:

- app-builder friendly,
- receipt-bearing,
- capability-truthful,
- provider-route truthful,
- strict at boundaries,
- safe by default,
- profile-driven but not magical,
- daemon/UI/queue aware without letting those shells own runtime truth.

## Do not skip

Before implementation, verify:

```bash
pwd
ls ~/Coding/Recall
ls ~/Coding/Recall/recall-session/src
ls ~/Coding/Recall/recall-contracts/src
```

## Phase priority

Do not start with memory, daemon, queue, graph, or Tauri. Start with the law/receipts/core app path.

1. Contracts
2. Boundary parsing/repair
3. Config
4. Receipts
5. Capability truth
6. Provider truth
7. Tool exposure
8. Security/permit
9. Arbiter/budget
10. Runner/app/CLI
11. Tests
