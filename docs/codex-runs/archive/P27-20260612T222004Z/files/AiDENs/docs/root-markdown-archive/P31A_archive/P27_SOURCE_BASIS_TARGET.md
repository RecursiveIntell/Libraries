# P27 Source Basis Target

Record date: `2026-05-04`

This file should replace stale active-run source-basis language during P27.

## Current run

- Current run: `P27 Super-Pass`
- Prior run: `P26 Advanced Local Agent Spine`
- P24/P25/P26 materials: historical evidence unless explicitly referenced by P27 docs.

## Required workspace layout

AiDENs is not currently a standalone source checkout. It depends on sibling crates under the parent Libraries workspace.

Expected local layout:

```text
/home/sikmindz/Coding/Libraries/
  AiDENs/
  stack-ids/
  semantic-memory-forge/
  forge-memory-bridge/
  semantic-memory/
  knowledge-runtime/
  llm-tool-runtime/
  verification-control/
  verification-policy/
  verification-calibration/
  verification-adjudication/
  recursive-kernel-core/
  constraint-compiler/
  kernel-execution/
  kernel-oracles/
  kernel-conformance/
  contract-schema-gen/
  ...
```

If those sibling crates are absent, cargo verification should classify the result as `sibling_workspace_missing`, not `package_self_replay_pass`.

## Active truth docs

- `AGENTS.md`
- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`
- `P27_*`

## Historical evidence docs

- `P24_*`
- `P25_*`
- `P26_*`
- archived codex run handoffs

Historical docs may be cited as evidence. They are not active instructions.
