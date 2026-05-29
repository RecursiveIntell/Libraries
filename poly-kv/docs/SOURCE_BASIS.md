# Source Basis and Evidence Classification

This bundle is based on current conversation research and official docs. It is not proof that any current target repo already has these files or capabilities.

## Uploaded research basis

- `quant-compression-governance.md`: recommends governed codec infrastructure, `quant-codec-core`, `quant-governor`, delayed `scr-runtime-compression`, and no raw lossy operators in app truth stores.
- `governed-compression.md`: frames governed compression as interchangeable, evaluated, receipt-bearing, replayable, and forbidden from becoming hidden truth.
- `harness research.md`: recommends local-first replayable receipt-bearing benchmark substrate for measuring memory/provenance/policy/compression behavior.
- Permanent project doctrine: material operations require receipts; source-of-truth boundaries, no shadow truth, strict boundaries, and verification outrank inference.

## Current web basis checked for Codex mechanics

- OpenAI Codex AGENTS.md docs: https://developers.openai.com/codex/guides/agents-md
- OpenAI Codex skills docs: https://developers.openai.com/codex/skills
- OpenAI Codex hooks docs: https://developers.openai.com/codex/hooks
- OpenAI Codex best practices: https://developers.openai.com/codex/learn/best-practices

## Current web basis checked for PolyKV

- PolyKV arXiv: https://arxiv.org/abs/2604.24971

## Evidence classes used in this bundle

| Grade | Meaning |
|---|---|
| A | Current repo files/logs/tests inspected in target run |
| B | Accepted project doctrine or official product docs |
| C | Structured research synthesis |
| D | Public research/paper support |
| Q | Quarantine/speculative until reproduced |
| F | Deprecated/stale or not current |

## Claims promoted into implementation guidance for this run

| Claim | Status | Scope | Acceptance test |
|---|---|---|---|
| `quant-codec-core` should own codec/profile/shape traits | Promoted | Workspace architecture | No duplicate IDs/shapes in `poly-kv`; compile tests use shared types |
| `poly-kv` should own shared-pool manifests/reader receipts | Promoted | `poly-kv` crate | Pool build, reader attach, fallback, decode receipts exist and serialize |
| Lossy compression must be explicit and fallback-capable | Promoted | all compressed paths | Fallback receipt tests and shape/error tests pass |
| App integrations are out of scope | Promoted | pass boundary | no references to semantic-memory/Gloss/Recall/AiDENs runtime code |
| Adaptive controller is deferred | Promoted | pass boundary | no policy routing engine beyond passive schema/types |
