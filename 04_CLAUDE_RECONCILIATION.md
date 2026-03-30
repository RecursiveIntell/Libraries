
# 04_CLAUDE_RECONCILIATION

This pass reconciles the supplied Claude analysis against the current `libraries-source-clean-20260323.zip` snapshot.

## Bottom line

Claude was directionally right about the stack’s real center of gravity and several credibility risks.

But several of the sharpest negatives are now stale because the current snapshot is materially ahead of the code Claude was describing.

| finding | status | note |
|---|---|---|
| forge-pilot has 7 tests and path normalization only | stale / closed | Current snapshot has 54 forge-pilot tests spanning observation, scoring, halts, verification control, execution evidence, repo chat, bootstrap, and roundtrip flows. |
| loop_runner.rs is oversized | still true | `forge-pilot/src/loop_runner.rs` remains 1034 LOC; `main_support/mod.rs` remains 1592 LOC. |
| Rust symbol extractor is fragile | still true | `forge-pilot/src/bootstrap/extract/rust.rs` still uses a line/prefix parser. |
| LLM integration is minimal | still true | `llm-refinement` exists but current decide-path behavior is a hint append, not real model-guided refinement. |
| policy/approval system is configured, not dynamic | mostly still true | The control plane is richer now, but there is still no evidence here of an interactive human-in-the-loop wait state being exercised in this audit environment. |
| governance/runtime naming is misleading | still true but narrower | The problem is no longer “11 empty runtimes”; it is a smaller but still real shell layer. |
| OODA loop has zero meaningful tests | stale / closed in part | The current test surface is materially broader. I did not build-run it here, but the source-level coverage surface is no longer trivial. |
| core pipeline is real | still true | This remains the strongest positive claim in the Claude pass. |

## What remains the most valuable Claude carry-forward

- the core stack is real and structurally differentiated,
- the release surface can still undersell or oversell the repo,
- the giant files are still giant,
- the symbol-extraction surface is still fragile,
- and the LLM refinement story is still thinner than the feature/config surface implies.

## What I would not repeat unchanged

I would **not** repeat the earlier claims that:
- `forge-pilot` is basically untested,
- the tracked core crates are still near-zero doc surfaces,
- or the current snapshot still looks like all-governance-shells and no real system.

That would be inaccurate for this archive.
