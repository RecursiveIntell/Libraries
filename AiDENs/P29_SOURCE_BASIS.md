# P29 Source Basis

## Current known package facts

The latest AiDENs package was generated on 2026-05-06 and reported:

- strict packaging;
- 1,390 included files;
- 512 Rust files;
- 40 external Cargo path dependency roots;
- zero validation findings;
- codex archive enabled;
- 26 stale Codex artifacts moved.

However, the package/evidence boundary was not reliable because P28 current-run artifacts were archived as stale and the package verifier was not self-replay-clean.

## Uploaded audit basis

The Claude hard audit reported:

- 200 confirmed bugs;
- 10 critical examples;
- 40 high examples;
- 50 medium examples;
- 50 low/code-quality examples;
- 50+ unaudited high-risk surfaces;
- estimated 100–300 additional bugs in unaudited components.

## Scope

P29 is allowed to modify:

- AiDENs docs, handoffs, scripts, verifier, status files;
- AiDENs contracts/runner/receipts/boundary/proof/semantic surfaces;
- semantic-memory and knowledge-runtime only where required by critical/high bugs;
- tests and fixtures;
- packaging scripts/assertions.

P29 must not attempt to fully repair every unaudited component. It must quarantine high-risk unaudited surfaces and block overclaims.
