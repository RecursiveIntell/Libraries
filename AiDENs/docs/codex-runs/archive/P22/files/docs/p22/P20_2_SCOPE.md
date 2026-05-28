# P20.2 Scope — Closure + Test Agent + v0.1 Certification

## Mission

Convert AiDENs from a mostly-correct canonical wiring foundation into a source/package-clean, test-agent-proven, v0.1-certifiable agent-builder layer.

## In scope

- Restore missing package artifacts referenced by code.
- Ensure all `include_str!` / `include_bytes!` targets exist.
- Correct `MANIFEST.txt` / `MANIFEST.json` or repository contents.
- Split/purify `aidens-testkit`.
- Add `aidens-integration-tests` for production-dependent vertical tests.
- Prove a canonical test agent.
- Certify provider/tool/permit/boundary/agency/receipt paths.
- Expand agency eval fixture enough to block obvious influence regressions.
- Generate final audit/release artifacts.
- Add archive recheck so zip output cannot omit required files.
- If all gates pass, execute stretch work limited to v0.1 usability and profile smoke tests.

## Out of scope until all core gates pass

- native OpenAI/Anthropic/OpenRouter tool loops;
- regional fixpoint runtime;
- multi-agent fanout;
- full Recall/Recall-Coding extraction;
- desktop daemon polish;
- federation/mechanism/theory runtime expansion.

## Stretch lane

Allowed only after all P0/P1 gates pass:

- minimal profile smoke tests;
- basic examples for coding/research/memory profiles;
- improved operator quickstart;
- P21 planning artifact for provider expansion.
